use crate::graph::Graph;
use crate::plan::RenderConfig;
use std::error::Error;

/// Renders a graph hierarchy into a PlantUML WBS (Work Breakdown Structure) diagram.
pub fn render(graph: &Graph, render_config: &RenderConfig) -> Result<String, Box<dyn Error>> {
    super::renderer::render_template(graph, render_config, &get_template())
}

pub fn get_template() -> String {
    include_str!("to_plantuml_wbs.hbs").to_string()
}
