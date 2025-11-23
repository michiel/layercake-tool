use anyhow::{anyhow, Result};
use indexmap::IndexMap;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

use crate::database::entities::{data_sets, project_layers, sequences, stories};
use crate::export::sequence_renderer::{
    SequenceDataset, SequenceEdge, SequenceGraphData, SequenceLayer, SequenceNode,
    SequenceParticipant, SequenceParticipantGroup, SequenceParticipantRef, SequenceRender,
    SequenceRenderConfigResolved, SequenceRenderContext, SequenceStep, SequenceStorySummary,
    SequenceStoryWithSequences,
};
use crate::graphql::types::sequence::{NotePosition, SequenceEdgeRef};
use crate::graphql::types::story::StoryLayerConfig;
use crate::plan::LayerSourceStyle;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SequenceStoryContext {
    pub story: SequenceStorySummary,
    pub sequences: Vec<SequenceRender>,
    pub participants: Vec<SequenceParticipant>,
    pub graph_data: SequenceGraphData,
    pub layers: Vec<SequenceLayer>,
}

pub async fn build_story_context(
    db: &DatabaseConnection,
    project_id: i32,
    story_id: i32,
) -> Result<SequenceStoryContext> {
    let story = stories::Entity::find_by_id(story_id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("Story {} not found", story_id))?;

    if story.project_id != project_id {
        return Err(anyhow!(
            "Story {} does not belong to project {}",
            story_id,
            project_id
        ));
    }

    let sequence_models = sequences::Entity::find()
        .filter(sequences::Column::StoryId.eq(story_id))
        .order_by_asc(sequences::Column::Id)
        .all(db)
        .await?;

    let dataset_ids = parse_story_dataset_ids(&story.enabled_dataset_ids);
    let datasets = if dataset_ids.is_empty() {
        Vec::new()
    } else {
        data_sets::Entity::find()
            .filter(data_sets::Column::Id.is_in(dataset_ids.clone()))
            .all(db)
            .await?
    };

    let dataset_contexts = build_dataset_contexts(&datasets);
    let story_layer_config = parse_story_layer_config(&story.layer_config);
    let layer_overrides = build_story_layer_overrides(&story_layer_config);

    let project_layers_list = project_layers::Entity::find()
        .filter(project_layers::Column::ProjectId.eq(project_id))
        .all(db)
        .await?;
    let project_layer_map = build_project_layer_map(&project_layers_list);

    let story_summary = SequenceStorySummary {
        id: story.id,
        name: story.name.clone(),
        description: story.description.clone(),
        tags: serde_json::from_str::<Vec<String>>(&story.tags).unwrap_or_default(),
    };

    let (sequence_entries, participants_map) = build_sequence_entries(
        &sequence_models,
        &dataset_contexts,
        &layer_overrides,
        &project_layer_map,
        true,
    );

    let mut participants: Vec<SequenceParticipant> = participants_map.values().cloned().collect();
    let graph_data = build_sequence_graph_data(&dataset_contexts);
    if participants.is_empty() && !graph_data.nodes.is_empty() {
        participants = graph_data
            .nodes
            .iter()
            .map(|node| SequenceParticipant {
                alias: format!("p{}_{}", node.dataset_id, sanitize_identifier(&node.id)),
                id: node.id.clone(),
                label: node.label.clone(),
                dataset_id: node.dataset_id,
                dataset_name: node.dataset_name.clone(),
                layer: node.layer.clone(),
                layer_color: None,
                partition_id: node.belongs_to.clone(),
                partition_label: node.partition_label.clone(),
                partition_layer_color: None,
            })
            .collect();
    }

    let mut layer_entries: Vec<SequenceLayer> = graph_data.layers.clone();
    layer_entries.extend(build_project_layer_entries(&project_layers_list));

    Ok(SequenceStoryContext {
        story: story_summary,
        sequences: sequence_entries,
        participants,
        graph_data,
        layers: layer_entries,
    })
}

