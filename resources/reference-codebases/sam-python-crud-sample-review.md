## SAM Python CRUD Sample — Codebase Decomposition

Repository: `resources/reference-codebases/sam-python-crud-sample`

### Comments
- Straightforward SAM stack: Lambda-per-verb plus DynamoDB table; API Gateway fronts handlers. CloudWatch logging assumed via SAM defaults.
- Use coalescing to collapse per-function nodes into file-level if desired; edge weights are scaled (`relative_weight`) in Graphviz exports to avoid extreme pen widths.
- Tests exercise API and DynamoDB directly; keep them included for full coverage unless using support-file exclusions.
- When exporting to DOT/Graphviz, enable `use_edge_weight` to use `relative_weight` for pen width; leave raw `weight` available for analytics or aggregation.

### Context
- AWS SAM-based CRUD sample with Lambda functions per CRUD verb, DynamoDB table backing, API Gateway exposure, and tests.
- Functions live under `src/*_activity/app.py`; API definitions in `template.yaml`; tests in `tests/`.

### Component Inventory (nodes)
```csv
id,label,layer,is_partition,belongs_to,comment
root_scope,Codebase Root,SCOPE,true,,Logical root for all nodes
comp_create_fn,Create Activity Function,COMPUTE,false,root_scope,Lambda handler defined in template.yaml and src/create_activity/app.py
comp_get_fn,Get Activity Function,COMPUTE,false,root_scope,Lambda handler defined in template.yaml and src/get_activity/app.py
comp_list_fn,List Activities Function,COMPUTE,false,root_scope,Lambda handler defined in template.yaml and src/list_activities/app.py
comp_update_fn,Update Activity Function,COMPUTE,false,root_scope,Lambda handler defined in template.yaml and src/update_activity/app.py
comp_delete_fn,Delete Activity Function,COMPUTE,false,root_scope,Lambda handler defined in template.yaml and src/delete_activity/app.py
comp_tests,Activity Integration Tests,COMPUTE,false,root_scope,Test suite in tests/*.py invoking API/Lambda
data_activity_request,Activity Request,DATA,false,root_scope,HTTP request payloads (template.yaml + tests)
data_activity_record,Activity Record,DATA,false,root_scope,DynamoDB item representing an activity (app.py files)
data_activity_list,Activity List,DATA,false,root_scope,Collection returned by list handler (list_activities/app.py)
data_activity_response,Activity Response,DATA,false,root_scope,Successful response bodies (handlers/tests)
data_error_response,Error Response,DATA,false,root_scope,Error payloads for validation/missing items (handlers)
aws_dynamodb_table,Activities Table,AWS,false,root_scope,DynamoDB table in template.yaml
aws_api_gateway,API Gateway CRUD,AWS,false,root_scope,API Gateway defined in template.yaml
aws_lambda_runtime,Lambda Runtime,AWS,false,root_scope,Lambda runtime environment for handlers (SAM)
aws_cloudwatch,CloudWatch Logs,AWS,false,root_scope,Logging for Lambda and API Gateway (SAM defaults)
```

### Relationship Inventory (edges)
```csv
id,source,target,layer,label,comment
edge_1,data_activity_request,comp_create_fn,DATA,create_request,Request payload into create handler (tests + template)
edge_2,data_activity_request,comp_update_fn,DATA,update_request,Request payload into update handler
edge_3,data_activity_request,comp_delete_fn,DATA,delete_request,Request payload into delete handler
edge_4,data_activity_request,comp_get_fn,DATA,get_request,Request payload into get handler
edge_5,data_activity_request,comp_list_fn,DATA,list_request,Request payload into list handler
edge_6,comp_create_fn,data_activity_record,DATA,persisted_item,Create writes item to DynamoDB (src/create_activity/app.py)
edge_7,comp_update_fn,data_activity_record,DATA,updated_item,Update writes updated item
edge_8,comp_delete_fn,data_activity_response,DATA,delete_result,Delete returns deletion result
edge_9,comp_get_fn,data_activity_response,DATA,retrieved_item,Get returns fetched item
edge_10,comp_list_fn,data_activity_list,DATA,list_result,List returns list of items
edge_11,comp_create_fn,data_error_response,DATA,validation_error,Create returns error payloads on validation failure
edge_12,comp_update_fn,data_error_response,DATA,validation_error,Update returns error payloads on validation failure
edge_13,comp_get_fn,data_error_response,DATA,missing_error,Get returns missing/error payloads
edge_14,comp_delete_fn,data_error_response,DATA,missing_error,Delete returns missing/error payloads
edge_15,comp_list_fn,data_error_response,DATA,query_error,List returns error payloads on scan error
edge_16,comp_create_fn,aws_dynamodb_table,AWS,put_item,Create uses DynamoDB put_item (app.py + template)
edge_17,comp_update_fn,aws_dynamodb_table,AWS,update_item,Update uses DynamoDB update_item
edge_18,comp_delete_fn,aws_dynamodb_table,AWS,delete_item,Delete uses DynamoDB delete_item
edge_19,comp_get_fn,aws_dynamodb_table,AWS,query_item,Get uses DynamoDB query/get
edge_20,comp_list_fn,aws_dynamodb_table,AWS,scan_items,List uses DynamoDB scan
edge_21,comp_create_fn,aws_cloudwatch,AWS,emit_logs,Create logs via Lambda/SAM defaults
edge_22,comp_update_fn,aws_cloudwatch,AWS,emit_logs,Update logs via Lambda/SAM defaults
edge_23,comp_delete_fn,aws_cloudwatch,AWS,emit_logs,Delete logs via Lambda/SAM defaults
edge_24,comp_get_fn,aws_cloudwatch,AWS,emit_logs,Get logs via Lambda/SAM defaults
edge_25,comp_list_fn,aws_cloudwatch,AWS,emit_logs,List logs via Lambda/SAM defaults
edge_26,aws_api_gateway,comp_create_fn,AWS,invoke_handler,API Gateway routes to create handler (template.yaml)
edge_27,aws_api_gateway,comp_update_fn,AWS,invoke_handler,API Gateway routes to update handler
edge_28,aws_api_gateway,comp_delete_fn,AWS,invoke_handler,API Gateway routes to delete handler
edge_29,aws_api_gateway,comp_get_fn,AWS,invoke_handler,API Gateway routes to get handler
edge_30,aws_api_gateway,comp_list_fn,AWS,invoke_handler,API Gateway routes to list handler
edge_31,comp_tests,aws_api_gateway,AWS,exercise_endpoints,Tests call API endpoints (tests/*.py)
edge_32,comp_tests,aws_dynamodb_table,AWS,seed_and_assert,Tests seed/query DynamoDB for assertions
edge_33,comp_tests,aws_lambda_runtime,AWS,invoke_locally,Tests may invoke Lambda locally via SAM tooling
```

### Notes
- Compute nodes map to Lambda handlers defined in `template.yaml` and implemented in `src/<verb>_activity/app.py`.
- Data nodes represent HTTP request/response payloads and DynamoDB items flowing through handlers.
- Platform nodes capture the SAM stack resources: DynamoDB table, API Gateway, Lambda runtime, and CloudWatch logging.
- Edges emphasize API → handler invocation, handler ↔ DynamoDB data flow, error/response outputs, and test harness interactions.
