# Backend Integration Notes: Execution Status Publishing

**Date:** 2025-10-20
**Related:** Implementation of real-time execution status updates (Phase 1)
**Status:** Subscription infrastructure complete, publishing integration pending

## Overview

The subscription infrastructure for real-time execution status updates has been implemented on both backend and frontend. However, the **publish calls** to broadcast execution status changes need to be integrated into the backend wherever datasource processing or graph computation occurs.

## What Was Implemented

### Backend

1. **Subscription Endpoint** (`layercake-core/src/graphql/subscriptions/mod.rs:311-348`)
   - New GraphQL subscription: `nodeExecutionStatusChanged(projectId: Int!)`
   - Returns `NodeExecutionStatusEvent` with execution metadata
   - WebSocket-based real-time broadcasting

2. **Event Type** (`layercake-core/src/graphql/types/plan_dag.rs:745-758`)
   ```rust
   pub struct NodeExecutionStatusEvent {
       pub project_id: i32,
       pub node_id: String,
       pub node_type: PlanDagNodeType,
       pub datasource_execution: Option<DataSourceExecutionMetadata>,
       pub graph_execution: Option<GraphExecutionMetadata>,
       pub timestamp: String,
   }
   ```

3. **Broadcaster Infrastructure** (`layercake-core/src/graphql/subscriptions/mod.rs:487-525`)
   - `get_execution_status_broadcaster(project_id)` - gets or creates broadcast channel
   - `publish_execution_status_event(event)` - broadcasts to all subscribers

### Frontend

1. **GraphQL Subscription** (`frontend/src/graphql/plan-dag.ts:314-339`)
   - Subscription query for execution status changes

2. **Query Service** (`frontend/src/services/PlanDagQueryService.ts:244-285`)
   - `subscribeToExecutionStatus()` method handles subscription

3. **CQRS Service** (`frontend/src/services/PlanDagCQRSService.ts:192-205`)
   - `subscribeToExecutionStatusUpdates()` wrapper method

4. **React Hook Integration** (`frontend/src/components/editors/PlanVisualEditor/hooks/usePlanDagCQRS.ts:330-367`)
   - Subscribes to execution status on mount
   - Updates Plan DAG state with execution metadata
   - Updates stable refs to prevent stale data

## What Needs Backend Integration

The `publish_execution_status_event()` function needs to be called whenever execution state changes. Based on the codebase analysis, this occurs in:

### Priority 1: Datasource Processing

**Location:** TBD (needs investigation)

When a datasource is processed and its execution state changes, publish an event:

```rust
use crate::graphql::subscriptions::publish_execution_status_event;
use crate::graphql::types::{NodeExecutionStatusEvent, DataSourceExecutionMetadata, PlanDagNodeType};

// After datasource execution state changes
let event = NodeExecutionStatusEvent {
    project_id,
    node_id: datasource_node_id.clone(),
    node_type: PlanDagNodeType::DataSource,
    datasource_execution: Some(DataSourceExecutionMetadata {
        data_source_id: datasource.id,
        filename: datasource.file_path.clone(),
        status: datasource.status.clone(),
        processed_at: datasource.processed_at.map(|dt| dt.to_rfc3339()),
        execution_state: datasource.execution_state.clone(),
        error_message: datasource.error_message.clone(),
    }),
    graph_execution: None,
    timestamp: chrono::Utc::now().to_rfc3339(),
};

publish_execution_status_event(event).await.ok(); // Fire and forget
```

**Trigger Points:**
- When datasource status changes to `Processing`
- When datasource status changes to `Completed`
- When datasource status changes to `Error`

### Priority 2: Graph Computation

**Location:** TBD (needs investigation)

When a graph is computed and its execution state changes:

```rust
let event = NodeExecutionStatusEvent {
    project_id,
    node_id: graph_node_id.clone(),
    node_type: PlanDagNodeType::Graph,
    datasource_execution: None,
    graph_execution: Some(GraphExecutionMetadata {
        graph_id: graph.id,
        node_count: graph.node_count,
        edge_count: graph.edge_count,
        execution_state: graph.execution_state.clone(),
        computed_date: graph.computed_date.map(|dt| dt.to_rfc3339()),
        error_message: graph.error_message.clone(),
    }),
    timestamp: chrono::Utc::now().to_rfc3339(),
};

publish_execution_status_event(event).await.ok();
```

**Trigger Points:**
- When graph computation starts (`Processing`)
- When graph computation completes (`Completed`)
- When graph computation fails (`Error`)

## Implementation Strategy

### Step 1: Identify Execution State Change Points

Search the codebase for where execution state is updated:

```bash
# Find datasource execution state updates
grep -r "execution_state.*Set\|set_execution_state" layercake-core/src --include="*.rs"

# Find graph execution state updates
grep -r "ExecutionState::" layercake-core/src --include="*.rs"
```