pub fn apply_render_config(
    base: &SequenceStoryContext,
    render_config: SequenceRenderConfigResolved,
) -> SequenceRenderContext {
    let sequences = if render_config.render_all_sequences {
        base.sequences.clone()
    } else {
        let allowed: HashSet<i32> = render_config.enabled_sequence_ids.iter().copied().collect();
        base.sequences
            .iter()
            .cloned()
            .filter(|seq| allowed.contains(&seq.id))
            .collect()
    };

    let mut participants = base.participants.clone();
    if !render_config.use_story_layers {
        for participant in participants.iter_mut() {
            participant.layer_color = None;
        }
    }

    let participant_groups =
        build_participant_groups(&base.story.name, &participants, &render_config);
    let first_participant_alias = participants.first().map(|p| p.alias.clone());
    let last_participant_alias = if participants.len() > 1 {
        participants.last().map(|p| p.alias.clone())
    } else {
        first_participant_alias.clone()
    };

    SequenceRenderContext {
        config: render_config.clone(),
        story: base.story.clone(),
        stories: vec![SequenceStoryWithSequences {
            story: base.story.clone(),
            sequences: sequences.clone(),
        }],
        sequences,
        participants,
        participant_groups,
        graph_data: base.graph_data.clone(),
        layers: base.layers.clone(),
        first_participant_alias,
        last_participant_alias,
    }
}

pub fn parse_story_dataset_ids(value: &str) -> Vec<i32> {
    serde_json::from_str(value).unwrap_or_default()
}

pub fn parse_story_layer_config(value: &str) -> Vec<StoryLayerConfig> {
    serde_json::from_str(value).unwrap_or_default()
}

