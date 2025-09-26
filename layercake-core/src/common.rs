use handlebars::{handlebars_helper, Handlebars};
use serde_json::Value;
use tracing::{error, info};

use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn create_path_if_not_exists(path: &str) -> anyhow::Result<()> {
    //
    // remove the file name from the path

    let path = Path::new(path).parent()
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
        !v.is_null() &&
        match v {
            serde_json::Value::String(s) => {
                !s.is_empty() && s != "null"
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
            _ => false, // Return false if not an array
        }
    });
    handlebars.register_helper("is_empty", Box::new(is_empty));

    handlebars_helper!(puml_render_tree: |node: Value, layermap: Value| {
        // define inline to allow recursive reference
        fn render_tree(node: Value, layermap: &serde_json::Map<String, Value>, acc: i32) -> String {
            if let Value::Object(map) = node {
                let id = map.get("id").and_then(|v| v.as_str()).unwrap_or("no-id");
                let label = map.get("label").and_then(|v| v.as_str()).unwrap_or("Unnamed");
                let layer = map.get("layer").and_then(|v| v.as_str()).unwrap_or("no-layer");
                let empty_vec = vec![];
                let children = map.get("children").and_then(|v| v.as_array()).unwrap_or(&empty_vec);

                let indent = " ".repeat((acc * 2) as usize);

                let mut result = format!("{}rectangle \"{}\" as {} <<{}>> ", indent, label, id, layer);
                if !children.is_empty() {
                    result += "{\n";
                    let children_rendered: Vec<String> = children.iter().map(|child| {
                        render_tree(child.clone(), layermap, acc + 1)
                    }).collect();
                    result += &children_rendered.join("").to_string();
                    result += &format!("{}}}\n", indent);
                } else {
                    result += "\n";
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

        render_tree(node, &layermap, 0)
    });
    handlebars.register_helper("puml_render_tree", Box::new(puml_render_tree));

    handlebars_helper!(mermaid_render_tree: |node: Value| {
        fn render_tree(node: Value, acc: i32) -> String {
            if let Value::Object(map) = node {
                let id = map.get("id").and_then(|v| v.as_str()).unwrap_or("no-id");
                let label = map.get("label").and_then(|v| v.as_str()).unwrap_or("Unnamed");
                let empty_vec = vec![];
                let children = map.get("children").and_then(|v| v.as_array()).unwrap_or(&empty_vec);

                let indent = " ".repeat((acc * 2) as usize);
                let mut result = String::new();

                if !children.is_empty() {
                    result += &format!("{}subgraph {}\n", indent, label);
                    let children_rendered: Vec<String> = children.iter().map(|child| {
                        render_tree(child.clone(), acc + 1)
                    }).collect();
                    result += &children_rendered.join("").to_string();
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

        render_tree(node, 0)
    });
    handlebars.register_helper("mermaid_render_tree", Box::new(mermaid_render_tree));

    handlebars_helper!(dot_render_tree: |node: Value, layermap: Value| {
        fn render_tree(node: Value, layermap: &serde_json::Map<String, Value>, acc: i32) -> String {
            if let Value::Object(map) = node {
                let id = map.get("id").and_then(|v| v.as_str()).unwrap_or("no-id");
                let label = map.get("label").and_then(|v| v.as_str()).unwrap_or("Unnamed");
                let layer = map.get("layer").and_then(|v| v.as_str()).unwrap_or("no-layer");
                let empty_vec = vec![];
                let children = map.get("children").and_then(|v| v.as_array()).unwrap_or(&empty_vec);

                let indent = " ".repeat((acc * 2) as usize);
                let mut result = String::new();

                if !children.is_empty() {
                    result += &format!("{}subgraph cluster_{} {{\n", indent, id);
                    result += &format!("{}  label=\"{}\"\n", indent, label);

                    if let Some(layer_props) = layermap.get(layer) {
                        result += &format!("{}  style=filled\n", indent);
                        if let Some(background_color) = layer_props.get("background_color").and_then(|v| v.as_str()) {
                            result += &format!("{}  fillcolor=\"#{}\"\n", indent, background_color);
                        }
                        if let Some(border_color) = layer_props.get("border_color").and_then(|v| v.as_str()) {
                            result += &format!("{}  color=\"#{}\"\n", indent, border_color);
                        }
                        if let Some(text_color) = layer_props.get("text_color").and_then(|v| v.as_str()) {
                            result += &format!("{}  fontcolor=\"#{}\"\n", indent, text_color);
                        }
                    }

                    let children_rendered: Vec<String> = children.iter().map(|child| {
                        render_tree(child.clone(), layermap, acc + 1)
                    }).collect();
                    result += &children_rendered.join("").to_string();
                    result += &format!("{}}}\n", indent);
                } else {
                    result += &format!("{}{}[label=\"{}\"]\n", indent, id, label);
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

        render_tree(node, &layermap, 0)
    });

    handlebars.register_helper("dot_render_tree", Box::new(dot_render_tree));

    handlebars
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn handlebars_can_render() {
        let handlebars = get_handlebars();
        let res = handlebars
            .render_template("Hello {{name}}", &json!({"name": "foo"}))
            .expect("This to render");
        assert_eq!(res, "Hello foo");
    }

    #[test]
    fn handlebars_can_iterate() {
        let handlebars = get_handlebars();
        let res = handlebars
            .render_template(
                r#"{{#each names as |name|}}
Hello {{name}}
{{/each}}"#,
                &json!({"names": ["foo", "bar", "baz"]}),
            )
            .expect("This to render");
        assert_eq!(res, "Hello foo\nHello bar\nHello baz\n");
    }

    #[test]
    fn handlebars_can_iterate_objects() {
        let handlebars = get_handlebars();
        let res = handlebars
            .render_template(
                r#"{{#each people as |person|}}
Hello {{person.name}}
{{/each}}"#,
                &json!({"people": [
                {
                    "name": "foo"
                },
                {
                    "name": "bar"
                },
                {
                    "name": "baz"
                }
                ]}),
            )
            .expect("This to render");
        assert_eq!(res, "Hello foo\nHello bar\nHello baz\n");
    }

    #[test]
    fn handlebars_helper_stringeq_can_render() {
        let handlebars = get_handlebars();
        let res = handlebars
            .render_template(
                r#"{{#if (stringeq "A label" node.label) }}
  {{node.label}};
{{/if}}"#,
                &json!({
                    "node": {
                        "label": "A label",
                    }
                }),
            )
            .expect("This to render");
        assert_eq!(res, "  A label;\n");
    }

    #[test]
    fn handlebars_helper_isnull_can_render() {
        let handlebars = get_handlebars();
        let res = handlebars
            .render_template(
                r#"{{#if (isnull node.id) }}
  {{node.label}};
{{/if}}"#,
                &json!({
                    "node": {
                        "label": "A label"
                    }
                }),
            )
            .expect("This to render");
        assert_eq!(res, "  A label;\n");
    }
}
