# Transformation System Integration Analysis

## Current Transformation System Overview

The existing layercake transformation system is robust and well-designed, located primarily in:
- `src/plan_execution.rs` - Main transformation pipeline
- `src/graph.rs` - Core transformation methods
- `src/export/` - Multiple output format renderers

### Key Strengths of Current System

1. **Comprehensive Graph Operations**: `src/graph.rs:258-466`
   - `modify_graph_limit_partition_depth()` - Hierarchical depth limiting
   - `modify_graph_limit_partition_width()` - Width-based aggregation
   - `invert_graph()` - Node/edge inversion
   - `aggregate_edges()` - Duplicate edge handling
   - Label transformation methods (truncate, newlines)

2. **Robust Export Pipeline**: `src/export/mod.rs:13-48`
   - Template-based rendering with Handlebars
   - Standardized context creation
   - Support for 11+ output formats (DOT, GML, JSON, PlantUML, etc.)
   - Custom template support

3. **Configuration Integration**: `src/plan_execution.rs:112-184`
   - Maps `ExportProfileGraphConfig` to transformation operations
   - Conditional application based on configuration values
   - Comprehensive logging and error handling

## Integration Strategy for Plan DAG

### 1. TransformNode Mapping

The new TransformNode must integrate seamlessly with existing transformations:

```rust
// New TransformNode execution that leverages existing system
impl TransformNode {
    pub async fn execute(
        &self,
        input_graph: Graph,
        graph_service: &GraphService,
    ) -> Result<Graph, TransformError> {
        let mut transformed_graph = input_graph;

        // Convert TransformNodeConfig to existing GraphConfig
        let graph_config = self.config.to_legacy_graph_config();

        // Use existing transformation pipeline
        crate::plan_execution::apply_graph_transformations(
            &mut transformed_graph,
            &graph_config,
        )?;

        // Store result in database via graph_service
        let stored_graph = graph_service
            .create_graph_from_data(transformed_graph, self.config.output_graph_ref.clone())
            .await?;

        Ok(stored_graph)
    }
}

// Conversion from new config to existing config
impl TransformNodeConfig {
    pub fn to_legacy_graph_config(&self) -> crate::plan::GraphConfig {
        crate::plan::GraphConfig {
            generate_hierarchy: self.transform_config.generate_hierarchy.unwrap_or(false),
            max_partition_depth: self.transform_config.max_partition_depth.unwrap_or(0),
            max_partition_width: self.transform_config.max_partition_width.unwrap_or(0),
            invert_graph: self.transform_config.invert_graph.unwrap_or(false),
            node_label_max_length: self.transform_config.node_label_max_length.unwrap_or(0),
            node_label_insert_newlines_at: self.transform_config.node_label_insert_newlines_at.unwrap_or(0),
            edge_label_max_length: self.transform_config.edge_label_max_length.unwrap_or(0),
            edge_label_insert_newlines_at: self.transform_config.edge_label_insert_newlines_at.unwrap_or(0),
        }
    }
}
```

### 2. OutputNode Integration

OutputNode leverages the existing export system:

```rust
impl OutputNode {
    pub async fn execute(
        &self,
        source_graph: Graph,
        export_service: &ExportService,
    ) -> Result<String, OutputError> {
        // Convert OutputNodeConfig to existing ExportProfileItem
        let export_profile = ExportProfileItem {
            filename: self.config.output_path.clone(),
            exporter: self.config.render_target.to_legacy_export_type(),
            render_config: self.config.render_config.as_ref().map(|rc| rc.to_legacy()),
            graph_config: self.config.graph_config.as_ref().map(|gc| gc.to_legacy()),
        };

        // Use existing export pipeline
        let result = crate::plan_execution::export_graph(&source_graph, &export_profile)?;

        // Track export in database
        export_service.record_export(
            source_graph.id,
            &self.config.output_path,
            &self.config.render_target.to_string(),
        ).await?;

        Ok(self.config.output_path.clone())
    }
}

// Convert new render target to existing export type
impl RenderTarget {
    pub fn to_legacy_export_type(&self) -> crate::plan::ExportFileType {
        match self {
            RenderTarget::DOT => crate::plan::ExportFileType::DOT,
            RenderTarget::GML => crate::plan::ExportFileType::GML,
            RenderTarget::JSON => crate::plan::ExportFileType::JSON,
            RenderTarget::PlantUML => crate::plan::ExportFileType::PlantUML,
            RenderTarget::Mermaid => crate::plan::ExportFileType::Mermaid,
            RenderTarget::CSVNodes => crate::plan::ExportFileType::CSVNodes,
            RenderTarget::CSVEdges => crate::plan::ExportFileType::CSVEdges,
            RenderTarget::CSVMatrix => crate::plan::ExportFileType::CSVMatrix,
            RenderTarget::JSGraph => crate::plan::ExportFileType::JSGraph,
            RenderTarget::Custom => crate::plan::ExportFileType::Custom(
                crate::plan::CustomExportProfile {
                    template: "".to_string(), // Will be populated from config
                    partials: None,
                }
            ),
        }
    }
}
```

