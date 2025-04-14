import { useParams, Link } from 'react-router-dom';
import { useQuery, useMutation } from '@apollo/client';
import { Card, Typography, Spin, Alert, Tabs, Space, Button, Modal, Form, Input, Select, Checkbox, message } from 'antd';
import { ReactGrid, Column, Row, DropdownCell, DefaultCellTypes, TextCell, CellChange } from '@silevis/reactgrid';
import '@silevis/reactgrid/styles.css';
import { 
  GET_GRAPH, 
  UPDATE_GRAPH,
  ADD_NODE,
  UPDATE_NODE,
  DELETE_NODE,
  ADD_EDGE,
  UPDATE_EDGE,
  DELETE_EDGE,
  ADD_LAYER,
  UPDATE_LAYER,
  DELETE_LAYER
} from '../graphql/queries';
import GraphVisualizer from './GraphVisualizer';
import { useState, useCallback } from 'react';

const { Title, Text } = Typography;
const { Option } = Select;

// Type definitions for graph data
interface Node {
  id: string;
  label: string;
  layer: string;
  isPartition: boolean;
  belongsTo: string | null;
  weight: number;
  comment?: string | null;
}

interface Edge {
  id: string;
  source: string;
  target: string;
  label: string;
  layer: string;
  weight: number;
  comment?: string | null;
}

interface Layer {
  id: string;
  label: string;
  backgroundColor: string;
  textColor: string;
  borderColor: string;
}

interface GraphData {
  id: string;
  projectId: string;
  nodes: Node[];
  edges: Edge[];
  layers: Layer[];
}

