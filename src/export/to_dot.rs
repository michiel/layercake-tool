use crate::graph::Graph;
use std::error::Error;

pub fn render(graph: Graph) -> Result<String, Box<dyn Error>> {
    use serde_json::json;

    let tree = graph.build_json_tree();
    let handlebars = crate::common::get_handlebars();

    let res = handlebars.render_template(
        &get_template(),
        &json!({
            "hierarchy_nodes": graph.nodes,
            "hierarchy_edges": graph.get_hierarchy_edges(),
            "hierarchy_tree": tree,
            "flow_nodes": graph.get_non_partition_nodes(),
            "flow_edges": graph.get_non_partition_edges(),
            "layers": graph.get_layer_map(),
        }),
    )?;
    Ok(res)
}

pub fn get_template() -> String {
    include_str!("to_dot.hbs").to_string()
}

#[cfg(test)]
mod tests {

    // #[test]
    // fn graphviz_template_can_render() {
    //     let handlebars = get_handlebars();
    //     let res = handlebars
    //         .render_template(
    //             &get_template(),
    //             &json!({
    //                 "nodes": [
    //                     {
    //                         "id": "id1",
    //                         "layer": "layer1",
    //                         "label": "Node 1"
    //                     },
    //                     {
    //                         "id": "id2",
    //                         "layer": "layer1",
    //                         "label": "Node 2"
    //                     },
    //                     {
    //                         "id": "id3",
    //                         "layer": "layer2",
    //                         "label": "Node 3"
    //                     },
    //                     {
    //                         "id": "id4",
    //                         "label": "Node 4"
    //                     }
    //                 ],
    //                 "edges": [
    //                 ]
    //             }),
    //         )
    //         .expect("This to render");
    //
    //     assert_eq!(
    //         res,
    //         "\n\ndigraph G {\n    rankdir=\"TB\";\n    splines=true;\n    overlap=false;\n    nodesep=\"0.3\";\n    ranksep=\"1.2\";\n    labelloc=\"t\";\n    fontname=\"Lato\";\n    node [ shape=\"plaintext\" style=\"filled, rounded\" fontname=\"Lato\" margin=0.2 ]\n    edge [ fontname=\"Lato\" color=\"#2B303A\" ]\n\n\n}\n    "
    //     );
    // }
}
