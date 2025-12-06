Implementation Document: Cloud Infrastructure Analysis & Correlation Engine1. Executive SummaryThis document details the extension of the existing Rust-based static analysis engine to support Infrastructure as Code (IaC) and Cloud Resource Identification. The goal is to detect cloud resources (AWS, Azure, GCP) defined in Terraform, CloudFormation, SAM, Bicep, and CDK, and map their relationships to application code in both Python and JavaScript/TypeScript.The proposed architecture uses native Rust crates to parse specific file formats into a unified Resource Graph. This graph allows for advanced queries, such as "Which Lambda function uses this S3 bucket?" or "Does this Python/JS code reference an undefined DynamoDB table?"2. Research & Technology SelectionTo achieve high performance and memory safety, we evaluated native Rust parsers for the required formats.FormatRecommended CrateProsConsTerraform (.tf)hcl-rsNative Serde support; easy data extraction into Structs.Stricter than Tree-Sitter; fails on invalid syntax.CloudFormation (.yaml)serde_yamlStandard Rust YAML parser; robust.Requires custom logic for Intrinsic Functions (!Ref).Bicep (.bicep)tree-sitter-bicepBest option for DSLs; error tolerant.Requires traversal logic (CST) rather than deserialization.CDK (Python)rustpython-parserReuses existing engine; deep analysis.Limited to Python CDK.CDK (TypeScript)oxc_parser or swcTODO: High-performance AST generation for JS/TS.Requires complex AST traversal for class patterns.GraphingNative CollectionsZero dependencies; fully serializable.Requires manual implementation of traversal (BFS/DFS).Recommendation: Specialized Parser StrategyInstead of using tree-sitter for everything (which results in generic "nodes"), we recommend Specialized Parsers where available (hcl-rs for HCL, serde for YAML, oxc for JS/TS). This allows us to strictly type the infrastructure definitions, making the extraction of properties (like bucket_name or runtime) significantly easier.3. Data Model: The Unified Resource GraphRegardless of the input format (Terraform vs. CloudFormation), all resources are normalized into a generic ResourceNode. The structs derive Serialize to support external rendering pipelines.use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
enum ResourceType {
    AwsS3Bucket,
    AwsLambdaFunction,
    AzureStorageAccount,
    GcpComputeInstance,
    // ...
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResourceNode {
    id: String,                  // Unique ID (e.g., "module.vpc.aws_subnet.main")
    resource_type: ResourceType, // Normalized Type
    name: String,                // Logical Name
    source_file: String,         // File path
    properties: HashMap<String, String>, // Key-Value pairs (region, runtime, etc.)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum EdgeType {
    DependsOn,    // Explicit dependency
    References,   // Implicit reference (e.g., passing an ARN)
    CodeLink,     // Code file implements this resource (e.g., Lambda handler)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GraphEdge {
    from: String, // Source Node ID
    to: String,   // Target Node ID
    edge_type: EdgeType,
}
4. Technical Implementation Examples4.1 Parsing Terraform (hcl-rs)This example demonstrates how to extract AWS resources and their dependencies from a .tf file.Dependencies:[dependencies]
hcl-rs = "0.18"
serde = { version = "1.0", features = ["derive"] }
Implementation:use hcl::{Body, Attribute, Block};
use std::fs;

pub fn parse_terraform(path: &str) {
    let content = fs::read_to_string(path).unwrap();
    let body: Body = hcl::from_str(&content).expect("Failed to parse HCL");

    for block in body.blocks() {
        if block.identifier() == "resource" {
            // Labels: ["aws_s3_bucket", "my_bucket"]
            let labels = block.labels(); 
            let provider_type = labels.get(0).unwrap().as_str();
            let logical_name = labels.get(1).unwrap().as_str();

            println!("Found Resource: {} ({})", logical_name, provider_type);

            // Extract Attributes
            if let Some(bucket_attr) = block.body().attributes().find(|a| a.key() == "bucket") {
                println!(" -> Bucket Name defined: {:?}", bucket_attr.value());
            }
        }
    }
}
4.2 Parsing CloudFormation (serde_yaml)CloudFormation uses YAML tags like !Ref which complicate standard parsing. We treat them as generic Enums or traverse the serde_yaml::Value tree manually.Dependencies:[dependencies]
serde_yaml = "0.9"
Implementation:use serde_yaml::Value;
use std::fs;

pub fn parse_cloudformation(path: &str) {
    let content = fs::read_to_string(path).unwrap();
    let parsed: Value = serde_yaml::from_str(&content).expect("Invalid YAML");

    if let Some(resources) = parsed.get("Resources").and_then(|v| v.as_mapping()) {
        for (name, body) in resources {
            let res_type = body.get("Type").and_then(|t| t.as_str()).unwrap_or("Unknown");
            println!("Found CFN Resource: {:?} ({})", name, res_type);

            // Detect Dependencies via !Ref (simplified traversal)
            if let Some(props) = body.get("Properties").and_then(|p| p.as_mapping()) {
                for (_key, val) in props {
                    // Check for implicit refs in properties
                    if let Value::String(s) = val {
                         // Very basic intrinsic function detection
                         if s.starts_with("!Ref") || s.contains("${") {
                             println!(" -> Possible Reference found: {}", s);
                         }
                    }
                }
            }
        }
    }
}
4.3 Parsing Bicep (tree-sitter)Bicep is a Domain Specific Language (DSL). We use tree-sitter with the Bicep grammar to query the syntax tree for resource declarations.Dependencies:[dependencies]
tree-sitter = "0.20"
tree-sitter-bicep = "0.1" // Or build from grammar source
Implementation:use tree_sitter::{Parser, Query, QueryCursor};

pub fn parse_bicep(path: &str) {
    let source_code = std::fs::read_to_string(path).unwrap();
    let mut parser = Parser::new();
    let language = tree_sitter_bicep::language();
    parser.set_language(language).expect("Error loading Bicep grammar");

    let tree = parser.parse(&source_code, None).unwrap();
    
    // S-Expression Query to find resource declarations
    // Matches: resource <name> '<type>'
    let query_str = "(resource_declaration 
                        name: (identifier) @res_name
                        type: (resource_type) @res_type)";
                        