### 3. Enhanced Plan Execution Engine

Create new execution engine that integrates with existing system:

```rust
pub struct PlanDAGExecutor {
    graph_service: Arc<GraphService>,
    import_service: Arc<ImportService>,
    export_service: Arc<ExportService>,
    db: DatabaseConnection,
}

impl PlanDAGExecutor {
    pub async fn execute_plan_dag(
        &self,
        project_id: i32,
        plan_dag: &PlanDAG,
    ) -> Result<ExecutionResult, ExecutionError> {
        // Get execution order (topological sort)
        let execution_order = plan_dag.get_execution_order()?;

        let mut execution_context = ExecutionContext::new();
        let mut executed_nodes = HashMap::new();

        for node_id in execution_order {
            let node = plan_dag.get_node(&node_id)?;

            tracing::info!("Executing node: {} ({})", node.metadata.label, node.node_type);

            let result = match &node.node_type {
                PlanDAGNodeType::InputNode => {
                    self.execute_input_node(node, &mut execution_context).await?
                },
                PlanDAGNodeType::GraphNode => {
                    self.execute_graph_node(node, &mut execution_context).await?
                },
                PlanDAGNodeType::TransformNode => {
                    self.execute_transform_node(node, &mut execution_context).await?
                },
                PlanDAGNodeType::MergeNode => {
                    self.execute_merge_node(node, &mut execution_context).await?
                },
                PlanDAGNodeType::CopyNode => {
                    self.execute_copy_node(node, &mut execution_context).await?
                },
                PlanDAGNodeType::OutputNode => {
                    self.execute_output_node(node, &mut execution_context).await?
                },
            };

            executed_nodes.insert(node_id, result);
        }

        Ok(ExecutionResult {
            executed_nodes,
            execution_time: execution_context.elapsed(),
            errors: execution_context.errors,
        })
    }

    async fn execute_transform_node(
        &self,
        node: &PlanDAGNode,
        context: &mut ExecutionContext,
    ) -> Result<ExecutionNodeResult, ExecutionError> {
        let config: TransformNodeConfig = serde_json::from_value(node.config.clone())?;

        // Get input graph from context
        let input_graph = context.get_graph(&config.input_graph_ref)?;

        // Apply transformations using existing system
        let mut transformed_graph = input_graph.clone();
        let graph_config = config.to_legacy_graph_config();

        // Use existing transformation pipeline
        crate::plan_execution::apply_graph_transformations(
            &mut transformed_graph,
            &graph_config,
        )?;

        // Store result in context for downstream nodes
        context.store_graph(config.output_graph_ref.clone(), transformed_graph.clone());

        // Optionally persist to database
        if config.persist_intermediate_results.unwrap_or(false) {
            let stored_graph = self.graph_service
                .create_graph_from_data(transformed_graph, Some(config.output_graph_ref.clone()))
                .await?;

            context.store_graph_reference(config.output_graph_ref.clone(), stored_graph.id);
        }

        Ok(ExecutionNodeResult::Graph(transformed_graph))
    }

    async fn execute_output_node(
        &self,
        node: &PlanDAGNode,
        context: &mut ExecutionContext,
    ) -> Result<ExecutionNodeResult, ExecutionError> {
        let config: OutputNodeConfig = serde_json::from_value(node.config.clone())?;

        // Get source graph from context
        let source_graph = context.get_graph(&config.source_graph_ref)?;

        // Create export profile from config
        let export_profile = ExportProfileItem {
            filename: config.output_path.clone(),
            exporter: config.render_target.to_legacy_export_type(),
            render_config: config.render_config.as_ref().map(|rc| rc.to_legacy()),
            graph_config: config.graph_config.as_ref().map(|gc| gc.to_legacy()),
        };

        // Use existing export system
        crate::plan_execution::export_graph(&source_graph, &export_profile)?;

        // Record export in database
        self.export_service.record_export(
            source_graph.id.unwrap_or(0), // Handle in-memory graphs
            &config.output_path,
            &config.render_target.to_string(),
        ).await?;

        Ok(ExecutionNodeResult::Export(config.output_path))
    }
}

pub struct ExecutionContext {
    graphs: HashMap<String, Graph>,          // In-memory graphs by reference
    graph_ids: HashMap<String, i32>,         // Database graph IDs by reference
    start_time: std::time::Instant,
    pub errors: Vec<ExecutionError>,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            graphs: HashMap::new(),
            graph_ids: HashMap::new(),
            start_time: std::time::Instant::now(),
            errors: Vec::new(),
        }
    }

    pub fn store_graph(&mut self, reference: String, graph: Graph) {
        self.graphs.insert(reference, graph);
    }

    pub fn get_graph(&self, reference: &str) -> Result<&Graph, ExecutionError> {
        self.graphs.get(reference)
            .ok_or_else(|| ExecutionError::GraphNotFound(reference.to_string()))
    }

    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
}
```