fn build_dataset_contexts(models: &[data_sets::Model]) -> HashMap<i32, ParsedDatasetContext> {
    models
        .iter()
        .map(|model| {
            let parsed = parse_dataset_context(model);
            (parsed.dataset_id, parsed)
        })
        .collect()
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ParsedDatasetContext {
    pub dataset_id: i32,
    pub name: String,
    pub nodes: IndexMap<String, SequenceNode>,
    pub edges: IndexMap<String, SequenceEdge>,
    pub layers: Vec<SequenceLayer>,
    pub partitions: HashMap<String, String>,
}

#[derive(Clone)]
pub struct ProjectLayerStyle {
    pub background: String,
    pub text: String,
    pub border: String,
    pub dataset_id: Option<i32>,
    pub name: String,
}

fn parse_dataset_context(model: &data_sets::Model) -> ParsedDatasetContext {
    let parsed: Value = serde_json::from_str(&model.graph_json).unwrap_or_else(|_| json!({}));
    let nodes = parsed
        .get("nodes")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let edges = parsed
        .get("edges")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let layers = parsed
        .get("layers")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut nodes_map: IndexMap<String, SequenceNode> = IndexMap::new();
    let mut partitions: HashMap<String, String> = HashMap::new();

    for node in nodes {
        if let Some(id) = value_to_string(node.get("id")) {
            let belongs_to = value_to_string(node.get("belongs_to"))
                .or_else(|| value_to_string(node.get("belongsTo")))
                .or_else(|| {
                    node.get("attrs")
                        .and_then(|attrs| value_to_string(attrs.get("belongs_to")))
                });
            let label = extract_label(&node, &id);
            let layer = value_to_string(node.get("layer")).or_else(|| {
                node.get("attrs")
                    .and_then(|attrs| value_to_string(attrs.get("layer")))
            });
            let is_partition = value_to_bool(node.get("is_partition"))
                || node
                    .get("attrs")
                    .and_then(|attrs| attrs.get("is_partition"))
                    .map(|val| value_to_bool(Some(val)))
                    .unwrap_or(false);

            if is_partition {
                partitions.insert(id.clone(), label.clone());
            }

            nodes_map.insert(
                id.clone(),
                SequenceNode {
                    id,
                    label,
                    layer,
                    dataset_id: model.id,
                    dataset_name: model.name.clone(),
                    belongs_to,
                    partition_label: None,
                    is_partition,
                },
            );
        }
    }

    for node in nodes_map.values_mut() {
        if let Some(parent) = node.belongs_to.as_ref() {
            if let Some(label) = partitions.get(parent) {
                node.partition_label = Some(label.clone());
            }
        }
    }

    let mut edges_map: IndexMap<String, SequenceEdge> = IndexMap::new();
    for edge in edges {
        let edge_id = value_to_string(edge.get("id")).unwrap_or_else(|| {
            format!(
                "{}:{}",
                value_to_string(edge.get("source")).unwrap_or_default(),
                value_to_string(edge.get("target")).unwrap_or_default()
            )
        });
        let source = value_to_string(edge.get("source")).unwrap_or_default();
        let target = value_to_string(edge.get("target")).unwrap_or_default();
        if source.is_empty() || target.is_empty() {
            continue;
        }
        let label = extract_label(&edge, &edge_id);
        edges_map.insert(
            edge_id.clone(),
            SequenceEdge {
                id: edge_id,
                source,
                target,
                label,
                dataset_id: model.id,
                dataset_name: model.name.clone(),
            },
        );
    }

    let mut layer_entries = Vec::new();
    for layer in layers {
        if let Some(id) = value_to_string(layer.get("id")) {
            let name = value_to_string(layer.get("name")).unwrap_or_else(|| id.clone());
            let background = normalize_color(value_to_string(layer.get("background_color")));
            let text = normalize_color(value_to_string(layer.get("text_color")));
            let border = normalize_color(value_to_string(layer.get("border_color")));
            layer_entries.push(SequenceLayer {
                id,
                name,
                background_color: background,
                text_color: text,
                border_color: border,
                dataset_id: Some(model.id),
                dataset_name: Some(model.name.clone()),
            });
        }
    }

    ParsedDatasetContext {
        dataset_id: model.id,
        name: model.name.clone(),
        nodes: nodes_map,
        edges: edges_map,
        layers: layer_entries,
        partitions,
    }
}

fn normalize_color(value: Option<String>) -> Option<String> {
    value.map(|color| {
        if color.starts_with('#') {
            color
        } else {
            format!("#{}", color)
        }
    })
}

fn extract_label(node: &Value, default_id: &str) -> String {
    value_to_string(node.get("label"))
        .or_else(|| {
            node.get("attrs")
                .and_then(|attrs| value_to_string(attrs.get("label")))
        })
        .unwrap_or_else(|| default_id.to_string())
}

fn build_project_layer_entries(layers: &[project_layers::Model]) -> Vec<SequenceLayer> {
    layers
        .iter()
        .map(|layer| SequenceLayer {
            id: layer.layer_id.clone(),
            name: layer.name.clone(),
            background_color: Some(format!(
                "#{}",
                layer.background_color.trim_start_matches('#')
            )),
            text_color: Some(format!("#{}", layer.text_color.trim_start_matches('#'))),
            border_color: Some(format!("#{}", layer.border_color.trim_start_matches('#'))),
            dataset_id: layer.source_dataset_id,
            dataset_name: None,
        })
        .collect()
}

fn build_sequence_graph_data(
    dataset_contexts: &HashMap<i32, ParsedDatasetContext>,
) -> SequenceGraphData {
    let mut datasets = Vec::new();
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut layers = Vec::new();

    for ctx in dataset_contexts.values() {
        nodes.extend(ctx.nodes.values().cloned());
        edges.extend(ctx.edges.values().cloned());
        layers.extend(ctx.layers.clone());

        datasets.push(SequenceDataset {
            id: ctx.dataset_id,
            name: ctx.name.clone(),
            nodes: ctx.nodes.values().cloned().collect(),
            edges: ctx.edges.values().cloned().collect(),
            layers: ctx.layers.clone(),
        });
    }

    SequenceGraphData {
        nodes,
        edges,
        layers: layers.clone(),
        datasets,
    }
}

fn build_sequence_entries(
    sequences: &[sequences::Model],
    dataset_contexts: &HashMap<i32, ParsedDatasetContext>,
    layer_overrides: &HashMap<Option<i32>, LayerSourceStyle>,
    project_layer_map: &HashMap<(Option<i32>, String), ProjectLayerStyle>,
    use_story_layers: bool,
) -> (
    Vec<SequenceRender>,
    IndexMap<(i32, String), SequenceParticipant>,
) {
    let mut participants: IndexMap<(i32, String), SequenceParticipant> = IndexMap::new();
    let mut alias_set: HashSet<String> = HashSet::new();
    let mut results = Vec::new();

    for sequence in sequences {
        let edge_order: Vec<SequenceEdgeRef> =
            serde_json::from_str(&sequence.edge_order).unwrap_or_default();
        let mut steps = Vec::new();

        for (index, edge_ref) in edge_order.into_iter().enumerate() {
            let dataset = match dataset_contexts.get(&edge_ref.dataset_id) {
                Some(ctx) => ctx,
                None => {
                    continue;
                }
            };

            let edge = match dataset.edges.get(&edge_ref.edge_id) {
                Some(edge) => edge,
                None => continue,
            };

            let source_node = match dataset.nodes.get(&edge.source) {
                Some(node) => node,
                None => continue,
            };
            let target_node = match dataset.nodes.get(&edge.target) {
                Some(node) => node,
                None => continue,
            };

            let source_ref = {
                let participant = get_or_create_participant(
                    dataset,
                    source_node,
                    &mut participants,
                    &mut alias_set,
                    layer_overrides,
                    project_layer_map,
                    use_story_layers,
                );
                build_participant_ref(participant)
            };
            let target_ref = {
                let participant = get_or_create_participant(
                    dataset,
                    target_node,
                    &mut participants,
                    &mut alias_set,
                    layer_overrides,
                    project_layer_map,
                    use_story_layers,
                );
                build_participant_ref(participant)
            };

            let note_position = edge_ref.note_position.as_ref().map(note_position_to_str);

            steps.push(SequenceStep {
                id: format!("seq{}_{}", sequence.id, index),
                dataset_id: dataset.dataset_id,
                dataset_name: dataset.name.clone(),
                source: source_ref,
                target: target_ref,
                label: edge.label.clone(),
                note: edge_ref.note.clone(),
                note_position,
            });
        }

        results.push(SequenceRender {
            id: sequence.id,
            name: sequence.name.clone(),
            description: sequence.description.clone(),
            steps,
        });
    }

    (results, participants)
}

fn get_or_create_participant<'a>(
    dataset: &ParsedDatasetContext,
    node: &SequenceNode,
    participants: &'a mut IndexMap<(i32, String), SequenceParticipant>,
    alias_set: &mut HashSet<String>,
    layer_overrides: &HashMap<Option<i32>, LayerSourceStyle>,
    project_layer_map: &HashMap<(Option<i32>, String), ProjectLayerStyle>,
    use_story_layers: bool,
) -> &'a SequenceParticipant {
    let key = (dataset.dataset_id, node.id.clone());
    if !participants.contains_key(&key) {
        let sanitized = sanitize_identifier(&node.id);
        let alias_base = if sanitized.is_empty() {
            format!("p{}", dataset.dataset_id)
        } else {
            sanitized
        };
        let alias = make_unique_alias(&alias_base, alias_set);
        let layer_color = resolve_layer_color(
            node.layer.as_ref(),
            dataset.dataset_id,
            use_story_layers,
            layer_overrides,
            project_layer_map,
        );
        let partition_layer_color = node
            .belongs_to
            .as_ref()
            .and_then(|parent_id| dataset.nodes.get(parent_id))
            .and_then(|parent_node| {
                resolve_layer_color(
                    parent_node.layer.as_ref(),
                    dataset.dataset_id,
                    use_story_layers,
                    layer_overrides,
                    project_layer_map,
                )
            });

        participants.insert(
            key.clone(),
            SequenceParticipant {
                alias,
                id: node.id.clone(),
                label: node.label.clone(),
                dataset_id: dataset.dataset_id,
                dataset_name: dataset.name.clone(),
                layer: node.layer.clone(),
                layer_color,
                partition_id: node.belongs_to.clone(),
                partition_label: node.partition_label.clone(),
                partition_layer_color,
            },
        );
    }
    participants.get(&key).unwrap()
}

