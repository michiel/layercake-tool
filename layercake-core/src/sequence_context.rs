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
use crate::plan::LayerSourceStyle;
use crate::sequence_types::{NotePosition, SequenceEdgeRef};
use crate::story_types::StoryLayerConfig;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SequenceStoryContext {
    pub story: SequenceStorySummary,
    pub sequences: Vec<SequenceRender>,
    pub participants: Vec<SequenceParticipant>,
    pub graph_data: SequenceGraphData,
    pub layers: Vec<SequenceLayer>,
    /// Non-fatal problems encountered while building the context (e.g. a
    /// sequence edge reference that didn't resolve). Surfaced so an empty or
    /// partial diagram is never silently reported as success.
    #[serde(default)]
    pub warnings: Vec<String>,
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

    let mut dataset_contexts = build_dataset_contexts(&datasets);

    // A story may also source from computed graphs (a GraphNode's output). Load
    // each enabled graph_data id as an additional context, keyed by its id so
    // sequence edge refs can target it via `datasetId`.
    let graph_ids = parse_story_graph_ids(&story.enabled_graph_ids);
    for graph_id in graph_ids {
        if let Some(ctx) = build_graph_data_context(db, graph_id).await? {
            dataset_contexts.insert(ctx.dataset_id, ctx);
        }
    }
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

    let (sequence_entries, participants_map, mut warnings) = build_sequence_entries(
        &sequence_models,
        &dataset_contexts,
        &layer_overrides,
        &project_layer_map,
        true,
    );

    // Participants are ONLY the nodes that appear as endpoints of the edges
    // referenced by the story's sequences (built in build_sequence_entries).
    // We intentionally do NOT fall back to "all nodes in the datasets" — a
    // story renders its selected sequence, not the entire graph. An empty
    // result means the sequences reference no resolvable edges, which should
    // render as an empty diagram, not a wall of unconnected boxes.
    let participants: Vec<SequenceParticipant> = participants_map.values().cloned().collect();
    let mut graph_data = build_sequence_graph_data(&dataset_contexts);
    graph_data.layers = build_project_layer_entries(&project_layers_list);

    let layer_entries = graph_data.layers.clone();

    if !sequence_models.is_empty() && participants.is_empty() {
        warnings.push(format!(
            "story '{}' (id {}): {} sequence(s) produced no participants — the diagram will be empty",
            story_summary.name,
            story_summary.id,
            sequence_models.len()
        ));
    }

    Ok(SequenceStoryContext {
        story: story_summary,
        sequences: sequence_entries,
        participants,
        graph_data,
        layers: layer_entries,
        warnings,
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

pub fn parse_story_graph_ids(value: &str) -> Vec<i32> {
    serde_json::from_str(value).unwrap_or_default()
}

/// Load a computed graph (graph_data + its nodes/edges) into the same
/// `ParsedDatasetContext` shape used for datasets, so a story can reference its
/// edges. Keyed by the graph_data id. Returns None if the graph doesn't exist.
async fn build_graph_data_context(
    db: &DatabaseConnection,
    graph_id: i32,
) -> Result<Option<ParsedDatasetContext>> {
    use crate::database::entities::{graph_data, graph_data_edges, graph_data_nodes};

    let graph = match graph_data::Entity::find_by_id(graph_id).one(db).await? {
        Some(g) => g,
        None => return Ok(None),
    };

    let node_models = graph_data_nodes::Entity::find()
        .filter(graph_data_nodes::Column::GraphDataId.eq(graph_id))
        .all(db)
        .await?;
    let edge_models = graph_data_edges::Entity::find()
        .filter(graph_data_edges::Column::GraphDataId.eq(graph_id))
        .all(db)
        .await?;

    let mut nodes: IndexMap<String, SequenceNode> = IndexMap::new();
    let mut partitions: HashMap<String, String> = HashMap::new();
    for n in &node_models {
        let label = n.label.clone().unwrap_or_else(|| n.external_id.clone());
        if n.is_partition {
            partitions.insert(n.external_id.clone(), label.clone());
        }
        nodes.insert(
            n.external_id.clone(),
            SequenceNode {
                id: n.external_id.clone(),
                label,
                layer: n.layer.clone(),
                dataset_id: graph.id,
                dataset_name: graph.name.clone(),
                belongs_to: n.belongs_to.clone(),
                partition_label: None,
                is_partition: n.is_partition,
            },
        );
    }
    for node in nodes.values_mut() {
        if let Some(parent) = node.belongs_to.as_ref() {
            if let Some(label) = partitions.get(parent) {
                node.partition_label = Some(label.clone());
            }
        }
    }

    let mut edges: IndexMap<String, SequenceEdge> = IndexMap::new();
    for e in &edge_models {
        let edge_id = if e.external_id.is_empty() {
            format!("{}:{}", e.source, e.target)
        } else {
            e.external_id.clone()
        };
        edges.insert(
            edge_id.clone(),
            SequenceEdge {
                id: edge_id,
                source: e.source.clone(),
                target: e.target.clone(),
                label: e.label.clone().unwrap_or_default(),
                comment: e.comment.clone().filter(|c| !c.trim().is_empty()),
                dataset_id: graph.id,
                dataset_name: graph.name.clone(),
            },
        );
    }

    Ok(Some(ParsedDatasetContext {
        dataset_id: graph.id,
        name: graph.name,
        nodes,
        edges,
        partitions,
    }))
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
    pub partitions: HashMap<String, String>,
}

#[derive(Clone)]
#[allow(dead_code)]
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
        let comment = value_to_string(edge.get("comment"))
            .or_else(|| value_to_string(edge.get("comments")))
            .or_else(|| {
                edge.get("attrs")
                    .and_then(|attrs| value_to_string(attrs.get("comment")))
            })
            .filter(|c| !c.trim().is_empty());
        edges_map.insert(
            edge_id.clone(),
            SequenceEdge {
                id: edge_id,
                source,
                target,
                label,
                comment,
                dataset_id: model.id,
                dataset_name: model.name.clone(),
            },
        );
    }

    ParsedDatasetContext {
        dataset_id: model.id,
        name: model.name.clone(),
        nodes: nodes_map,
        edges: edges_map,
        partitions,
    }
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

    for ctx in dataset_contexts.values() {
        nodes.extend(ctx.nodes.values().cloned());
        edges.extend(ctx.edges.values().cloned());

        datasets.push(SequenceDataset {
            id: ctx.dataset_id,
            name: ctx.name.clone(),
            nodes: ctx.nodes.values().cloned().collect(),
            edges: ctx.edges.values().cloned().collect(),
            layers: Vec::new(),
        });
    }

    SequenceGraphData {
        nodes,
        edges,
        layers: Vec::new(),
        datasets,
    }
}

/// Resolve a sequence's stored `edge_id` against a dataset's edges.
///
/// The stored id is whatever the UI captured from the dataset's graph JSON
/// (the raw edge `id`). If that direct lookup misses — e.g. the graph JSON had
/// no edge `id`, so the backend keyed the edge by its `source:target` composite
/// while the UI stored something else — fall back to matching by the
/// `source:target` composite so the sequence still resolves.
fn resolve_edge<'a>(dataset: &'a ParsedDatasetContext, edge_id: &str) -> Option<&'a SequenceEdge> {
    if let Some(edge) = dataset.edges.get(edge_id) {
        return Some(edge);
    }
    // Fallback 1: the stored id is a "source:target" composite.
    if let Some((source, target)) = edge_id.split_once(':') {
        if let Some(edge) = dataset
            .edges
            .values()
            .find(|e| e.source == source && e.target == target)
        {
            return Some(edge);
        }
    }
    None
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
    Vec<String>,
) {
    let mut participants: IndexMap<(i32, String), SequenceParticipant> = IndexMap::new();
    let mut alias_set: HashSet<String> = HashSet::new();
    let mut results = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    let mut all_nodes: HashMap<String, &SequenceNode> = HashMap::new();
    for ctx in dataset_contexts.values() {
        for (node_id, node) in &ctx.nodes {
            all_nodes.insert(node_id.clone(), node);
        }
    }

    for sequence in sequences {
        let edge_order: Vec<SequenceEdgeRef> = match serde_json::from_str(&sequence.edge_order) {
            Ok(order) => order,
            Err(e) => {
                warnings.push(format!(
                    "sequence '{}' (id {}): could not parse edge_order ({}); it will render empty",
                    sequence.name, sequence.id, e
                ));
                Vec::new()
            }
        };
        let requested_steps = edge_order.len();
        let mut steps = Vec::new();

        for (index, edge_ref) in edge_order.into_iter().enumerate() {
            let dataset = match dataset_contexts.get(&edge_ref.dataset_id) {
                Some(ctx) => ctx,
                None => {
                    warnings.push(format!(
                        "sequence '{}' step {}: dataset {} is not enabled for this story",
                        sequence.name, index, edge_ref.dataset_id
                    ));
                    continue;
                }
            };

            let edge = match resolve_edge(dataset, &edge_ref.edge_id) {
                Some(edge) => edge,
                None => {
                    warnings.push(format!(
                        "sequence '{}' step {}: edge '{}' not found in dataset {}",
                        sequence.name, index, edge_ref.edge_id, edge_ref.dataset_id
                    ));
                    continue;
                }
            };

            let source_node = match all_nodes.get(&edge.source) {
                Some(node) => node,
                None => {
                    warnings.push(format!(
                        "sequence '{}' step {}: source node '{}' of edge '{}' not found",
                        sequence.name, index, edge.source, edge_ref.edge_id
                    ));
                    continue;
                }
            };
            let source_dataset = dataset_contexts.get(&source_node.dataset_id).unwrap();

            let target_node = match all_nodes.get(&edge.target) {
                Some(node) => node,
                None => {
                    warnings.push(format!(
                        "sequence '{}' step {}: target node '{}' of edge '{}' not found",
                        sequence.name, index, edge.target, edge_ref.edge_id
                    ));
                    continue;
                }
            };
            let target_dataset = dataset_contexts.get(&target_node.dataset_id).unwrap();

            let source_ref = {
                let participant = get_or_create_participant(
                    source_dataset,
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
                    target_dataset,
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

            // Surface the edge's comment alongside its label on the message,
            // matching the interactive preview ("label: comment").
            let label = match &edge.comment {
                Some(comment) if !edge.label.is_empty() => {
                    format!("{}: {}", edge.label, comment)
                }
                Some(comment) => comment.clone(),
                None => edge.label.clone(),
            };

            steps.push(SequenceStep {
                id: format!("seq{}_{}", sequence.id, index),
                dataset_id: dataset.dataset_id,
                dataset_name: dataset.name.clone(),
                source: source_ref,
                target: target_ref,
                label,
                note: edge_ref.note.clone(),
                note_position,
            });
        }

        // A sequence that requested steps but resolved none is the classic
        // "green but empty" case — make it loud.
        if requested_steps > 0 && steps.is_empty() {
            warnings.push(format!(
                "sequence '{}' (id {}): all {} step(s) were skipped; it will render empty",
                sequence.name, sequence.id, requested_steps
            ));
        }

        results.push(SequenceRender {
            id: sequence.id,
            name: sequence.name.clone(),
            description: sequence.description.clone(),
            steps,
        });
    }

    (results, participants, warnings)
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

#[cfg(test)]
mod story_participant_tests {
    use super::*;

    fn node(id: &str, label: &str) -> SequenceNode {
        SequenceNode { id: id.into(), label: label.into(), layer: None, dataset_id: 1, dataset_name: "d".into(), belongs_to: None, partition_label: None, is_partition: false }
    }
    fn edge(id: &str, s: &str, t: &str, label: &str) -> SequenceEdge {
        SequenceEdge { id: id.into(), source: s.into(), target: t.into(), label: label.into(), comment: None, dataset_id: 1, dataset_name: "d".into() }
    }
    fn edge_c(id: &str, s: &str, t: &str, label: &str, comment: &str) -> SequenceEdge {
        SequenceEdge { id: id.into(), source: s.into(), target: t.into(), label: label.into(), comment: Some(comment.into()), dataset_id: 1, dataset_name: "d".into() }
    }
    fn ctx_with(edges: Vec<SequenceEdge>) -> HashMap<i32, ParsedDatasetContext> {
        let mut nodes = IndexMap::new();
        nodes.insert("n1".to_string(), node("n1","Alice"));
        nodes.insert("n2".to_string(), node("n2","Bob"));
        nodes.insert("n3".to_string(), node("n3","Unused")); // should NOT appear as participant
        let mut em = IndexMap::new();
        for e in edges { em.insert(e.id.clone(), e); }
        let mut m = HashMap::new();
        m.insert(1, ParsedDatasetContext { dataset_id: 1, name: "d".into(), nodes, edges: em, partitions: HashMap::new() });
        m
    }
    fn seq(edge_ids: &[&str]) -> sequences::Model {
        let order: Vec<serde_json::Value> = edge_ids.iter().map(|id| json!({"datasetId":1,"edgeId":id,"note":"hi"})).collect();
        sequences::Model {
            id: 1, story_id: 1, name: "Seq".into(), description: None,
            enabled_dataset_ids: "[1]".into(),
            edge_order: serde_json::to_string(&order).unwrap(),
            created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn only_story_edges_become_participants_and_steps() {
        let contexts = ctx_with(vec![edge("e1","n1","n2","calls")]);
        let sequences = vec![seq(&["e1"])];
        let (results, participants, _warnings) = build_sequence_entries(&sequences, &contexts, &HashMap::new(), &HashMap::new(), true);
        // Exactly 2 participants (n1,n2) — NOT n3.
        assert_eq!(participants.len(), 2, "participants: {:?}", participants.keys().collect::<Vec<_>>());
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].steps.len(), 1, "expected 1 step");
        assert_eq!(results[0].steps[0].label, "calls");
        assert_eq!(results[0].steps[0].note.as_deref(), Some("hi"));
    }

    #[test]
    fn resolves_edge_by_source_target_when_stored_id_is_composite() {
        // Dataset edges keyed "n1:n2" (graph JSON had no edge id), and the story
        // stored the same composite. The source:target fallback must resolve it
        // so we still get the story's participants/steps — not zero.
        let contexts = ctx_with(vec![edge("n1:n2", "n1", "n2", "calls")]);
        let sequences = vec![seq(&["n1:n2"])];
        let (results, participants, _warnings) =
            build_sequence_entries(&sequences, &contexts, &HashMap::new(), &HashMap::new(), true);
        assert_eq!(participants.len(), 2);
        assert_eq!(results[0].steps.len(), 1);
        assert_eq!(results[0].steps[0].label, "calls");
    }

    #[test]
    fn resolves_edge_when_stored_composite_but_dataset_has_real_id() {
        // Dataset edge has a real id "e1" but the story stored "n1:n2" (composite).
        // The fallback matches by endpoints.
        let contexts = ctx_with(vec![edge("e1", "n1", "n2", "calls")]);
        let sequences = vec![seq(&["n1:n2"])];
        let (results, participants, _warnings) =
            build_sequence_entries(&sequences, &contexts, &HashMap::new(), &HashMap::new(), true);
        assert_eq!(participants.len(), 2);
        assert_eq!(results[0].steps.len(), 1);
    }

    #[test]
    fn edge_comment_is_appended_to_message_label() {
        let contexts = ctx_with(vec![edge_c("e1", "n1", "n2", "calls", "async")]);
        let sequences = vec![seq(&["e1"])];
        let (results, _, _) =
            build_sequence_entries(&sequences, &contexts, &HashMap::new(), &HashMap::new(), true);
        assert_eq!(results[0].steps[0].label, "calls: async");
    }

    #[test]
    fn unresolvable_edge_yields_no_participants_not_all_nodes() {
        // A truly missing edge reference must NOT fall back to dumping every node
        // as a participant — it renders empty.
        let contexts = ctx_with(vec![edge("e1", "n1", "n2", "calls")]);
        let sequences = vec![seq(&["does-not-exist"])];
        let (results, participants, warnings) =
            build_sequence_entries(&sequences, &contexts, &HashMap::new(), &HashMap::new(), true);
        assert_eq!(participants.len(), 0, "must not include unrelated nodes");
        assert_eq!(results[0].steps.len(), 0);
        // ...and it must be surfaced as a warning, not silently green.
        assert!(
            warnings.iter().any(|w| w.contains("does-not-exist")),
            "expected an unresolved-edge warning, got: {warnings:?}"
        );
        assert!(
            warnings.iter().any(|w| w.contains("all 1 step(s) were skipped")),
            "expected an empty-sequence warning, got: {warnings:?}"
        );
    }
}

#[cfg(test)]
mod graph_source_tests {
    use super::*;
    use crate::database::entities::{
        graph_data, graph_data_edges, graph_data_nodes, projects, stories,
    };
    use crate::database::test_utils::setup_test_db;
    use chrono::Utc;
    use sea_orm::{ActiveModelTrait, Set};

    #[tokio::test]
    async fn story_sources_edges_from_a_computed_graph() {
        let db = setup_test_db().await;
        projects::ActiveModel {
            id: Set(1), name: Set("P".into()), description: Set(None), tags: Set("[]".into()),
            import_export_path: Set(None), created_at: Set(Utc::now().into()), updated_at: Set(Utc::now().into()),
        }.insert(&db).await.unwrap();

        // A computed graph (graph_data id 500) with 2 nodes + 1 edge.
        graph_data::ActiveModel {
            id: Set(500), project_id: Set(1), name: Set("merged".into()),
            source_type: Set("computed".into()), dag_node_id: Set(Some("gnode".into())),
            last_edit_sequence: Set(0), has_pending_edits: Set(false),
            node_count: Set(2), edge_count: Set(1), status: Set("active".into()),
            created_at: Set(Utc::now()), updated_at: Set(Utc::now()),
            ..Default::default()
        }.insert(&db).await.unwrap();
        for (ext, label) in [("a", "Alice"), ("b", "Bob")] {
            graph_data_nodes::ActiveModel {
                graph_data_id: Set(500), external_id: Set(ext.into()), label: Set(Some(label.into())),
                layer: Set(None), weight: Set(Some(1.0)), is_partition: Set(false),
                belongs_to: Set(None), comment: Set(None), source_dataset_id: Set(None),
                attributes: Set(None), created_at: Set(Utc::now()),
                ..Default::default()
            }.insert(&db).await.unwrap();
        }
        graph_data_edges::ActiveModel {
            graph_data_id: Set(500), external_id: Set("e1".into()), source: Set("a".into()),
            target: Set("b".into()), label: Set(Some("calls".into())), layer: Set(None),
            weight: Set(Some(1.0)), comment: Set(None), source_dataset_id: Set(None),
            attributes: Set(None), created_at: Set(Utc::now()),
            ..Default::default()
        }.insert(&db).await.unwrap();

        // Story sources from the computed graph (not a dataset), sequence uses e1.
        stories::ActiveModel {
            id: Set(1), project_id: Set(1), name: Set("S".into()), description: Set(None),
            tags: Set("[]".into()), enabled_dataset_ids: Set("[]".into()),
            enabled_graph_ids: Set("[500]".into()), layer_config: Set("[]".into()),
            created_at: Set(Utc::now().into()), updated_at: Set(Utc::now().into()),
        }.insert(&db).await.unwrap();
        sequences::ActiveModel {
            id: Set(1), story_id: Set(1), name: Set("seq".into()), description: Set(None),
            enabled_dataset_ids: Set("[500]".into()),
            edge_order: Set(r#"[{"datasetId":500,"edgeId":"e1"}]"#.into()),
            created_at: Set(Utc::now()), updated_at: Set(Utc::now()),
        }.insert(&db).await.unwrap();

        let ctx = build_story_context(&db, 1, 1).await.unwrap();
        assert_eq!(ctx.participants.len(), 2, "expected Alice+Bob from computed graph");
        assert_eq!(ctx.sequences[0].steps.len(), 1);
        assert_eq!(ctx.sequences[0].steps[0].label, "calls");
        assert!(ctx.warnings.is_empty(), "warnings: {:?}", ctx.warnings);
    }
}
