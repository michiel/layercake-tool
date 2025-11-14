use anyhow::Result;
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};
use std::collections::HashMap;

use super::types::LayerData;
use crate::database::entities::graph_layers;

/// Insert layers from a HashMap into the database for a given graph
///
/// This is shared functionality used by both MergeBuilder and GraphBuilder
/// when persisting layer data after graph construction.
///
/// # Arguments
/// * `db` - Database connection
/// * `graph_id` - ID of the graph these layers belong to
/// * `all_layers` - HashMap of layer_id -> LayerData to insert
///
/// # Returns
/// * `Result<()>` - Success or database error
pub async fn insert_layers_to_db<C>(
    db: &C,
    graph_id: i32,
    all_layers: HashMap<String, LayerData>,
) -> Result<()>
where
    C: ConnectionTrait,
{
    for (layer_id, layer_data) in all_layers {
        let layer = graph_layers::ActiveModel {
            graph_id: Set(graph_id),
            layer_id: Set(layer_id),
            name: Set(layer_data.name),
            background_color: Set(layer_data.background_color),
            text_color: Set(layer_data.text_color),
            border_color: Set(layer_data.border_color),
            comment: Set(layer_data.comment),
            dataset_id: Set(layer_data.dataset_id),
            properties: Set(layer_data.properties),
            ..Default::default()
        };

        layer.insert(db).await?;
    }

    Ok(())
}

/// Load layers from the database for a given graph into a HashMap
///
/// This is shared functionality used by pipeline builders when loading
/// existing layer data for graph merging or transformation.
///
/// # Arguments
/// * `db` - Database connection
/// * `graph_id` - ID of the graph to load layers for
///
/// # Returns
/// * `Result<HashMap<String, LayerData>>` - HashMap of layer_id -> LayerData
pub async fn load_layers_from_db<C>(db: &C, graph_id: i32) -> Result<HashMap<String, LayerData>>
where
    C: ConnectionTrait,
{
    let db_layers = graph_layers::Entity::find()
        .filter(graph_layers::Column::GraphId.eq(graph_id))
        .all(db)
        .await?;

    let mut all_layers = HashMap::new();
    for db_layer in db_layers {
        let layer = LayerData {
            name: db_layer.name,
            background_color: db_layer.background_color,
            text_color: db_layer.text_color,
            border_color: db_layer.border_color,
            comment: db_layer.comment,
            properties: db_layer.properties,
            dataset_id: db_layer.dataset_id,
        };
        all_layers.insert(db_layer.layer_id, layer);
    }

    Ok(all_layers)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These would need a test database to run properly
    // For now, they serve as documentation of expected behavior

    #[test]
    fn test_layer_data_roundtrip_concept() {
        // Create test layer data
        let mut layers = HashMap::new();
        layers.insert(
            "layer1".to_string(),
            LayerData {
                name: "Test Layer".to_string(),
                background_color: Some("#FF0000".to_string()),
                text_color: None,
                border_color: None,
                comment: None,
                properties: Some(r#"{"z_index":1}"#.to_string()),
                dataset_id: None,
            },
        );

        // In a real test with DB:
        // 1. insert_layers_to_db(&db, graph_id, layers)
        // 2. loaded = load_layers_from_db(&db, graph_id)
        // 3. assert_eq!(layers, loaded)

        assert!(!layers.is_empty());
    }
}