fn build_participant_groups(
    story_name: &str,
    participants: &[SequenceParticipant],
    config: &SequenceRenderConfigResolved,
) -> Vec<SequenceParticipantGroup> {
    if participants.is_empty() {
        return Vec::new();
    }

    match config.contain_nodes.as_str() {
        "one" => {
            let mut groups: IndexMap<String, SequenceParticipantGroup> = IndexMap::new();
            for participant in participants {
                let key = participant
                    .partition_id
                    .clone()
                    .unwrap_or_else(|| format!("participant-{}", participant.alias));
                let label = participant
                    .partition_label
                    .clone()
                    .unwrap_or_else(|| participant.dataset_name.clone());
                let entry = groups
                    .entry(key.clone())
                    .or_insert_with(|| SequenceParticipantGroup {
                        id: key.clone(),
                        label: label.clone(),
                        color: participant
                            .partition_layer_color
                            .clone()
                            .or_else(|| participant.layer_color.clone()),
                        participants: Vec::new(),
                    });
                entry.participants.push(participant.clone());
            }
            groups.into_iter().map(|(_, group)| group).collect()
        }
        "all" => vec![SequenceParticipantGroup {
            id: "sequence-participants".to_string(),
            label: story_name.to_string(),
            color: participants
                .iter()
                .find_map(|participant| participant.layer_color.clone()),
            participants: participants.to_vec(),
        }],
        _ => Vec::new(),
    }
}

