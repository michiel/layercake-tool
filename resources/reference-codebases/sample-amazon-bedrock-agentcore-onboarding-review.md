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
id,label,layer,comment
comp_code_interpreter,cost_estimator_agent,COMPUTE,From 01_code_interpreter/cost_estimator_agent implementation in README
comp_runtime_preparer,runtime_preparer,COMPUTE,From 02_runtime/prepare_agent.py described in README
comp_identity_authorizer,identity_oauth_setup,COMPUTE,From 03_identity/setup_inbound_authorizer.py and README
comp_gateway_lambda,gateway_lambda_handler,COMPUTE,From 04_gateway/src/app.py lambda handler
comp_observability_runner,observability_runner,COMPUTE,From 05_observability/test_observability.py running agent calls
comp_memory_agent,memory_agent,COMPUTE,From 06_memory/test_memory.py memory client usage
comp_custom_weather,custom_weather_agent,COMPUTE,From a1_custom/weather_agent README
data_architecture_input,architecture_description,DATA,Input architecture text passed to cost estimator (01_code_interpreter README)
data_cost_output,cost_estimate,DATA,Cost estimate output from code interpreter agent (01_code_interpreter README)
data_oauth_tokens,oauth_tokens,DATA,OAuth tokens handled in identity samples (03_identity README)
data_gateway_requests,gateway_requests,DATA,API requests flowing through gateway sample (04_gateway README/app.py)
data_observability_logs,observability_events,DATA,Log/trace events collected in observability sample (05_observability README)
data_memory_events,memory_events,DATA,Memory events persisted in 06_memory/test_memory.py
data_weather_payload,weather_payload,DATA,Payloads served by custom weather agent (a1_custom README)
aws_agent_runtime,bedrock_agent_runtime,AWS,Bedrock Agent Runtime referenced in 02_runtime README/template
aws_gateway,bedrock_gateway,AWS,Bedrock Agent Gateway from 04_gateway README
aws_cognito,cognito_oauth,AWS,Cognito OAuth provider setup in 03_identity README
aws_dynamodb,dynamodb_memory_table,AWS,DynamoDB used for memory persistence in 06_memory README
aws_cloudwatch,cloudwatch_observability,AWS,CloudWatch metrics/traces in 05_observability README
aws_s3,s3_artifacts,AWS,S3 artifact storage mentioned in deployment steps (02_runtime README)
aws_lambda,lambda_runtime,AWS,Lambda runtime for gateway/custom agents (04_gateway/src/app.py, a1_custom)
aws_api_gateway,api_gateway_edge,AWS,API Gateway fronting the Lambda gateway (04_gateway README)
```

### Relationship Inventory (edges)
```csv
id,source,target,layer,label,comment
edge_1,data_architecture_input,comp_code_interpreter,DATA,estimation_request,Architecture text passed into cost estimator agent (01_code_interpreter README)
edge_2,comp_code_interpreter,data_cost_output,DATA,cost_estimate,Estimator returns cost estimate output (01_code_interpreter README)
edge_3,comp_runtime_preparer,aws_agent_runtime,AWS,package_deploy,Runtime packaging/deploy step in 02_runtime/prepare_agent.py
edge_4,comp_identity_authorizer,aws_cognito,AWS,configure_oauth,Cognito setup in 03_identity/setup_inbound_authorizer.py
edge_5,data_oauth_tokens,comp_identity_authorizer,DATA,token_handling,Tokens handled in identity sample (03_identity README)
edge_6,comp_gateway_lambda,aws_gateway,AWS,gateway_binding,GW binding described in 04_gateway README/app.py
edge_7,data_gateway_requests,comp_gateway_lambda,DATA,invoke_agent,Requests passed through gateway Lambda (04_gateway)
edge_8,comp_observability_runner,aws_cloudwatch,AWS,emit_metrics_traces,Observability runner sends metrics/traces (05_observability README)
edge_9,comp_memory_agent,aws_dynamodb,AWS,persist_memory,Memory client persists state to DynamoDB (06_memory README)
edge_10,comp_memory_agent,data_memory_events,DATA,store_context,Memory agent produces memory events (06_memory test_memory.py)
edge_11,comp_custom_weather,aws_lambda,AWS,weather_logic_runtime,Custom weather agent runs on Lambda (a1_custom README)
edge_12,comp_custom_weather,data_weather_payload,DATA,weather_response,Weather agent returns payload (a1_custom README)
edge_13,comp_gateway_lambda,aws_api_gateway,AWS,api_frontend,API Gateway fronts gateway Lambda (04_gateway README)
edge_14,comp_code_interpreter,aws_s3,AWS,artifact_fetch_store,Estimator uses S3 artifacts per deployment steps
edge_15,comp_code_interpreter,aws_agent_runtime,AWS,invoke_runtime,Estimator invokes Agent Runtime (01/02 README)
edge_16,comp_observability_runner,data_observability_logs,DATA,run_sessions,Runner collects logs after invocations (05_observability README)
```

### Notes
- Compute nodes align to the README’s capability folders (`01_code_interpreter`, `02_runtime`, `03_identity`, `04_gateway`, `05_observability`, `06_memory`, `a1_custom`).
- Data nodes capture the primary inputs/outputs used across the samples (architecture descriptions, estimates, tokens, requests, memory, weather payloads).
- AWS nodes reflect platform services referenced throughout the guides (Agent Runtime/Gateway, Cognito OAuth, DynamoDB, CloudWatch, S3, Lambda, API Gateway).
- Edge labels summarize the dominant flow or binding; Graphviz/DOT renderers should use `relative_weight` when present for pen widths, but here all edges are single-weight logical links.
