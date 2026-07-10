use sea_orm::DatabaseConnection;

pub struct ImportService {
    #[allow(dead_code)]
    db: DatabaseConnection,
}

impl ImportService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[allow(dead_code)] // Reserved for future import result tracking
#[derive(Debug)]
pub struct ImportResult {
    pub nodes_imported: usize,
    pub edges_imported: usize,
    pub layers_imported: usize,
    pub errors: Vec<String>,
}
