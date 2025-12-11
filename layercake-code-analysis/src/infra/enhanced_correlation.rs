use std::collections::HashMap;

use crate::analyzer::AnalysisResult;
use serde::{Deserialize, Serialize};

use super::graph::InfrastructureGraph;
use super::model::CorrelationMatch;

/// Enhanced correlation that links external calls, env vars, and data flows to infrastructure
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EnhancedCorrelationReport {
    pub handler_matches: Vec<CorrelationMatch>,
    pub external_call_matches: Vec<ExternalCallCorrelation>,
    pub env_var_matches: Vec<EnvVarCorrelation>,
    pub data_flow_matches: Vec<DataFlowCorrelation>,
    pub unresolved_external_calls: Vec<String>,
    pub unresolved_env_vars: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalCallCorrelation {
    pub external_call_id: String,
    pub infra_resource_id: String,
    pub call_target: String,
    pub resource_name: String,
    pub reason: String,
    pub confidence: u8,
    pub operation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVarCorrelation {
    pub env_var_name: String,
    pub code_file: String,
    pub infra_resource_id: String,
    pub reason: String,
    pub confidence: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFlowCorrelation {
    pub from_code: Option<String>,
    pub to_code: Option<String>,
    pub from_infra: Option<String>,
    pub to_infra: Option<String>,
    pub flow_type: String,
    pub confidence: u8,
}

pub fn enhanced_correlate(
    code: &AnalysisResult,
    infra: &InfrastructureGraph,
) -> EnhancedCorrelationReport {
    let mut report = EnhancedCorrelationReport::default();

    // Build lookup maps for fast correlation
    let resource_by_type: HashMap<String, Vec<String>> =
        infra
            .resources
            .iter()
            .fold(HashMap::new(), |mut acc, (id, resource)| {
                let type_key = format!("{:?}", resource.resource_type).to_lowercase();
                acc.entry(type_key).or_default().push(id.clone());
                acc
            });

    let _resource_by_name: HashMap<String, String> = infra
        .resources
        .iter()
        .map(|(id, resource)| (resource.name.to_lowercase(), id.clone()))
        .collect();

    // 1. Correlate external calls to infrastructure resources
    for ext_call in &code.external_calls {
        let target_lc = ext_call.target.to_ascii_lowercase();
        let mut matched = false;

        // Match AWS SDK calls to specific resources
        if target_lc.contains("s3") {
            if let Some(buckets) = resource_by_type.get("aws(\"aws_s3_bucket\")") {
                for bucket_id in buckets {
                    if let Some(resource) = infra.resources.get(bucket_id) {
                        // Check if bucket name appears in the call path
                        let bucket_name = resource.name.to_lowercase();
                        if let Some(path) = &ext_call.path {
                            if path.to_lowercase().contains(&bucket_name) {
                                report.external_call_matches.push(ExternalCallCorrelation {
                                    external_call_id: format!(
                                        "{}:{}",
                                        ext_call.file_path, ext_call.line_number
                                    ),
                                    infra_resource_id: bucket_id.clone(),
                                    call_target: ext_call.target.clone(),
                                    resource_name: resource.name.clone(),
                                    reason: format!(
                                        "S3 call references bucket '{}'",
                                        resource.name
                                    ),
                                    confidence: 80,
                                    operation: ext_call.method.clone(),
                                });
                                matched = true;
                            }
                        }

                        // Also check properties for bucket references
                        if !matched {
                            for (_, prop_val) in &resource.properties {
                                if ext_call
                                    .path
                                    .as_ref()
                                    .map(|p| p.contains(prop_val))
                                    .unwrap_or(false)
                                {
                                    report.external_call_matches.push(ExternalCallCorrelation {
                                        external_call_id: format!(
                                            "{}:{}",
                                            ext_call.file_path, ext_call.line_number
                                        ),
                                        infra_resource_id: bucket_id.clone(),
                                        call_target: ext_call.target.clone(),
                                        resource_name: resource.name.clone(),
                                        reason: "S3 call matches bucket property".to_string(),
                                        confidence: 70,
                                        operation: ext_call.method.clone(),
                                    });
                                    matched = true;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Match DynamoDB calls
        if target_lc.contains("dynamodb") {
            if let Some(tables) = resource_by_type.get("aws(\"aws_dynamodb_table\")") {
                for table_id in tables {
                    if let Some(resource) = infra.resources.get(table_id) {
                        let table_name = resource.name.to_lowercase();
                        if let Some(path) = &ext_call.path {
                            if path.to_lowercase().contains(&table_name) {
                                report.external_call_matches.push(ExternalCallCorrelation {
                                    external_call_id: format!(
                                        "{}:{}",
                                        ext_call.file_path, ext_call.line_number
                                    ),
                                    infra_resource_id: table_id.clone(),
                                    call_target: ext_call.target.clone(),
                                    resource_name: resource.name.clone(),
                                    reason: format!(
                                        "DynamoDB call references table '{}'",
                                        resource.name
                                    ),
                                    confidence: 85,
                                    operation: ext_call.method.clone(),
                                });
                                matched = true;
                            }
                        }
                    }
                }
            }
        }

        // Match Lambda invocations
        if target_lc.contains("lambda") && target_lc.contains("invoke") {
            if let Some(functions) = resource_by_type.get("aws(\"aws_lambda_function\")") {
                for func_id in functions {
                    if let Some(resource) = infra.resources.get(func_id) {
                        let func_name = resource.name.to_lowercase();
                        if let Some(path) = &ext_call.path {
                            if path.to_lowercase().contains(&func_name) {
                                report.external_call_matches.push(ExternalCallCorrelation {
                                    external_call_id: format!(
                                        "{}:{}",
                                        ext_call.file_path, ext_call.line_number
                                    ),
                                    infra_resource_id: func_id.clone(),
                                    call_target: ext_call.target.clone(),
                                    resource_name: resource.name.clone(),
                                    reason: format!(
                                        "Lambda invocation targets function '{}'",
                                        resource.name
                                    ),
                                    confidence: 90,
                                    operation: Some("invoke".to_string()),
                                });
                                matched = true;
                            }
                        }
                    }
                }
            }
        }

        // Match SQS/SNS calls
        if target_lc.contains("sqs") || target_lc.contains("sns") {
            let resource_type = if target_lc.contains("sqs") {
                "aws(\"aws_sqs_queue\")"
            } else {
                "aws(\"aws_sns_topic\")"
            };

            if let Some(queues) = resource_by_type.get(resource_type) {
                for queue_id in queues {
                    if let Some(resource) = infra.resources.get(queue_id) {
                        let queue_name = resource.name.to_lowercase();
                        if let Some(path) = &ext_call.path {
                            if path.to_lowercase().contains(&queue_name) {
                                report.external_call_matches.push(ExternalCallCorrelation {
                                    external_call_id: format!(
                                        "{}:{}",
                                        ext_call.file_path, ext_call.line_number
                                    ),
                                    infra_resource_id: queue_id.clone(),
                                    call_target: ext_call.target.clone(),
                                    resource_name: resource.name.clone(),
                                    reason: format!(
                                        "Messaging call references '{}'",
                                        resource.name
                                    ),
                                    confidence: 75,
                                    operation: ext_call.method.clone(),
                                });
                                matched = true;
                            }
                        }
                    }
                }
            }
        }

        if !matched {
            report
                .unresolved_external_calls
                .push(ext_call.target.clone());
        }
    }

    // 2. Correlate environment variables to infrastructure configuration
    for env_var in &code.env_vars {
        let var_name_lc = env_var.name.to_ascii_lowercase();
        let mut matched = false;

        // Search all infrastructure resources for matching environment variable names
        for (resource_id, resource) in &infra.resources {
            // Check if resource has environment configuration
            for (prop_key, prop_val) in &resource.properties {
                let key_lc = prop_key.to_ascii_lowercase();

                // Match environment variable definitions
                if (key_lc.contains("environment") || key_lc.contains("env"))
                    && prop_val.to_lowercase().contains(&var_name_lc)
                {
                    report.env_var_matches.push(EnvVarCorrelation {
                        env_var_name: env_var.name.clone(),
                        code_file: env_var.file_path.clone(),
                        infra_resource_id: resource_id.clone(),
                        reason: format!(
                            "Environment variable '{}' defined in resource '{}'",
                            env_var.name, resource.name
                        ),
                        confidence: 85,
                    });
                    matched = true;
                }

                // Match specific patterns like TABLE_NAME -> DynamoDB table
                if var_name_lc.contains("table") && var_name_lc.contains("name") {
                    if format!("{:?}", resource.resource_type)
                        .to_lowercase()
                        .contains("dynamodb")
                    {
                        let resource_name_lc = resource.name.to_lowercase();
                        if prop_val.to_lowercase().contains(&resource_name_lc)
                            || prop_val.to_lowercase().contains("table")
                        {
                            report.env_var_matches.push(EnvVarCorrelation {
                                env_var_name: env_var.name.clone(),
                                code_file: env_var.file_path.clone(),
                                infra_resource_id: resource_id.clone(),
                                reason: format!(
                                    "Env var '{}' likely references DynamoDB table '{}'",
                                    env_var.name, resource.name
                                ),
                                confidence: 70,
                            });
                            matched = true;
                        }
                    }
                }

                // Match BUCKET patterns
                if var_name_lc.contains("bucket") {
                    if format!("{:?}", resource.resource_type)
                        .to_lowercase()
                        .contains("s3")
                    {
                        report.env_var_matches.push(EnvVarCorrelation {
                            env_var_name: env_var.name.clone(),
                            code_file: env_var.file_path.clone(),
                            infra_resource_id: resource_id.clone(),
                            reason: format!(
                                "Env var '{}' likely references S3 bucket '{}'",
                                env_var.name, resource.name
                            ),
                            confidence: 70,
                        });
                        matched = true;
                    }
                }
            }
        }

        if !matched {
            report.unresolved_env_vars.push(env_var.name.clone());
        }
    }

    // 3. Infer data flow between code and infrastructure based on patterns
    // For example: function reads from S3, writes to DynamoDB
    for function in &code.functions {
        let _func_name_lc = function.name.to_lowercase();

        // Check if function interacts with specific resource types
        for call in &function.calls {
            let call_lc = call.to_lowercase();

            // S3 read operations
            if call_lc.contains("s3") && (call_lc.contains("get") || call_lc.contains("read")) {
                if let Some(buckets) = resource_by_type.get("aws(\"aws_s3_bucket\")") {
                    for bucket_id in buckets {
                        report.data_flow_matches.push(DataFlowCorrelation {
                            from_infra: Some(bucket_id.clone()),
                            to_code: Some(format!("{}::{}", function.file_path, function.name)),
                            from_code: None,
                            to_infra: None,
                            flow_type: "s3_read".to_string(),
                            confidence: 60,
                        });
                    }
                }
            }

            // DynamoDB operations
            if call_lc.contains("dynamodb")
                && (call_lc.contains("query")
                    || call_lc.contains("get")
                    || call_lc.contains("scan"))
            {
                if let Some(tables) = resource_by_type.get("aws(\"aws_dynamodb_table\")") {
                    for table_id in tables {
                        report.data_flow_matches.push(DataFlowCorrelation {
                            from_infra: Some(table_id.clone()),
                            to_code: Some(format!("{}::{}", function.file_path, function.name)),
                            from_code: None,
                            to_infra: None,
                            flow_type: "dynamodb_read".to_string(),
                            confidence: 60,
                        });
                    }
                }
            }

            // Write operations
            if call_lc.contains("dynamodb")
                && (call_lc.contains("put") || call_lc.contains("update"))
            {
                if let Some(tables) = resource_by_type.get("aws(\"aws_dynamodb_table\")") {
                    for table_id in tables {
                        report.data_flow_matches.push(DataFlowCorrelation {
                            from_code: Some(format!("{}::{}", function.file_path, function.name)),
                            to_infra: Some(table_id.clone()),
                            from_infra: None,
                            to_code: None,
                            flow_type: "dynamodb_write".to_string(),
                            confidence: 60,
                        });
                    }
                }
            }
        }
    }

    if report.external_call_matches.is_empty()
        && !code.external_calls.is_empty()
        && !infra.resources.is_empty()
    {
        report.warnings.push(
            "External calls detected but could not correlate to infrastructure resources"
                .to_string(),
        );
    }

    if report.env_var_matches.is_empty() && !code.env_vars.is_empty() {
        report.warnings.push(
            "Environment variables detected but could not correlate to infrastructure".to_string(),
        );
    }

    report
}
