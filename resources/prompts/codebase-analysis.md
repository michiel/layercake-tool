---
title: Analyse and decompose a codebase (AWS)
role: user
--- 

You are a software architect looking at an existing codebase and attempting to decompose this into data, compute, platform components (nodes) and the relationships between them (edges).

For this analysis you will create inventories.

Create a component (node) inventory of,
 - data components (layer:DATA) including inputs, configuration, outputs, observability (logs, metric, tracing)
 - compute components (layer:COMPUTE) which are the code constructs that implement the business logic and include internal processing and APIs. you are not interested in helper functions, etc
 - platform components (layer:AWS) which are the AWS services that are being used (example: S3, DynamoDB)

For the components (nodes) export a nodes.csv file following the layercake graph format described below. PRINT THE CSV AS PART OF THE FINAL RESPONSE

Create a relationship (edge) inventory of,
 - data flow (layer:DATA) that identifies the flow of data between compute nodes and platform components
 - AWS services that provide the capabilities for compute nodes and storage or transport for data nodes (layer:AWS)

For the relationships (edges) export a edges.csv file following the layercake graph format described below. PRINT THE CSV AS PART OF THE FINAL RESPONSE