### 4. Backward Compatibility Layer

Maintain compatibility with existing YAML plans:

```rust
pub struct LegacyPlanAdapter;

impl LegacyPlanAdapter {
    /// Convert existing YAML Plan to new Plan DAG format
    pub fn yaml_plan_to_dag(plan: &Plan) -> PlanDAG {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut node_counter = 0;

        // Convert import profiles to InputNodes
        for import_profile in &plan.import.profiles {
            nodes.push(PlanDAGNode {
                id: format!("input_{}", node_counter),
                node_type: PlanDAGNodeType::InputNode,
                config: serde_json::to_value(InputNodeConfig {
                    input_type: match import_profile.filetype {
                        ImportFileType::Nodes => "CSVNodesFromFile".to_string(),
                        ImportFileType::Edges => "CSVEdgesFromFile".to_string(),
                        ImportFileType::Layers => "CSVLayersFromFile".to_string(),
                    },
                    source: import_profile.filename.clone(),
                    data_type: match import_profile.filetype {
                        ImportFileType::Nodes => "Nodes".to_string(),
                        ImportFileType::Edges => "Edges".to_string(),
                        ImportFileType::Layers => "Layers".to_string(),
                    },
                    output_graph_ref: format!("graph_{}", node_counter),
                    csv_options: None,
                    rest_options: None,
                    sql_options: None,
                }).unwrap(),
                position: Position { x: 100.0 * node_counter as f64, y: 100.0 },
                metadata: NodeMetadata {
                    label: format!("Import {}", import_profile.filename),
                    description: Some(format!("Import {} data from CSV",
                        match import_profile.filetype {
                            ImportFileType::Nodes => "nodes",
                            ImportFileType::Edges => "edges",
                            ImportFileType::Layers => "layers",
                        }
                    )),
                },
            });
            node_counter += 1;
        }

        // Add MergeNode if multiple inputs
        if plan.import.profiles.len() > 1 {
            let input_refs: Vec<String> = (0..plan.import.profiles.len())
                .map(|i| format!("input_{}", i))
                .collect();

            nodes.push(PlanDAGNode {
                id: format!("merge_{}", node_counter),
                node_type: PlanDAGNodeType::MergeNode,
                config: serde_json::to_value(MergeNodeConfig {
                    input_refs,
                    output_graph_ref: "graph_merged".to_string(),
                    merge_strategy: "Union".to_string(),
                    conflict_resolution: Some("PreferFirst".to_string()),
                    merge_options: None,
                }).unwrap(),
                position: Position { x: 100.0 * node_counter as f64, y: 200.0 },
                metadata: NodeMetadata {
                    label: "Merge Imports".to_string(),
                    description: Some("Combine imported data".to_string()),
                },
            });
            node_counter += 1;
        }

        // Convert export profiles to TransformNode + OutputNode pairs
        for export_profile in &plan.export.profiles {
            let source_ref = if plan.import.profiles.len() > 1 {
                "graph_merged".to_string()
            } else {
                "graph_0".to_string()
            };

            // Add TransformNode if transformations needed
            if export_profile.graph_config.is_some() {
                let transform_config = export_profile.graph_config.unwrap();

                nodes.push(PlanDAGNode {
                    id: format!("transform_{}", node_counter),
                    node_type: PlanDAGNodeType::TransformNode,
                    config: serde_json::to_value(TransformNodeConfig {
                        input_graph_ref: source_ref.clone(),
                        output_graph_ref: format!("graph_transformed_{}", node_counter),
                        transform_type: "Custom".to_string(),
                        transform_config: TransformConfig {
                            max_partition_depth: if transform_config.max_partition_depth.unwrap_or(0) > 0 {
                                transform_config.max_partition_depth
                            } else { None },
                            max_partition_width: if transform_config.max_partition_width.unwrap_or(0) > 0 {
                                transform_config.max_partition_width
                            } else { None },
                            generate_hierarchy: transform_config.generate_hierarchy,
                            invert_graph: transform_config.invert_graph,
                            aggregate_edges: Some(true), // Always enabled
                            node_label_max_length: if transform_config.node_label_max_length.unwrap_or(0) > 0 {
                                transform_config.node_label_max_length
                            } else { None },
                            node_label_insert_newlines_at: if transform_config.node_label_insert_newlines_at.unwrap_or(0) > 0 {
                                transform_config.node_label_insert_newlines_at
                            } else { None },
                            edge_label_max_length: if transform_config.edge_label_max_length.unwrap_or(0) > 0 {
                                transform_config.edge_label_max_length
                            } else { None },
                            edge_label_insert_newlines_at: if transform_config.edge_label_insert_newlines_at.unwrap_or(0) > 0 {
                                transform_config.edge_label_insert_newlines_at
                            } else { None },
                            custom_config: None,
                        },
                    }).unwrap(),
                    position: Position { x: 100.0 * node_counter as f64, y: 300.0 },
                    metadata: NodeMetadata {
                        label: "Transform Graph".to_string(),
                        description: Some("Apply graph transformations".to_string()),
                    },
                });

                // Create edge from source to transform
                edges.push(PlanDAGEdge {
                    id: format!("edge_to_transform_{}", node_counter),
                    source: source_ref,
                    target: format!("transform_{}", node_counter),
                    metadata: EdgeMetadata {
                        label: "Input Graph".to_string(),
                        data_type: "GraphData".to_string(),
                    },
                });

                node_counter += 1;
            }

            // Add OutputNode
            let output_source = if export_profile.graph_config.is_some() {
                format!("graph_transformed_{}", node_counter - 1)
            } else {
                source_ref
            };

            nodes.push(PlanDAGNode {
                id: format!("output_{}", node_counter),
                node_type: PlanDAGNodeType::OutputNode,
                config: serde_json::to_value(OutputNodeConfig {
                    source_graph_ref: output_source.clone(),
                    render_target: export_profile.exporter.to_string(),
                    output_path: export_profile.filename.clone(),
                    render_config: export_profile.render_config.map(|rc| rc.into()),
                    graph_config: export_profile.graph_config.map(|gc| gc.into()),
                }).unwrap(),
                position: Position { x: 100.0 * node_counter as f64, y: 400.0 },
                metadata: NodeMetadata {
                    label: format!("Export {}", export_profile.filename),
                    description: Some(format!("Export to {} format", export_profile.exporter.to_string())),
                },
            });

            // Create edge to output
            edges.push(PlanDAGEdge {
                id: format!("edge_to_output_{}", node_counter),
                source: output_source,
                target: format!("output_{}", node_counter),
                metadata: EdgeMetadata {
                    label: "Export Data".to_string(),
                    data_type: "GraphData".to_string(),
                },
            });

            node_counter += 1;
        }

        PlanDAG {
            version: "1.0".to_string(),
            nodes,
            edges,
            metadata: PlanDAGMetadata {
                created_at: chrono::Utc::now(),
                last_modified: chrono::Utc::now(),
                version: "1.0".to_string(),
                description: plan.meta.as_ref()
                    .and_then(|m| m.name.as_ref())
                    .unwrap_or(&"Converted from YAML plan".to_string())
                    .clone(),
            },
        }
    }

    /// Execute Plan DAG using existing pipeline for compatibility
    pub async fn execute_legacy_compatible(
        plan_dag: &PlanDAG,
        project_path: &Path,
    ) -> Result<(), ExecutionError> {
        // Extract compatible Plan structure from Plan DAG
        let legacy_plan = Self::dag_to_yaml_plan(plan_dag)?;

        // Use existing execution pipeline
        crate::plan_execution::execute_plan(&legacy_plan, project_path)
            .map_err(|e| ExecutionError::LegacyExecution(e.to_string()))
    }
}
```

