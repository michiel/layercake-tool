import { useParams, Link } from 'react-router-dom';
import { useQuery } from '@apollo/client';
import { Card, Typography, Spin, Alert, Tabs, Space, Button } from 'antd';
import { ReactGrid, Column, Row, DropdownCell, DefaultCellTypes, TextCell } from '@silevis/reactgrid';
import '@silevis/reactgrid/styles.css';
import { GET_GRAPH } from '../graphql/queries';
import GraphVisualizer from './GraphVisualizer';

const { Title } = Typography;

const GraphDetail = () => {
  const { projectId } = useParams<{ projectId: string }>();
  const { loading, error, data } = useQuery(GET_GRAPH, {
    variables: { projectId },
    skip: !projectId
  });

  if (loading) return <Spin size="large" />;
  if (error) return <Alert message="Error loading graph" description={error.message} type="error" showIcon />;
  if (!data || !data.graph) return <Alert message="Graph not found" type="warning" showIcon />;

  const { graph } = data;

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
      ]
    };

    // Create data rows
    const dataRows: Row[] = graph.nodes.map((node, idx) => ({
      rowId: idx,
      cells: [
        createTextCell(node.id),
        createTextCell(node.label),
        createTextCell(node.layer),
        createTextCell(node.isPartition ? 'Yes' : 'No'),
        createTextCell(node.belongsTo),
        createTextCell(node.weight),
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
      ]
    };

    // Create data rows
    const dataRows: Row[] = graph.edges.map((edge, idx) => ({
      rowId: idx,
      cells: [
        createTextCell(edge.id),
        createTextCell(edge.source),
        createTextCell(edge.target),
        createTextCell(edge.label),
        createTextCell(edge.layer),
        createTextCell(edge.weight),
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
      ]
    };

    // Create data rows
    const dataRows: Row[] = graph.layers.map((layer, idx) => ({
      rowId: idx,
      cells: [
        createTextCell(layer.id),
        createTextCell(layer.label),
        createColorCell(layer.backgroundColor),
        createColorCell(layer.textColor),
        createColorCell(layer.borderColor),
      ]
    }));

    return [headerRow, ...dataRows];
  };

  // Graph Visualization Tab Content
  const graphVisualizationContent = (
    <GraphVisualizer graph={graph} />
  );

  // Handler for cell changes (read-only for now)
  const handleChanges = (changes: any[]) => {
    // For read-only, no action needed
    console.log('Cell changes:', changes);
  };

  // Graph Data Tables Tab Content
  const graphDataContent = (
    <Tabs
      defaultActiveKey="nodes"
      items={[
        {
          key: 'nodes',
          label: `Nodes (${graph.nodes.length})`,
          children: (
            <div style={{ height: '400px', overflow: 'auto' }}>
              <ReactGrid 
                rows={getNodeRows()} 
                columns={getNodeColumns()} 
                onCellsChanged={handleChanges}
                stickyTopRows={1}
                stickyLeftColumns={1}
              />
            </div>
          ),
        },
        {
          key: 'edges',
          label: `Edges (${graph.edges.length})`,
          children: (
            <div style={{ height: '400px', overflow: 'auto' }}>
              <ReactGrid 
                rows={getEdgeRows()} 
                columns={getEdgeColumns()} 
                onCellsChanged={handleChanges}
                stickyTopRows={1}
                stickyLeftColumns={1}
              />
            </div>
          ),
        },
        {
          key: 'layers',
          label: `Layers (${graph.layers.length})`,
          children: (
            <div style={{ height: '400px', overflow: 'auto' }}>
              <ReactGrid 
                rows={getLayerRows()} 
                columns={getLayerColumns()} 
                onCellsChanged={handleChanges}
                stickyTopRows={1}
                stickyLeftColumns={1}
              />
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
    </Card>
  );
};

export default GraphDetail;
