import { useParams, Link } from 'react-router-dom';
import { useQuery, useMutation } from '@apollo/client';
import { Card, Typography, Spin, Alert, Tabs, Space, Button, Modal, Form, Input, Select, Checkbox, message } from 'antd';
import { ReactGrid, Column, Row, DropdownCell, DefaultCellTypes, TextCell, CellChange } from '@silevis/reactgrid';
import '@silevis/reactgrid/styles.css';
import { GET_GRAPH, UPDATE_GRAPH } from '../graphql/queries';
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
  
  // Update graph mutation
  const [updateGraph, { loading: updating }] = useMutation(UPDATE_GRAPH, {
    onCompleted: () => {
      message.success('Graph updated successfully');
      refetch();
    },
    onError: (error) => {
      message.error(`Failed to update graph: ${error.message}`);
    }
  });

  // Keep local state of the graph data to handle changes
  const [graphData, setGraphData] = useState<GraphData | null>(null);

  // Update local state when data is fetched
  if (data?.graph && !graphData) {
    setGraphData(data.graph);
  }

  if (loading) return <Spin size="large" />;
  if (error) return <Alert message="Error loading graph" description={error.message} type="error" showIcon />;
  if (!graphData) return <Alert message="Graph not found" type="warning" showIcon />;

  // Function to save changes to the backend
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

  // Functions to handle adding new items
  const addNode = (node: Omit<Node, 'id'>) => {
    if (!graphData) return;
    
    // Generate a unique ID (in production, this would be handled by the backend)
    const newId = `n${Date.now()}`;
    const newNode: Node = {
      ...node,
      id: newId,
      isPartition: node.isPartition || false,
      weight: node.weight || 0
    };
    
    setGraphData({
      ...graphData,
      nodes: [...graphData.nodes, newNode]
    });
    
    return newNode;
  };
  
  const updateNode = (nodeId: string, updates: Partial<Node>) => {
    if (!graphData) return;
    
    const updatedNodes = graphData.nodes.map(node => 
      node.id === nodeId ? { ...node, ...updates } : node
    );
    
    setGraphData({
      ...graphData,
      nodes: updatedNodes
    });
  };
  
  const deleteNode = (nodeId: string) => {
    if (!graphData) return;
    
    // Also delete any edges connected to this node
    const updatedEdges = graphData.edges.filter(
      edge => edge.source !== nodeId && edge.target !== nodeId
    );
    
    setGraphData({
      ...graphData,
      nodes: graphData.nodes.filter(node => node.id !== nodeId),
      edges: updatedEdges
    });
  };
  
  const addEdge = (edge: Omit<Edge, 'id'>) => {
    if (!graphData) return;
    
    // Generate a unique ID
    const newId = `e${Date.now()}`;
    const newEdge: Edge = {
      ...edge,
      id: newId,
      weight: edge.weight || 0
    };
    
    setGraphData({
      ...graphData,
      edges: [...graphData.edges, newEdge]
    });
    
    return newEdge;
  };
  
  const updateEdge = (edgeId: string, updates: Partial<Edge>) => {
    if (!graphData) return;
    
    const updatedEdges = graphData.edges.map(edge => 
      edge.id === edgeId ? { ...edge, ...updates } : edge
    );
    
    setGraphData({
      ...graphData,
      edges: updatedEdges
    });
  };
  
  const deleteEdge = (edgeId: string) => {
    if (!graphData) return;
    
    setGraphData({
      ...graphData,
      edges: graphData.edges.filter(edge => edge.id !== edgeId)
    });
  };
  
  const addLayer = (layer: Omit<Layer, 'id'>) => {
    if (!graphData) return;
    
    // Generate a unique ID
    const newId = `l${Date.now()}`;
    const newLayer: Layer = {
      ...layer,
      id: newId
    };
    
    setGraphData({
      ...graphData,
      layers: [...graphData.layers, newLayer]
    });
    
    return newLayer;
  };
  
  const updateLayer = (layerId: string, updates: Partial<Layer>) => {
    if (!graphData) return;
    
    const updatedLayers = graphData.layers.map(layer => 
      layer.id === layerId ? { ...layer, ...updates } : layer
    );
    
    setGraphData({
      ...graphData,
      layers: updatedLayers
    });
  };
  
  const deleteLayer = (layerId: string) => {
    if (!graphData) return;
    
    // Check if any nodes or edges are using this layer
    const nodesUsingLayer = graphData.nodes.some(node => node.layer === layerId);
    const edgesUsingLayer = graphData.edges.some(edge => edge.layer === layerId);
    
    if (nodesUsingLayer || edgesUsingLayer) {
      message.error('Cannot delete layer that is in use by nodes or edges');
      return;
    }
    
    setGraphData({
      ...graphData,
      layers: graphData.layers.filter(layer => layer.id !== layerId)
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

    const updatedNodes = [...graphData.nodes];
    
    changes.forEach(change => {
      // Skip header row
      if (change.rowId === 'header') return;
      
      const nodeIndex = change.rowId as number;
      const node = updatedNodes[nodeIndex];
      const property = getNodeColumns()[change.columnId as number].columnId;
      
      if (node && property) {
        const value = change.newCell.text;
        switch (property) {
          case 'label':
          case 'layer':
          case 'belongsTo':
            (node as any)[property] = value;
            break;
          case 'isPartition':
            node.isPartition = value === 'Yes';
            break;
          case 'weight':
            node.weight = Number(value) || 0;
            break;
        }
      }
    });
    
    setGraphData({
      ...graphData,
      nodes: updatedNodes
    });
  };
  
  const handleEdgeChanges = (changes: CellChange[]) => {
    if (!graphData) return;

    const updatedEdges = [...graphData.edges];
    
    changes.forEach(change => {
      // Skip header row
      if (change.rowId === 'header') return;
      
      const edgeIndex = change.rowId as number;
      const edge = updatedEdges[edgeIndex];
      const property = getEdgeColumns()[change.columnId as number].columnId;
      
      if (edge && property) {
        const value = change.newCell.text;
        switch (property) {
          case 'source':
          case 'target':
          case 'label':
          case 'layer':
            (edge as any)[property] = value;
            break;
          case 'weight':
            edge.weight = Number(value) || 0;
            break;
        }
      }
    });
    
    setGraphData({
      ...graphData,
      edges: updatedEdges
    });
  };
  
  const handleLayerChanges = (changes: CellChange[]) => {
    if (!graphData) return;

    const updatedLayers = [...graphData.layers];
    
    changes.forEach(change => {
      // Skip header row
      if (change.rowId === 'header') return;
      
      const layerIndex = change.rowId as number;
      const layer = updatedLayers[layerIndex];
      const property = getLayerColumns()[change.columnId as number].columnId;
      
      if (layer && property) {
        const value = change.newCell.text;
        switch (property) {
          case 'label':
          case 'backgroundColor':
          case 'textColor':
          case 'borderColor':
            (layer as any)[property] = value;
            break;
        }
      }
    });
    
    setGraphData({
      ...graphData,
      layers: updatedLayers
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
                <Button type="default" onClick={saveGraphChanges} loading={updating}>Save Changes</Button>
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
                <Button type="default" onClick={saveGraphChanges} loading={updating}>Save Changes</Button>
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
                <Button type="default" onClick={saveGraphChanges} loading={updating}>Save Changes</Button>
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
