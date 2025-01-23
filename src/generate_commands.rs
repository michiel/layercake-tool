use tracing::{error, info};

pub fn generate_template(exporter: String) -> () {
    info!("Generating exporter template: {}", exporter);
    match exporter.as_str() {
        "mermaid" => {
            println!("{}", crate::export::to_mermaid::get_template());
        }
        "dot" => {
            println!("{}", crate::export::to_dot::get_template());
        }
        "plantuml" => {
            println!("{}", crate::export::to_plantuml::get_template());
        }
        "gml" => {
            println!("{}", crate::export::to_gml::get_template());
        }
        _ => {
            error!(
                "Unsupported exporter: {} - use mermaid, dot, plantuml, gml",
                exporter
            );
        }
    }
}
