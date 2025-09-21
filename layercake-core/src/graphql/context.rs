use std::sync::Arc;
use sea_orm::DatabaseConnection;
use crate::services::{ImportService, ExportService, GraphService};

#[derive(Clone)]
pub struct GraphQLContext {
    pub db: DatabaseConnection,
    pub import_service: Arc<ImportService>,
    pub export_service: Arc<ExportService>,
    pub graph_service: Arc<GraphService>,
}

impl GraphQLContext {
    pub fn new(
        db: DatabaseConnection,
        import_service: Arc<ImportService>,
        export_service: Arc<ExportService>,
        graph_service: Arc<GraphService>,
    ) -> Self {
        Self {
            db,
            import_service,
            export_service,
            graph_service,
        }
    }
}