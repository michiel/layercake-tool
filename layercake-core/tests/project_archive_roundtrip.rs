use anyhow::Result;
use chrono::Utc;
use layercake as layercake_core;
use layercake_core::app_context::{AppContext, PlanDagNodeRequest};
use layercake_core::auth::SystemActor;
use layercake_core::database::entities::{
    data_sets, layer_aliases, plan_dag_nodes, plans, project_layers, sequences, stories,
};
use layercake_core::database::migrations::Migrator;
use layercake_core::graphql::types::plan_dag::{PlanDagNodeType, Position};
use layercake_core::graphql::types::sequence::SequenceEdgeRef;
use layercake_core::services::data_set_service::DataSetService;
use layercake_core::services::plan_service::PlanCreateRequest;
use sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, Database, EntityTrait, PaginatorTrait,
    QueryFilter, Set,
};
use sea_orm_migration::MigratorTrait;
use serde_json::{json, Value};
use std::io::{Cursor, Read};
use zip::ZipArchive;

#[tokio::test]
async fn project_export_import_roundtrip_restores_assets() -> Result<()> {
    let db = Database::connect("sqlite::memory:").await?;
    Migrator::up(&db, None).await?;
    let app = AppContext::new(db.clone());

    let project = app
        .create_project(
            &SystemActor::internal(),
            "Source Project".to_string(),
            Some("contains detached datasets".to_string()),
            None,
        )
        .await?;

    let data_set_service = DataSetService::new(db.clone());
    let dataset = data_set_service
        .create_empty(
            project.id,
            "Detached DataSet".to_string(),
            Some("not referenced in plan".to_string()),
        )
        .await?;

    let plan = app
        .create_plan(&SystemActor::internal(), PlanCreateRequest {
            project_id: project.id,
            name: "Roundtrip Plan".to_string(),
            description: Some("plan with detached dataset".to_string()),
            tags: Some(vec!["roundtrip".to_string()]),
            yaml_content: "steps: []".to_string(),
            dependencies: None,
            status: Some("draft".to_string()),
        })
        .await?;

    app.create_plan_dag_node(
        &SystemActor::internal(),
        project.id,
        Some(plan.id),
        PlanDagNodeRequest {
            node_type: PlanDagNodeType::DataSet,
            position: Position { x: 0.0, y: 0.0 },
            metadata: json!({ "label": "Detached dataset" }),
            config: json!({ "dataSetId": dataset.id }),
        },
    )
    .await?;

    let plan_count_before = plans::Entity::find()
        .filter(plans::Column::ProjectId.eq(project.id))
        .count(&db)
        .await?;

    let now = Utc::now();
    let layer = project_layers::ActiveModel {
        id: NotSet,
        project_id: Set(project.id),
        layer_id: Set("layer-1".to_string()),
        name: Set("Layer 1".to_string()),
        background_color: Set("#111111".to_string()),
        text_color: Set("#eeeeee".to_string()),
        border_color: Set("#222222".to_string()),
        alias: Set(Some("alias".to_string())),
        source_dataset_id: Set(Some(dataset.id)),
        enabled: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(&db)
    .await?;

    layer_aliases::ActiveModel {
        id: NotSet,
        project_id: Set(project.id),
        alias_layer_id: Set("alias-layer".to_string()),
        target_layer_id: Set(layer.id),
        created_at: Set(now),
    }
    .insert(&db)
    .await?;

    let initial_layers = project_layers::Entity::find()
        .filter(project_layers::Column::ProjectId.eq(project.id))
        .all(&db)
        .await?;
    assert_eq!(initial_layers.len(), 1, "setup should create palette layer");

    let story = stories::ActiveModel {
        id: NotSet,
        project_id: Set(project.id),
        name: Set("Story".to_string()),
        description: Set(Some("roundtrip story".to_string())),
        tags: Set(json!(["critical"]).to_string()),
        enabled_dataset_ids: Set(json!([dataset.id]).to_string()),
        layer_config: Set(json!({ "layer-1": { "visible": true } }).to_string()),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(&db)
    .await?;

    let edge_order_payload = serde_json::to_string(&vec![SequenceEdgeRef {
        dataset_id: dataset.id,
        edge_id: "edge-1".to_string(),
        note: None,
        note_position: None,
    }])?;

    sequences::ActiveModel {
        id: NotSet,
        story_id: Set(story.id),
        name: Set("Sequence".to_string()),
        description: Set(None),
        enabled_dataset_ids: Set(json!([dataset.id]).to_string()),
        edge_order: Set(edge_order_payload),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(&db)
    .await?;

    let archive = app.export_project_archive(project.id, false).await?;
    let archive_bytes = archive.bytes.clone();
    let mut archive_reader = ZipArchive::new(Cursor::new(archive_bytes.clone()))?;
    {
        let mut palette_file = archive_reader
            .by_name("layers/palette.json")
            .expect("palette export missing from archive");
        let mut palette_json = String::new();
        palette_file.read_to_string(&mut palette_json)?;
        let palette_value: Value = serde_json::from_str(&palette_json)?;
        let exported_layer_count = palette_value["layers"]
            .as_array()
            .map(|arr| arr.len())
            .unwrap_or(0);
        assert_eq!(
            exported_layer_count, 1,
            "palette export should include layer data"
        );
    }

    {
        let mut stories_file = archive_reader
            .by_name("stories/stories.json")
            .expect("stories export missing from archive");
        let mut stories_json = String::new();
        stories_file.read_to_string(&mut stories_json)?;
        let stories_value: Value = serde_json::from_str(&stories_json)?;
        let exported_story = stories_value["stories"]
            .as_array()
            .and_then(|arr| arr.first())
            .cloned()
            .expect("stories export should contain a story");
        let exported_sequences = exported_story["sequences"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        assert_eq!(
            exported_sequences.len(),
            1,
            "stories export should include sequence data"
        );
        let exported_sequence = exported_sequences
            .first()
            .expect("sequence export should contain data");
        assert_eq!(
            exported_sequence["edgeOrder"]
                .as_array()
                .map(|arr| arr.len())
                .unwrap_or(0),
            1,
            "sequence export should include edge order references"
        );
    }

    let imported = app
        .import_project_archive(archive_bytes, Some("Imported Copy".to_string()))
        .await?;

    let imported_datasets = data_sets::Entity::find()
        .filter(data_sets::Column::ProjectId.eq(imported.id))
        .all(&db)
        .await?;
    assert_eq!(imported_datasets.len(), 1, "dataset count should match");
    let cloned_dataset = imported_datasets.first().unwrap();
    assert_eq!(cloned_dataset.name, dataset.name);

    let imported_layers = project_layers::Entity::find()
        .filter(project_layers::Column::ProjectId.eq(imported.id))
        .all(&db)
        .await?;
    assert_eq!(imported_layers.len(), 1, "layer palette should be restored");
    let imported_layer = imported_layers
        .first()
        .expect("expected layer row for imported project");
    assert_eq!(imported_layer.source_dataset_id, Some(cloned_dataset.id));

    let imported_aliases = layer_aliases::Entity::find()
        .filter(layer_aliases::Column::ProjectId.eq(imported.id))
        .all(&db)
        .await?;
    assert_eq!(imported_aliases.len(), 1, "layer alias should be restored");
    let imported_alias = imported_aliases
        .first()
        .expect("expected alias row for imported project");
    assert_eq!(imported_alias.target_layer_id, imported_layer.id);

    let imported_stories = stories::Entity::find()
        .filter(stories::Column::ProjectId.eq(imported.id))
        .all(&db)
        .await?;
    assert_eq!(imported_stories.len(), 1, "story should be restored");
    let imported_story = imported_stories
        .first()
        .expect("expected story row for imported project");
    let story_datasets: Vec<i32> =
        serde_json::from_str(&imported_story.enabled_dataset_ids).unwrap();
    assert_eq!(
        story_datasets,
        vec![cloned_dataset.id],
        "story dataset IDs should be remapped"
    );

    let imported_sequences = sequences::Entity::find()
        .filter(sequences::Column::StoryId.eq(imported_story.id))
        .all(&db)
        .await?;
    assert_eq!(
        imported_sequences.len(),
        1,
        "sequence should be restored for story"
    );
    let sequence_record = imported_sequences
        .first()
        .expect("expected sequence row for imported story");
    let sequence_datasets: Vec<i32> =
        serde_json::from_str(&sequence_record.enabled_dataset_ids).unwrap();
    assert_eq!(
        sequence_datasets,
        vec![cloned_dataset.id],
        "sequence dataset IDs should be remapped"
    );
    let edge_refs: Vec<SequenceEdgeRef> =
        serde_json::from_str(&sequence_record.edge_order).unwrap();
    assert_eq!(
        edge_refs.len(),
        1,
        "edge order references should be restored"
    );
    let first_edge = edge_refs
        .first()
        .expect("expected edge reference for imported sequence");
    assert_eq!(
        first_edge.dataset_id, cloned_dataset.id,
        "edge order dataset references should be remapped"
    );

    let imported_plans = plans::Entity::find()
        .filter(plans::Column::ProjectId.eq(imported.id))
        .all(&db)
        .await?;
    assert_eq!(
        imported_plans.len(),
        plan_count_before as usize,
        "plans index should be recreated"
    );
    let imported_plan = imported_plans
        .first()
        .expect("expected plan row for imported project");

    let imported_nodes = plan_dag_nodes::Entity::find()
        .filter(plan_dag_nodes::Column::PlanId.eq(imported_plan.id))
        .all(&db)
        .await?;
    assert_eq!(imported_nodes.len(), 1, "plan DAG nodes should be restored");
    let imported_node = imported_nodes
        .first()
        .expect("expected plan node for imported project");
    let node_config: Value = serde_json::from_str(&imported_node.config_json).unwrap();
    assert_eq!(
        node_config["dataSetId"].as_i64().unwrap() as i32,
        cloned_dataset.id,
        "plan node dataset reference should be remapped"
    );

    Ok(())
}
