use crate::graph::Graph;
use std::error::Error;

pub fn render(graph: Graph) -> Result<String, Box<dyn Error>> {
    use serde_json::json;

    let handlebars = crate::common::get_handlebars();

    let res = handlebars.render_template(
        &get_template(),
        &json!({
            "nodes": graph.nodes,
            "edges": graph.get_hierarchy_edges(),
            "layers": graph.get_layer_map(),
        }),
    )?;
    Ok(res)
}

pub fn get_template() -> String {
    let template = r##"

digraph G {
    rankdir="TB";
    splines=true;
    overlap=false;
    nodesep="0.3";
    ranksep="1.3";
    labelloc="t";
    fontname="Lato";
    node [ shape="plaintext" style="filled, rounded" fontsize=12]
    edge [ fontname="Lato" color="#2B303A" fontsize=8]

  {{#each layers as |layer|}}
  node [style="filled, dashed" fillcolor="#{{layer.background_color}}" fontcolor="#{{layer.text_color}}" penwidth=1 color="#{{layer.border_color}}"]; {
    {{#each ../nodes as |node|}}
        {{#if (eq node.layer layer.id)}}
            {{node.id}}[label="{{node.label}}"];
        {{/if}}
    {{/each}}
    }
  {{/each}}

node [style="filled, rounded" fillcolor="#dddddd" fontcolor="#000000"];

  {{#each edges as |edge|}}
    {{#if (exists edge.label)}}
      {{edge.source}} -> {{edge.target}} [label="{{edge.label}}" {{#each layer in ../layers}} {{#if (eq edge.layer layer.id)}} fontcolor="#{{layer.background_color}}" {{/if}} {{/each}}];
    {{else}}
      {{edge.source}} -> {{edge.target}};
    {{/if}}
  {{/each}}
}
    "##;

    template.to_string()
}
