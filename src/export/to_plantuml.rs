use crate::graph::Graph;
use crate::plan::RenderConfig;
use std::error::Error;

/// Renders a graph to PlantUML format
pub fn render(graph: Graph, render_config: RenderConfig) -> Result<String, Box<dyn Error>> {
    super::renderer::render_template(graph, render_config, &get_template())
}

/// Returns the Handlebars template for PlantUML format
pub fn get_template() -> String {
    include_str!("to_plantuml.hbs").to_string()
}

#[cfg(test)]
mod tests {
    // These tests are commented out but could be rewritten
    // using the new rendering approach
}