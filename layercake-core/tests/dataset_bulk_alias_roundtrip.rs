use anyhow::Result;
use layercake::database::entities::{data_sets, projects};
use layercake::services::dataset_bulk_service::DataSetBulkService;
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait, Set};
use serde_json::json;

#[tokio::test]
async fn dataset_layers_alias_roundtrip_xlsx_and_ods() -> Result<()> {
    let db = setup_in_memory_db().await.expect("test database");
    let service = DataSetBulkService::new(db.clone());

    // XLSX roundtrip
    let project_xlsx = insert_project(&db, "Alias XLSX Project").await?;
    let dataset_xlsx = insert_layer_dataset(&db, project_xlsx.id, "Alias XLSX Dataset").await?;
    let xlsx_bytes = service
        .export_to_xlsx(&[dataset_xlsx.id])
        .await
        .expect("export XLSX");
    let xlsx_result = service
        .import_from_xlsx(project_xlsx.id, &xlsx_bytes)
        .await
        .expect("import XLSX");
    assert_eq!(
        xlsx_result.created_count, 1,
        "expected new dataset from XLSX import"
    );
    assert_alias_preserved(&db, xlsx_result.imported_ids[0]).await?;

    // ODS roundtrip
    let project_ods = insert_project(&db, "Alias ODS Project").await?;
    let dataset_ods = insert_layer_dataset(&db, project_ods.id, "Alias ODS Dataset").await?;
    let ods_bytes = service
        .export_to_ods(&[dataset_ods.id])
        .await
        .expect("export ODS");
    let ods_result = service
        .import_from_ods(project_ods.id, &ods_bytes)
        .await
        .expect("import ODS");
    assert_eq!(
        ods_result.created_count, 1,
        "expected new dataset from ODS import"
    );
    assert_alias_preserved(&db, ods_result.imported_ids[0]).await?;

    Ok(())
}

async fn insert_project(db: &DatabaseConnection, name: &str) -> Result<projects::Model> {
    let mut project = projects::ActiveModel::new();
    project.name = Set(name.to_string());
    project.description = Set(Some("Alias verification project".to_string()));
    Ok(project.insert(db).await?)
}

async fn insert_layer_dataset(
    db: &DatabaseConnection,
    project_id: i32,
    name: &str,
) -> Result<data_sets::Model> {
    use chrono::Utc;

    let graph_json = json!({
        "nodes": [],
        "edges": [],
        "layers": [
            {
                "id": "layer_a",
                "label": "Layer A",
                "background_color": "#ffffff",
                "text_color": "#000000",
                "alias": "primary"
            },
            {
                "id": "primary",
                "label": "Primary Palette",
                "background_color": "#f7f7f8",
                "text_color": "#0f172a"
            }
        ]
    });

    let mut dataset = data_sets::ActiveModel::new();
    dataset.project_id = Set(project_id);
    dataset.name = Set(name.to_string());
    dataset.description = Set(Some("Alias dataset".to_string()));
    dataset.file_format = Set("json".to_string());
    dataset.data_type = Set("graph".to_string());
    dataset.origin = Set("manual_edit".to_string());
    dataset.filename = Set(format!("{name}.json"));
    dataset.blob = Set(Vec::new());
    dataset.graph_json = Set(graph_json.to_string());
    dataset.status = Set("active".to_string());
    dataset.error_message = Set(None);
    dataset.file_size = Set(0);
    dataset.processed_at = Set(Some(Utc::now()));
    dataset.created_at = Set(Utc::now());
    dataset.updated_at = Set(Utc::now());

    Ok(dataset.insert(db).await?)
}

async fn assert_alias_preserved(db: &DatabaseConnection, dataset_id: i32) -> Result<()> {
    let dataset = data_sets::Entity::find_by_id(dataset_id)
        .one(db)
        .await?
        .expect("dataset should exist");
    let parsed = serde_json::from_str::<serde_json::Value>(&dataset.graph_json)?;
    let alias = parsed["layers"][0]["alias"].as_str();
    assert_eq!(alias, Some("primary"), "alias should survive roundtrip");
    Ok(())
}

async fn setup_in_memory_db() -> Result<DatabaseConnection> {
    let db = Database::connect("sqlite::memory:").await?;
    use sea_orm_migration::MigratorTrait;
    layercake::database::migrations::Migrator::up(&db, None).await?;
    Ok(db)
}
