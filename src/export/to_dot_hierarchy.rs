use crate::graph::Graph;
use crate::plan::legacy_plan::RenderConfig;
use std::error::Error;

pub fn render(graph: Graph, render_config: RenderConfig) -> Result<String, Box<dyn Error>> {
    super::renderer::render_template(graph, render_config, &get_template())
}

pub fn get_template() -> String {
    include_str!("to_dot_hierarchy.hbs").to_string()
}
