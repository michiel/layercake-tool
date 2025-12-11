use serde::{Deserialize, Serialize};

use super::graph::InfrastructureGraph;
use super::model::{EdgeType, GraphEdge};

/// Event source mapping between triggers and handlers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSourceMapping {
    pub trigger_resource_id: String,
    pub handler_resource_id: String,
    pub event_type: String,
    pub filter_pattern: Option<String>,
    pub batch_size: Option<usize>,
}

/// Detect event source mappings in infrastructure
pub fn detect_event_sources(infra: &InfrastructureGraph) -> Vec<EventSourceMapping> {
    let mut mappings = Vec::new();

    for (_resource_id, resource) in &infra.resources {
        let resource_type_str = format!("{:?}", resource.resource_type).to_lowercase();

        // AWS Lambda Event Source Mapping
        if resource_type_str.contains("aws_lambda_event_source_mapping") {
            if let (Some(event_source_arn), Some(function_name)) = (
                resource.properties.get("event_source_arn"),
                resource.properties.get("function_name"),
            ) {
                let event_type = determine_event_type(event_source_arn);
                let batch_size = resource
                    .properties
                    .get("batch_size")
                    .and_then(|s| s.parse().ok());
                let filter_pattern = resource.properties.get("filter_criteria").cloned();

                // Find trigger and handler resources
                if let (Some(trigger_id), Some(handler_id)) = (
                    find_resource_by_arn_or_name(infra, event_source_arn),
                    find_resource_by_name(infra, function_name),
                ) {
                    mappings.push(EventSourceMapping {
                        trigger_resource_id: trigger_id,
                        handler_resource_id: handler_id,
                        event_type,
                        filter_pattern,
                        batch_size,
                    });
                }
            }
        }

        // S3 Bucket Notifications
        if resource_type_str.contains("aws_s3_bucket_notification") {
            if let Some(bucket_id) = resource.properties.get("bucket") {
                // Parse lambda function configurations
                for (key, value) in &resource.properties {
                    if key.starts_with("lambda_function") {
                        if let Some(function_arn) = extract_function_arn(value) {
                            if let Some(handler_id) =
                                find_resource_by_arn_or_name(infra, &function_arn)
                            {
                                let event_type = extract_s3_event_type(key, value);
                                mappings.push(EventSourceMapping {
                                    trigger_resource_id: bucket_id.clone(),
                                    handler_resource_id: handler_id,
                                    event_type,
                                    filter_pattern: None,
                                    batch_size: None,
                                });
                            }
                        }
                    }
                }
            }
        }

        // DynamoDB Stream
        if resource_type_str.contains("aws_lambda_event_source_mapping")
            && resource
                .properties
                .get("event_source_arn")
                .map(|s| s.contains("dynamodb"))
                .unwrap_or(false)
        {
            // Already handled above, but could add DynamoDB-specific logic
        }

        // SNS Topic Subscription
        if resource_type_str.contains("aws_sns_topic_subscription") {
            if let (Some(topic_arn), Some(endpoint)) = (
                resource.properties.get("topic_arn"),
                resource.properties.get("endpoint"),
            ) {
                if let (Some(trigger_id), Some(handler_id)) = (
                    find_resource_by_arn_or_name(infra, topic_arn),
                    find_resource_by_arn_or_name(infra, endpoint),
                ) {
                    mappings.push(EventSourceMapping {
                        trigger_resource_id: trigger_id,
                        handler_resource_id: handler_id,
                        event_type: "sns:Notification".to_string(),
                        filter_pattern: resource.properties.get("filter_policy").cloned(),
                        batch_size: None,
                    });
                }
            }
        }

        // SQS Queue as trigger
        if resource_type_str.contains("aws_lambda_event_source_mapping")
            && resource
                .properties
                .get("event_source_arn")
                .map(|s| s.contains("sqs"))
                .unwrap_or(false)
        {
            // Already handled above
        }

        // EventBridge Rule Target
        if resource_type_str.contains("aws_cloudwatch_event_target")
            || resource_type_str.contains("aws_eventbridge_target")
        {
            if let (Some(rule), Some(arn)) = (
                resource.properties.get("rule"),
                resource.properties.get("arn"),
            ) {
                if let (Some(trigger_id), Some(handler_id)) = (
                    find_resource_by_name(infra, rule),
                    find_resource_by_arn_or_name(infra, arn),
                ) {
                    mappings.push(EventSourceMapping {
                        trigger_resource_id: trigger_id,
                        handler_resource_id: handler_id,
                        event_type: "eventbridge:Event".to_string(),
                        filter_pattern: None,
                        batch_size: None,
                    });
                }
            }
        }
    }

    mappings
}

fn determine_event_type(arn: &str) -> String {
    if arn.contains(":dynamodb:") {
        "dynamodb:StreamRecord".to_string()
    } else if arn.contains(":kinesis:") {
        "kinesis:Record".to_string()
    } else if arn.contains(":sqs:") {
        "sqs:Message".to_string()
    } else if arn.contains(":kafka:") {
        "kafka:Record".to_string()
    } else {
        "unknown".to_string()
    }
}

fn extract_s3_event_type(_key: &str, value: &str) -> String {
    if value.contains("ObjectCreated") {
        "s3:ObjectCreated".to_string()
    } else if value.contains("ObjectRemoved") {
        "s3:ObjectRemoved".to_string()
    } else {
        "s3:Event".to_string()
    }
}

fn extract_function_arn(config: &str) -> Option<String> {
    // Simple extraction - would need more sophisticated parsing
    if config.contains("arn:aws:lambda") {
        Some(config.to_string())
    } else {
        None
    }
}

fn find_resource_by_arn_or_name(infra: &InfrastructureGraph, arn_or_name: &str) -> Option<String> {
    // Try exact ID match first
    if infra.resources.contains_key(arn_or_name) {
        return Some(arn_or_name.to_string());
    }

    // Try name match
    for (id, resource) in &infra.resources {
        if resource.name.to_lowercase() == arn_or_name.to_lowercase() {
            return Some(id.clone());
        }

        // Try ARN match in properties
        if arn_or_name.contains(&resource.name) {
            return Some(id.clone());
        }

        // Check if any property contains the ARN
        for value in resource.properties.values() {
            if value.to_lowercase() == arn_or_name.to_lowercase() {
                return Some(id.clone());
            }
        }
    }

    None
}

fn find_resource_by_name(infra: &InfrastructureGraph, name: &str) -> Option<String> {
    for (id, resource) in &infra.resources {
        if resource.name.to_lowercase() == name.to_lowercase() {
            return Some(id.clone());
        }
    }
    None
}

/// Add event source mappings as edges to infrastructure graph
pub fn enrich_with_event_sources(infra: &mut InfrastructureGraph) -> Vec<EventSourceMapping> {
    let mappings = detect_event_sources(infra);

    for mapping in &mappings {
        infra.add_edge(GraphEdge {
            from: mapping.trigger_resource_id.clone(),
            to: mapping.handler_resource_id.clone(),
            edge_type: EdgeType::References,
            label: Some(mapping.event_type.clone()),
        });
    }

    mappings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_event_type() {
        assert_eq!(
            determine_event_type("arn:aws:dynamodb:us-east-1:123:table/MyTable/stream/2024"),
            "dynamodb:StreamRecord"
        );
        assert_eq!(
            determine_event_type("arn:aws:kinesis:us-east-1:123:stream/MyStream"),
            "kinesis:Record"
        );
        assert_eq!(
            determine_event_type("arn:aws:sqs:us-east-1:123:MyQueue"),
            "sqs:Message"
        );
    }
}
