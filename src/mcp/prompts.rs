//! Graph analysis prompt templates for Layercake MCP
//! Provides intelligent prompts for analyzing graph structure, connectivity, and patterns

use axum_mcp::prelude::*;
use std::collections::HashMap;

/// Layercake prompt registry for graph analysis templates
#[derive(Debug, Clone)]
pub struct LayercakePromptRegistry {
    prompts: HashMap<String, Prompt>,
    categories: Vec<PromptCategory>,
}

impl LayercakePromptRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            prompts: HashMap::new(),
            categories: Vec::new(),
        };
        
        registry.initialize_graph_analysis_prompts();
        registry
    }
    
    fn initialize_graph_analysis_prompts(&mut self) {
        // Graph structure analysis prompt
        self.add_graph_analysis_prompt(
            "analyze_graph_structure",
            "Analyze graph structure and provide insights about connectivity, hierarchy, and patterns",
            r#"Analyze this graph structure with {{node_count}} nodes and {{edge_count}} edges.

Graph Details:
- Nodes: {{node_count}}
- Edges: {{edge_count}}
- Layers: {{layer_count}}
- Graph Type: {{graph_type}}
- Density: {{density}}

Please provide:
1. **Connectivity Analysis**: Identify connected components, isolated nodes, and hub nodes
2. **Hierarchical Structure**: Analyze layer distribution and hierarchy patterns  
3. **Network Metrics**: Calculate density, average degree, and clustering patterns
4. **Anomaly Detection**: Identify unusual patterns or outliers
5. **Optimization Suggestions**: Recommend improvements for graph layout or structure

{{#if focus_areas}}Focus specifically on: {{focus_areas}}{{/if}}

Use the layercake:// URI scheme to access additional data:
- Project details: layercake://projects/{{project_id}}
- Graph export: layercake://graphs/{{project_id}}/json
- Connectivity analysis: layercake://analysis/{{project_id}}/connectivity"#,
            vec![
                PromptParameter {
                    name: "project_id".to_string(),
                    description: "ID of the project to analyze".to_string(),
                    required: true,
                    schema: Some(serde_json::json!({"type": "integer", "minimum": 1})),
                },
                PromptParameter {
                    name: "node_count".to_string(),
                    description: "Number of nodes in the graph".to_string(),
                    required: true,
                    schema: Some(serde_json::json!({"type": "integer", "minimum": 0})),
                },
                PromptParameter {
                    name: "edge_count".to_string(),
                    description: "Number of edges in the graph".to_string(),
                    required: true,
                    schema: Some(serde_json::json!({"type": "integer", "minimum": 0})),
                },
                PromptParameter {
                    name: "layer_count".to_string(),
                    description: "Number of layers in the graph".to_string(),
                    required: false,
                    schema: Some(serde_json::json!({"type": "integer", "minimum": 0})),
                    default: Some(serde_json::json!(1)),
                },
                PromptParameter {
                    name: "density".to_string(),
                    description: "Graph density (0.0 to 1.0)".to_string(),
                    required: false,
                    schema: Some(serde_json::json!({"type": "number", "minimum": 0.0, "maximum": 1.0})),
                    default: Some(serde_json::json!(0.0)),
                },
                PromptParameter {
                    name: "graph_type".to_string(),
                    description: "Type of graph (directed, undirected, layered, etc.)".to_string(),
                    required: false,
                    schema: Some(serde_json::json!({
                        "type": "string",
                        "enum": ["directed", "undirected", "layered", "hierarchical", "mixed"]
                    })),
                    default: Some(serde_json::Value::String("layered".to_string())),
                },
                PromptParameter {
                    name: "focus_areas".to_string(),
                    description: "Specific areas to focus analysis on".to_string(),
                    required: false,
                    schema: Some(serde_json::json!({
                        "type": "array",
                        "items": {"type": "string"}
                    })),
                },
            ],
        );

        // Path analysis prompt
        self.add_graph_analysis_prompt(
            "analyze_paths",
            "Analyze paths and relationships between specific nodes in the graph",
            r#"Analyze paths between {{source_node}} and {{target_node}} in project {{project_id}}.

Path Analysis Request:
- Source Node: {{source_node}}
- Target Node: {{target_node}}
- Maximum Path Length: {{max_path_length}}
- Path Type: {{path_type}}

Please provide:
1. **Direct Paths**: List all direct paths between the nodes
2. **Shortest Paths**: Identify the shortest path(s) and their length
3. **Alternative Routes**: Describe alternative paths and their characteristics
4. **Bottlenecks**: Identify potential bottleneck nodes in the paths
5. **Path Patterns**: Analyze common patterns in the discovered paths
6. **Recommendations**: Suggest optimizations for connectivity

{{#if include_intermediate_analysis}}
Also analyze the intermediate nodes and their roles in the network.
{{/if}}

Use the find_paths tool to get detailed path data:
- Tool: find_paths
- Parameters: {"project_id": {{project_id}}, "source_node": "{{source_node}}", "target_node": "{{target_node}}", "max_paths": {{max_path_length}}}"#,
            vec![
                PromptParameter {
                    name: "project_id".to_string(),
                    description: "ID of the project containing the graph".to_string(),
                    required: true,
                    schema: Some(serde_json::json!({"type": "integer", "minimum": 1})),
                },
                PromptParameter {
                    name: "source_node".to_string(),
                    description: "ID of the source node".to_string(),
                    required: true,
                    schema: Some(serde_json::json!({"type": "string"})),
                },
                PromptParameter {
                    name: "target_node".to_string(),
                    description: "ID of the target node".to_string(),
                    required: true,
                    schema: Some(serde_json::json!({"type": "string"})),
                },
                PromptParameter {
                    name: "max_path_length".to_string(),
                    description: "Maximum path length to consider".to_string(),
                    required: false,
                    schema: Some(serde_json::json!({"type": "integer", "minimum": 1, "maximum": 20})),
                    default: Some(serde_json::json!(10)),
                },
                PromptParameter {
                    name: "path_type".to_string(),
                    description: "Type of paths to analyze".to_string(),
                    required: false,
                    schema: Some(serde_json::json!({
                        "type": "string",
                        "enum": ["shortest", "all", "optimal", "diverse"]
                    })),
                    default: Some(serde_json::Value::String("all".to_string())),
                },
                PromptParameter {
                    name: "include_intermediate_analysis".to_string(),
                    description: "Whether to include analysis of intermediate nodes".to_string(),
                    required: false,
                    schema: Some(serde_json::json!({"type": "boolean"})),
                    default: Some(serde_json::json!(false)),
                },
            ],
        );

        // Layer analysis prompt
        self.add_graph_analysis_prompt(
            "analyze_layers",
            "Analyze layer structure and inter-layer relationships in hierarchical graphs",
            r#"Analyze the layer structure of this {{layer_count}}-layer graph in project {{project_id}}.

Layer Configuration:
- Total Layers: {{layer_count}}
- Analysis Focus: {{analysis_focus}}
- Include Cross-Layer Edges: {{include_cross_layer}}

Please provide:
1. **Layer Distribution**: Analyze node distribution across layers
2. **Inter-Layer Connectivity**: Examine connections between layers
3. **Layer Hierarchy**: Identify hierarchical patterns and dependencies
4. **Layer Balance**: Assess whether layers are balanced in terms of content
5. **Optimization Opportunities**: Suggest layer reorganization improvements
6. **Visual Recommendations**: Recommend optimal visualization approaches

{{#if specific_layers}}
Focus detailed analysis on layers: {{specific_layers}}
{{/if}}

Access layer data using layercake resources:
- Project details: layercake://projects/{{project_id}}
- Graph structure: layercake://graphs/{{project_id}}/json"#,
            vec![
                PromptParameter {
                    name: "project_id".to_string(),
                    description: "ID of the project containing the layered graph".to_string(),
                    required: true,
                    schema: Some(serde_json::json!({"type": "integer", "minimum": 1})),
                },
                PromptParameter {
                    name: "layer_count".to_string(),
                    description: "Number of layers in the graph".to_string(),
                    required: true,
                    schema: Some(serde_json::json!({"type": "integer", "minimum": 1})),
                },
                PromptParameter {
                    name: "analysis_focus".to_string(),
                    description: "Primary focus of the layer analysis".to_string(),
                    required: false,
                    schema: Some(serde_json::json!({
                        "type": "string",
                        "enum": ["distribution", "connectivity", "hierarchy", "balance", "optimization"]
                    })),
                    default: Some(serde_json::Value::String("connectivity".to_string())),
                },
                PromptParameter {
                    name: "include_cross_layer".to_string(),
                    description: "Whether to include cross-layer edge analysis".to_string(),
                    required: false,
                    schema: Some(serde_json::json!({"type": "boolean"})),
                    default: Some(serde_json::json!(true)),
                },
                PromptParameter {
                    name: "specific_layers".to_string(),
                    description: "Specific layer IDs to focus on".to_string(),
                    required: false,
                    schema: Some(serde_json::json!({
                        "type": "array",
                        "items": {"type": "string"}
                    })),
                },
            ],
        );

        // Graph transformation recommendations prompt
        self.add_graph_analysis_prompt(
            "recommend_transformations",
            "Generate recommendations for graph transformations and optimizations",
            r#"Analyze project {{project_id}} and recommend transformations to improve the graph structure.

Current Graph Metrics:
- Nodes: {{node_count}}
- Edges: {{edge_count}}
- Layers: {{layer_count}}
- Connected Components: {{component_count}}
- Transformation Goal: {{transformation_goal}}

Please provide:
1. **Structural Improvements**: Suggest modifications to improve connectivity or hierarchy
2. **Layout Optimizations**: Recommend changes for better visualization
3. **Data Quality Enhancements**: Identify opportunities to improve data consistency
4. **Performance Optimizations**: Suggest changes to improve analysis performance  
5. **Export Recommendations**: Recommend optimal export formats for different use cases
6. **Implementation Plan**: Provide step-by-step transformation recommendations

{{#if constraint_requirements}}
Consider these constraints: {{constraint_requirements}}
{{/if}}

Use the analyze_connectivity tool to get detailed structural analysis before making recommendations."#,
            vec![
                PromptParameter {
                    name: "project_id".to_string(),
                    description: "ID of the project to analyze for transformations".to_string(),
                    required: true,
                    schema: Some(serde_json::json!({"type": "integer", "minimum": 1})),
                },
                PromptParameter {
                    name: "node_count".to_string(),
                    description: "Current number of nodes".to_string(),
                    required: true,
                    schema: Some(serde_json::json!({"type": "integer", "minimum": 0})),
                },
                PromptParameter {
                    name: "edge_count".to_string(),
                    description: "Current number of edges".to_string(),
                    required: true,
                    schema: Some(serde_json::json!({"type": "integer", "minimum": 0})),
                },
                PromptParameter {
                    name: "layer_count".to_string(),
                    description: "Current number of layers".to_string(),
                    required: false,
                    schema: Some(serde_json::json!({"type": "integer", "minimum": 0})),
                    default: Some(serde_json::json!(1)),
                },
                PromptParameter {
                    name: "component_count".to_string(),
                    description: "Number of connected components".to_string(),
                    required: false,
                    schema: Some(serde_json::json!({"type": "integer", "minimum": 0})),
                    default: Some(serde_json::json!(1)),
                },
                PromptParameter {
                    name: "transformation_goal".to_string(),
                    description: "Primary goal of the transformation".to_string(),
                    required: false,
                    schema: Some(serde_json::json!({
                        "type": "string",
                        "enum": ["improve_connectivity", "optimize_layout", "enhance_hierarchy", "reduce_complexity", "improve_performance"]
                    })),
                    default: Some(serde_json::Value::String("improve_connectivity".to_string())),
                },
                PromptParameter {
                    name: "constraint_requirements".to_string(),
                    description: "Any constraints or requirements for the transformation".to_string(),
                    required: false,
                    schema: Some(serde_json::json!({
                        "type": "array",
                        "items": {"type": "string"}
                    })),
                },
            ],
        );

        // Add category
        self.categories.push(PromptCategory {
            id: "graph_analysis".to_string(),
            name: "Graph Analysis".to_string(),
            description: "Prompts for analyzing graph structure, connectivity, and patterns".to_string(),
            prompts: vec![
                "analyze_graph_structure".to_string(),
                "analyze_paths".to_string(),
                "analyze_layers".to_string(),
                "recommend_transformations".to_string(),
            ],
        });
    }
    
    fn add_graph_analysis_prompt(&mut self, name: &str, description: &str, template: &str, parameters: Vec<PromptParameter>) {
        let prompt = Prompt {
            name: name.to_string(),
            description: description.to_string(),
            version: "1.0.0".to_string(),
            parameters,
            messages: vec![
                PromptMessage {
                    role: MessageRole::System,
                    content: PromptContent::Text {
                        text: "You are an expert graph analyst specializing in network analysis, graph theory, and data visualization. You have deep knowledge of layered graph structures, connectivity patterns, and transformation techniques. Provide detailed, actionable insights about graph structures and patterns with specific recommendations for improvements.".to_string(),
                    },
                },
                PromptMessage {
                    role: MessageRole::User,
                    content: PromptContent::Text {
                        text: template.to_string(),
                    },
                },
            ],
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("type".to_string(), serde_json::Value::String("graph_analysis".to_string()));
                meta.insert("domain".to_string(), serde_json::Value::String("layercake".to_string()));
                meta.insert("version".to_string(), serde_json::Value::String("1.0.0".to_string()));
                meta
            },
        };
        
        self.prompts.insert(name.to_string(), prompt);
    }
}

impl Default for LayercakePromptRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PromptRegistry for LayercakePromptRegistry {
    async fn list_prompts(&self, _context: &SecurityContext) -> McpResult<Vec<Prompt>> {
        Ok(self.prompts.values().cloned().collect())
    }
    
    async fn get_prompt(&self, name: &str, _context: &SecurityContext) -> McpResult<Option<Prompt>> {
        Ok(self.prompts.get(name).cloned())
    }
    
    async fn get_prompt_with_args(&self, request: GetPromptRequest, _context: &SecurityContext) -> McpResult<GetPromptResult> {
        let prompt = self.prompts.get(&request.name)
            .ok_or_else(|| McpError::PromptNotFound {
                name: request.name.clone(),
            })?;
        
        let params = request.arguments.unwrap_or_default();
        
        // Validate required parameters
        for param in &prompt.parameters {
            if param.required && !params.contains_key(&param.name) {
                return Err(McpError::Validation {
                    message: format!("Required parameter '{}' not provided", param.name),
                });
            }
        }
        
        // Simple template substitution for {{variable}} patterns
        let mut rendered_messages = Vec::new();
        for message in &prompt.messages {
            let rendered_content = match &message.content {
                PromptContent::Text { text } => {
                    let mut rendered_text = text.clone();
                    
                    // Substitute template variables
                    for (key, value) in &params {
                        let placeholder = format!("{{{{{}}}}}", key);
                        let replacement = match value {
                            serde_json::Value::String(s) => s.clone(),
                            serde_json::Value::Number(n) => n.to_string(),
                            serde_json::Value::Bool(b) => b.to_string(),
                            serde_json::Value::Array(arr) => {
                                arr.iter()
                                    .map(|v| match v {
                                        serde_json::Value::String(s) => s.clone(),
                                        other => other.to_string(),
                                    })
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            },
                            other => other.to_string(),
                        };
                        rendered_text = rendered_text.replace(&placeholder, &replacement);
                    }
                    
                    // Handle simple conditional blocks {{#if condition}}...{{/if}}
                    for (key, value) in &params {
                        let if_start = format!("{{{{#if {}}}}}", key);
                        let if_end = "{{/if}}";
                        
                        if let Some(start_pos) = rendered_text.find(&if_start) {
                            if let Some(end_pos) = rendered_text.find(if_end) {
                                let before = &rendered_text[..start_pos];
                                let content = &rendered_text[start_pos + if_start.len()..end_pos];
                                let after = &rendered_text[end_pos + if_end.len()..];
                                
                                // Check if condition is truthy
                                let include_content = match value {
                                    serde_json::Value::Bool(b) => *b,
                                    serde_json::Value::Null => false,
                                    serde_json::Value::String(s) => !s.is_empty(),
                                    serde_json::Value::Array(arr) => !arr.is_empty(),
                                    serde_json::Value::Object(obj) => !obj.is_empty(),
                                    serde_json::Value::Number(_) => true,
                                };
                                
                                rendered_text = if include_content {
                                    format!("{}{}{}", before, content, after)
                                } else {
                                    format!("{}{}", before, after)
                                };
                            }
                        }
                    }
                    
                    PromptContent::Text { text: rendered_text }
                },
                other => other.clone(),
            };
            
            rendered_messages.push(PromptMessage {
                role: message.role.clone(),
                content: rendered_content,
            });
        }
        
        // Render description
        let mut rendered_description = prompt.description.clone();
        for (key, value) in &params {
            let placeholder = format!("{{{{{}}}}}", key);
            let replacement = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                other => other.to_string(),
            };
            rendered_description = rendered_description.replace(&placeholder, &replacement);
        }
        
        Ok(GetPromptResult {
            name: request.name,
            messages: rendered_messages,
            description: rendered_description,
        })
    }
    
    async fn list_categories(&self, _context: &SecurityContext) -> McpResult<Vec<PromptCategory>> {
        Ok(self.categories.clone())
    }
    
    async fn prompt_exists(&self, name: &str, _context: &SecurityContext) -> McpResult<bool> {
        Ok(self.prompts.contains_key(name))
    }
    
    async fn validate_prompt_parameters(&self, name: &str, params: &HashMap<String, serde_json::Value>, _context: &SecurityContext) -> McpResult<()> {
        let prompt = self.prompts.get(name)
            .ok_or_else(|| McpError::PromptNotFound {
                name: name.to_string(),
            })?;
        
        for param in &prompt.parameters {
            if param.required && !params.contains_key(&param.name) {
                return Err(McpError::Validation {
                    message: format!("Required parameter '{}' not provided", param.name),
                });
            }
        }
        
        Ok(())
    }
}