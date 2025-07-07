# Layercake Implementation Roadmap
## Transition from Linear Pipeline to DAG-Based Graph Platform

### 🎯 **Vision**: Transform Layercake from a traditional plan-runner into a sophisticated DAG-based graph inspection and collaboration platform

---

## **Phase 1: Core DAG Architecture Foundation** 
**Priority: CRITICAL | Timeline: 4-5 weeks**

### **1.1 Database Schema Migration (Week 1-2)**
**Target**: Implement plan-centric graph artifact model

- [ ] **Create `plan_nodes` table** - Core DAG node entities
  - `id`, `plan_id`, `node_type` (import/transform/export), `name`, `configuration`
  - `graph_id` (nullable - set after execution), `position_x`, `position_y`
  - Relations: `plan_id` → `plans`, `graph_id` → `graphs`

- [ ] **Create `graphs` table** - Plan execution artifacts  
  - `id`, `plan_id`, `plan_node_id`, `name`, `description`
  - `graph_data` (JSON), `metadata` (execution stats), timestamps
  - Relations: `plan_id` → `plans`, `plan_node_id` → `plan_nodes`

- [ ] **Migrate existing data** 
  - Convert current project-linked nodes/edges/layers to plan-centric graphs
  - Create default plan nodes for existing plans
  - Update foreign key relationships

- [ ] **Add plan execution scope tracking**
  - Track which graphs are available at each execution step
  - Store plan node execution status and graph references

### **1.2 DAG Plan Execution Engine (Week 2-3)**
**Target**: Replace linear YAML execution with JSON DAG processing

- [ ] **JSON DAG Plan Schema Implementation**
  - Define flat DAG structure: `{ nodes: [], edges: [] }` 
  - Plan node types: import, transform, export with configuration
  - Position data for visual layout (`position: {x, y}`)
  - Render context for output formatting

- [ ] **DAG Execution Engine**
  - Topological sort for execution order
  - Parallel execution support for independent branches  
  - Graph artifact generation at each plan node
  - Execution state tracking and recovery

- [ ] **Legacy YAML Support**
  - YAML-to-JSON conversion for backward compatibility
  - Migration utilities for existing YAML plans
  - Dual execution path during transition period

### **1.3 Graph Artifact Management (Week 3-4)**
**Target**: Store and retrieve graph snapshots at plan nodes

- [ ] **Graph Generation Service**
  - Generate graph artifacts during plan execution
  - Store graph data with metadata (node count, execution time)
  - Link graphs to specific plan nodes and executions

- [ ] **Graph Snapshot API**
  - Retrieve graph state at any plan node
  - Support execution-specific vs latest snapshots
  - Graph versioning for plan modifications

- [ ] **Graph Inspection Service** 
  - Validate graph integrity at each plan node
  - Provide graph statistics and metadata
  - Support graph diff operations between plan nodes

### **1.4 API Architecture Updates (Week 4-5)**
**Target**: Support hierarchical navigation and graph inspection

- [ ] **Hierarchical REST Endpoints**
  ```
  GET /api/v1/projects/{id}/plans/{plan_id}/plan-nodes/{node_id}/graph
  GET /api/v1/projects/{id}/plans/{plan_id}/execution/{exec_id}/graphs  
  GET /api/v1/plans/{id}/dag - Get plan DAG structure
  ```

- [ ] **GraphQL Schema Enhancement**
  - Add `graphAtPlanNode(planNodeId, executionId)` resolver
  - Implement `inspectionPoints` query for available graphs
  - Add execution path navigation queries

- [ ] **Plan Node CRUD Operations**
  - Create, read, update, delete plan nodes within DAG
  - Validate DAG structure (no cycles, connected components)
  - Update plan node positions and configurations

---

## **Phase 2: Frontend DAG Support & Visual Editor**
**Priority: HIGH | Timeline: 4-5 weeks**

### **2.1 ReactFlow DAG Plan Editor (Week 1-2)**
**Target**: Replace text-based plan editing with visual DAG editor

- [ ] **ReactFlow Integration**
  - Install and configure ReactFlow for DAG editing
  - Custom node types: ImportNode, TransformNode, ExportNode
  - Custom edge rendering with execution flow indicators
  - Drag-and-drop node creation and connection

