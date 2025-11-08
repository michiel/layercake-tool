use crate::graph::Graph;
use crate::plan::RenderConfig;
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;

#[derive(Serialize)]
struct JsGraph {
    nodes: Vec<JsNode>,
    links: Vec<JsEdge>,
}

#[derive(Serialize)]
struct JsNode {
    id: String,
    name: String,
    layer: String,
    attrs: HashMap<String, String>,
}

#[derive(Serialize)]
struct JsEdge {
    id: String,
    source: String,
    target: String,
    name: String,
    layer: String,
    attrs: HashMap<String, String>,
}

fn from_graph(graph: &Graph) -> JsGraph {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let flow_nodes = graph.get_non_partition_nodes();
    let flow_edges = graph.get_non_partition_edges();

    for node in &flow_nodes {
        let mut attrs = HashMap::new();
        attrs.insert("is_partition".to_string(), node.is_partition.to_string());
        attrs.insert("weight".to_string(), node.weight.to_string());
        attrs.insert("type".to_string(), node.layer.to_string());
        nodes.push(JsNode {
            id: node.id.clone(),
            name: node.label.clone(),
            layer: node.layer.clone(),
            attrs,
        });
    }

    for edge in &flow_edges {
        let mut attrs = HashMap::new();
        attrs.insert("weight".to_string(), edge.weight.to_string());
        attrs.insert("type".to_string(), edge.layer.to_string());
        edges.push(JsEdge {
            id: edge.id.clone(),
            source: edge.source.clone(),
            target: edge.target.clone(),
            name: edge.label.clone(),
            layer: edge.layer.clone(),
            attrs,
        });
    }

    JsGraph {
        nodes,
        links: edges,
    }
}

pub fn render(graph: &Graph, render_config: &RenderConfig) -> Result<String, Box<dyn Error>> {
    let layers = graph.get_layer_map();
    let name = &graph.name;
    let data = from_graph(graph);
    let config = json!({
        "name": name,
        "layers": layers,
        "config": render_config,
    });

    let handlebars = crate::common::get_handlebars();
    let res = handlebars.render_template(
        &get_template(),
        &json!({
                "graphData": serde_json::to_string_pretty(&data)?,
                "graphConfig": serde_json::to_string_pretty(&config)?,
        }),
    )?;
    Ok(res)
}

pub fn get_template() -> String {
    include_str!("to_jsgraph.hbs").to_string()
}
