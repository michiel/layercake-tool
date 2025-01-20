use crate::graph::Graph;
use tracing::debug;

pub fn render(graph: Graph) -> String {
    use serde_json::json;

    let data = graph.build_json_tree();
    debug!("Data: {:?}", data);

    let handlebars = crate::common::get_handlebars();
    let res = handlebars.render_template(
        &get_template(),
        &json!({
        "tree": data,
        "nodes": graph.nodes,
        "edges": graph.get_non_partition_edges(),
        }),
    );
    res.unwrap()
}

pub fn get_template() -> String {
    let template = r##"
flowchart LR

  {{#each tree as |rootnode|}}
{{{mermaid_render_tree rootnode}}}
  {{/each}}
  {{#each edges as |edge|}}
 {{edge.source}} --> {{edge.target}}
  {{/each}}

    "##;

    template.to_string()
}
