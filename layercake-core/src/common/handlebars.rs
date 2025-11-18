use handlebars::{handlebars_helper, Handlebars};
use serde_json::Value;
use tracing::{error, info};

use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn create_path_if_not_exists(path: &str) -> anyhow::Result<()> {
    let path = Path::new(path)
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Invalid path: no parent directory for '{}'", path))?;
    if !path.exists() {
        info!("Creating path: {:?}", path);
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

pub fn write_string_to_file(filename: &str, content: &str) -> anyhow::Result<()> {
    create_path_if_not_exists(filename)?;
    let path = Path::new(filename);
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

pub fn get_handlebars() -> Handlebars<'static> {
    let mut handlebars = Handlebars::new();

    handlebars_helper!(exists: |v: Value| {
        match v {
            serde_json::Value::Null => false,
            serde_json::Value::String(s) => {
                let trimmed = s.trim();
                !trimmed.is_empty() && trimmed != "null"
            }
            _ => true,
        }
    });
    handlebars.register_helper("exists", Box::new(exists));

    handlebars_helper!(isnull: |v: Value| v.is_null());
    handlebars.register_helper("isnull", Box::new(isnull));

    handlebars_helper!(stringeq: |s1: String, s2: String| s1.eq(&s2));
    handlebars.register_helper("stringeq", Box::new(stringeq));

    handlebars_helper!(is_empty: |v: Value| {
        match v {
            serde_json::Value::Array(arr) => arr.is_empty(),
            _ => false,
        }
    });
    handlebars.register_helper("is_empty", Box::new(is_empty));

    handlebars_helper!(mindmap_prefix: |depth: i64| {
        let depth = if depth < 0 { 0 } else { depth };
        "*".repeat((depth + 2) as usize)
    });
    handlebars.register_helper("mindmap_prefix", Box::new(mindmap_prefix));

    handlebars_helper!(mindmap_indent: |depth: i64| {
        let depth = if depth < 0 { 0 } else { depth };
        "  ".repeat((depth + 1) as usize)
    });
    handlebars.register_helper("mindmap_indent", Box::new(mindmap_indent));

    handlebars_helper!(treemap_value: |weight: i64| {
        if weight <= 0 {
            1
        } else {
            weight
        }
    });
    handlebars.register_helper("treemap_value", Box::new(treemap_value));

    handlebars_helper!(sanitize_id: |id: String| {
        id.chars()
            .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
            .collect::<String>()
    });
    handlebars.register_helper("sanitize_id", Box::new(sanitize_id));

    handlebars_helper!(puml_render_tree: |node: Value, layermap: Value, style_config: Value| {
        fn render_tree(
            node: Value,
            layermap: &serde_json::Map<String, Value>,
            acc: i32,
            apply_layers: bool,
            add_notes: bool,
            note_position: &str,
        ) -> String {
            if let Value::Object(map) = node {
                let id = map.get("id").and_then(|v| v.as_str()).unwrap_or("no-id");
                let label = map.get("label").and_then(|v| v.as_str()).unwrap_or("Unnamed");
                let layer = map.get("layer").and_then(|v| v.as_str()).unwrap_or("no-layer");
                let comment = map
                    .get("comment")
                    .and_then(|v| v.as_str())
                    .map(|s| s.trim())
                    .unwrap_or("");
                let has_comment = !comment.is_empty() && comment != "null";
                let empty_vec = vec![];
                let children = map.get("children").and_then(|v| v.as_array()).unwrap_or(&empty_vec);

                let indent = " ".repeat((acc * 2) as usize);

                let mut result = if apply_layers {
                    format!("{}rectangle \"{}\" as {} <<{}>> ", indent, label, id, layer)
                } else {
                    format!("{}rectangle \"{}\" as {} ", indent, label, id)
                };
                if !children.is_empty() {
                    result += "{\n";
                    let children_rendered: Vec<String> = children.iter().map(|child| {
                        render_tree(child.clone(), layermap, acc + 1, apply_layers, add_notes, note_position)
                    }).collect();
                    result += &children_rendered.join("");
                    result += &format!("{}}}\n", indent);
                } else {
                    result += "\n";
                }
                if add_notes && has_comment {
                    result += &format!(
                        "{}note {} of {} : {}\n",
                        indent, note_position, id, comment
                    );
                }
                result
            } else {
                error!("Expected object, got: {:?}", node);
                String::new()
            }
        }

        let layermap = match layermap {
            serde_json::Value::Object(map) => map,
            _ => {
                error!("Expected layer map object, got: {:?}", layermap);
                serde_json::Map::new()
            }
        };

        let style_map = match style_config {
            serde_json::Value::Object(map) => map,
            _ => serde_json::Map::new(),
        };

        let apply_layers = style_map
            .get("apply_layers")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let add_notes = style_map
            .get("add_node_comments_as_notes")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let note_position = style_map
            .get("note_position")
            .and_then(|v| v.as_str())
            .map(|v| v.to_lowercase())
            .unwrap_or_else(|| "left".to_string());

        render_tree(
            node,
            &layermap,
            0,
            apply_layers,
            add_notes,
            note_position.as_str(),
        )
    });
    handlebars.register_helper("puml_render_tree", Box::new(puml_render_tree));

    handlebars_helper!(mermaid_render_tree: |node: Value, layermap: Value, style_config: Value| {
        fn render_tree(
            node: Value,
            _layermap: &serde_json::Map<String, Value>,
            acc: i32,
            _apply_layers: bool,
        ) -> String {
            if let Value::Object(map) = node {
                let id = map.get("id").and_then(|v| v.as_str()).unwrap_or("no-id");
                let label = map.get("label").and_then(|v| v.as_str()).unwrap_or("Unnamed");
                let empty_vec = vec![];
                let children = map.get("children").and_then(|v| v.as_array()).unwrap_or(&empty_vec);

                let indent = " ".repeat((acc * 2) as usize);
                let mut result = String::new();

                if !children.is_empty() {
                    result += &format!("{}subgraph \"{}\"\n", indent, label);
                    let children_rendered: Vec<String> = children.iter().map(|child| {
                        render_tree(child.clone(), _layermap, acc + 1, _apply_layers)
                    }).collect();
                    result += &children_rendered.join("");
                    result += &format!("{}end\n", indent);
                } else {
                    result += &format!("{}{}[\"{}\"]\n", indent, id, label);
                }

                result
            } else {
                error!("Expected object, got: {:?}", node);
                String::new()
            }
        }

        let layermap = match layermap {
            serde_json::Value::Object(map) => map,
            _ => {
                error!("Expected layer map object, got: {:?}", layermap);
                serde_json::Map::new()
            }
        };

        let style_map = match style_config {
            serde_json::Value::Object(map) => map,
            _ => serde_json::Map::new(),
        };

        let apply_layers = style_map
            .get("apply_layers")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        render_tree(node, &layermap, 0, apply_layers)
    });
    handlebars.register_helper("mermaid_render_tree", Box::new(mermaid_render_tree));

    handlebars_helper!(dot_render_tree: |node: Value, layermap: Value, style_config: Value| {
        fn theme_palette(style: &str) -> (&'static str, &'static str, &'static str, &'static str) {
            match style.to_lowercase().as_str() {
                "dark" => ("#1e1e1e", "#f5f5f5", "#444444", "#888888"),
                "none" => ("#ffffff", "#000000", "#cccccc", "#444444"),
                _ => ("#f7f7f8", "#0f172a", "#1f2933", "#cccccc"),
            }
        }

        fn render_tree(
            node: Value,
            layermap: &serde_json::Map<String, Value>,
            acc: i32,
            apply_layers: bool,
            built_in_style: &str,
            add_notes: bool,
            comment_style: &str,
        ) -> String {
            if let Value::Object(map) = node {
                let id = map.get("id").and_then(|v| v.as_str()).unwrap_or("no-id");
                let label = map.get("label").and_then(|v| v.as_str()).unwrap_or("Unnamed");
                let layer = map.get("layer").and_then(|v| v.as_str()).unwrap_or("no-layer");
                let comment = map
                    .get("comment")
                    .and_then(|v| v.as_str())
                    .map(|s| s.trim())
                    .unwrap_or("");
                let has_comment = add_notes && !comment.is_empty() && comment != "null";
                let empty_vec = vec![];
                let children = map.get("children").and_then(|v| v.as_array()).unwrap_or(&empty_vec);

                let indent = " ".repeat((acc * 2) as usize);
                let mut result = String::new();

                let (default_fill, default_font, default_border, default_container_border) =
                    theme_palette(built_in_style);

                if !children.is_empty() {
                    result += &format!("{}subgraph cluster_{} {{\n", indent, id);
                    result += &format!("{}  label=\"{}\"\n", indent, label);

                    if apply_layers {
                        let mut fillcolor = default_fill.trim_start_matches('#').to_string();
                        let mut bordercolor =
                            default_container_border.trim_start_matches('#').to_string();
                        let mut fontcolor = default_font.trim_start_matches('#').to_string();

                        if let Some(layer_props) = layermap.get(layer) {
                            if let Some(background_color) =
                                layer_props.get("background_color").and_then(|v| v.as_str())
                            {
                                fillcolor = background_color.to_string();
                            }
                            if let Some(border_color) =
                                layer_props.get("border_color").and_then(|v| v.as_str())
                            {
                                bordercolor = border_color.to_string();
                            }
                            if let Some(text_color) =
                                layer_props.get("text_color").and_then(|v| v.as_str())
                            {
                                fontcolor = text_color.to_string();
                            }
                        }

                        result += &format!("{}  style=filled\n", indent);
                        result += &format!("{}  fillcolor=\"#{}\"\n", indent, fillcolor);
                        result += &format!("{}  color=\"#{}\"\n", indent, bordercolor);
                        result += &format!("{}  fontcolor=\"#{}\"\n", indent, fontcolor);
                    }

                    let children_rendered: Vec<String> = children.iter().map(|child| {
                        render_tree(
                            child.clone(),
                            layermap,
                            acc + 1,
                            apply_layers,
                            built_in_style,
                            add_notes,
                            comment_style,
                        )
                    }).collect();
                    result += &children_rendered.join("");
                    result += &format!("{}  }}\n", indent);
                } else if apply_layers {
                    let mut fillcolor = default_fill.trim_start_matches('#').to_string();
                    let mut fontcolor = default_font.trim_start_matches('#').to_string();
                    let mut bordercolor = default_border.trim_start_matches('#').to_string();

                    if let Some(layer_props) = layermap.get(layer) {
                        if let Some(background_color) =
                            layer_props.get("background_color").and_then(|v| v.as_str())
                        {
                            fillcolor = background_color.to_string();
                        }
                        if let Some(text_color) =
                            layer_props.get("text_color").and_then(|v| v.as_str())
                        {
                            fontcolor = text_color.to_string();
                        }
                        if let Some(border_color) =
                            layer_props.get("border_color").and_then(|v| v.as_str())
                        {
                            bordercolor = border_color.to_string();
                        }
                    }

                    let escaped_comment = comment.replace('"', "\\\"");
                    let comment_attr = if has_comment {
                        match comment_style {
                            "tooltip" => format!(", tooltip=\"{}\"", escaped_comment),
                            _ => format!(", xlabel=\"{}\"", escaped_comment),
                        }
                    } else {
                        String::new()
                    };
                    result += &format!(
                        "{}{} [label=\"{}\", layer=\"{}\", style=\"filled,rounded\", fillcolor=\"#{}\", fontcolor=\"#{}\", color=\"#{}\"{}];\n",
                        indent, id, label, layer, fillcolor, fontcolor, bordercolor, comment_attr
                    );
                } else {
                    let escaped_comment = comment.replace('"', "\\\"");
                    let comment_attr = if has_comment {
                        match comment_style {
                            "tooltip" => format!(", tooltip=\"{}\"", escaped_comment),
                            _ => format!(", xlabel=\"{}\"", escaped_comment),
                        }
                    } else {
                        String::new()
                    };
                    result += &format!(
                        "{}{} [label=\"{}\", layer=\"{}\", style=\"rounded\"{}];\n",
                        indent, id, label, layer, comment_attr
                    );
                }

                result
            } else {
                error!("Expected object, got: {:?}", node);
                String::new()
            }
        }

        let layermap = match layermap {
            serde_json::Value::Object(map) => map,
            _ => {
                error!("Expected layer map object, got: {:?}", layermap);
                serde_json::Map::new()
            }
        };

        let style_map = match style_config {
            serde_json::Value::Object(map) => map,
            _ => serde_json::Map::new(),
        };

        let apply_layers = style_map
            .get("apply_layers")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let built_in_style = style_map
            .get("built_in_styles")
            .and_then(|v| v.as_str())
            .unwrap_or("light");
        let add_notes = style_map
            .get("add_node_comments_as_notes")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let comment_style = style_map
            .get("target_options")
            .and_then(|v| v.get("graphviz"))
            .and_then(|v| v.get("comment_style"))
            .and_then(|v| v.as_str())
            .unwrap_or("label");

        render_tree(
            node,
            &layermap,
            0,
            apply_layers,
            built_in_style,
            add_notes,
            comment_style,
        )
    });
    handlebars.register_helper("dot_render_tree", Box::new(dot_render_tree));

    handlebars_helper!(puml_link: |layer: Value, link_type: Value| {
        let default_color = "black";
        if let (Value::String(layer), Value::String(link_type)) = (layer, link_type) {
            match (layer.as_str(), link_type.as_str()) {
                ("data", "parent_of") => "#FF5733",
                ("data", "child_of") => "#33FF57",
                ("data", "related_to") => "#3357FF",
                ("control", _) => "#FFC300",
                ("application", _) => "#FF33A8",
                ("infrastructure", _) => "#33FFF0",
                ("threat", _) => "#FF3333",
                _ => default_color,
            }
        } else {
            default_color
        }
    });
    handlebars.register_helper("puml_link_color", Box::new(puml_link));

    handlebars_helper!(layer_bg_color: |layermap: Value, layer_id: String| {
        if let Value::Object(map) = layermap {
            if let Some(layer_obj) = map.get(&layer_id) {
                if let Some(bg_color) = layer_obj.get("background_color").and_then(|v| v.as_str()) {
                    bg_color.to_string()
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    });
    handlebars.register_helper("layer_bg_color", Box::new(layer_bg_color));

    handlebars
}
