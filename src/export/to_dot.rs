use crate::graph::Graph;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Serialize, Deserialize, Debug)]
pub struct LayerConfig {
    pub key: String,
    #[serde(default = "zoneconfig_puml_default_fillcolor")]
    pub fillcolor: String,
    #[serde(default = "zoneconfig_puml_default_fontcolor")]
    pub fontcolor: String,
    #[serde(default = "zoneconfig_puml_default_style")]
    pub style: String,
    #[serde(default = "zoneconfig_puml_default_shape")]
    pub shape: String,
}

fn zoneconfig_puml_default_fillcolor() -> String {
    "white".to_string()
}

fn zoneconfig_puml_default_fontcolor() -> String {
    "black".to_string()
}

fn zoneconfig_puml_default_style() -> String {
    "filled".to_string()
}

fn zoneconfig_puml_default_shape() -> String {
    "rectangle".to_string()
}

pub fn render(graph: Graph) -> String {
    use serde_json::json;

    let tree = graph.build_json_tree();
    let handlebars = crate::common::get_handlebars();

    let res = handlebars.render_template(
        &get_template(),
        &json!({
            "nodes": graph.nodes,
            "edges": graph.get_non_partition_edges(),
            "tree": tree,
        }),
    );
    res.unwrap()
}

pub fn get_template() -> String {
    let template = r##"

digraph G {
    rankdir="TB";
    splines=true;
    overlap=false;
    nodesep="0.3";
    ranksep="1.2";
    labelloc="t";
    fontname="Lato";
    node [ shape="plaintext" style="filled, rounded" fontname="Lato" margin=0.2 ]
    edge [ fontname="Lato" color="#2B303A" ]

  {{#each tree as |rootnode|}}
{{{dot_render_tree rootnode}}}
  {{/each}}

  {{#each edges as |edge|}}
    {{#if (exists edge.label)}}
      {{edge.source}} -> {{edge.target}} [label="{{edge.label}}"];
    {{else}}
      {{edge.source}} -> {{edge.target}};
    {{/if}}
  {{/each}}
}
    "##;

    template.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::get_handlebars;
    use serde_json::json;

    #[test]
    fn graphviz_template_can_render() {
        let handlebars = get_handlebars();
        let res = handlebars
            .render_template(
                &get_template(),
                &json!({
                    "nodes": [
                        {
                            "id": "id1",
                            "layer": "layer1",
                            "label": "Node 1"
                        },
                        {
                            "id": "id2",
                            "layer": "layer1",
                            "label": "Node 2"
                        },
                        {
                            "id": "id3",
                            "layer": "layer2",
                            "label": "Node 3"
                        },
                        {
                            "id": "id4",
                            "label": "Node 4"
                        }
                    ],
                    "edges": [
                    ],
                    "nodeconfigs": [
                        {
                            "key": "layer1",
                            "fillcolor": "white",
                            "fontcolor": "blue",
                            "style": "filled",
                            "shape": "diamond"
                        },
                        {
                            "key": "layer2",
                            "fillcolor": "pink",
                            "fontcolor": "white",
                            "style": "filled",
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
            "\n\ndigraph G {\n\n    layout=\"neato\";\n    rankdir=\"TB\";\n    splines=true;\n    overlap=false;\n    nodesep=\"0.2\";\n    ranksep=\"0.4\";\n    labelloc=\"t\";\n    fontname=\"Lato\";\n    node [ shape=\"plaintext\" style=\"filled, rounded\" fontname=\"Lato\" margin=0.2 ]\n    edge [ fontname=\"Lato\" color=\"#2B303A\" ]\n\n  // Apply styling to each layer\n      {\n        node[ fillcolor=\"\" fontcolor=\"\" style=\"\" shape=\"\" ];\n      }\n      {\n        node[ fillcolor=\"\" fontcolor=\"\" style=\"\" shape=\"\" ];\n      }\n\n  // Place elements in zones\n\n      id1[label=\"Node 1\"];\n      id2[label=\"Node 2\"];\n      id3[label=\"Node 3\"];\n      id4[label=\"Node 4\"];\n\n}\n    "
            );
    }
}