fn build_project_layer_map(
    layers: &[project_layers::Model],
) -> HashMap<(Option<i32>, String), ProjectLayerStyle> {
    let mut map = HashMap::new();
    for layer in layers {
        let key = (layer.source_dataset_id, layer.layer_id.clone());
        map.insert(
            key,
            ProjectLayerStyle {
                background: format!("#{}", layer.background_color.trim_start_matches('#')),
                text: format!("#{}", layer.text_color.trim_start_matches('#')),
                border: format!("#{}", layer.border_color.trim_start_matches('#')),
                dataset_id: layer.source_dataset_id,
                name: layer.name.clone(),
            },
        );
    }
    map
}

fn build_story_layer_overrides(
    configs: &[StoryLayerConfig],
) -> HashMap<Option<i32>, LayerSourceStyle> {
    configs
        .iter()
        .filter_map(|config| {
            let mode = match config.mode.to_lowercase().as_str() {
                "dark" => LayerSourceStyle::Dark,
                "light" => LayerSourceStyle::Light,
                _ => LayerSourceStyle::Default,
            };
            Some((config.source_dataset_id, mode))
        })
        .collect()
}

fn build_participant_ref(participant: &SequenceParticipant) -> SequenceParticipantRef {
    SequenceParticipantRef {
        alias: participant.alias.clone(),
        label: participant.label.clone(),
    }
}

fn note_position_to_str(position: &NotePosition) -> String {
    match position {
        NotePosition::Source => "source".to_string(),
        NotePosition::Target => "target".to_string(),
        NotePosition::Both => "both".to_string(),
    }
}

fn sanitize_identifier(input: &str) -> String {
    let mut output = String::new();
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            output.push(ch);
        } else if ch.is_ascii_whitespace() || ch == '-' {
            output.push('_');
        }
    }
    if output.is_empty() {
        "participant".to_string()
    } else {
        output
    }
}

fn make_unique_alias(base: &str, set: &mut HashSet<String>) -> String {
    let mut alias = base.to_string();
    let mut counter = 1;
    while set.contains(&alias) {
        alias = format!("{}_{}", base, counter);
        counter += 1;
    }
    set.insert(alias.clone());
    alias
}

fn resolve_layer_color(
    layer_id: Option<&String>,
    dataset_id: i32,
    use_story_layers: bool,
    overrides: &HashMap<Option<i32>, LayerSourceStyle>,
    project_layers: &HashMap<(Option<i32>, String), ProjectLayerStyle>,
) -> Option<String> {
    if !use_story_layers {
        return None;
    }
    let override_key = overrides
        .get(&Some(dataset_id))
        .or_else(|| overrides.get(&None));
    if let Some(mode) = override_key {
        let (background, _, _) = palette_for_mode(mode);
        return Some(background.to_string());
    }
    let layer_id = layer_id?;
    project_layers
        .get(&(Some(dataset_id), layer_id.clone()))
        .or_else(|| project_layers.get(&(None, layer_id.clone())))
        .map(|style| style.background.clone())
}

fn palette_for_mode(mode: &LayerSourceStyle) -> (&'static str, &'static str, &'static str) {
    match mode {
        LayerSourceStyle::Dark => ("#1f2933", "#f8fafc", "#94a3b8"),
        LayerSourceStyle::Light => ("#f7f7f8", "#0f172a", "#e2e8f0"),
        LayerSourceStyle::Default => ("#f7f7f8", "#0f172a", "#dddddd"),
    }
}

fn value_to_string(value: Option<&Value>) -> Option<String> {
    value.and_then(|val| match val {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    })
}

fn value_to_bool(value: Option<&Value>) -> bool {
    value
        .map(|val| match val {
            Value::Bool(b) => *b,
            Value::String(s) => s.eq_ignore_ascii_case("true"),
            Value::Number(n) => n.as_i64().map(|i| i != 0).unwrap_or(false),
            _ => false,
        })
        .unwrap_or(false)
}
