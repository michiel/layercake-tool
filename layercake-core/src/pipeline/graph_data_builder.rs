use std::sync::Arc;

use anyhow::{bail, Result};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

use crate::database::entities::graph_data;
use crate::services::{GraphDataService, LayerPaletteService};

/// Experimental builder that operates on the unified graph_data model.
///
/// This is the Phase 3 replacement for the legacy GraphBuilder that targets
/// `graphs`/`graph_nodes`/`graph_edges`. For now it is a placeholder that will
/// be fleshed out as the pipeline migrates.
pub struct GraphDataBuilder {
    pub graph_data_service: Arc<GraphDataService>,
    layer_palette_service: Arc<LayerPaletteService>,
}

impl GraphDataBuilder {
    pub fn new(
        graph_data_service: Arc<GraphDataService>,
        layer_palette_service: Arc<LayerPaletteService>,
    ) -> Self {
        Self {
            graph_data_service,
            layer_palette_service,
        }
    }

    /// Build a graph_data record from upstream unified sources.
    ///
    /// Placeholder implementation until the DAG executor is migrated.
    pub async fn build_graph(
        &self,
        project_id: i32,
        dag_node_id: String,
        name: String,
        upstream_ids: Vec<i32>,
    ) -> Result<graph_data::Model> {
        // Load upstream graph_data (datasets or computed)
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        for id in upstream_ids {
            let (_g, mut g_nodes, mut g_edges) = self.graph_data_service.load_full(id).await?;
            nodes.append(&mut g_nodes);
            edges.append(&mut g_edges);
        }

        // Validate layer references are present in project palette
        let layer_ids: HashSet<String> = nodes
            .iter()
            .filter_map(|n| n.layer.clone())
            .chain(edges.iter().filter_map(|e| e.layer.clone()))
            .collect();

        let validation = self
            .layer_palette_service
            .validate_layer_references(project_id, &layer_ids)
            .await?;
        if !validation.missing_layers.is_empty() {
            bail!(
                "Missing layers in project palette: {:?}",
                validation.missing_layers
            );
        }

        // Compute source hash for change detection
        let source_hash = self.compute_source_hash(&nodes, &edges);

        // Reuse existing graph_data by dag_node_id if hash matches
        if let Some(existing) = self
            .graph_data_service
            .get_by_dag_node(&dag_node_id)
            .await?
        {
            if existing.source_hash.as_deref() == Some(&source_hash)
                && existing.status == graph_data::GraphDataStatus::Active.as_str()
            {
                return Ok(existing);
            }

            // Mark processing before replacing content
            self.graph_data_service
                .mark_status(existing.id, graph_data::GraphDataStatus::Processing, None)
                .await?;

            // Persist merged nodes/edges onto existing record
            self.graph_data_service
                .replace_nodes(
                    existing.id,
                    nodes
                        .into_iter()
                        .map(|n| crate::services::GraphDataNodeInput {
                            external_id: n.external_id,
                            label: n.label,
                            layer: n.layer,
                            weight: n.weight,
                            is_partition: Some(n.is_partition),
                            belongs_to: n.belongs_to,
                            comment: n.comment,
                            source_dataset_id: n.source_dataset_id,
                            attributes: n.attributes,
                            created_at: Some(n.created_at),
                        })
                        .collect(),
                )
                .await?;

            self.graph_data_service
                .replace_edges(
                    existing.id,
                    edges
                        .into_iter()
                        .map(|e| crate::services::GraphDataEdgeInput {
                            external_id: e.external_id,
                            source: e.source,
                            target: e.target,
                            label: e.label,
                            layer: e.layer,
                            weight: e.weight,
                            comment: e.comment,
                            source_dataset_id: e.source_dataset_id,
                            attributes: e.attributes,
                            created_at: Some(e.created_at),
                        })
                        .collect(),
                )
                .await?;

            self.graph_data_service
                .mark_complete(existing.id, source_hash)
                .await?;

            let (graph, _, _) = self.graph_data_service.load_full(existing.id).await?;
            return Ok(graph);
        }

        // Create the new computed graph_data shell using convenience method
        let created = self
            .graph_data_service
            .create_computed(project_id, dag_node_id, name)
            .await?;

        // Persist merged nodes/edges
        self.graph_data_service
            .replace_nodes(
                created.id,
                nodes
                    .into_iter()
                    .map(|n| crate::services::GraphDataNodeInput {
                        external_id: n.external_id,
                        label: n.label,
                        layer: n.layer,
                        weight: n.weight,
                        is_partition: Some(n.is_partition),
                        belongs_to: n.belongs_to,
                        comment: n.comment,
                        source_dataset_id: n.source_dataset_id,
                        attributes: n.attributes,
                        created_at: Some(n.created_at),
                    })
                    .collect(),
            )
            .await?;

        self.graph_data_service
            .replace_edges(
                created.id,
                edges
                    .into_iter()
                    .map(|e| crate::services::GraphDataEdgeInput {
                        external_id: e.external_id,
                        source: e.source,
                        target: e.target,
                        label: e.label,
                        layer: e.layer,
                        weight: e.weight,
                        comment: e.comment,
                        source_dataset_id: e.source_dataset_id,
                        attributes: e.attributes,
                        created_at: Some(e.created_at),
                    })
                    .collect(),
            )
            .await?;

        // Mark complete with no hash (hashing to be added later)
        self.graph_data_service
            .mark_complete(created.id, source_hash)
            .await?;

        // Reload full record with counts
        let (graph, _, _) = self.graph_data_service.load_full(created.id).await?;
        Ok(graph)
    }

    fn compute_source_hash(
        &self,
        nodes: &[crate::database::entities::graph_data_nodes::Model],
        edges: &[crate::database::entities::graph_data_edges::Model],
    ) -> String {
        let mut hasher = Sha256::new();

        let mut sorted_nodes = nodes.to_owned();
        sorted_nodes.sort_by(|a, b| a.external_id.cmp(&b.external_id));
        for n in sorted_nodes {
            hasher.update(n.external_id.as_bytes());
            if let Some(label) = &n.label {
                hasher.update(label.as_bytes());
            }
            if let Some(layer) = &n.layer {
                hasher.update(layer.as_bytes());
            }
            if let Some(weight) = n.weight {
                hasher.update(weight.to_le_bytes());
            }
        }

        let mut sorted_edges = edges.to_owned();
        sorted_edges.sort_by(|a, b| a.external_id.cmp(&b.external_id));
        for e in sorted_edges {
            hasher.update(e.external_id.as_bytes());
            hasher.update(e.source.as_bytes());
            hasher.update(e.target.as_bytes());
            if let Some(label) = &e.label {
                hasher.update(label.as_bytes());
            }
            if let Some(layer) = &e.layer {
                hasher.update(layer.as_bytes());
            }
            if let Some(weight) = e.weight {
                hasher.update(weight.to_le_bytes());
            }
        }

        format!("{:x}", hasher.finalize())
    }
}
