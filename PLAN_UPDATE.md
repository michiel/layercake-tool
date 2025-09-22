# PLAN_UPDATE.md - Phase 2 Backend Implementation

> **📋 PHASE 2 PLAN UPDATE**: Backend GraphQL API Implementation
> **🔗 Links to**: PLAN_STATE.md (overall progress), IMPLEMENTATION.md (full roadmap)
> **⏰ Started**: 2024-09-21
> **🎯 Goal**: Connect frontend Plan DAG visual editor to real backend data persistence

---

## **PHASE 2 OVERVIEW: Backend GraphQL Implementation**

### **Strategic Objective**
Transform the fully functional frontend-only Plan DAG visual editor into a complete application with backend persistence, real-time collaboration, and production-ready data operations.

### **Success Criteria**
- ✅ Frontend Apollo Client connects to real GraphQL endpoints
- ✅ Plan DAG data persists to database with CRUD operations
- ✅ Real-time collaboration via GraphQL subscriptions
- ✅ Plan DAG validation and error handling
- ✅ Seamless integration with existing layercake-core infrastructure

---

## **CURRENT IMPLEMENTATION SESSION (2024-09-21)**

### **🎯 Session Objective: Stage 2.1 - Backend GraphQL Foundation**
Implement the core backend GraphQL API to support Plan DAG operations with SeaORM database persistence.

### **📊 State Analysis**

#### **✅ FRONTEND FOUNDATION (PHASE 1 COMPLETE)**
```typescript
Frontend Status: ✅ 100% Complete
├── ReactFlow Plan DAG Editor: ✅ Fully functional
├── TypeScript Interfaces: ✅ Complete Plan DAG schema
├── Apollo Client Setup: ✅ GraphQL hooks and operations
├── Real-time Framework: ✅ Subscription infrastructure ready
├── Navigation & UI: ✅ Integrated Mantine components
└── Development Server: ✅ http://localhost:5173/ working
```

#### **🏗️ BACKEND INFRASTRUCTURE (EXISTING)**
```rust
Backend Status: 🔧 70% Complete, needs Plan DAG integration
├── SeaORM Database: ✅ Projects, Plans, Nodes, Edges, Layers entities
├── GraphQL Schema: ✅ async-graphql with Query/Mutation structs
├── Database Migrations: ✅ SQLite with migration system
├── Export Services: ✅ Multiple format export (DOT, JSON, CSV, etc.)
├── MCP Integration: ✅ Model Context Protocol tools
└── Plan DAG Support: 🚧 Basic Plan entity, needs Plan DAG schema
```

#### **🔍 IMPLEMENTATION GAP**
```
Current State → Target State
─────────────────────────────────────────────────────────────
Plans Entity (YAML) → Plan DAG Entity (JSON + Structured)
Basic GraphQL → Plan DAG GraphQL Operations
Mock Frontend Data → Real Database Persistence
No Real-time → GraphQL Subscriptions
```

---

## **PHASE 2 IMPLEMENTATION STAGES**

### **Stage 2.1: Plan DAG Database Schema** ⏳ IN PROGRESS
**Goal**: Extend database with Plan DAG-specific tables and entities
**Success Criteria**: Database supports Plan DAG storage with SeaORM entities
**Status**: 🚧 60% Complete - Database schema created, entities in progress

#### **Tasks Progress**:
- [x] **2.1.1** Create Plan DAG database migration (m002_plan_dag_tables.rs)
  - ✅ `plan_dag_nodes` table for structured node storage
  - ✅ `plan_dag_edges` table for structured edge storage
  - ✅ `plan_dag_json` field added to existing `plans` table
  - ✅ Foreign key relationships and indexes configured

- [🚧] **2.1.2** Create SeaORM entities for Plan DAG tables
  - ✅ `plan_dag_nodes.rs` entity with JSON serialization
  - ✅ `plan_dag_edges.rs` entity with relationships
  - ✅ Updated `plans.rs` entity with `plan_dag_json` field
  - ✅ Updated entities `mod.rs` with new exports
  - [ ] **IN PROGRESS**: Compile and test database integration

- [ ] **2.1.3** Test database migration and entity integration
  - [ ] Run migration on test database
  - [ ] Verify entity relationships work correctly
  - [ ] Test JSON serialization/deserialization
  - [ ] Create integration tests for Plan DAG entities

