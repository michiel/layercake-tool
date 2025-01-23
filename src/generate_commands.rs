use include_dir::{include_dir, Dir};
use std::fs;
use std::path::Path;
use tracing::{error, info};

static SAMPLE_DIR: Dir = include_dir!("sample");

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

pub fn generate_sample(dir: String) -> () {
    info!("Generating sample project: {:?}", dir);
    let target_path = Path::new(&dir);
    if let Err(e) = fs::create_dir_all(target_path) {
        error!("Failed to create target directory: {:?}", e);
        return;
    }

    fn write_dir_contents(dir: &Dir, target_path: &Path) {
        for file in dir.files() {
            let relative_path = file.path();
            let target_file_path = target_path.join(relative_path);

            if let Some(parent) = target_file_path.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    error!("Failed to create directory: {:?}", e);
                    return;
                }
            }

            if let Err(e) = fs::write(&target_file_path, file.contents()) {
                error!("Failed to write file: {:?}", e);
                return;
            }
        }

        for sub_dir in dir.dirs() {
            let sub_dir_path = target_path.join(sub_dir.path());
            if let Err(e) = fs::create_dir_all(&sub_dir_path) {
                error!("Failed to create subdirectory: {:?}", e);
                return;
            }
            write_dir_contents(sub_dir, &sub_dir_path);
        }
    }

    write_dir_contents(&SAMPLE_DIR, target_path);

    info!("Sample project generated successfully at: {:?}", dir);
}