- [ ] **Plan Node Configuration**
  - Modal dialogs for node configuration editing
  - Form validation for node-specific settings
  - Preview mode for configuration changes
  - Copy/paste and duplicate node operations

- [ ] **DAG Validation UI**
  - Real-time cycle detection with visual feedback
  - Connection validation (types, required inputs)
  - Plan execution simulation and preview
  - Save/load plan versions with change tracking

### **2.2 Hierarchical Navigation System (Week 2-3)**
**Target**: Implement project → plan → workflow → node drilling

- [ ] **Navigation Architecture**
  - Breadcrumb navigation with context switching
  - URL routing: `/projects/{id}/plans/{id}/nodes/{id}`
  - Deep linking to specific plan nodes and executions
  - Navigation state management with React Router

- [ ] **Plan Node Inspection View**
  - Display graph state at selected plan node
  - Show execution history and statistics
  - Graph visualization with plan node context
  - Switch between different execution snapshots

- [ ] **Execution Path Visualization**
  - Timeline view of plan execution progress
  - Clickable execution steps with graph inspection
  - Real-time execution monitoring with live updates
  - Error tracking and debugging support

### **2.3 Graph Data Grid Interface (Week 3-4)**
**Target**: Spreadsheet-like editing interface for graph data

- [ ] **Data Grid Component (TanStack Table/AG-Grid)**
  - Tabbed interface: Nodes, Edges, Layers
  - Excel-like editing with validation
  - Copy/paste support from external sources
  - Bulk operations (delete, transform, import)

- [ ] **Grid Column Definitions**
  - Nodes: id, label, layer, position (x,y), weight, metadata
  - Edges: id, source, target, label, layer, weight
  - Layers: id, name, color, description, visibility
  - Custom cell editors: dropdowns, color pickers, JSON editors

- [ ] **Transformation Node Strategy**
  - Edit operations create transformation plan nodes
  - Preview changes before applying to DAG
  - Commit strategy selection: transformation vs in-place
  - Undo/redo with plan node removal

### **2.4 Real-time Integration (Week 4-5)**
**Target**: Live updates and bidirectional sync

- [ ] **GraphQL Subscriptions**
  - Real-time plan execution updates
  - Live graph data changes during execution
  - Multi-user presence indicators
  - Conflict notification system

- [ ] **Bidirectional Graph Sync**
  - Data grid changes update visualization
  - Visualization selections highlight grid rows
  - Focus synchronization between views
  - Cross-view selection and highlighting

---

## **Phase 3: Enhanced Graph Operations & Analysis**
**Priority: MEDIUM | Timeline: 3-4 weeks**

### **3.1 Advanced Graph Transformations (Week 1-2)**
**Target**: Sophisticated graph manipulation within DAG nodes

- [ ] **Transformation Node Types**
  - Filter operations: node/edge/layer filtering with conditions
  - Transform operations: field mapping, computed fields
  - Merge operations: combine multiple input graphs
  - Split operations: partition graphs by criteria

- [ ] **Transformation Pipeline**
  - Chain multiple transformations in sequence
  - Validation and preview before execution
  - Rollback capabilities with plan node removal
  - Performance optimization for large graphs

- [ ] **Custom Transformation Scripts**
  - JavaScript/TypeScript scripting for complex operations
  - Sandbox execution environment with API access
  - Library of common transformation patterns
  - User-defined transformation templates

### **3.2 Enhanced MCP Integration (Week 2-3)**
**Target**: Advanced AI agent capabilities for graph analysis

- [ ] **Graph Analysis MCP Tools**
  - Connectivity analysis and pathfinding
  - Community detection and clustering
  - Centrality measures and influence analysis
  - Graph similarity and comparison tools

- [ ] **Plan Execution MCP Tools**
  - Execute plans with parameter override
  - Monitor execution progress and status
  - Retrieve execution results and artifacts
  - Debug failed executions with detailed logs

- [ ] **Graph Intelligence Features**
  - Automated graph quality assessment
  - Suggest optimizations and improvements
  - Detect anomalies and inconsistencies
  - Generate insights and summary reports

### **3.3 File Upload & Import Enhancement (Week 3-4)**
**Target**: Support file uploads for plan creation workflow

- [ ] **Multipart File Upload API**
  - Handle CSV, JSON, YAML file uploads
  - Validation and preprocessing before import
  - Progress tracking for large file uploads
  - Error handling with detailed feedback