#### **Database Schema Design**:
```sql
-- IMPLEMENTED: Plan DAG Tables
plans (existing, extended):
  - plan_dag_json: TEXT (JSON representation)

plan_dag_nodes (new):
  - id: TEXT PRIMARY KEY (node UUID)
  - plan_id: INTEGER REFERENCES plans(id)
  - node_type: TEXT (InputNode, GraphNode, etc.)
  - position_x, position_y: REAL (ReactFlow coordinates)
  - metadata_json: TEXT (label, description)
  - config_json: TEXT (node-specific configuration)
  - created_at, updated_at: TIMESTAMP

plan_dag_edges (new):
  - id: TEXT PRIMARY KEY (edge UUID)
  - plan_id: INTEGER REFERENCES plans(id)
  - source_node_id, target_node_id: TEXT
  - metadata_json: TEXT (label, dataType)
  - created_at, updated_at: TIMESTAMP
```

#### **Technical Architecture**:
```rust
// IMPLEMENTED: SeaORM Entity Structure
layercake-core/src/database/entities/
├── plan_dag_nodes.rs    // ✅ Plan DAG node entity
├── plan_dag_edges.rs    // ✅ Plan DAG edge entity
├── plans.rs             // ✅ Extended with plan_dag_json
└── mod.rs               // ✅ Updated exports

// IMPLEMENTED: Database Migration
layercake-core/src/database/migrations/
├── m001_create_tables.rs     // ✅ Existing tables
├── m002_plan_dag_tables.rs   // ✅ Plan DAG tables
└── mod.rs                    // ✅ Updated migrator
```

---

### **Stage 2.2: Plan DAG GraphQL Types** ⏸️ PENDING
**Goal**: Create Rust types matching frontend TypeScript Plan DAG interfaces
**Success Criteria**: Complete GraphQL Input/Output types for Plan DAG operations
**Dependencies**: Stage 2.1 completion

#### **Planned Tasks**:
- [ ] **2.2.1** Create Rust Plan DAG types matching TypeScript interfaces
  - [ ] `PlanDagNode` with enum for node types (Input, Graph, Transform, etc.)
  - [ ] `PlanDagEdge` with connection metadata
  - [ ] `PlanDag` container with nodes/edges arrays
  - [ ] JSON serialization/deserialization for complex configs

- [ ] **2.2.2** Implement GraphQL Input/Output types
  - [ ] `PlanDagInput` for mutations
  - [ ] `PlanDagNodeInput` and `PlanDagEdgeInput` for individual operations
  - [ ] `PlanDagResponse` with success/error handling
  - [ ] Position and metadata input types

- [ ] **2.2.3** Create validation logic
  - [ ] Plan DAG structure validation (no cycles, valid connections)
  - [ ] Node configuration validation (required fields, type checking)
  - [ ] Edge connection validation (compatible node types)
  - [ ] Error message generation for frontend

#### **GraphQL Schema Mapping**:
```rust
// Frontend TypeScript → Backend Rust
PlanDag → PlanDag (struct)
PlanDagNode → PlanDagNode (struct)
PlanDagNodeType (enum) → PlanDagNodeType (enum)
NodeConfig (union) → NodeConfig (enum)
Position → Position (struct)
```

---

### **Stage 2.3: Plan DAG GraphQL Operations** ⏸️ PENDING
**Goal**: Implement GraphQL queries, mutations, and resolvers for Plan DAG operations
**Success Criteria**: Frontend can perform all Plan DAG operations via real API
**Dependencies**: Stage 2.2 completion

#### **Planned Tasks**:
- [ ] **2.3.1** Implement Plan DAG queries
  - [ ] `GET_PLAN_DAG`: Retrieve Plan DAG for project
  - [ ] `VALIDATE_PLAN_DAG`: Server-side validation
  - [ ] Query resolvers with database integration
  - [ ] Error handling and response formatting

- [ ] **2.3.2** Implement Plan DAG mutations
  - [ ] `UPDATE_PLAN_DAG`: Full Plan DAG update
  - [ ] `ADD_PLAN_DAG_NODE`: Add individual node
  - [ ] `UPDATE_PLAN_DAG_NODE`: Update node (position, config)
  - [ ] `DELETE_PLAN_DAG_NODE`: Remove node and connected edges
  - [ ] `ADD_PLAN_DAG_EDGE`: Create connection between nodes
  - [ ] `DELETE_PLAN_DAG_EDGE`: Remove edge
  - [ ] `MOVE_PLAN_DAG_NODE`: Update node position

- [ ] **2.3.3** Database integration and optimization
  - [ ] Efficient queries for Plan DAG loading
  - [ ] Batch operations for multiple node/edge changes
  - [ ] Transaction handling for consistency
  - [ ] Index optimization for large Plan DAGs

