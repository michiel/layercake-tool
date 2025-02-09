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
        "layers": graph.get_layer_map(),
        }),
    )?;
    Ok(res)
}

pub fn get_template() -> String {
    let template = r##"
@startuml

hide stereotype

<style>
{{#each layers as |layer|}}
    .{{layer.id}} {
        BackgroundColor #{{layer.background_color}};
        BorderColor #{{layer.border_color}};
        FontColor #{{layer.text_color}};
    }
{{/each}}
</style>

{{#each tree as |rootnode|}}
{{{puml_render_tree rootnode ../layers}}}
{{/each}}
{{#each edges as |edge|}}
    {{#if (exists edge.label)}}
 {{edge.source}} --> {{edge.target}} : "{{edge.label}}"
    {{else}}
 {{edge.source}} --> {{edge.target}}
    {{/if}}
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

    //     #[test]
    //     fn plantuml_template_can_render() {
    //         let handlebars = get_handlebars();
    //         let res = handlebars
    //             .render_template(
    //                 &get_template(),
    //                 &json!({
    //                     "nodes": [{
    //                             "id": "id1",
    //                             "label": "Root",
    //                             "layer": "Layer1",
    //                             "is_partition": true,
    //                             "belongs_to": null,
    //                             "comment": null,
    //                         },
    //                         {
    //                             "id": "id2",
    //                             "label": "Child1",
    //                             "layer": "Layer1",
    //                             "is_partition": false,
    //                             "belongs_to": "id1",
    //                             "comment": null,
    //                         },
    //                         {
    //                             "id": "3",
    //                             "label": "Child2",
    //                             "layer": "Layer1",
    //                             "is_partition": false,
    //                             "belongs_to": "id1",
    //                             "comment": null,
    //                         }
    //                     ],
    //                     "edges": [
    //                         {
    //                             "source": "id1",
    //                             "target": "id2",
    //                             "label": "belongs_to",
    //                             "layer": "nesting",
    //                             "comment": null,
    //                         },
    //                         {
    //                             "source": "id1",
    //                             "target": "id3",
    //                             "label": "belongs_to",
    //                             "layer": "nesting",
    //                             "comment": null,
    //                         }
    //                     ],
    //                     "tree" : [{
    //                             "id": "id1",
    //                             "label": "Root",
    //                             "layer": "Layer1",
    //                             "is_partition": true,
    //                             "belongs_to": null,
    //                             "comment": null,
    //                             "children": [
    //                                 {
    //                                     "id": "id2",
    //                                     "label": "Child1",
    //                                     "layer": "Layer1",
    //                                     "is_partition": false,
    //                                     "belongs_to": "1",
    //                                     "comment": null,
    //                                     "children": []
    //                                 },
    //                                 {
    //                                     "id": "id3",
    //                                     "label": "Child2",
    //                                     "layer": "Layer1",
    //                                     "is_partition": false,
    //                                     "belongs_to": "1",
    //                                     "comment": null,
    //                                     "children": []
    //                                 }
    //                             ]
    //                         }]
    //                 }),
    //             )
    //             .expect("This to render");
    //
    //         assert_eq!(
    //             res,
    //             r##"
    // @startuml
    //
    // rectangle "Root" as id1 {
    //   rectangle "Child1" as id2
    //   rectangle "Child2" as id3
    // }
    //
    //  id1 --> id2
    //  id1 --> id3
    //
    // @enduml
    //     "##
    //         );
    //     }
}
