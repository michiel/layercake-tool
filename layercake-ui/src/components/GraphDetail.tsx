import { useParams, Link } from 'react-router-dom';
import { useQuery } from '@apollo/client';
import { Card, Typography, Spin, Alert, Tabs, Table, Space, Button } from 'antd';
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

  const nodeColumns = [
    {
      title: 'ID',
      dataIndex: 'id',
      key: 'id',
    },
    {
      title: 'Label',
      dataIndex: 'label',
      key: 'label',
    },
    {
      title: 'Layer',
      dataIndex: 'layer',
      key: 'layer',
    },
    {
      title: 'Is Partition',
      dataIndex: 'isPartition',
      key: 'isPartition',
      render: (value: boolean) => value ? 'Yes' : 'No',
    },
    {
      title: 'Belongs To',
      dataIndex: 'belongsTo',
      key: 'belongsTo',
    },
    {
      title: 'Weight',
      dataIndex: 'weight',
      key: 'weight',
    },
  ];

  const edgeColumns = [
    {
      title: 'ID',
      dataIndex: 'id',
      key: 'id',
    },
    {
      title: 'Source',
      dataIndex: 'source',
      key: 'source',
    },
    {
      title: 'Target',
      dataIndex: 'target',
      key: 'target',
    },
    {
      title: 'Label',
      dataIndex: 'label',
      key: 'label',
    },
    {
      title: 'Layer',
      dataIndex: 'layer',
      key: 'layer',
    },
    {
      title: 'Weight',
      dataIndex: 'weight',
      key: 'weight',
    },
  ];

  const layerColumns = [
    {
      title: 'ID',
      dataIndex: 'id',
      key: 'id',
    },
    {
      title: 'Label',
      dataIndex: 'label',
      key: 'label',
    },
    {
      title: 'Background Color',
      dataIndex: 'backgroundColor',
      key: 'backgroundColor',
      render: (color: string) => (
        <div style={{ backgroundColor: color, width: 20, height: 20, display: 'inline-block', marginRight: 10 }} />
      ),
    },
    {
      title: 'Text Color',
      dataIndex: 'textColor',
      key: 'textColor',
      render: (color: string) => (
        <div style={{ backgroundColor: color, width: 20, height: 20, display: 'inline-block', marginRight: 10 }} />
      ),
    },
    {
      title: 'Border Color',
      dataIndex: 'borderColor',
      key: 'borderColor',
      render: (color: string) => (
        <div style={{ backgroundColor: color, width: 20, height: 20, display: 'inline-block', marginRight: 10 }} />
      ),
    },
  ];

  // Graph Visualization Tab Content
  const graphVisualizationContent = (
    <GraphVisualizer graph={graph} />
  );

  // Graph Data Tables Tab Content
  const graphDataContent = (
    <Tabs
      defaultActiveKey="nodes"
      items={[
        {
          key: 'nodes',
          label: `Nodes (${graph.nodes.length})`,
          children: (
            <Table 
              dataSource={graph.nodes} 
              columns={nodeColumns} 
              rowKey="id"
              pagination={{ pageSize: 10 }}
              size="small"
            />
          ),
        },
        {
          key: 'edges',
          label: `Edges (${graph.edges.length})`,
          children: (
            <Table 
              dataSource={graph.edges} 
              columns={edgeColumns} 
              rowKey="id"
              pagination={{ pageSize: 10 }}
              size="small"
            />
          ),
        },
        {
          key: 'layers',
          label: `Layers (${graph.layers.length})`,
          children: (
            <Table 
              dataSource={graph.layers} 
              columns={layerColumns} 
              rowKey="id"
              pagination={{ pageSize: 10 }}
              size="small"
            />
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