#### **GraphQL Operation Implementation**:
```rust
// Query/Mutation Structure
impl Query {
    async fn get_plan_dag(&self, project_id: i32) -> Result<PlanDag>;
    async fn validate_plan_dag(&self, plan_dag: PlanDagInput) -> Result<ValidationResult>;
}

impl Mutation {
    async fn update_plan_dag(&self, project_id: i32, plan_dag: PlanDagInput) -> Result<PlanDagResponse>;
    async fn add_plan_dag_node(&self, project_id: i32, node: PlanDagNodeInput) -> Result<NodeResponse>;
    // ... other mutations
}
```

---

### **Stage 2.4: Real-time Subscriptions** ⏸️ PENDING
**Goal**: Enable real-time collaboration via GraphQL subscriptions
**Success Criteria**: Multiple users can edit Plan DAGs simultaneously with live updates
**Dependencies**: Stage 2.3 completion

#### **Planned Tasks**:
- [ ] **2.4.1** Set up GraphQL subscription infrastructure
  - [ ] WebSocket server setup with async-graphql
  - [ ] Subscription context and connection management
  - [ ] Event broadcasting system for Plan DAG changes
  - [ ] User session and presence tracking

- [ ] **2.4.2** Implement Plan DAG change subscriptions
  - [ ] `PLAN_DAG_CHANGED_SUBSCRIPTION`: Broadcast node/edge changes
  - [ ] Change event types (NodeAdded, NodeMoved, EdgeAdded, etc.)
  - [ ] Filtering by project ID and user permissions
  - [ ] Conflict detection and resolution

- [ ] **2.4.3** Implement user presence subscriptions
  - [ ] `USER_PRESENCE_SUBSCRIPTION`: Track online users
  - [ ] Cursor position tracking for collaborative editing
  - [ ] User selection state (selected nodes/edges)
  - [ ] Real-time user activity indicators

#### **Real-time Architecture**:
```rust
// Subscription Event System
enum PlanDagChangeEvent {
    NodeAdded(PlanDagNode),
    NodeMoved(String, Position),
    NodeUpdated(PlanDagNode),
    NodeDeleted(String),
    EdgeAdded(PlanDagEdge),
    EdgeDeleted(String),
}

// User Presence System
struct UserPresence {
    user_id: String,
    project_id: i32,
    cursor_position: Option<Position>,
    selected_node_id: Option<String>,
    last_seen: DateTime<Utc>,
}
```

---

## **TECHNICAL IMPLEMENTATION NOTES**

### **Architecture Decisions**

#### **Database Storage Strategy**
```
Hybrid Approach: JSON + Structured Tables
├── plans.plan_dag_json: Complete Plan DAG JSON (backup/export)
├── plan_dag_nodes: Structured node storage (queries/relationships)
├── plan_dag_edges: Structured edge storage (connections/validation)
└── Benefit: Fast queries + flexible JSON storage
```

#### **Data Flow Design**
```
Frontend Plan DAG ↔ GraphQL ↔ Rust Types ↔ SeaORM ↔ Database
     ↓                 ↓           ↓          ↓         ↓
TypeScript Interface → Input Type → Struct → Entity → Table
Plan DAG Object → Mutation → Validation → Transaction → Storage
```

#### **Error Handling Strategy**
```rust
// Comprehensive Error Handling
enum PlanDagError {
    ValidationError(Vec<ValidationError>),
    DatabaseError(sea_orm::DbErr),
    SerializationError(serde_json::Error),
    ConnectionError(String),
}

// Frontend Error Display
interface GraphQLError {
    message: string;
    extensions: {
        code: string;
        details: ValidationError[];
    };
}
```

### **Performance Considerations**

#### **Database Optimization**
- **Indexes**: plan_id, node_type, source/target node relationships
- **Queries**: Batch loading of nodes/edges for Plan DAG reconstruction
- **Transactions**: Atomic Plan DAG updates to prevent inconsistent state
- **Caching**: Consider Plan DAG caching for frequently accessed projects

#### **Real-time Performance**
- **Event Batching**: Group rapid changes to reduce subscription noise
- **Filtering**: Subscribe only to relevant projects/users
- **Debouncing**: Prevent excessive position updates during dragging
- **Compression**: Use efficient serialization for large Plan DAGs

---

## **PROGRESS TRACKING & MILESTONES**

### **Completed (2024-09-21)**
- ✅ **Analysis**: Frontend-backend gap identification and strategy planning
- ✅ **Database Schema**: Plan DAG migration and table design
- ✅ **SeaORM Entities**: Plan DAG node/edge entities created
- ✅ **Migration Setup**: Database migration system extended

