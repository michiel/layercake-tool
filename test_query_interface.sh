#!/bin/bash
# Layercake Query Interface Test Suite
# Tests all Phase 1 and Phase 2 features

set -e

PROJECT_ID=34
PLAN_ID=37
DB="layercake.db"

echo "=========================================="
echo "Layercake Query Interface Test Suite"
echo "=========================================="
echo ""

# Phase 1.1: Node Query Filters
echo "Test 1.1: Filter nodes by type"
echo "--------------------"
layercake query --database $DB --entity nodes --action list \
  --project $PROJECT_ID --plan $PLAN_ID \
  --payload-json '{"nodeType":"GraphNode"}' --pretty
echo ""

echo "Test 1.1b: Filter nodes by label pattern"
echo "--------------------"
layercake query --database $DB --entity nodes --action list \
  --project $PROJECT_ID --plan $PLAN_ID \
  --payload-json '{"labelPattern":"copilot"}' --pretty
echo ""

echo "Test 1.1c: Filter nodes by position bounds"
echo "--------------------"
layercake query --database $DB --entity nodes --action list \
  --project $PROJECT_ID --plan $PLAN_ID \
  --payload-json '{"bounds":{"minX":0,"maxX":500,"minY":0,"maxY":500}}' --pretty
echo ""

# Phase 1.2: Single Node GET
echo "Test 1.2: Get single node with metadata"
echo "--------------------"
layercake query --database $DB --entity nodes --action get \
  --project $PROJECT_ID --plan $PLAN_ID \
  --payload-json '{"nodeId":"graph_42b0374af121"}' --pretty
echo ""

# Phase 1.3: Graph Traversal
echo "Test 1.3a: Traverse downstream from dataset"
echo "--------------------"
layercake query --database $DB --entity nodes --action traverse \
  --project $PROJECT_ID --plan $PLAN_ID \
  --payload-json '{"startNode":"dataset_fb5f819c7089","direction":"downstream","maxDepth":3}' --pretty
echo ""

echo "Test 1.3b: Traverse upstream from artefact"
echo "--------------------"
layercake query --database $DB --entity nodes --action traverse \
  --project $PROJECT_ID --plan $PLAN_ID \
  --payload-json '{"startNode":"graphartefact_af32487bb03c","direction":"upstream","maxDepth":5}' --pretty
echo ""

echo "Test 1.3c: Find path between nodes"
echo "--------------------"
layercake query --database $DB --entity nodes --action traverse \
  --project $PROJECT_ID --plan $PLAN_ID \
  --payload-json '{"startNode":"dataset_fb5f819c7089","endNode":"graphartefact_af32487bb03c","findPath":true}' --pretty
echo ""

# Phase 1.4: Schema Introspection
echo "Test 1.4a: Get node schema for GraphNode"
echo "--------------------"
layercake query --database $DB --entity schema --action get \
  --payload-json '{"type":"node","nodeType":"GraphNode"}' --pretty
echo ""

echo "Test 1.4b: Get edge schema"
echo "--------------------"
layercake query --database $DB --entity schema --action get \
  --payload-json '{"type":"edge"}' --pretty
echo ""

echo "Test 1.4c: List available node types"
echo "--------------------"
layercake query --database $DB --entity schema --action list \
  --payload-json '{"type":"nodeTypes"}' --pretty
echo ""

echo "Test 1.4d: List available actions for nodes"
echo "--------------------"
layercake query --database $DB --entity schema --action list \
  --payload-json '{"type":"actions","entity":"nodes"}' --pretty
echo ""

# Phase 1.5: Improved Error Messages
echo "Test 1.5: Trigger helpful error message (missing nodeId)"
echo "--------------------"
layercake query --database $DB --entity nodes --action get \
  --project $PROJECT_ID --plan $PLAN_ID \
  --payload-json '{}' --pretty || true
echo ""

# Phase 1.6: Validation and Dry-Run
echo "Test 1.6a: Validate node creation (should pass)"
echo "--------------------"
layercake query --database $DB --entity nodes --action create \
  --project $PROJECT_ID --plan $PLAN_ID \
  --payload-json '{"nodeType":"GraphNode","position":{"x":100,"y":200},"metadata":{"label":"Test"},"config":{}}' \
  --dry-run --pretty
echo ""

echo "Test 1.6b: Validate node creation (should fail - missing position)"
echo "--------------------"
layercake query --database $DB --entity nodes --action create \
  --project $PROJECT_ID --plan $PLAN_ID \
  --payload-json '{"nodeType":"GraphNode","metadata":{"label":"Test"},"config":{}}' \
  --dry-run --pretty || true
echo ""

# Phase 2.1: Batch Operations
echo "Test 2.1: Batch create nodes and edges"
echo "--------------------"
cat > /tmp/batch_test.json <<'EOF'
{
  "operations": [
    {
      "op": "createNode",
      "id": "temp_node_1",
      "data": {
        "nodeType": "GraphNode",
        "position": {"x": 1500, "y": 500},
        "metadata": {"label": "Batch Test Node 1"},
        "config": {"metadata": {}}
      }
    },
    {
      "op": "createNode",
      "id": "temp_node_2",
      "data": {
        "nodeType": "GraphNode",
        "position": {"x": 1700, "y": 500},
        "metadata": {"label": "Batch Test Node 2"},
        "config": {"metadata": {}}
      }
    },
    {
      "op": "createEdge",
      "data": {
        "source": "$temp_node_1",
        "target": "$temp_node_2",
        "metadata": {"label": "Test Edge", "data_type": "GraphData"}
      }
    }
  ],
  "atomic": false
}
EOF

layercake query --database $DB --entity nodes --action batch \
  --project $PROJECT_ID --plan $PLAN_ID \
  --payload-file /tmp/batch_test.json --pretty
echo ""

echo "=========================================="
echo "All tests completed!"
echo "=========================================="