    let query = Query::new(language, query_str).unwrap();
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, tree.root_node(), source_code.as_bytes()) {
        let name_node = m.captures[0].node;
        let type_node = m.captures[1].node;
        
        println!("Found Bicep Resource: {} ({})", 
            name_node.utf8_text(source_code.as_bytes()).unwrap(),
            type_node.utf8_text(source_code.as_bytes()).unwrap()
        );
    }
}
4.4 Parsing CDK (Python & TypeScript)CDK infrastructure is defined in standard programming languages.Python Implementation (rustpython)For Python CDK, we leverage our existing rustpython integration to detect class instantiations that match CDK patterns (e.g., s3.Bucket(...)).Dependencies:[dependencies]
rustpython-parser = "0.3.0"
rustpython-ast = { version = "0.3.0", features = ["visitor"] }
Implementation:use rustpython_ast::{Visitor, ExprCall, Expr};

struct CdkVisitor;

impl<'a> Visitor<'a> for CdkVisitor {
    fn visit_expr_call(&mut self, node: &'a ExprCall) {
        // We look for calls like: s3.Bucket(self, "MyBucket", ...)
        // node.func is likely an Expr::Attribute (s3.Bucket)
        
        if let Expr::Attribute(attr) = &*node.func {
            // Check if the attribute name suggests a resource
            let construct_name = &attr.attr; // e.g., "Bucket" or "Function"
            
            // Heuristic: Check if the base is a known AWS module (s3, lambda, aws_s3)
            // This requires a helper to resolve "s3" from the attribute value
            
            if is_cloud_construct(construct_name) {
                // The second argument in CDK is usually the logical ID
                let logical_id = if node.args.len() > 1 {
                    format!("{:?}", node.args[1]) // Simplified extraction
                } else {
                    "Unknown".to_string()
                };

                println!("Found CDK Construct: {} (ID: {})", construct_name, logical_id);
            }
        }
        self.generic_visit_expr_call(node);
    }
}

fn is_cloud_construct(name: &str) -> bool {
    let cloud_types = ["Bucket", "Function", "Table", "Queue", "Topic"];
    cloud_types.contains(&name)
}
TypeScript Implementation (TODO)Requirements: Parse .ts files to identify new s3.Bucket(...) patterns.Recommended Crate: oxc_parser or swc_ecma_parser.// TODO: Implement TypeScript CDK Visitor
// 1. Ingest TypeScript file using Oxc/SWC
// 2. Traverse AST looking for `NewExpression`
// 3. Match `callee` against known CDK constructs (e.g., "s3.Bucket")
// 4. Extract first argument as `scope` and second as `id`
4.5 Graph Construction (Custom Adjacency List)We implement a custom InfrastructureGraph using a HashMap for node storage and an Adjacency List (Vector of Edges) for relationships. This structure is natively serializable to JSON without requiring petgraph.use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct InfrastructureGraph {
    // Map of NodeID -> ResourceNode
    nodes: HashMap<String, ResourceNode>,
    // Map of NodeID -> List of Outgoing Edges
    adjacency_list: HashMap<String, Vec<GraphEdge>>,
}

