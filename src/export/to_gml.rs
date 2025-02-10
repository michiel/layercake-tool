use crate::graph::Graph;
use std::error::Error;

pub fn render(graph: Graph) -> Result<String, Box<dyn Error>> {
    use serde_json::json;

    let handlebars = crate::common::get_handlebars();
    let res = handlebars.render_template(
        &get_template(),
        &json!({
        "hierarchy_nodes": graph.nodes,
        "hierarchy_edges": graph.get_hierarchy_edges(),
        // "hierarchy_tree": tree,
        "flow_nodes": graph.get_non_partition_nodes(),
        "flow_edges": graph.get_non_partition_edges(),
        "layers": graph.get_layer_map(),
        }),
    )?;
    Ok(res)
}

pub fn get_template() -> String {
    include_str!("to_gml.hbs").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::get_handlebars;
    use serde_json::json;

    #[test]
    fn template_can_render() {
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
                    ]
                }),
            )
            .expect("This to render");

        // TODO : Fix this test after changes
        // assert_eq!(res, "\n\ngraph [\n    id 0\n    label \"Graph\"\n    node [\n      id id1\n      label \"Node 1\"\n      layer \"layer1\"\n    ]\n    node [\n      id id2\n      label \"Node 2\"\n      layer \"layer1\"\n    ]\n    node [\n      id id3\n      label \"Node 3\"\n      layer \"layer2\"\n    ]\n    node [\n      id id4\n      label \"Node 4\"\n    ]\n\n]\n    ");
    }
}