Expected locations:
- `layercake-core/src/database/entities/datasources.rs` - entity methods
- `layercake-core/src/database/entities/graphs.rs` - entity methods
- `layercake-core/src/pipeline/dag_executor.rs` - execution orchestration
- `layercake-core/src/services/data_source_service.rs` - datasource service
- `layercake-core/src/plan_execution.rs` - plan execution

### Step 2: Add Publishing Calls

For each execution state change:

1. **Import dependencies:**
   ```rust
   use crate::graphql::subscriptions::publish_execution_status_event;
   use crate::graphql::types::{NodeExecutionStatusEvent, DataSourceExecutionMetadata, GraphExecutionMetadata, PlanDagNodeType};
   ```

2. **Get node_id mapping:**
   - Execution occurs on datasource/graph entities
   - Need to map from datasource_id/graph_id to Plan DAG node_id
   - Query `plan_dag_nodes` table where config contains the datasource_id/graph_id

3. **Construct and publish event:**
   ```rust
   // Get Plan DAG node ID (example - actual query will vary)
   let node_id = get_plan_dag_node_id_for_datasource(db, project_id, datasource_id).await?;

   let event = NodeExecutionStatusEvent { /* ... */ };

   // Fire and forget - don't block on subscription
   tokio::spawn(async move {
       publish_execution_status_event(event).await.ok();
   });
   ```

### Step 3: Testing

1. **Unit Tests:**
   - Mock execution state changes
   - Verify events are published with correct data

2. **Integration Tests:**
   - Start datasource processing
   - Subscribe to execution status
   - Verify status change events received

3. **Manual Testing:**
   - Open Plan DAG editor
   - Trigger datasource processing
   - Observe execution status badges update in real-time (without reload)

## Helper Function Template

Create a helper module to simplify publishing:

```rust
// layercake-core/src/graphql/execution_events.rs

use crate::database::entities::{data_sources, graphs, plan_dag_nodes};
use crate::graphql::subscriptions::publish_execution_status_event;
use crate::graphql::types::*;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait};

/// Publish datasource execution status change
pub async fn publish_datasource_status(
    db: &DatabaseConnection,
    project_id: i32,
    datasource: &data_sources::Model,
) -> anyhow::Result<()> {
    // Find Plan DAG node containing this datasource
    let node = plan_dag_nodes::Entity::find()
        .filter(plan_dag_nodes::Column::PlanId.eq(project_id))
        .filter(plan_dag_nodes::Column::NodeType.eq("DataSourceNode"))
        .all(db)
        .await?
        .into_iter()
        .find(|n| {
            // Parse config JSON and check if dataSourceId matches
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&n.config_json) {
                config.get("dataSourceId").and_then(|v| v.as_i64()) == Some(datasource.id as i64)
            } else {
                false
            }
        });

    if let Some(node) = node {
        let event = NodeExecutionStatusEvent {
            project_id,
            node_id: node.id.clone(),
            node_type: PlanDagNodeType::DataSource,
            datasource_execution: Some(DataSourceExecutionMetadata {
                data_source_id: datasource.id,
                filename: datasource.file_path.clone(),
                status: datasource.status.clone(),
                processed_at: datasource.processed_at.map(|dt| dt.to_rfc3339()),
                execution_state: datasource.execution_state.clone(),
                error_message: datasource.error_message.clone(),
            }),
            graph_execution: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        // Fire and forget
        tokio::spawn(async move {
            publish_execution_status_event(event).await.ok();
        });
    }

    Ok(())
}

/// Publish graph execution status change
pub async fn publish_graph_status(
    db: &DatabaseConnection,
    project_id: i32,
    graph: &graphs::Model,
) -> anyhow::Result<()> {
    // Similar implementation for graph nodes
    // ...
    Ok(())
}
```

Then use in execution code:

```rust
// When datasource execution state changes
datasource.set_execution_state(ExecutionState::Processing);
datasource.update(db).await?;

// Publish status change
execution_events::publish_datasource_status(db, project_id, &datasource).await.ok();
```

## Current Status

✅ **Completed:**
- Backend subscription infrastructure
- Frontend subscription client
- React state integration
- Type definitions and GraphQL schema

⏸️ **Pending:**
- Identify all execution state change points
- Implement node_id mapping logic
- Add publish calls to execution code
- Test end-to-end real-time updates

## Expected Behaviour After Integration

1. User creates datasource node in Plan DAG editor
2. User triggers datasource processing (via separate UI)
3. **Without reload**, datasource node badge updates:
   - "Pending" → "Processing" (with spinner)
   - "Processing" → "Completed" (with success indicator)
   - Or "Processing" → "Error" (with error indicator)
4. Multiple users see updates in real-time

## References

- Subscription implementation: `layercake-core/src/graphql/subscriptions/mod.rs:311-525`
- Event types: `layercake-core/src/graphql/types/plan_dag.rs:713-758`
- Frontend integration: `frontend/src/components/editors/PlanVisualEditor/hooks/usePlanDagCQRS.ts:330-367`
- Technical review: `docs/docs/reviews/2025-10-20-editor_events.md`