impl InfrastructureGraph {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            adjacency_list: HashMap::new(),
        }
    }

    fn add_resource(&mut self, res: ResourceNode) {
        self.nodes.insert(res.id.clone(), res);
    }

    fn add_dependency(&mut self, from_id: &str, to_id: &str, edge_type: EdgeType) {
        // Validation: Ensure both nodes exist before linking
        if self.nodes.contains_key(from_id) && self.nodes.contains_key(to_id) {
            let edge = GraphEdge {
                from: from_id.to_string(),
                to: to_id.to_string(),
                edge_type,
            };
            
            self.adjacency_list
                .entry(from_id.to_string())
                .or_insert_with(Vec::new)
                .push(edge);
        } else {
            eprintln!("Warning: Attempted to link undefined nodes: {} -> {}", from_id, to_id);
        }
    }
    
    // Example: Helper to find all dependencies of a node
    fn get_dependencies(&self, id: &str) -> Option<&Vec<GraphEdge>> {
        self.adjacency_list.get(id)
    }
}
4.6 The "Linker" (Code-to-Infra)This is the bridge. We use the Code Analyzers to find os.getenv("TABLE_NAME") (Python) or process.env.TABLE_NAME (JS/TS) and match it against Terraform's environment { variables = { TABLE_NAME = ... } }.// Pseudo-code logic for the Linker
fn link_code_to_infra(infra_graph: &InfrastructureGraph, code_ast: &AnalysisResult) {
    // 1. Identify Environment Variable usage in Code
    // (Assumes existing Python analyzer extracts `os.getenv("FOO")`)
    // TODO: Ensure JS/TS analyzer extracts `process.env.FOO`
    let env_vars_used = &code_ast.extracted_env_vars; 

    // 2. Iterate through all Infrastructure Nodes (Values of the HashMap)
    for node in infra_graph.nodes.values() {
        if let Some(env_block) = node.properties.get("environment_variables") {
            for var in env_vars_used {
                if env_block.contains(var) {
                    println!("LINK: Code ({}) uses env var '{}' defined in Infra ({})", 
                        "app.py", var, node.name);
                }
            }
        }
        
        // 3. Check for Lambda Handler filenames
        if let Some(handler_path) = node.properties.get("filename") {
            if code_ast.files.contains(handler_path) {
                 println!("LINK: Infra resource '{}' runs code file '{}'", node.name, handler_path);
            }
        }
    }
}
5. Implementation RoadmapPhase 1: The Parsers (Weeks 1-2)Goal: Ingest .tf and .yaml files into Rust structs.Action: Create scanner::terraform and scanner::cloudformation modules.Deliverable: A robust ResourceNode extraction pipeline that outputs a list of resources found in the target directory.Phase 2: App Code Analyzers (Week 3)Goal: Extract Cloud constructs and Env Vars from App Code.Action (Python): Integrate rustpython CDK visitor (see 4.4).Action (JS/TS): TODO Integrate oxc_parser or swc to traverse TypeScript ASTs for CDK constructs (new s3.Bucket) and process.env usage.Deliverable: AST Visitors capable of listing "Constructs Found" and "EnvVars Used".Phase 3: The Graph (Week 4)Goal: Establish relationships.Action: Implement the InfrastructureGraph struct (HashMap + Adjacency List). Write logic to resolve Terraform depends_on and var.x references.Deliverable: The fully populated InfrastructureGraph data structure, serialized (e.g., JSON) for consumption by the external rendering engine.Phase 4: Code Correlation (Week 5)Goal: Link the AST analysis with the Infra Graph.Action: Extend the Analyzer struct to collect os.environ (Py) and process.env (JS) calls. Create the matching logic demonstrated in section 4.6.Deliverable: A structured data object (e.g., CorrelationReport) linking Code Nodes to Infra Nodes, identifying orphaned resources and configuration mismatches.

