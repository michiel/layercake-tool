use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::graph::InfrastructureGraph;
use super::model::{EdgeType, GraphEdge, ResourceNode, ResourceType};

/// API Gateway route definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRoute {
    pub path: String,
    pub method: String,
    pub integration_type: String,
    pub target: Option<String>,
}

/// Enhanced infrastructure analysis with API Gateway route detection
pub fn detect_api_routes(infra: &InfrastructureGraph) -> Vec<(ApiRoute, Vec<GraphEdge>)> {
    let mut routes = Vec::new();

    // Scan for API Gateway resources
    for (resource_id, resource) in &infra.resources {
        let resource_type_str = format!("{:?}", resource.resource_type).to_lowercase();

        // AWS API Gateway REST API
        if resource_type_str.contains("aws_apigatewayv2_route")
            || resource_type_str.contains("aws_api_gateway_method")
        {
            let route = extract_api_route(resource);
            let edges = create_route_edges(resource_id, &route, infra);
            routes.push((route, edges));
        }

        // SAM API definitions
        if resource_type_str.contains("aws::serverless::api")
            || resource_type_str.contains("aws_serverlessrepo_cloudformation_stack")
        {
            if let Some(definition) = resource.properties.get("DefinitionBody") {
                let sam_routes = parse_sam_definition(definition, resource_id);
                for (route, edges) in sam_routes {
                    routes.push((route, edges));
                }
            }
        }
    }

    routes
}

fn extract_api_route(resource: &ResourceNode) -> ApiRoute {
    let path = resource
        .properties
        .get("route_key")
        .or_else(|| resource.properties.get("resource_path"))
        .or_else(|| resource.properties.get("path"))
        .cloned()
        .unwrap_or_else(|| "/".to_string());

    let method = resource
        .properties
        .get("http_method")
        .or_else(|| resource.properties.get("method"))
        .cloned()
        .unwrap_or_else(|| {
            // Extract method from route_key like "GET /users"
            if let Some(route_key) = resource.properties.get("route_key") {
                route_key
                    .split_whitespace()
                    .next()
                    .unwrap_or("ANY")
                    .to_string()
            } else {
                "ANY".to_string()
            }
        });

    let integration_type = resource
        .properties
        .get("integration_type")
        .or_else(|| resource.properties.get("type"))
        .cloned()
        .unwrap_or_else(|| "AWS_PROXY".to_string());

    let target = resource
        .properties
        .get("integration_uri")
        .or_else(|| resource.properties.get("uri"))
        .cloned();

    ApiRoute {
        path,
        method,
        integration_type,
        target,
    }
}

fn create_route_edges(
    route_resource_id: &str,
    route: &ApiRoute,
    infra: &InfrastructureGraph,
) -> Vec<GraphEdge> {
    let mut edges = Vec::new();

    if let Some(target_uri) = &route.target {
        // Extract Lambda function ARN or name from integration URI
        // Format: arn:aws:apigateway:region:lambda:path/2015-03-31/functions/arn:aws:lambda:region:account:function:FunctionName/invocations
        // Or: ${FunctionArn}
        // Or: function-name

        let function_name = extract_function_name_from_uri(target_uri);

        // Try to find matching Lambda function in infrastructure
        for (lambda_id, lambda_resource) in &infra.resources {
            let resource_type_str = format!("{:?}", lambda_resource.resource_type).to_lowercase();

            if resource_type_str.contains("lambda") && resource_type_str.contains("function") {
                let lambda_name = lambda_resource.name.to_lowercase();
                let function_name_lower = function_name.to_lowercase();

                if lambda_name.contains(&function_name_lower)
                    || function_name_lower.contains(&lambda_name)
                    || target_uri.to_lowercase().contains(&lambda_name)
                {
                    edges.push(GraphEdge {
                        from: route_resource_id.to_string(),
                        to: lambda_id.clone(),
                        edge_type: EdgeType::References,
                        label: Some(format!("{} {}", route.method, route.path)),
                    });
                    break;
                }
            }
        }
    }

    edges
}

fn extract_function_name_from_uri(uri: &str) -> String {
    // Handle various URI formats
    if uri.contains("/functions/") {
        // ARN format
        if let Some(pos) = uri.rfind(":function:") {
            let after = &uri[pos + 10..];
            if let Some(end) = after.find('/') {
                return after[..end].to_string();
            }
            return after.to_string();
        }
    }

    // Handle ${FunctionArn} or ${Function.Arn}
    if uri.starts_with("${") && uri.ends_with('}') {
        let var_name = &uri[2..uri.len() - 1];
        if let Some(dot_pos) = var_name.find('.') {
            return var_name[..dot_pos].to_string();
        }
        return var_name.to_string();
    }

    // Fallback: use the whole URI as function name hint
    uri.to_string()
}

fn parse_sam_definition(
    _definition_body: &str,
    _api_id: &str,
) -> Vec<(ApiRoute, Vec<GraphEdge>)> {
    // Placeholder for SAM template parsing
    // Would parse OpenAPI/Swagger definition from SAM template
    // and extract routes with x-amazon-apigateway-integration
    Vec::new()
}

/// Add API Gateway routes to infrastructure graph
pub fn enrich_with_api_routes(
    infra: &mut InfrastructureGraph,
) -> Vec<(String, ApiRoute)> {
    let routes_with_edges = detect_api_routes(infra);
    let mut enriched_routes = Vec::new();

    for (route, edges) in routes_with_edges {
        // Add edges to the graph
        for edge in edges {
            infra.add_edge(edge);
            enriched_routes.push((route.path.clone(), route.clone()));
        }
    }

    enriched_routes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_function_name_from_arn() {
        let arn = "arn:aws:apigateway:us-east-1:lambda:path/2015-03-31/functions/arn:aws:lambda:us-east-1:123456789012:function:MyFunction/invocations";
        assert_eq!(extract_function_name_from_uri(arn), "MyFunction");
    }

    #[test]
    fn test_extract_function_name_from_variable() {
        assert_eq!(
            extract_function_name_from_uri("${MyFunction.Arn}"),
            "MyFunction"
        );
        assert_eq!(extract_function_name_from_uri("${MyFunctionArn}"), "MyFunctionArn");
    }
}
