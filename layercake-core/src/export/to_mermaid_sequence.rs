use super::sequence_renderer::{render_sequence_template, SequenceRenderContext};
use std::error::Error;

pub fn render(context: &SequenceRenderContext) -> Result<String, Box<dyn Error>> {
    render_sequence_template(context, include_str!("to_mermaid_sequence.hbs"))
}
