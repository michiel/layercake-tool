use anyhow::{anyhow, Result};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{AppContext, StoryExportResult, StoryImportResult, StoryImportSummary};
use crate::database::entities::{data_sets, sequences, stories};
use crate::graphql::types::sequence::{NotePosition, SequenceEdgeRef};
use crate::graphql::types::story::StoryLayerConfig;

// ===== CSV Export =====

#[derive(Debug, Serialize)]
struct StoryCsvRow {
    story_id: i32,
    story_name: String,
    story_description: String,
    story_tags: String,
    story_enabled_dataset_ids: String,
    sequence_id: i32,
    sequence_name: String,
    sequence_description: String,
    sequence_enabled_dataset_ids: String,
    sequence_item_id: usize,
    dataset_id: i32,
    edge_id: String,
    note: String,
    note_position: String,
}

// ===== JSON Export =====

#[derive(Debug, Serialize, Deserialize)]
struct StoryExportJson {
    version: String,
    #[serde(rename = "exportedAt")]
    exported_at: String,
    stories: Vec<StoryExportData>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StoryExportData {
    id: i32,
    name: String,
    description: Option<String>,
    tags: Vec<String>,
    #[serde(rename = "enabledDatasetIds")]
    enabled_dataset_ids: Vec<i32>,
    #[serde(rename = "layerConfig")]
    layer_config: Vec<StoryLayerConfig>,
    sequences: Vec<SequenceExportData>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SequenceExportData {
    id: i32,
    name: String,
    description: Option<String>,
    #[serde(rename = "enabledDatasetIds")]
    enabled_dataset_ids: Vec<i32>,
    #[serde(rename = "edgeOrder")]
    edge_order: Vec<SequenceEdgeRef>,
}

// ===== CSV Import =====

#[derive(Debug, Deserialize)]
struct StoryCsvImportRow {
    story_id: i32,
    story_name: String,
    #[serde(default)]
    story_description: String,
    #[serde(default)]
    story_tags: String,
    #[serde(default)]
    story_enabled_dataset_ids: String,
    sequence_id: i32,
    sequence_name: String,
    #[serde(default)]
    sequence_description: String,
    #[serde(default)]
    sequence_enabled_dataset_ids: String,
    sequence_item_id: usize,
    dataset_id: i32,
    edge_id: String,
    #[serde(default)]
    note: String,
    #[serde(default)]
    note_position: String,
}

impl AppContext {
    /// Export a story to CSV format
    pub async fn export_story_csv(&self, story_id: i32) -> Result<StoryExportResult> {
        let story = stories::Entity::find_by_id(story_id)
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to fetch story {}: {}", story_id, e))?
            .ok_or_else(|| anyhow!("Story {} not found", story_id))?;

        let story_sequences = sequences::Entity::find()
            .filter(sequences::Column::StoryId.eq(story_id))
            .all(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to fetch sequences for story {}: {}", story_id, e))?;

        let mut csv_rows = Vec::new();

        for sequence in story_sequences {
            let edge_order: Vec<SequenceEdgeRef> =
                serde_json::from_str(&sequence.edge_order).unwrap_or_default();

            for (idx, edge_ref) in edge_order.iter().enumerate() {
                csv_rows.push(StoryCsvRow {
                    story_id: story.id,
                    story_name: story.name.clone(),
                    story_description: story.description.clone().unwrap_or_default(),
                    story_tags: story.tags.clone(),
                    story_enabled_dataset_ids: story.enabled_dataset_ids.clone(),
                    sequence_id: sequence.id,
                    sequence_name: sequence.name.clone(),
                    sequence_description: sequence.description.clone().unwrap_or_default(),
                    sequence_enabled_dataset_ids: sequence.enabled_dataset_ids.clone(),
                    sequence_item_id: idx,
                    dataset_id: edge_ref.dataset_id,
                    edge_id: edge_ref.edge_id.clone(),
                    note: edge_ref.note.clone().unwrap_or_default(),
                    note_position: edge_ref
                        .note_position
                        .map(|p| format!("{:?}", p))
                        .unwrap_or_default(),
                });
            }
        }

        let mut wtr = csv::Writer::from_writer(Vec::new());
        for row in csv_rows {
            wtr.serialize(row)?;
        }
        let csv_bytes = wtr
            .into_inner()
            .map_err(|e| anyhow!("Failed to finalize CSV: {}", e))?;

        let filename = format!("story_{}_export.csv", story_id);

        Ok(StoryExportResult {
            filename,
            content: csv_bytes,
            mime_type: "text/csv".to_string(),
        })
    }

    /// Export a story to JSON format
    pub async fn export_story_json(&self, story_id: i32) -> Result<StoryExportResult> {
        let story = stories::Entity::find_by_id(story_id)
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to fetch story {}: {}", story_id, e))?
            .ok_or_else(|| anyhow!("Story {} not found", story_id))?;

        let story_sequences = sequences::Entity::find()
            .filter(sequences::Column::StoryId.eq(story_id))
            .all(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to fetch sequences for story {}: {}", story_id, e))?;

        let tags: Vec<String> = serde_json::from_str(&story.tags).unwrap_or_default();
        let enabled_dataset_ids: Vec<i32> =
            serde_json::from_str(&story.enabled_dataset_ids).unwrap_or_default();
        let layer_config: Vec<StoryLayerConfig> =
            serde_json::from_str(&story.layer_config).unwrap_or_default();

        let mut sequences_data = Vec::new();
        for sequence in story_sequences {
            let seq_enabled_dataset_ids: Vec<i32> =
                serde_json::from_str(&sequence.enabled_dataset_ids).unwrap_or_default();
            let edge_order: Vec<SequenceEdgeRef> =
                serde_json::from_str(&sequence.edge_order).unwrap_or_default();

            sequences_data.push(SequenceExportData {
                id: sequence.id,
                name: sequence.name,
                description: sequence.description,
                enabled_dataset_ids: seq_enabled_dataset_ids,
                edge_order,
            });
        }

        let export_json = StoryExportJson {
            version: "1.0".to_string(),
            exported_at: Utc::now().to_rfc3339(),
            stories: vec![StoryExportData {
                id: story.id,
                name: story.name,
                description: story.description,
                tags,
                enabled_dataset_ids,
                layer_config,
                sequences: sequences_data,
            }],
        };

        let json_bytes = serde_json::to_vec_pretty(&export_json)?;
        let filename = format!("story_{}_export.json", story_id);

        Ok(StoryExportResult {
            filename,
            content: json_bytes,
            mime_type: "application/json".to_string(),
        })
    }

    /// Import stories from CSV format
    pub async fn import_story_csv(
        &self,
        project_id: i32,
        content: &str,
    ) -> Result<StoryImportResult> {
        let mut rdr = csv::Reader::from_reader(content.as_bytes());
        let mut rows: Vec<StoryCsvImportRow> = Vec::new();

        for result in rdr.deserialize() {
            let row: StoryCsvImportRow = result?;
            rows.push(row);
        }

        if rows.is_empty() {
            return Ok(StoryImportResult {
                imported_stories: Vec::new(),
                created_count: 0,
                updated_count: 0,
                errors: Vec::new(),
            });
        }

        let mut errors = Vec::new();

        // Group rows by story
        let mut story_map: std::collections::HashMap<(i32, String), Vec<StoryCsvImportRow>> =
            std::collections::HashMap::new();

        for row in rows {
            story_map
                .entry((row.story_id, row.story_name.clone()))
                .or_default()
                .push(row);
        }

        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| anyhow!("Failed to start transaction: {}", e))?;

        let mut imported_stories = Vec::new();
        let mut created_count = 0;
        let mut updated_count = 0;

        for ((story_id, story_name), story_rows) in story_map {
            let first_row = &story_rows[0];

            // Validate dataset IDs
            let story_dataset_ids = parse_csv_int_list(&first_row.story_enabled_dataset_ids);
            for dataset_id in &story_dataset_ids {
                if !self
                    .validate_dataset_exists(project_id, *dataset_id)
                    .await?
                {
                    errors.push(format!(
                        "Dataset {} not found in project {}",
                        dataset_id, project_id
                    ));
                    continue;
                }
            }

            // Create or update story
            let story_model = if story_id <= 0 {
                // Create new story
                let new_story = stories::ActiveModel {
                    project_id: Set(project_id),
                    name: Set(story_name.clone()),
                    description: Set(if first_row.story_description.is_empty() {
                        None
                    } else {
                        Some(first_row.story_description.clone())
                    }),
                    tags: Set(first_row.story_tags.clone()),
                    enabled_dataset_ids: Set(serde_json::to_string(&story_dataset_ids)?),
                    layer_config: Set("[]".to_string()),
                    created_at: Set(Utc::now()),
                    updated_at: Set(Utc::now()),
                    ..Default::default()
                };

                let model = new_story
                    .insert(&txn)
                    .await
                    .map_err(|e| anyhow!("Failed to create story '{}': {}", story_name, e))?;
                created_count += 1;
                model
            } else {
                // Try to update existing story
                match stories::Entity::find_by_id(story_id)
                    .one(&txn)
                    .await
                    .map_err(|e| anyhow!("Failed to fetch story {}: {}", story_id, e))?
                {
                    Some(existing) => {
                        let mut active: stories::ActiveModel = existing.into();
                        active.name = Set(story_name.clone());
                        active.description = Set(if first_row.story_description.is_empty() {
                            None
                        } else {
                            Some(first_row.story_description.clone())
                        });
                        active.tags = Set(first_row.story_tags.clone());
                        active.enabled_dataset_ids =
                            Set(serde_json::to_string(&story_dataset_ids)?);
                        active.updated_at = Set(Utc::now());

                        let model = active
                            .update(&txn)
                            .await
                            .map_err(|e| anyhow!("Failed to update story {}: {}", story_id, e))?;
                        updated_count += 1;
                        model
                    }
                    None => {
                        // Create as new if ID doesn't exist
                        let new_story = stories::ActiveModel {
                            project_id: Set(project_id),
                            name: Set(story_name.clone()),
                            description: Set(if first_row.story_description.is_empty() {
                                None
                            } else {
                                Some(first_row.story_description.clone())
                            }),
                            tags: Set(first_row.story_tags.clone()),
                            enabled_dataset_ids: Set(serde_json::to_string(&story_dataset_ids)?),
                            layer_config: Set("[]".to_string()),
                            created_at: Set(Utc::now()),
                            updated_at: Set(Utc::now()),
                            ..Default::default()
                        };

                        let model = new_story.insert(&txn).await.map_err(|e| {
                            anyhow!("Failed to create story '{}': {}", story_name, e)
                        })?;
                        created_count += 1;
                        model
                    }
                }
            };

            // Group rows by sequence
            let mut sequence_map: std::collections::HashMap<
                (i32, String),
                Vec<&StoryCsvImportRow>,
            > = std::collections::HashMap::new();

            for row in &story_rows {
                sequence_map
                    .entry((row.sequence_id, row.sequence_name.clone()))
                    .or_default()
                    .push(row);
            }

            let mut sequence_count = 0;

            for ((sequence_id, sequence_name), mut sequence_rows) in sequence_map {
                // Sort by sequence_item_id
                sequence_rows.sort_by_key(|r| r.sequence_item_id);

                let first_seq_row = sequence_rows[0];
                let seq_dataset_ids =
                    parse_csv_int_list(&first_seq_row.sequence_enabled_dataset_ids);

                // Build edge_order
                let mut edge_order = Vec::new();
                for seq_row in &sequence_rows {
                    // Validate dataset and edge
                    if let Err(e) = self
                        .validate_edge_exists(seq_row.dataset_id, &seq_row.edge_id)
                        .await
                    {
                        errors.push(format!(
                            "Edge validation failed for {}/{}: {}",
                            seq_row.dataset_id, seq_row.edge_id, e
                        ));
                        continue;
                    }

                    let note_position = if !seq_row.note_position.is_empty() {
                        Some(parse_note_position(&seq_row.note_position))
                    } else {
                        None
                    };

                    edge_order.push(SequenceEdgeRef {
                        dataset_id: seq_row.dataset_id,
                        edge_id: seq_row.edge_id.clone(),
                        note: if seq_row.note.is_empty() {
                            None
                        } else {
                            Some(seq_row.note.clone())
                        },
                        note_position,
                    });
                }

                // Create or update sequence
                if sequence_id <= 0 {
                    // Create new sequence
                    let new_sequence = sequences::ActiveModel {
                        story_id: Set(story_model.id),
                        name: Set(sequence_name.clone()),
                        description: Set(if first_seq_row.sequence_description.is_empty() {
                            None
                        } else {
                            Some(first_seq_row.sequence_description.clone())
                        }),
                        enabled_dataset_ids: Set(serde_json::to_string(&seq_dataset_ids)?),
                        edge_order: Set(serde_json::to_string(&edge_order)?),
                        created_at: Set(Utc::now()),
                        updated_at: Set(Utc::now()),
                        ..Default::default()
                    };

                    new_sequence.insert(&txn).await.map_err(|e| {
                        anyhow!(
                            "Failed to create sequence '{}' for story {}: {}",
                            sequence_name,
                            story_model.id,
                            e
                        )
                    })?;
                    sequence_count += 1;
                } else {
                    // Update existing sequence
                    match sequences::Entity::find_by_id(sequence_id)
                        .one(&txn)
                        .await
                        .map_err(|e| anyhow!("Failed to fetch sequence {}: {}", sequence_id, e))?
                    {
                        Some(existing) => {
                            let mut active: sequences::ActiveModel = existing.into();
                            active.name = Set(sequence_name.clone());
                            active.description =
                                Set(if first_seq_row.sequence_description.is_empty() {
                                    None
                                } else {
                                    Some(first_seq_row.sequence_description.clone())
                                });
                            active.enabled_dataset_ids =
                                Set(serde_json::to_string(&seq_dataset_ids)?);
                            active.edge_order = Set(serde_json::to_string(&edge_order)?);
                            active.updated_at = Set(Utc::now());

                            active.update(&txn).await.map_err(|e| {
                                anyhow!("Failed to update sequence {}: {}", sequence_id, e)
                            })?;
                            sequence_count += 1;
                        }
                        None => {
                            // Create new if doesn't exist
                            let new_sequence = sequences::ActiveModel {
                                story_id: Set(story_model.id),
                                name: Set(sequence_name.clone()),
                                description: Set(
                                    if first_seq_row.sequence_description.is_empty() {
                                        None
                                    } else {
                                        Some(first_seq_row.sequence_description.clone())
                                    },
                                ),
                                enabled_dataset_ids: Set(serde_json::to_string(&seq_dataset_ids)?),
                                edge_order: Set(serde_json::to_string(&edge_order)?),
                                created_at: Set(Utc::now()),
                                updated_at: Set(Utc::now()),
                                ..Default::default()
                            };

                            new_sequence.insert(&txn).await.map_err(|e| {
                                anyhow!(
                                    "Failed to create sequence '{}' for story {}: {}",
                                    sequence_name,
                                    story_model.id,
                                    e
                                )
                            })?;
                            sequence_count += 1;
                        }
                    }
                }
            }

            imported_stories.push(StoryImportSummary {
                id: story_model.id,
                name: story_model.name,
                sequence_count,
            });
        }

        txn.commit()
            .await
            .map_err(|e| anyhow!("Failed to commit transaction: {}", e))?;

        Ok(StoryImportResult {
            imported_stories,
            created_count,
            updated_count,
            errors,
        })
    }

    /// Import stories from JSON format
    pub async fn import_story_json(
        &self,
        project_id: i32,
        content: &str,
    ) -> Result<StoryImportResult> {
        let import_data: StoryExportJson =
            serde_json::from_str(content).map_err(|e| anyhow!("Failed to parse JSON: {}", e))?;

        let mut errors = Vec::new();
        let mut imported_stories = Vec::new();
        let mut created_count = 0;
        let mut updated_count = 0;

        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| anyhow!("Failed to start transaction: {}", e))?;

        for story_data in import_data.stories {
            // Validate dataset IDs
            for dataset_id in &story_data.enabled_dataset_ids {
                if !self
                    .validate_dataset_exists(project_id, *dataset_id)
                    .await?
                {
                    errors.push(format!(
                        "Dataset {} not found in project {}",
                        dataset_id, project_id
                    ));
                    continue;
                }
            }

            // Validate layer config
            for layer_cfg in &story_data.layer_config {
                if let Some(source_dataset_id) = layer_cfg.source_dataset_id {
                    if !self
                        .validate_dataset_exists(project_id, source_dataset_id)
                        .await?
                    {
                        errors.push(format!(
                            "Layer config source dataset {} not found in project {}",
                            source_dataset_id, project_id
                        ));
                    }
                }
            }

            // Create or update story
            let story_model = if story_data.id <= 0 {
                // Create new story
                let new_story = stories::ActiveModel {
                    project_id: Set(project_id),
                    name: Set(story_data.name.clone()),
                    description: Set(story_data.description.clone()),
                    tags: Set(serde_json::to_string(&story_data.tags)?),
                    enabled_dataset_ids: Set(serde_json::to_string(
                        &story_data.enabled_dataset_ids,
                    )?),
                    layer_config: Set(serde_json::to_string(&story_data.layer_config)?),
                    created_at: Set(Utc::now()),
                    updated_at: Set(Utc::now()),
                    ..Default::default()
                };

                let model = new_story
                    .insert(&txn)
                    .await
                    .map_err(|e| anyhow!("Failed to create story '{}': {}", story_data.name, e))?;
                created_count += 1;
                model
            } else {
                // Try to update existing story
                match stories::Entity::find_by_id(story_data.id)
                    .one(&txn)
                    .await
                    .map_err(|e| anyhow!("Failed to fetch story {}: {}", story_data.id, e))?
                {
                    Some(existing) => {
                        let mut active: stories::ActiveModel = existing.into();
                        active.name = Set(story_data.name.clone());
                        active.description = Set(story_data.description.clone());
                        active.tags = Set(serde_json::to_string(&story_data.tags)?);
                        active.enabled_dataset_ids =
                            Set(serde_json::to_string(&story_data.enabled_dataset_ids)?);
                        active.layer_config = Set(serde_json::to_string(&story_data.layer_config)?);
                        active.updated_at = Set(Utc::now());

                        let model = active.update(&txn).await.map_err(|e| {
                            anyhow!("Failed to update story {}: {}", story_data.id, e)
                        })?;
                        updated_count += 1;
                        model
                    }
                    None => {
                        // Create as new if ID doesn't exist
                        let new_story = stories::ActiveModel {
                            project_id: Set(project_id),
                            name: Set(story_data.name.clone()),
                            description: Set(story_data.description.clone()),
                            tags: Set(serde_json::to_string(&story_data.tags)?),
                            enabled_dataset_ids: Set(serde_json::to_string(
                                &story_data.enabled_dataset_ids,
                            )?),
                            layer_config: Set(serde_json::to_string(&story_data.layer_config)?),
                            created_at: Set(Utc::now()),
                            updated_at: Set(Utc::now()),
                            ..Default::default()
                        };

                        let model = new_story.insert(&txn).await.map_err(|e| {
                            anyhow!("Failed to create story '{}': {}", story_data.name, e)
                        })?;
                        created_count += 1;
                        model
                    }
                }
            };

            let mut sequence_count = 0;

            for sequence_data in story_data.sequences {
                // Validate edges
                for edge_ref in &sequence_data.edge_order {
                    if let Err(e) = self
                        .validate_edge_exists(edge_ref.dataset_id, &edge_ref.edge_id)
                        .await
                    {
                        errors.push(format!(
                            "Edge validation failed for {}/{}: {}",
                            edge_ref.dataset_id, edge_ref.edge_id, e
                        ));
                        continue;
                    }
                }

                // Create or update sequence
                if sequence_data.id <= 0 {
                    // Create new sequence
                    let new_sequence = sequences::ActiveModel {
                        story_id: Set(story_model.id),
                        name: Set(sequence_data.name.clone()),
                        description: Set(sequence_data.description.clone()),
                        enabled_dataset_ids: Set(serde_json::to_string(
                            &sequence_data.enabled_dataset_ids,
                        )?),
                        edge_order: Set(serde_json::to_string(&sequence_data.edge_order)?),
                        created_at: Set(Utc::now()),
                        updated_at: Set(Utc::now()),
                        ..Default::default()
                    };

                    new_sequence.insert(&txn).await.map_err(|e| {
                        anyhow!(
                            "Failed to create sequence '{}' for story {}: {}",
                            sequence_data.name,
                            story_model.id,
                            e
                        )
                    })?;
                    sequence_count += 1;
                } else {
                    // Update existing sequence
                    match sequences::Entity::find_by_id(sequence_data.id)
                        .one(&txn)
                        .await
                        .map_err(|e| {
                            anyhow!("Failed to fetch sequence {}: {}", sequence_data.id, e)
                        })? {
                        Some(existing) => {
                            let mut active: sequences::ActiveModel = existing.into();
                            active.name = Set(sequence_data.name.clone());
                            active.description = Set(sequence_data.description.clone());
                            active.enabled_dataset_ids =
                                Set(serde_json::to_string(&sequence_data.enabled_dataset_ids)?);
                            active.edge_order =
                                Set(serde_json::to_string(&sequence_data.edge_order)?);
                            active.updated_at = Set(Utc::now());

                            active.update(&txn).await.map_err(|e| {
                                anyhow!("Failed to update sequence {}: {}", sequence_data.id, e)
                            })?;
                            sequence_count += 1;
                        }
                        None => {
                            // Create new if doesn't exist
                            let new_sequence = sequences::ActiveModel {
                                story_id: Set(story_model.id),
                                name: Set(sequence_data.name.clone()),
                                description: Set(sequence_data.description.clone()),
                                enabled_dataset_ids: Set(serde_json::to_string(
                                    &sequence_data.enabled_dataset_ids,
                                )?),
                                edge_order: Set(serde_json::to_string(&sequence_data.edge_order)?),
                                created_at: Set(Utc::now()),
                                updated_at: Set(Utc::now()),
                                ..Default::default()
                            };

                            new_sequence.insert(&txn).await.map_err(|e| {
                                anyhow!(
                                    "Failed to create sequence '{}' for story {}: {}",
                                    sequence_data.name,
                                    story_model.id,
                                    e
                                )
                            })?;
                            sequence_count += 1;
                        }
                    }
                }
            }

            imported_stories.push(StoryImportSummary {
                id: story_model.id,
                name: story_model.name,
                sequence_count,
            });
        }

        txn.commit()
            .await
            .map_err(|e| anyhow!("Failed to commit transaction: {}", e))?;

        Ok(StoryImportResult {
            imported_stories,
            created_count,
            updated_count,
            errors,
        })
    }

    // ===== Helper methods =====

    async fn validate_dataset_exists(&self, project_id: i32, dataset_id: i32) -> Result<bool> {
        let dataset = data_sets::Entity::find_by_id(dataset_id)
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to check dataset {}: {}", dataset_id, e))?;

        match dataset {
            Some(ds) if ds.project_id == project_id => Ok(true),
            _ => Ok(false),
        }
    }

    async fn validate_edge_exists(&self, dataset_id: i32, edge_id: &str) -> Result<()> {
        let dataset = data_sets::Entity::find_by_id(dataset_id)
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to fetch dataset {}: {}", dataset_id, e))?
            .ok_or_else(|| anyhow!("Dataset {} not found", dataset_id))?;

        let graph_json: Value = serde_json::from_str(&dataset.graph_json).map_err(|e| {
            anyhow!(
                "Failed to parse graph_json for dataset {}: {}",
                dataset_id,
                e
            )
        })?;

        let edges = graph_json
            .get("edges")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow!("No edges array in dataset {}", dataset_id))?;

        let edge_exists = edges.iter().any(|edge| {
            edge.get("id")
                .and_then(|id| id.as_str())
                .map(|id| id == edge_id)
                .unwrap_or(false)
        });

        if edge_exists {
            Ok(())
        } else {
            Err(anyhow!(
                "Edge '{}' not found in dataset {}",
                edge_id,
                dataset_id
            ))
        }
    }
}

// ===== Utility functions =====

fn parse_csv_int_list(s: &str) -> Vec<i32> {
    if s.is_empty() {
        return Vec::new();
    }

    // Try parsing as JSON array first
    if let Ok(arr) = serde_json::from_str::<Vec<i32>>(s) {
        return arr;
    }

    // Otherwise parse as comma-separated
    s.split(',')
        .filter_map(|part| part.trim().parse::<i32>().ok())
        .collect()
}

fn parse_note_position(s: &str) -> NotePosition {
    match s.to_lowercase().as_str() {
        "source" => NotePosition::Source,
        "target" => NotePosition::Target,
        "both" => NotePosition::Both,
        _ => NotePosition::Source, // Default
    }
}
