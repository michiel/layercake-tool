use anyhow::Result;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::collections::HashMap;

use super::types::LayerData;
use crate::database::entities::layers;

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
pub async fn insert_layers_to_db(
    db: &DatabaseConnection,
    graph_id: i32,
    all_layers: HashMap<String, LayerData>,
) -> Result<()> {
    for (layer_id, layer_data) in all_layers {
        let layer = layers::ActiveModel {
            graph_id: Set(graph_id),
            layer_id: Set(layer_id),
            name: Set(layer_data.name),
            color: Set(layer_data.color),
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
pub async fn load_layers_from_db(
    db: &DatabaseConnection,
    graph_id: i32,
) -> Result<HashMap<String, LayerData>> {
    let db_layers = layers::Entity::find()
        .filter(layers::Column::GraphId.eq(graph_id))
        .all(db)
        .await?;

    let mut all_layers = HashMap::new();
    for db_layer in db_layers {
        let layer = LayerData {
            name: db_layer.name,
            color: db_layer.color,
            properties: db_layer.properties,
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
                color: Some("#FF0000".to_string()),
                properties: Some(r#"{"z_index":1}"#.to_string()),
            },
        );

        // In a real test with DB:
        // 1. insert_layers_to_db(&db, graph_id, layers)
        // 2. loaded = load_layers_from_db(&db, graph_id)
        // 3. assert_eq!(layers, loaded)

        assert!(!layers.is_empty());
    }
}
