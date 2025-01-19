use crate::graph::Graph;
use serde::{Deserialize, Serialize};
use tracing::error;

// #[derive(Serialize, Deserialize)]
// struct PumlNode {
//     id: String,
//     label: String,
//     zone: String,
//     layer: String,
//     shape: String,
// }
//
// #[derive(Serialize, Deserialize, Debug)]
// pub struct LayerConfig {
//     pub key: String,
//     #[serde(default = "layerconfig_default_fillcolor")]
//     pub fillcolor: String,
//     #[serde(default = "layerconfig_default_fontcolor")]
//     pub fontcolor: String,
//     #[serde(default = "layerconfig_default_style")]
//     pub style: String,
//     #[serde(default = "layerconfig_default_shape")]
//     pub shape: String,
// }
//
// fn layerconfig_default_fillcolor() -> String {
//     "white".to_string()
// }
//
// fn layerconfig_default_fontcolor() -> String {
//     "black".to_string()
// }
//
// fn layerconfig_default_style() -> String {
//     "filled".to_string()
// }
//
// fn layerconfig_default_shape() -> String {
//     "rectangle".to_string()
// }

pub fn render(graph: Graph) -> String {
    use serde_json::json;

    let data = graph.build_json_tree();
    error!("Data: {:?}", data);

    let handlebars = crate::common::get_handlebars();
    let res = handlebars.render_template(
        &get_template(),
        &json!({
        "tree": data,
        "nodes": graph.nodes,
        "edges": graph.edges,
        }),
    );
    res.unwrap()
}

pub fn get_template() -> String {
    let template = r##"
@startuml

  {{#each tree as |rootnode|}}
{{{puml_render_tree rootnode}}}
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
                    "nodes": [{
                            "id": "1",
                            "label": "Root",
                            "layer": "Layer1",
                            "is_container": true,
                            "belongs_to": null,
                            "comment": null,
                        },
                        {
                            "id": "2",
                            "label": "Child1",
                            "layer": "Layer1",
                            "is_container": false,
                            "belongs_to": "1",
                            "comment": null,
                        },
                        {
                            "id": "3",
                            "label": "Child2",
                            "layer": "Layer1",
                            "is_container": false,
                            "belongs_to": "1",
                            "comment": null,
                        }
                    ],
                    "edges": [
                    ],
                    "tree" : [{
                            "id": "id1",
                            "label": "Root",
                            "layer": "Layer1",
                            "is_container": true,
                            "belongs_to": null,
                            "comment": null,
                            "children": [
                                {
                                    "id": "id2",
                                    "label": "Child1",
                                    "layer": "Layer1",
                                    "is_container": false,
                                    "belongs_to": "1",
                                    "comment": null,
                                    "children": []
                                },
                                {
                                    "id": "id3",
                                    "label": "Child2",
                                    "layer": "Layer1",
                                    "is_container": false,
                                    "belongs_to": "1",
                                    "comment": null,
                                    "children": []
                                }
                            ]
                        }]
                }),
            )
            .expect("This to render");

        assert_eq!(
            res,
            r##"
@startuml

  rectangle "Root" as id1
  rectangle "Child1" as id2
  rectangle "Child2" as id3

  id1 --> id2
  id1 --> id3

@enduml
            "##
        );
    }
}
