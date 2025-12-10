use std::sync::Arc;

use anyhow::{bail, Result};

use crate::database::entities::graph_data;
use crate::services::{GraphDataService, LayerPaletteService};

/// Experimental builder that operates on the unified graph_data model.
///
/// This is the Phase 3 replacement for the legacy GraphBuilder that targets
/// `graphs`/`graph_nodes`/`graph_edges`. For now it is a placeholder that will
/// be fleshed out as the pipeline migrates.
pub struct GraphDataBuilder {
    graph_data_service: Arc<GraphDataService>,
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
        _project_id: i32,
        _dag_node_id: String,
        _name: String,
        _upstream_ids: Vec<i32>,
    ) -> Result<graph_data::Model> {
        bail!("GraphDataBuilder is not implemented yet (Phase 3 in progress)");
    }
}
