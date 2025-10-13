use include_dir::{include_dir, Dir};
use std::fs;
use std::path::Path;
use tracing::{error, info};

static SAMPLE_DIR_KVM_CONTROL_FLOW: Dir = include_dir!("../sample/kvm_control_flow");
static SAMPLE_DIR_ATTACK_TREE: Dir = include_dir!("../sample/attack_tree");
static SAMPLE_DIR_REF: Dir = include_dir!("../sample/ref");
static SAMPLE_DIR_DISTRIBUTED_MONOLITH: Dir = include_dir!("../sample/distributed-monolith");

pub fn generate_template(exporter: String) {
    info!("Generating exporter template: {}", exporter);
    match exporter.as_str() {
        "mermaid" => {
            println!("{}", crate::export::to_mermaid::get_template());
        }
        "dot" => {
            println!("{}", crate::export::to_dot::get_template());
        }
        "dothierarchy" => {
            println!("{}", crate::export::to_dot_hierarchy::get_template());
        }
        "jsgraph" => {
            println!("{}", crate::export::to_jsgraph::get_template());
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

pub fn generate_sample(sample: String, dir: String) {
    info!("Generating sample project: {:?} in {:?}", sample, dir);
    let target_path = Path::new(&dir);
    if let Err(e) = fs::create_dir_all(target_path) {
        error!("Failed to create target directory: {:?}", e);
        return;
    }

    fn write_dir_contents(dir: &Dir, target_path: &Path) {
        for file in dir.files() {
            let relative_path = match file.path().strip_prefix(dir.path()) {
                Ok(path) => path,
                Err(e) => {
                    error!(
                        "Failed to create relative path for {:?}: {}",
                        file.path(),
                        e
                    );
                    continue;
                }
            };
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
            let relative_path = match sub_dir.path().strip_prefix(dir.path()) {
                Ok(path) => path,
                Err(e) => {
                    error!(
                        "Failed to create relative path for {:?}: {}",
                        sub_dir.path(),
                        e
                    );
                    continue;
                }
            };
            let sub_dir_path = target_path.join(relative_path);
            if let Err(e) = fs::create_dir_all(&sub_dir_path) {
                error!("Failed to create subdirectory: {:?}", e);
                return;
            }
            write_dir_contents(sub_dir, &sub_dir_path);
        }
    }

    match sample.to_lowercase().as_str() {
        "reference" => write_dir_contents(&SAMPLE_DIR_REF, target_path),
        "ref" => write_dir_contents(&SAMPLE_DIR_REF, target_path),
        "attack-tree" => write_dir_contents(&SAMPLE_DIR_ATTACK_TREE, target_path),
        "attack_tree" => write_dir_contents(&SAMPLE_DIR_ATTACK_TREE, target_path),
        "kvm_control_flow" => write_dir_contents(&SAMPLE_DIR_KVM_CONTROL_FLOW, target_path),
        "kvm-control-flow" => write_dir_contents(&SAMPLE_DIR_KVM_CONTROL_FLOW, target_path),
        "distributed-monolith" => write_dir_contents(&SAMPLE_DIR_DISTRIBUTED_MONOLITH, target_path),
        _ => {
            error!(
                "Unsupported sample: {} - use ref, reference, kvm-control-flow, attack-tree, distributed-monolith",
                sample
            );
            return;
        }
    }

    info!("Sample project generated successfully at: {:?}", dir);
}