## Integration Benefits

### 1. **Zero Regression Risk**
- Existing transformation logic preserved unchanged
- All current transformations accessible through TransformNode
- Export system fully compatible through OutputNode

### 2. **Incremental Migration**
- YAML plans can be converted to Plan DAG format
- Existing CLI functionality maintained
- New interactive features build on proven foundation

### 3. **Performance Characteristics**
- Leverages optimized existing graph operations
- No performance degradation for transformations
- Template-based export system maintains efficiency

### 4. **Maintenance Benefits**
- Single source of truth for transformations
- Bug fixes automatically benefit both systems
- Existing test coverage protects functionality

## Implementation Sequence

### Phase 1: Core Integration (Weeks 3-4)
1. Create TransformNode executor using existing `apply_graph_transformations()`
2. Create OutputNode executor using existing export system
3. Build configuration conversion layers

### Phase 2: Plan DAG Execution (Weeks 5-6)
4. Implement PlanDAGExecutor with topological sorting
5. Create ExecutionContext for graph state management
6. Add comprehensive error handling and logging

### Phase 3: Compatibility & Testing (Weeks 7-8)
7. Build LegacyPlanAdapter for YAML conversion
8. Comprehensive integration testing
9. Performance benchmarking against existing system

This integration strategy ensures that the new Plan DAG system builds on the existing robust foundation while enabling the interactive features required by the specification. The existing transformation and export systems remain unchanged, eliminating regression risk while providing a clear migration path.