use crate::graph::Graph;
use std::error::Error;

pub fn render(graph: Graph) -> Result<String, Box<dyn Error>> {
    use serde_json::json;

    let data = graph.build_json_tree();

    let handlebars = crate::common::get_handlebars();
    let res = handlebars.render_template(
        &get_template(),
        &json!({
        "tree": data,
        "nodes": graph.nodes,
        "edges": graph.get_non_partition_edges(),
        }),
    )?;
    Ok(res)
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
