use layercake_code_analysis::analyzer::{AnalysisResult, EnvVarUsage, FunctionInfo};
use layercake_code_analysis::infra::{
    correlate_code_infra, EdgeType, GraphEdge, InfrastructureGraph, ResourceNode, ResourceType,
};

fn sample_infra() -> InfrastructureGraph {
    let mut graph = InfrastructureGraph::new("infra");
    let mut node = ResourceNode::new(
        "aws_lambda_func",
        ResourceType::Aws("aws_lambda_function".into()),
        "handler",
        "main.tf",
    );
    node.properties
        .insert("handler".into(), "app.lambda_handler".into());
    node.properties.insert("ENV".into(), "TABLE_NAME".into());
    node.properties.insert("file".into(), "src/app.py".into());
    graph.add_resource(node);
    graph.add_edge(GraphEdge {
        from: "aws_lambda_func".into(),
        to: "aws_s3_bucket_data".into(),
        edge_type: EdgeType::References,
        label: None,
    });
    graph
}

#[test]
fn correlates_env_and_handler() {
    let mut code = AnalysisResult::default();
    code.files = vec!["src/app.py".into()];
    code.functions.push(FunctionInfo {
        name: "lambda_handler".into(),
        file_path: "src/app.py".into(),
        ..Default::default()
    });
    code.env_vars.push(EnvVarUsage {
        name: "TABLE_NAME".into(),
        file_path: "src/app.py".into(),
        line_number: 1,
        kind: "os.getenv".into(),
    });

    let infra = sample_infra();
    let report = correlate_code_infra(&code, &infra);

    assert!(report.matches.iter().any(|m| m.reason.contains("handler")));
    assert!(report.matches.iter().any(|m| m.reason.contains("env var")));
    assert!(report.unresolved.is_empty());
}