- [ ] **Import Plan Node Workflow**
  - Drag-and-drop file upload in plan editor
  - Configure import settings with preview
  - Create import plan nodes from uploaded files
  - Batch import operations for multiple files

---

## **Phase 4: Real-time Collaboration & Advanced UX**  
**Priority: MEDIUM | Timeline: 4-5 weeks**

### **4.1 Multi-User Collaborative Editing (Week 1-3)**
**Target**: Teams working together on graph data simultaneously

- [ ] **Operational Transformation System**
  - Conflict resolution for concurrent edits
  - Operation queuing and transformation
  - Last-write-wins vs merge strategies
  - Manual conflict resolution interface

- [ ] **Real-time Presence System**
  - User cursors and active editing indicators
  - User list with current activity status
  - Locking mechanism for critical operations
  - Chat/comment system for collaboration

- [ ] **Collaborative Plan Editing**
  - Real-time DAG updates across browsers
  - Shared plan node configuration editing
  - Version control with branch/merge workflow
  - Team permission management

### **4.2 Advanced Export & Template System (Week 3-4)**
**Target**: Enhanced output generation and sharing

- [ ] **Advanced Export Configuration**
  - Custom export templates with Handlebars
  - Multi-format export from single plan execution
  - Export parameter configuration and preview
  - Scheduled export execution

- [ ] **Template Marketplace**
  - Share and discover plan templates
  - Template versioning and dependency management
  - Community ratings and reviews
  - Enterprise template repositories

### **4.3 Performance & Scale Optimization (Week 4-5)**
**Target**: Handle large graphs and complex DAGs efficiently

- [ ] **Large Graph Handling**
  - Virtualization for graph visualization (10k+ nodes)
  - Chunked data loading and pagination
  - Graph level-of-detail rendering
  - Memory management and garbage collection

- [ ] **Execution Performance**
  - Parallel DAG execution optimization
  - Caching strategies for intermediate results
  - Incremental execution for plan modifications
  - Resource monitoring and limits

---

## **Phase 5: Production Readiness & Operations**
**Priority: LOW | Timeline: 2-3 weeks**

### **5.1 Security & Authentication (Week 1-2)**
- [ ] User authentication with JWT/OAuth
- [ ] Role-based access control for projects/plans
- [ ] API security and rate limiting
- [ ] Audit logging for all operations

### **5.2 Production Infrastructure (Week 2-3)**
- [ ] Deployment automation with Docker/K8s
- [ ] Monitoring and alerting systems  
- [ ] Backup and disaster recovery
- [ ] Performance monitoring and optimization

---

## **Success Metrics & Validation**

### **Phase 1 Success Criteria**
- [ ] Can create DAG plans with visual editor
- [ ] Can execute DAG plans and inspect graphs at each node  
- [ ] Can navigate project → plan → workflow → node hierarchy
- [ ] All existing YAML plans work with new execution engine

### **Phase 2 Success Criteria**  
- [ ] Can edit graph data with spreadsheet interface
- [ ] Can see real-time execution progress with live updates
- [ ] Can inspect any graph state during/after execution
- [ ] Visual DAG editor supports all plan operations

### **Phase 3 Success Criteria**
- [ ] Advanced transformations work within DAG execution
- [ ] MCP agents can analyze graphs and provide insights  
- [ ] File upload workflow creates import plan nodes
- [ ] Complex multi-branch DAGs execute successfully

### **Phase 4 Success Criteria**
- [ ] Multiple users can collaborate on plans simultaneously
- [ ] Large graphs (10k+ nodes) render and edit smoothly
- [ ] Template sharing and reuse works across teams
- [ ] Export system generates publication-ready outputs

---

## **Technical Debt & Cleanup**

### **After Phase 2**
- [ ] Remove legacy YAML execution code paths
- [ ] Cleanup direct project-graph relationships
- [ ] Remove unused database entities and migrations
- [ ] Update API documentation for new endpoints

### **After Phase 4**  
- [ ] Performance audit and optimization
- [ ] Security review and penetration testing
- [ ] Code coverage analysis and improvement
- [ ] Documentation and knowledge transfer

---

**Estimated Total Timeline: 15-17 weeks**
**Critical Path: Phase 1 → Phase 2 (Core functionality)**
**Optional: Phase 3-5 (Enhanced features)**