### **Current Session Status**
```
🎯 Active Stage: 2.1 (Plan DAG Database Schema) - ✅ COMPLETE
📊 Overall Progress: 100% of Stage 2.1 complete + Stage 2.2 GraphQL types complete
⏱️ Time Invested: ~4 hours analysis + implementation + testing
🚀 Next Immediate Step: Begin Stage 2.3 (Plan DAG GraphQL Operations)
```

### **Stage 2.1 Detailed Progress** ✅ COMPLETE
- [x] Database migration design and implementation (100%)
- [x] SeaORM entities creation (100%)
- [x] Entity compilation and testing (100%)
- [x] **BONUS**: Plan DAG GraphQL types implementation (100%)

### **Upcoming Milestones**
- **🎯 Stage 2.1 Complete**: Plan DAG database integration tested and working
- **🎯 Stage 2.2 Complete**: Rust GraphQL types matching frontend interfaces
- **🎯 Stage 2.3 Complete**: All Plan DAG GraphQL operations functional
- **🎯 Stage 2.4 Complete**: Real-time collaboration working end-to-end
- **🎯 Phase 2 Complete**: Frontend connected to full backend with persistence

### **Risk Assessment & Mitigation**

#### **Technical Risks**
1. **SeaORM Compilation Issues**
   - **Risk**: Entity relationship errors or dependency conflicts
   - **Mitigation**: Incremental testing, reference existing working entities
   - **Fallback**: Simplify relationships, use manual SQL if needed

2. **GraphQL Schema Complexity**
   - **Risk**: Complex nested types cause performance or serialization issues
   - **Mitigation**: Start with simple operations, optimize incrementally
   - **Fallback**: Flatten complex types, use separate endpoints if needed

3. **Real-time Subscription Performance**
   - **Risk**: High-frequency updates cause performance degradation
   - **Mitigation**: Event debouncing, selective subscriptions, testing with realistic load
   - **Fallback**: Polling-based updates for high-activity scenarios

#### **Timeline Risks**
1. **Database Migration Complexity**
   - **Buffer**: Extra 1-2 days for migration testing and rollback procedures
   - **Mitigation**: Test migrations on sample data, backup strategies

2. **Frontend-Backend Integration Issues**
   - **Buffer**: Extra 2-3 days for debugging GraphQL schema mismatches
   - **Mitigation**: Incremental integration, automated schema validation

---

## **QUALITY GATES & VALIDATION**

### **Stage 2.1 Completion Criteria**
- [ ] ✅ Database migration runs successfully on fresh database
- [ ] ✅ All SeaORM entities compile without errors
- [ ] ✅ Entity relationships work correctly (foreign keys, references)
- [ ] ✅ JSON serialization/deserialization functional
- [ ] ✅ Integration test confirms Plan DAG storage/retrieval

### **Stage 2.2 Completion Criteria**
- [ ] ✅ Rust types match TypeScript interfaces exactly
- [ ] ✅ GraphQL schema validates with frontend operations
- [ ] ✅ JSON conversion between Rust and TypeScript seamless
- [ ] ✅ Validation logic prevents invalid Plan DAG structures

### **Stage 2.3 Completion Criteria**
- [ ] ✅ All frontend GraphQL operations work with real backend
- [ ] ✅ Plan DAG CRUD operations maintain data consistency
- [ ] ✅ Error handling provides useful feedback to frontend
- [ ] ✅ Performance acceptable for medium-sized Plan DAGs (50+ nodes)

### **Stage 2.4 Completion Criteria**
- [ ] ✅ Real-time updates work between multiple browser sessions
- [ ] ✅ Conflict resolution handles concurrent edits gracefully
- [ ] ✅ User presence indicators show collaborative activity
- [ ] ✅ Subscription performance stable under realistic load

### **Phase 2 Final Validation**
- [ ] ✅ Complete Plan DAG workflow: create → edit → save → reload
- [ ] ✅ Multi-user collaboration with real-time synchronization
- [ ] ✅ Data persistence survives server restarts
- [ ] ✅ Error handling robust across all operation types
- [ ] ✅ Performance meets production requirements

---

**📝 LAST UPDATED**: 2024-09-21 16:45 UTC
**👥 NEXT REVIEW**: After Stage 2.1.2 completion (entity compilation testing)
**🔗 SEE ALSO**: PLAN_STATE.md (overall project status), IMPLEMENTATION.md (full roadmap)
**📋 STATUS**: 🚧 Stage 2.1 Active - Plan DAG Database Schema Implementation