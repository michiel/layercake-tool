use crate::graph::Graph;
use crate::plan::legacy_plan::RenderConfig;
use std::error::Error;

/// Renders a graph to the DOT format for use with Graphviz
pub fn render(graph: Graph, render_config: RenderConfig) -> Result<String, Box<dyn Error>> {
    super::renderer::render_template(graph, render_config, &get_template())
}

/// Returns the Handlebars template for DOT format
pub fn get_template() -> String {
    include_str!("to_dot.hbs").to_string()
}

#[cfg(test)]
mod tests {
    // These tests are commented out but could be rewritten
    // using the new rendering approach
}