const GraphDetail = () => {
  const { projectId } = useParams<{ projectId: string }>();
  
  // State for managing modal forms
  const [nodeModalVisible, setNodeModalVisible] = useState(false);
  const [edgeModalVisible, setEdgeModalVisible] = useState(false);
  const [layerModalVisible, setLayerModalVisible] = useState(false);
  const [editingItem, setEditingItem] = useState<any>(null);
  const [isEditing, setIsEditing] = useState(false);
  
  // Form instances
  const [nodeForm] = Form.useForm();
  const [edgeForm] = Form.useForm();
  const [layerForm] = Form.useForm();
  
  // Fetch graph data
  const { loading, error, data, refetch } = useQuery(GET_GRAPH, {
    variables: { projectId },
    skip: !projectId
  });
  
  // Graph update mutations
  const [updateGraph, { loading: updatingGraph }] = useMutation(UPDATE_GRAPH, {
    onCompleted: () => {
      message.success('Graph updated successfully');
      refetch();
    },
    onError: (error) => {
      message.error(`Failed to update graph: ${error.message}`);
    }
  });
  
  // Node mutations
  const [addNodeMutation, { loading: addingNode }] = useMutation(ADD_NODE, {
    onCompleted: () => {
      message.success('Node added successfully');
      refetch();
    },
    onError: (error) => {
      message.error(`Failed to add node: ${error.message}`);
    }
  });
  
  const [updateNodeMutation, { loading: updatingNode }] = useMutation(UPDATE_NODE, {
    onCompleted: () => {
      message.success('Node updated successfully');
      refetch();
    },
    onError: (error) => {
      message.error(`Failed to update node: ${error.message}`);
    }
  });
  
  const [deleteNodeMutation, { loading: deletingNode }] = useMutation(DELETE_NODE, {
    onCompleted: () => {
      message.success('Node deleted successfully');
      refetch();
    },
    onError: (error) => {
      message.error(`Failed to delete node: ${error.message}`);
    }
  });
  
  // Edge mutations
  const [addEdgeMutation, { loading: addingEdge }] = useMutation(ADD_EDGE, {
    onCompleted: () => {
      message.success('Edge added successfully');
      refetch();
    },
    onError: (error) => {
      message.error(`Failed to add edge: ${error.message}`);
    }
  });
  
  const [updateEdgeMutation, { loading: updatingEdge }] = useMutation(UPDATE_EDGE, {
    onCompleted: () => {
      message.success('Edge updated successfully');
      refetch();
    },
    onError: (error) => {
      message.error(`Failed to update edge: ${error.message}`);
    }
  });
  
  const [deleteEdgeMutation, { loading: deletingEdge }] = useMutation(DELETE_EDGE, {
    onCompleted: () => {
      message.success('Edge deleted successfully');
      refetch();
    },
    onError: (error) => {
      message.error(`Failed to delete edge: ${error.message}`);
    }
  });
  
  // Layer mutations
  const [addLayerMutation, { loading: addingLayer }] = useMutation(ADD_LAYER, {
    onCompleted: () => {
      message.success('Layer added successfully');
      refetch();
    },
    onError: (error) => {
      message.error(`Failed to add layer: ${error.message}`);
    }
  });
  
  const [updateLayerMutation, { loading: updatingLayer }] = useMutation(UPDATE_LAYER, {
    onCompleted: () => {
      message.success('Layer updated successfully');
      refetch();
    },
    onError: (error) => {
      message.error(`Failed to update layer: ${error.message}`);
    }
  });
  
  const [deleteLayerMutation, { loading: deletingLayer }] = useMutation(DELETE_LAYER, {
    onCompleted: () => {
      message.success('Layer deleted successfully');
      refetch();
    },
    onError: (error) => {
      message.error(`Failed to delete layer: ${error.message}`);
    }
  });
  
  // Calculate combined loading state
  const updating = updatingGraph || addingNode || updatingNode || deletingNode || 
                  addingEdge || updatingEdge || deletingEdge || 
                  addingLayer || updatingLayer || deletingLayer;

  // Keep local state of the graph data to handle changes
  const [graphData, setGraphData] = useState<GraphData | null>(null);

  // Update local state when data is fetched
  if (data?.graph && !graphData) {
    setGraphData(data.graph);
  }

  if (loading) return <Spin size="large" />;
  if (error) return <Alert message="Error loading graph" description={error.message} type="error" showIcon />;
  if (!graphData) return <Alert message="Graph not found" type="warning" showIcon />;

  // Function to save changes to the backend using bulk update
  const saveGraphChanges = () => {
    if (!graphData || !projectId) return;
    
    // Convert the graph data to a string for the mutation
    updateGraph({
      variables: {
        projectId,
        graphData: JSON.stringify(graphData)
      }
    });
  };

  // Functions to handle adding new items using specific mutations
  const addNode = (node: Omit<Node, 'id'>) => {
    if (!projectId) return;
    
    // Call the addNode mutation
    addNodeMutation({
      variables: {
        projectId,
        node: {
          label: node.label,
          layer: node.layer,
          isPartition: node.isPartition || false,
          belongsTo: node.belongsTo,
          weight: node.weight || 0,
          comment: node.comment
        }
      }
    });
  };
  
  const updateNode = (nodeId: string, updates: Partial<Node>) => {
    if (!projectId) return;
    
    // Get current node data to merge with updates
    const currentNode = graphData?.nodes.find(n => n.id === nodeId);
    if (!currentNode) {
      message.error(`Node with ID ${nodeId} not found`);
      return;
    }
    
    // Call the updateNode mutation with merged data
    updateNodeMutation({
      variables: {
        projectId,
        nodeId,
        node: {
          label: updates.label || currentNode.label,
          layer: updates.layer || currentNode.layer,
          isPartition: updates.isPartition !== undefined ? updates.isPartition : currentNode.isPartition,
          belongsTo: updates.belongsTo !== undefined ? updates.belongsTo : currentNode.belongsTo,
          weight: updates.weight !== undefined ? updates.weight : currentNode.weight,
          comment: updates.comment !== undefined ? updates.comment : currentNode.comment
        }
      }
    });
  };
  
  const deleteNode = (nodeId: string) => {
    if (!projectId) return;
    
    // Call the deleteNode mutation
    deleteNodeMutation({
      variables: {
        projectId,
        nodeId
      }
    });
  };
  
  const addEdge = (edge: Omit<Edge, 'id'>) => {
    if (!projectId) return;
    
    // Call the addEdge mutation
    addEdgeMutation({
      variables: {
        projectId,
        edge: {
          source: edge.source,
          target: edge.target,
          label: edge.label,
          layer: edge.layer,
          weight: edge.weight || 0,
          comment: edge.comment
        }
      }
    });
  };
  
  const updateEdge = (edgeId: string, updates: Partial<Edge>) => {
    if (!projectId) return;
    
    // Get current edge data to merge with updates
    const currentEdge = graphData?.edges.find(e => e.id === edgeId);
    if (!currentEdge) {
      message.error(`Edge with ID ${edgeId} not found`);
      return;
    }
    
    // Call the updateEdge mutation with merged data
    updateEdgeMutation({
      variables: {
        projectId,
        edgeId,
        edge: {
          source: updates.source || currentEdge.source,
          target: updates.target || currentEdge.target,
          label: updates.label || currentEdge.label,
          layer: updates.layer || currentEdge.layer,
          weight: updates.weight !== undefined ? updates.weight : currentEdge.weight,
          comment: updates.comment !== undefined ? updates.comment : currentEdge.comment
        }
      }
    });
  };
  
  const deleteEdge = (edgeId: string) => {
    if (!projectId) return;
    
    // Call the deleteEdge mutation
    deleteEdgeMutation({
      variables: {
        projectId,
        edgeId
      }
    });
  };
  
  const addLayer = (layer: Omit<Layer, 'id'>) => {
    if (!projectId) return;
    
    // Call the addLayer mutation
    addLayerMutation({
      variables: {
        projectId,
        layer: {
          label: layer.label,
          backgroundColor: layer.backgroundColor,
          textColor: layer.textColor,
          borderColor: layer.borderColor
        }
      }
    });
  };
  
  const updateLayer = (layerId: string, updates: Partial<Layer>) => {
    if (!projectId) return;
    
    // Get current layer data to merge with updates
    const currentLayer = graphData?.layers.find(l => l.id === layerId);
    if (!currentLayer) {
      message.error(`Layer with ID ${layerId} not found`);
      return;
    }
    
    // Call the updateLayer mutation with merged data
    updateLayerMutation({
      variables: {
        projectId,
        layerId,
        layer: {
          label: updates.label || currentLayer.label,
          backgroundColor: updates.backgroundColor || currentLayer.backgroundColor,
          textColor: updates.textColor || currentLayer.textColor,
          borderColor: updates.borderColor || currentLayer.borderColor
        }
      }
    });
  };
  
  const deleteLayer = (layerId: string) => {
    if (!projectId) return;
    
    // With the backend integrity check, we don't need to check if the layer is in use
    deleteLayerMutation({
      variables: {
        projectId,
        layerId
      }
    });
  };

  // Helper to determine text color based on background color for contrast
  const getContrastColor = (hexColor: string): string => {
    // Default to black if color is invalid
    if (!hexColor || !hexColor.match(/^#([0-9A-F]{3}){1,2}$/i)) {
      return '#000000';
    }

    // Convert hex to RGB
    let r = 0, g = 0, b = 0;
    if (hexColor.length === 4) {
      r = parseInt(hexColor[1] + hexColor[1], 16);
      g = parseInt(hexColor[2] + hexColor[2], 16);
      b = parseInt(hexColor[3] + hexColor[3], 16);
    } else {
      r = parseInt(hexColor.slice(1, 3), 16);
      g = parseInt(hexColor.slice(3, 5), 16);
      b = parseInt(hexColor.slice(5, 7), 16);
    }

    // Calculate brightness
    const brightness = (r * 299 + g * 587 + b * 114) / 1000;
    
    // Return white for dark backgrounds, black for light
    return brightness > 128 ? '#000000' : '#ffffff';
  };

  // Define custom cell types
  type CustomCellTypes = DefaultCellTypes | { type: 'color'; text: string; color: string };
  
  // Helper function to create text cells
  const createTextCell = (text: string | number | boolean | null | undefined): TextCell => ({
    type: 'text',
    text: text === null || text === undefined ? '' : String(text)
  });

  // Helper function to create color cells
  const createColorCell = (color: string): CustomCellTypes => ({
    type: 'text', // Standard cell type with custom styles
    text: color || '',
    style: {
      backgroundColor: color || 'transparent',
      color: getContrastColor(color || '#ffffff'),
      textAlign: 'center'
    }
  });

  // ReactGrid - Nodes
  const getNodeColumns = (): Column[] => [
    { columnId: 'id', width: 80 },
    { columnId: 'label', width: 150 },
    { columnId: 'layer', width: 100 },
    { columnId: 'isPartition', width: 100 },
    { columnId: 'belongsTo', width: 100 },
    { columnId: 'weight', width: 80 },
    { columnId: 'actions', width: 120 },
  ];

  const getNodeRows = (): Row[] => {
    // Create header row
    const headerRow: Row = {
      rowId: 'header',
      cells: [
        { type: 'header', text: 'ID' },
        { type: 'header', text: 'Label' },
        { type: 'header', text: 'Layer' },
        { type: 'header', text: 'Is Partition' },
        { type: 'header', text: 'Belongs To' },
        { type: 'header', text: 'Weight' },
        { type: 'header', text: 'Actions' },
      ]
    };

    if (!graphData) return [headerRow];

    // Create data rows
    const dataRows: Row[] = graphData.nodes.map((node, idx) => ({
      rowId: idx,
      cells: [
        createTextCell(node.id),
        createTextCell(node.label),
        createTextCell(node.layer),
        createTextCell(node.isPartition ? 'Yes' : 'No'),
        createTextCell(node.belongsTo),
        createTextCell(node.weight),
        {
          type: 'text',
          text: 'Edit / Delete',
          nonEditable: true,
          style: { color: 'blue', cursor: 'pointer', textDecoration: 'underline' },
          onClick: () => {
            Modal.confirm({
              title: 'Actions',
              content: (
                <Space direction="vertical">
                  <Button onClick={() => { Modal.destroyAll(); showNodeModal(node); }}>Edit</Button>
                  <Button danger onClick={() => { Modal.destroyAll(); handleDeleteNode(node.id); }}>Delete</Button>
                </Space>
              ),
              footer: null
            });
          }
        },
      ]
    }));

    return [headerRow, ...dataRows];
  };

  // ReactGrid - Edges
  const getEdgeColumns = (): Column[] => [
    { columnId: 'id', width: 80 },
    { columnId: 'source', width: 100 },
    { columnId: 'target', width: 100 },
    { columnId: 'label', width: 150 },
    { columnId: 'layer', width: 100 },
    { columnId: 'weight', width: 80 },
    { columnId: 'actions', width: 120 },
  ];

  const getEdgeRows = (): Row[] => {
    // Create header row
    const headerRow: Row = {
      rowId: 'header',
      cells: [
        { type: 'header', text: 'ID' },
        { type: 'header', text: 'Source' },
        { type: 'header', text: 'Target' },
        { type: 'header', text: 'Label' },
        { type: 'header', text: 'Layer' },
        { type: 'header', text: 'Weight' },
        { type: 'header', text: 'Actions' },
      ]
    };

    if (!graphData) return [headerRow];

    // Create data rows
    const dataRows: Row[] = graphData.edges.map((edge, idx) => ({
      rowId: idx,
      cells: [
        createTextCell(edge.id),
        createTextCell(edge.source),
        createTextCell(edge.target),
        createTextCell(edge.label),
        createTextCell(edge.layer),
        createTextCell(edge.weight),
        {
          type: 'text',
          text: 'Edit / Delete',
          nonEditable: true,
          style: { color: 'blue', cursor: 'pointer', textDecoration: 'underline' },
          onClick: () => {
            Modal.confirm({
              title: 'Actions',
              content: (
                <Space direction="vertical">
                  <Button onClick={() => { Modal.destroyAll(); showEdgeModal(edge); }}>Edit</Button>
                  <Button danger onClick={() => { Modal.destroyAll(); handleDeleteEdge(edge.id); }}>Delete</Button>
                </Space>
              ),
              footer: null
            });
          }
        },
      ]
    }));

    return [headerRow, ...dataRows];
  };

  // ReactGrid - Layers
  const getLayerColumns = (): Column[] => [
    { columnId: 'id', width: 80 },
    { columnId: 'label', width: 150 },
    { columnId: 'backgroundColor', width: 150 },
    { columnId: 'textColor', width: 150 },
    { columnId: 'borderColor', width: 150 },
    { columnId: 'actions', width: 120 },
  ];

  const getLayerRows = (): Row[] => {
    // Create header row
    const headerRow: Row = {
      rowId: 'header',
      cells: [
        { type: 'header', text: 'ID' },
        { type: 'header', text: 'Label' },
        { type: 'header', text: 'Background Color' },
        { type: 'header', text: 'Text Color' },
        { type: 'header', text: 'Border Color' },
        { type: 'header', text: 'Actions' },
      ]
    };

    if (!graphData) return [headerRow];

    // Create data rows
    const dataRows: Row[] = graphData.layers.map((layer, idx) => ({
      rowId: idx,
      cells: [
        createTextCell(layer.id),
        createTextCell(layer.label),
        createColorCell(layer.backgroundColor),
        createColorCell(layer.textColor),
        createColorCell(layer.borderColor),
        {
          type: 'text',
          text: 'Edit / Delete',
          nonEditable: true,
          style: { color: 'blue', cursor: 'pointer', textDecoration: 'underline' },
          onClick: () => {
            Modal.confirm({
              title: 'Actions',
              content: (
                <Space direction="vertical">
                  <Button onClick={() => { Modal.destroyAll(); showLayerModal(layer); }}>Edit</Button>
                  <Button danger onClick={() => { Modal.destroyAll(); handleDeleteLayer(layer.id); }}>Delete</Button>
                </Space>
              ),
              footer: null
            });
          }
        },
      ]
    }));

    return [headerRow, ...dataRows];
  };

  // Graph Visualization Tab Content
  const graphVisualizationContent = (
    <GraphVisualizer graph={graphData} />
  );

  // Handler for cell changes
  const handleNodeChanges = (changes: CellChange[]) => {
    if (!graphData) return;
    
    changes.forEach(change => {
      // Skip header row
      if (change.rowId === 'header') return;
      
      const nodeIndex = change.rowId as number;
      const node = graphData.nodes[nodeIndex];
      const property = getNodeColumns()[change.columnId as number].columnId;
      
      if (node && property && property !== 'actions') {
        const value = change.newCell.text;
        const updates: Partial<Node> = {};
        
        switch (property) {
          case 'label':
          case 'layer':
            updates[property as keyof Node] = value;
            break;
          case 'belongsTo':
            updates.belongsTo = value || null;
            break;
          case 'isPartition':
            updates.isPartition = value === 'Yes';
            break;
          case 'weight':
            updates.weight = Number(value) || 0;
            break;
        }
        
        // Call the update mutation directly
        updateNode(node.id, updates);
      }
    });
  };
  
  const handleEdgeChanges = (changes: CellChange[]) => {
    if (!graphData) return;
    
    changes.forEach(change => {
      // Skip header row
      if (change.rowId === 'header') return;
      
      const edgeIndex = change.rowId as number;
      const edge = graphData.edges[edgeIndex];
      const property = getEdgeColumns()[change.columnId as number].columnId;
      
      if (edge && property && property !== 'actions') {
        const value = change.newCell.text;
        const updates: Partial<Edge> = {};
        
        switch (property) {
          case 'source':
          case 'target':
          case 'label':
          case 'layer':
            updates[property as keyof Edge] = value;
            break;
          case 'weight':
            updates.weight = Number(value) || 0;
            break;
        }
        
        // Call the update mutation directly
        updateEdge(edge.id, updates);
      }
    });
  };
  
  const handleLayerChanges = (changes: CellChange[]) => {
    if (!graphData) return;
    
    changes.forEach(change => {
      // Skip header row
      if (change.rowId === 'header') return;
      
      const layerIndex = change.rowId as number;
      const layer = graphData.layers[layerIndex];
      const property = getLayerColumns()[change.columnId as number].columnId;
      
      if (layer && property && property !== 'actions') {
        const value = change.newCell.text;
        const updates: Partial<Layer> = {};
        
        switch (property) {
          case 'label':
            updates.label = value;
            break;
          case 'backgroundColor':
            updates.backgroundColor = value;
            break;
          case 'textColor':
            updates.textColor = value;
            break;
          case 'borderColor':
            updates.borderColor = value;
            break;
        }
        
        // Call the update mutation directly
        updateLayer(layer.id, updates);
      }
    });
  };
  
  // Modal handlers
  const showNodeModal = (node?: Node) => {
    setIsEditing(!!node);
    setEditingItem(node || null);
    
    if (node) {
      nodeForm.setFieldsValue(node);
    } else {
      nodeForm.resetFields();
    }
    
    setNodeModalVisible(true);
  };
  
  const showEdgeModal = (edge?: Edge) => {
    setIsEditing(!!edge);
    setEditingItem(edge || null);
    
    if (edge) {
      edgeForm.setFieldsValue(edge);
    } else {
      edgeForm.resetFields();
    }
    
    setEdgeModalVisible(true);
  };
  
  const showLayerModal = (layer?: Layer) => {
    setIsEditing(!!layer);
    setEditingItem(layer || null);
    
    if (layer) {
      layerForm.setFieldsValue(layer);
    } else {
      layerForm.resetFields();
    }
    
    setLayerModalVisible(true);
  };
  
  const handleNodeFormSubmit = (values: any) => {
    if (isEditing && editingItem) {
      updateNode(editingItem.id, values);
    } else {
      addNode(values);
    }
    
    setNodeModalVisible(false);
  };
  
  const handleEdgeFormSubmit = (values: any) => {
    if (isEditing && editingItem) {
      updateEdge(editingItem.id, values);
    } else {
      addEdge(values);
    }
    
    setEdgeModalVisible(false);
  };
  
  const handleLayerFormSubmit = (values: any) => {
    if (isEditing && editingItem) {
      updateLayer(editingItem.id, values);
    } else {
      addLayer(values);
    }
    
    setLayerModalVisible(false);
  };
  
  const handleDeleteNode = (nodeId: string) => {
    Modal.confirm({
      title: 'Confirm Deletion',
      content: 'Are you sure you want to delete this node? This will also delete any edges connected to it.',
      onOk: () => {
        deleteNode(nodeId);
      }
    });
  };
  
  const handleDeleteEdge = (edgeId: string) => {
    Modal.confirm({
      title: 'Confirm Deletion',
      content: 'Are you sure you want to delete this edge?',
      onOk: () => {
        deleteEdge(edgeId);
      }
    });
  };
  
  const handleDeleteLayer = (layerId: string) => {
    Modal.confirm({
      title: 'Confirm Deletion',
      content: 'Are you sure you want to delete this layer?',
      onOk: () => {
        deleteLayer(layerId);
      }
    });
  };

  // Graph Data Tables Tab Content
  const graphDataContent = (
    <Tabs
      defaultActiveKey="nodes"
      items={[
        {
          key: 'nodes',
          label: `Nodes (${graphData.nodes.length})`,
          children: (
            <div>
              <Space style={{ marginBottom: '1rem' }}>
                <Button type="primary" onClick={() => showNodeModal()}>Add Node</Button>
                {updating && <Spin size="small" />}
              </Space>
              <div style={{ height: '400px', overflow: 'auto' }}>
                <ReactGrid 
                  rows={getNodeRows()} 
                  columns={getNodeColumns()} 
                  onCellsChanged={handleNodeChanges}
                  stickyTopRows={1}
                  stickyLeftColumns={1}
                />
              </div>
            </div>
          ),
        },
        {
          key: 'edges',
          label: `Edges (${graphData.edges.length})`,
          children: (
            <div>
              <Space style={{ marginBottom: '1rem' }}>
                <Button type="primary" onClick={() => showEdgeModal()}>Add Edge</Button>
                {updating && <Spin size="small" />}
              </Space>
              <div style={{ height: '400px', overflow: 'auto' }}>
                <ReactGrid 
                  rows={getEdgeRows()} 
                  columns={getEdgeColumns()} 
                  onCellsChanged={handleEdgeChanges}
                  stickyTopRows={1}
                  stickyLeftColumns={1}
                />
              </div>
            </div>
          ),
        },
        {
          key: 'layers',
          label: `Layers (${graphData.layers.length})`,
          children: (
            <div>
              <Space style={{ marginBottom: '1rem' }}>
                <Button type="primary" onClick={() => showLayerModal()}>Add Layer</Button>
                {updating && <Spin size="small" />}
              </Space>
              <div style={{ height: '400px', overflow: 'auto' }}>
                <ReactGrid 
                  rows={getLayerRows()} 
                  columns={getLayerColumns()} 
                  onCellsChanged={handleLayerChanges}
                  stickyTopRows={1}
                  stickyLeftColumns={1}
                />
              </div>
            </div>
          ),
        },
      ]}
    />
  );

  return (
    <Card>
      <Space direction="vertical" style={{ width: '100%' }} size="large">
        <Space style={{ justifyContent: 'space-between', width: '100%' }}>
          <Title level={2}>Graph Details</Title>
          <Button type="primary">
            <Link to={`/projects/${projectId}`}>Back to Project</Link>
          </Button>
        </Space>

        <Tabs
          defaultActiveKey="visualization"
          items={[
            {
              key: 'visualization',
              label: 'Visualization',
              children: graphVisualizationContent,
            },
            {
              key: 'data',
              label: 'Data Tables',
              children: graphDataContent,
            },
          ]}
        />
      </Space>

      {/* Node Form Modal */}
      <Modal
        title={isEditing ? 'Edit Node' : 'Add Node'}
        open={nodeModalVisible}
        onCancel={() => setNodeModalVisible(false)}
        footer={null}
      >
        <Form 
          form={nodeForm}
          layout="vertical"
          onFinish={handleNodeFormSubmit}
        >
          <Form.Item
            name="label"
            label="Label"
            rules={[{ required: true, message: 'Please enter a label' }]}
          >
            <Input />
          </Form.Item>
          
          <Form.Item
            name="layer"
            label="Layer"
            rules={[{ required: true, message: 'Please select a layer' }]}
          >
            <Select>
              {graphData?.layers.map(layer => (
                <Option key={layer.id} value={layer.id}>{layer.label}</Option>
              ))}
            </Select>
          </Form.Item>
          
          <Form.Item
            name="isPartition"
            valuePropName="checked"
          >
            <Checkbox>Is Partition</Checkbox>
          </Form.Item>
          
          <Form.Item
            name="belongsTo"
            label="Belongs To"
          >
            <Select allowClear>
              {graphData?.nodes
                .filter(n => n.isPartition && (!editingItem || n.id !== editingItem.id))
                .map(node => (
                <Option key={node.id} value={node.id}>{node.label}</Option>
              ))}
            </Select>
          </Form.Item>
          
          <Form.Item
            name="weight"
            label="Weight"
            initialValue={0}
          >
            <Input type="number" />
          </Form.Item>
          
          <Form.Item
            name="comment"
            label="Comment"
          >
            <Input.TextArea />
          </Form.Item>
          
          <Form.Item>
            <Space>
              <Button type="primary" htmlType="submit">
                {isEditing ? 'Update' : 'Add'}
              </Button>
              <Button onClick={() => setNodeModalVisible(false)}>
                Cancel
              </Button>
            </Space>
          </Form.Item>
        </Form>
      </Modal>

      {/* Edge Form Modal */}
      <Modal
        title={isEditing ? 'Edit Edge' : 'Add Edge'}
        open={edgeModalVisible}
        onCancel={() => setEdgeModalVisible(false)}
        footer={null}
      >
        <Form 
          form={edgeForm}
          layout="vertical"
          onFinish={handleEdgeFormSubmit}
        >
          <Form.Item
            name="source"
            label="Source Node"
            rules={[{ required: true, message: 'Please select a source node' }]}
          >
            <Select>
              {graphData?.nodes.map(node => (
                <Option key={node.id} value={node.id}>{node.label}</Option>
              ))}
            </Select>
          </Form.Item>
          
          <Form.Item
            name="target"
            label="Target Node"
            rules={[{ required: true, message: 'Please select a target node' }]}
          >
            <Select>
              {graphData?.nodes.map(node => (
                <Option key={node.id} value={node.id}>{node.label}</Option>
              ))}
            </Select>
          </Form.Item>
          
          <Form.Item
            name="label"
            label="Label"
            rules={[{ required: true, message: 'Please enter a label' }]}
          >
            <Input />
          </Form.Item>
          
          <Form.Item
            name="layer"
            label="Layer"
            rules={[{ required: true, message: 'Please select a layer' }]}
          >
            <Select>
              {graphData?.layers.map(layer => (
                <Option key={layer.id} value={layer.id}>{layer.label}</Option>
              ))}
            </Select>
          </Form.Item>
          
          <Form.Item
            name="weight"
            label="Weight"
            initialValue={0}
          >
            <Input type="number" />
          </Form.Item>
          
          <Form.Item
            name="comment"
            label="Comment"
          >
            <Input.TextArea />
          </Form.Item>
          
          <Form.Item>
            <Space>
              <Button type="primary" htmlType="submit">
                {isEditing ? 'Update' : 'Add'}
              </Button>
              <Button onClick={() => setEdgeModalVisible(false)}>
                Cancel
              </Button>
            </Space>
          </Form.Item>
        </Form>
      </Modal>

      {/* Layer Form Modal */}
      <Modal
        title={isEditing ? 'Edit Layer' : 'Add Layer'}
        open={layerModalVisible}
        onCancel={() => setLayerModalVisible(false)}
        footer={null}
      >
        <Form 
          form={layerForm}
          layout="vertical"
          onFinish={handleLayerFormSubmit}
        >
          <Form.Item
            name="label"
            label="Label"
            rules={[{ required: true, message: 'Please enter a label' }]}
          >
            <Input />
          </Form.Item>
          
          <Form.Item
            name="backgroundColor"
            label="Background Color"
            rules={[{ required: true, message: 'Please enter a background color' }]}
            initialValue="#FFFFFF"
          >
            <Input type="color" style={{ width: '100%' }} />
          </Form.Item>
          
          <Form.Item
            name="textColor"
            label="Text Color"
            rules={[{ required: true, message: 'Please enter a text color' }]}
            initialValue="#000000"
          >
            <Input type="color" style={{ width: '100%' }} />
          </Form.Item>
          
          <Form.Item
            name="borderColor"
            label="Border Color"
            rules={[{ required: true, message: 'Please enter a border color' }]}
            initialValue="#000000"
          >
            <Input type="color" style={{ width: '100%' }} />
          </Form.Item>
          
          <Form.Item>
            <Space>
              <Button type="primary" htmlType="submit">
                {isEditing ? 'Update' : 'Add'}
              </Button>
              <Button onClick={() => setLayerModalVisible(false)}>
                Cancel
              </Button>
            </Space>
          </Form.Item>
        </Form>
      </Modal>
    </Card>
  );
};

export default GraphDetail;
