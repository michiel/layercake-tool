use crate::graph::Graph;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct PumlNode {
    id: String,
    label: String,
    zone: String,
    layer: String,
    shape: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LayerConfig {
    pub key: String,
    #[serde(default = "layerconfig_default_fillcolor")]
    pub fillcolor: String,
    #[serde(default = "layerconfig_default_fontcolor")]
    pub fontcolor: String,
    #[serde(default = "layerconfig_default_style")]
    pub style: String,
    #[serde(default = "layerconfig_default_shape")]
    pub shape: String,
}

fn layerconfig_default_fillcolor() -> String {
    "white".to_string()
}

fn layerconfig_default_fontcolor() -> String {
    "black".to_string()
}

fn layerconfig_default_style() -> String {
    "filled".to_string()
}

fn layerconfig_default_shape() -> String {
    "rectangle".to_string()
}

fn nodes_from_raw_graphdata(graph: &Graph) -> Vec<PumlNode> {
    let mut pumlnodes = Vec::<PumlNode>::new();
    for node in &graph.nodes {
        let mut shape = "rectangle";

        let zone = match node.layer {
            Some(s) => s.to_string().clone(),
            None => "rectangle".to_string(),
        };

        pumlnodes.push(PumlNode {
            id: node.id.clone(),
            label: node.label.clone(),
            shape: shape.to_string(),
            zone,
            layer: node.layer.clone(),
        });
    }
    pumlnodes
}

pub fn render(graph: Graph) -> String {
    use serde_json::json;

    let nodes = nodes_from_raw_graphdata(&data, &config);

    let handlebars = crate::common::get_handlebars();
    let res = handlebars.render_template(
        &get_template(),
        &json!({
        "nodes": nodes,
        "edges": data.edges,
        "layerconfigs": config.layers,
        "zoneconfigs": config.zones,
        }),
    );
    res.unwrap()
}

pub fn get_template() -> String {
    let template = r##"
@startuml
  skinparam rectangle {
    BackgroundColor LightBlue
  }

  {{#each zoneconfigs as |config| ~}}
  rectangle "{{config.label}}" as {{config.key}} #White {
    {{#each ../nodes as |node| ~}}
      {{#if (exists node.zone) ~}}
        {{#if (stringeq config.key node.zone)}}
          {{node.shape}} "{{node.label}}" as {{node.id}}
        {{/if ~}}
      {{/if ~}}
    {{/each}}
  }
  {{/each}}

  {{#each nodes as |node|}}
    {{#if (isnull node.layer) }}
      rectangle "{{node.label}}" as {{node.id}}
    {{/if}}
  {{/each}}

  {{#each edges as |edge|}}
    {{edge.source_id}} --> {{edge.target_id}}
  {{/each}}

@enduml
    "##;

    template.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::get_handlebars;
    use serde_json::json;

    #[test]
    fn plantuml_template_can_render() {
        let handlebars = get_handlebars();
        let res = handlebars
            .render_template(
                &get_template(),
                &json!({
                    "nodes": [
                        {
                            "id": "id1",
                            "layer": "layer1",
                            "label": "PumlNode 1"
                        },
                        {
                            "id": "id2",
                            "layer": "layer1",
                            "label": "PumlNode 2"
                        },
                        {
                            "id": "id3",
                            "layer": "layer2",
                            "label": "PumlNode 3"
                        },
                        {
                            "id": "id4",
                            "label": "PumlNode 4"
                        }
                    ],
                    "edges": [
                    ],
                    "nodeconfigs": [
                        {
                            "key": "layer1",
                            "backgroundcolor": "white",
                            "fontcolor": "blue",
                            "shape": "rectangle"
                        },
                        {
                            "key": "layer2",
                            "backgroundcolor": "pink",
                            "fontcolor": "white",
                            "shape": "circle"
                        }
                    ],
                    "layerconfigs": [
                        {
                            "key": "layer1",
                            "label": "Layer 1"
                        },
                        {
                            "key": "layer2",
                            "label": "Layer 2"
                        }
                    ]
                }),
            )
            .expect("This to render");

        assert_eq!(
            res,
            "\n@startuml\n  skinparam rectangle {\n    BackgroundColor LightBlue\n  }\n\n\n      rectangle \"PumlNode 4\" as id4\n\n\n@enduml\n    "
        );
    }
}
