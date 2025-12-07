## Amazon Bedrock AgentCore Onboarding — Codebase Decomposition

Repository: `resources/reference-codebases/sample-amazon-bedrock-agentcore-onboarding`

### Comments
- Samples are runnable and well-documented; keeping infra/test exclusion toggles off is recommended when exploring full context.
- Graph export uses scaled edge weights (`relative_weight`) to avoid oversized pen widths; raw weights are preserved for other uses.
- Node IDs/labels are already file- and capability-oriented; coalescing can further simplify views by collapsing functions into files.
- If exporting to DOT, prefer rendering with `use_edge_weight=true` to leverage `relative_weight`; for JSON consumers, keep `relative_weight` and `weight` aligned.

### Context
- Progressive samples that demonstrate AgentCore capabilities: code interpreter, runtime deployment, identity, gateway, observability, memory, and a custom appendix.
- Workflows combine Python agents/tests with AWS platform resources (Agent Runtime, Cognito OAuth, API Gateway/Lambda, DynamoDB, CloudWatch).

### Component Inventory (nodes)
```csv
id,label,layer
comp_code_interpreter,cost_estimator_agent,COMPUTE
comp_runtime_preparer,runtime_preparer,COMPUTE
comp_identity_authorizer,identity_oauth_setup,COMPUTE
comp_gateway_lambda,gateway_lambda_handler,COMPUTE
comp_observability_runner,observability_runner,COMPUTE
comp_memory_agent,memory_agent,COMPUTE
comp_custom_weather,custom_weather_agent,COMPUTE
data_architecture_input,architecture_description,DATA
data_cost_output,cost_estimate,DATA
data_oauth_tokens,oauth_tokens,DATA
data_gateway_requests,gateway_requests,DATA
data_observability_logs,observability_events,DATA
data_memory_events,memory_events,DATA
data_weather_payload,weather_payload,DATA
aws_agent_runtime,bedrock_agent_runtime,AWS
aws_gateway,bedrock_gateway,AWS
aws_cognito,cognito_oauth,AWS
aws_dynamodb,dynamodb_memory_table,AWS
aws_cloudwatch,cloudwatch_observability,AWS
aws_s3,s3_artifacts,AWS
aws_lambda,lambda_runtime,AWS
aws_api_gateway,api_gateway_edge,AWS
```

### Relationship Inventory (edges)
```csv
id,source,target,layer,label
edge_1,data_architecture_input,comp_code_interpreter,DATA,estimation_request
edge_2,comp_code_interpreter,data_cost_output,DATA,cost_estimate
edge_3,comp_runtime_preparer,aws_agent_runtime,AWS,package_deploy
edge_4,comp_identity_authorizer,aws_cognito,AWS,configure_oauth
edge_5,data_oauth_tokens,comp_identity_authorizer,DATA,token_handling
edge_6,comp_gateway_lambda,aws_gateway,AWS,gateway_binding
edge_7,data_gateway_requests,comp_gateway_lambda,DATA,invoke_agent
edge_8,comp_observability_runner,aws_cloudwatch,AWS,emit_metrics_traces
edge_9,comp_memory_agent,aws_dynamodb,AWS,persist_memory
edge_10,comp_memory_agent,data_memory_events,DATA,store_context
edge_11,comp_custom_weather,aws_lambda,AWS,weather_logic_runtime
edge_12,comp_custom_weather,data_weather_payload,DATA,weather_response
edge_13,comp_gateway_lambda,aws_api_gateway,AWS,api_frontend
edge_14,comp_code_interpreter,aws_s3,AWS,artifact_fetch_store
edge_15,comp_code_interpreter,aws_agent_runtime,AWS,invoke_runtime
edge_16,comp_observability_runner,data_observability_logs,DATA,run_sessions
```

### Notes
- Compute nodes align to the README’s capability folders (`01_code_interpreter`, `02_runtime`, `03_identity`, `04_gateway`, `05_observability`, `06_memory`, `a1_custom`).
- Data nodes capture the primary inputs/outputs used across the samples (architecture descriptions, estimates, tokens, requests, memory, weather payloads).
- AWS nodes reflect platform services referenced throughout the guides (Agent Runtime/Gateway, Cognito OAuth, DynamoDB, CloudWatch, S3, Lambda, API Gateway).
- Edge labels summarize the dominant flow or binding; Graphviz/DOT renderers should use `relative_weight` when present for pen widths, but here all edges are single-weight logical links.
