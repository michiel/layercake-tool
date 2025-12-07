## SAM Python CRUD Sample — Codebase Decomposition

Repository: `resources/reference-codebases/sam-python-crud-sample`

### Comments
- Straightforward SAM stack: Lambda-per-verb plus DynamoDB table; API Gateway fronts handlers. CloudWatch logging assumed via SAM defaults.
- Use coalescing to collapse per-function nodes into file-level if desired; edge weights are scaled (`relative_weight`) in Graphviz exports to avoid extreme pen widths.
- Tests exercise API and DynamoDB directly; keep them included for full coverage unless using support-file exclusions.

### Context
- AWS SAM-based CRUD sample with Lambda functions per CRUD verb, DynamoDB table backing, API Gateway exposure, and tests.
- Functions live under `src/*_activity/app.py`; API definitions in `template.yaml`; tests in `tests/`.

### Component Inventory (nodes)
```csv
id,label,layer
comp_create_fn,CreateActivityFunction,COMPUTE
comp_get_fn,GetActivityFunction,COMPUTE
comp_list_fn,ListActivitiesFunction,COMPUTE
comp_update_fn,UpdateActivityFunction,COMPUTE
comp_delete_fn,DeleteActivityFunction,COMPUTE
comp_tests,ActivityIntegrationTests,COMPUTE
data_activity_request,ActivityRequest,DATA
data_activity_record,ActivityRecord,DATA
data_activity_list,ActivityList,DATA
data_activity_response,ActivityResponse,DATA
data_error_response,ErrorResponse,DATA
aws_dynamodb_table,ActivitiesTable,AWS
aws_api_gateway,APIGatewayCRUD,AWS
aws_lambda_runtime,LambdaRuntime,AWS
aws_cloudwatch,CloudWatchLogs,AWS
```

### Relationship Inventory (edges)
```csv
id,source,target,layer,label
edge_1,data_activity_request,comp_create_fn,DATA,create_request
edge_2,data_activity_request,comp_update_fn,DATA,update_request
edge_3,data_activity_request,comp_delete_fn,DATA,delete_request
edge_4,data_activity_request,comp_get_fn,DATA,get_request
edge_5,data_activity_request,comp_list_fn,DATA,list_request
edge_6,comp_create_fn,data_activity_record,DATA,persisted_item
edge_7,comp_update_fn,data_activity_record,DATA,updated_item
edge_8,comp_delete_fn,data_activity_response,DATA,delete_result
edge_9,comp_get_fn,data_activity_response,DATA,retrieved_item
edge_10,comp_list_fn,data_activity_list,DATA,list_result
edge_11,comp_create_fn,data_error_response,DATA,validation_error
edge_12,comp_update_fn,data_error_response,DATA,validation_error
edge_13,comp_get_fn,data_error_response,DATA,missing_error
edge_14,comp_delete_fn,data_error_response,DATA,missing_error
edge_15,comp_list_fn,data_error_response,DATA,query_error
edge_16,comp_create_fn,aws_dynamodb_table,AWS,put_item
edge_17,comp_update_fn,aws_dynamodb_table,AWS,update_item
edge_18,comp_delete_fn,aws_dynamodb_table,AWS,delete_item
edge_19,comp_get_fn,aws_dynamodb_table,AWS,query_item
edge_20,comp_list_fn,aws_dynamodb_table,AWS,scan_items
edge_21,comp_create_fn,aws_cloudwatch,AWS,emit_logs
edge_22,comp_update_fn,aws_cloudwatch,AWS,emit_logs
edge_23,comp_delete_fn,aws_cloudwatch,AWS,emit_logs
edge_24,comp_get_fn,aws_cloudwatch,AWS,emit_logs
edge_25,comp_list_fn,aws_cloudwatch,AWS,emit_logs
edge_26,aws_api_gateway,comp_create_fn,AWS,invoke_handler
edge_27,aws_api_gateway,comp_update_fn,AWS,invoke_handler
edge_28,aws_api_gateway,comp_delete_fn,AWS,invoke_handler
edge_29,aws_api_gateway,comp_get_fn,AWS,invoke_handler
edge_30,aws_api_gateway,comp_list_fn,AWS,invoke_handler
edge_31,comp_tests,aws_api_gateway,AWS,exercise_endpoints
edge_32,comp_tests,aws_dynamodb_table,AWS,seed_and_assert
edge_33,comp_tests,aws_lambda_runtime,AWS,invoke_locally
```

### Notes
- Compute nodes map to Lambda handlers defined in `template.yaml` and implemented in `src/<verb>_activity/app.py`.
- Data nodes represent HTTP request/response payloads and DynamoDB items flowing through handlers.
- Platform nodes capture the SAM stack resources: DynamoDB table, API Gateway, Lambda runtime, and CloudWatch logging.
- Edges emphasize API → handler invocation, handler ↔ DynamoDB data flow, error/response outputs, and test harness interactions.
