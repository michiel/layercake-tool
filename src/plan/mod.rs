pub mod dag_plan;
pub mod dag_execution;
pub mod legacy_plan;

pub use dag_plan::*;
pub use dag_execution::*;
pub use legacy_plan::*;

use serde::{Deserialize, Serialize};

/// Plan format enumeration to support both legacy YAML and new DAG JSON formats
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "format_version")]
pub enum PlanFormat {
    #[serde(rename = "legacy")]
    Legacy(legacy_plan::Plan),
    #[serde(rename = "dag-1.0")]
    Dag(dag_plan::DagPlan),
}

impl PlanFormat {
    /// Create a new DAG plan
    pub fn new_dag(name: String) -> Self {
        PlanFormat::Dag(DagPlan::new(name))
    }

    /// Create from legacy plan
    pub fn from_legacy(plan: legacy_plan::Plan) -> Self {
        PlanFormat::Legacy(plan)
    }

    /// Get the plan name regardless of format
    pub fn name(&self) -> String {
        match self {
            PlanFormat::Legacy(plan) => {
                plan.meta.as_ref()
                    .and_then(|m| m.name.clone())
                    .unwrap_or_else(|| "Legacy Plan".to_string())
            }
            PlanFormat::Dag(plan) => plan.name.clone(),
        }
    }

    /// Check if this is a DAG plan
    pub fn is_dag(&self) -> bool {
        matches!(self, PlanFormat::Dag(_))
    }

    /// Check if this is a legacy plan
    pub fn is_legacy(&self) -> bool {
        matches!(self, PlanFormat::Legacy(_))
    }

    /// Convert legacy plan to DAG plan (migration utility)
    pub fn migrate_to_dag(&self) -> Option<DagPlan> {
        match self {
            PlanFormat::Legacy(legacy) => Some(migrate_legacy_to_dag(legacy)),
            PlanFormat::Dag(dag) => Some(dag.clone()),
        }
    }
}

/// Convert a legacy YAML plan to a DAG plan
pub fn migrate_legacy_to_dag(legacy: &legacy_plan::Plan) -> DagPlan {
    let name = legacy.meta.as_ref()
        .and_then(|m| m.name.clone())
        .unwrap_or_else(|| "Migrated Plan".to_string());

    let mut dag_plan = DagPlan::new(name);

    // Create import nodes from legacy import profiles
    for (i, import_profile) in legacy.import.profiles.iter().enumerate() {
        let import_config = dag_plan::ImportNodeConfig {
            source_type: match import_profile.filetype {
                legacy_plan::ImportFileType::Nodes => "csv_nodes".to_string(),
                legacy_plan::ImportFileType::Edges => "csv_edges".to_string(),
                legacy_plan::ImportFileType::Layers => "csv_layers".to_string(),
            },
            source_path: Some(import_profile.filename.clone()),
            import_options: std::collections::HashMap::new(),
            field_mappings: None,
        };

        let node = DagPlanNode::new_import(
            format!("Import {}", import_profile.filename),
            import_config,
        ).with_position(100.0 * i as f64, 100.0);

        dag_plan.add_node(node);
    }

    // Create export nodes from legacy export profiles
    for (i, export_profile) in legacy.export.profiles.iter().enumerate() {
        let format = match &export_profile.exporter {
            legacy_plan::ExportFileType::GML => "gml",
            legacy_plan::ExportFileType::DOT => "dot",
            legacy_plan::ExportFileType::DOTHierarchy => "dot_hierarchy",
            legacy_plan::ExportFileType::JSON => "json",
            legacy_plan::ExportFileType::PlantUML => "plantuml",
            legacy_plan::ExportFileType::CSVNodes => "csv_nodes",
            legacy_plan::ExportFileType::CSVEdges => "csv_edges",
            legacy_plan::ExportFileType::CSVMatrix => "csv_matrix",
            legacy_plan::ExportFileType::Mermaid => "mermaid",
            legacy_plan::ExportFileType::JSGraph => "jsgraph",
            legacy_plan::ExportFileType::Custom(_) => "custom",
        };

        let mut export_options = std::collections::HashMap::new();
        
        // Convert graph config to export options
        if let Some(graph_config) = export_profile.graph_config {
            if let Some(hierarchy) = graph_config.generate_hierarchy {
                export_options.insert("generate_hierarchy".to_string(), 
                    serde_json::Value::Bool(hierarchy));
            }
            if let Some(depth) = graph_config.max_partition_depth {
                export_options.insert("max_partition_depth".to_string(), 
                    serde_json::Value::Number(depth.into()));
            }
            if let Some(width) = graph_config.max_partition_width {
                export_options.insert("max_partition_width".to_string(), 
                    serde_json::Value::Number(width.into()));
            }
            if let Some(invert) = graph_config.invert_graph {
                export_options.insert("invert_graph".to_string(), 
                    serde_json::Value::Bool(invert));
            }
        }

        let export_config = dag_plan::ExportNodeConfig {
            format: format.to_string(),
            output_path: Some(export_profile.filename.clone()),
            export_options,
            template: match &export_profile.exporter {
                legacy_plan::ExportFileType::Custom(custom) => Some(custom.template.clone()),
                _ => None,
            },
        };

        let node = DagPlanNode::new_export(
            format!("Export {}", export_profile.filename),
            export_config,
        ).with_position(300.0, 100.0 * i as f64);

        dag_plan.add_node(node);
    }

    // Connect import nodes to export nodes (simple linear flow for migration)
    if !dag_plan.nodes.is_empty() {
        let import_nodes: Vec<_> = dag_plan.nodes.iter()
            .filter(|n| matches!(n.config, dag_plan::PlanNodeConfig::Import(_)))
            .map(|n| n.id.clone())
            .collect();
        
        let export_nodes: Vec<_> = dag_plan.nodes.iter()
            .filter(|n| matches!(n.config, dag_plan::PlanNodeConfig::Export(_)))
            .map(|n| n.id.clone())
            .collect();

        // Connect each import to each export (fan-out pattern)
        for import_id in &import_nodes {
            for export_id in &export_nodes {
                let _ = dag_plan.connect_nodes(import_id, export_id);
            }
        }
    }

    dag_plan
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dag_plan_creation() {
        let plan = PlanFormat::new_dag("Test Plan".to_string());
        assert!(plan.is_dag());
        assert_eq!(plan.name(), "Test Plan");
    }

    #[test]
    fn test_legacy_migration() {
        let legacy = legacy_plan::Plan {
            meta: Some(legacy_plan::Meta {
                name: Some("Legacy Test".to_string()),
            }),
            import: legacy_plan::ImportConfig {
                profiles: vec![
                    legacy_plan::ImportProfile {
                        filename: "nodes.csv".to_string(),
                        filetype: legacy_plan::ImportFileType::Nodes,
                    },
                ],
            },
            export: legacy_plan::ExportProfile {
                profiles: vec![
                    legacy_plan::ExportProfileItem {
                        filename: "output.json".to_string(),
                        exporter: legacy_plan::ExportFileType::JSON,
                        render_config: None,
                        graph_config: None,
                    },
                ],
            },
        };

        let plan_format = PlanFormat::from_legacy(legacy);
        let dag_plan = plan_format.migrate_to_dag().unwrap();
        
        assert_eq!(dag_plan.name, "Legacy Test");
        assert_eq!(dag_plan.nodes.len(), 2); // 1 import + 1 export
        assert_eq!(dag_plan.edges.len(), 1); // connected
    }
}