import { useParams, Link } from 'react-router-dom';
import { useQuery } from '@apollo/client';
import { Card, Typography, Descriptions, Spin, Alert, Tabs, Space, Button } from 'antd';
import { GET_PROJECT } from '../graphql/queries';

const { Title } = Typography;

const ProjectDetail = () => {
  const { projectId } = useParams<{ projectId: string }>();
  const { loading, error, data } = useQuery(GET_PROJECT, {
    variables: { id: projectId },
    skip: !projectId
  });

  if (loading) return <Spin size="large" />;
  if (error) return <Alert message="Error loading project" description={error.message} type="error" showIcon />;
  if (!data || !data.project) return <Alert message="Project not found" type="warning" showIcon />;

  const { project } = data;

  return (
    <Card>
      <Space direction="vertical" style={{ width: '100%' }} size="large">
        <Title level={2}>{project.name}</Title>
        
        <Descriptions title="Project Details" bordered>
          <Descriptions.Item label="ID">{project.id}</Descriptions.Item>
          <Descriptions.Item label="Description">{project.description || 'No description'}</Descriptions.Item>
          <Descriptions.Item label="Created At">{new Date(project.createdAt).toLocaleString()}</Descriptions.Item>
          <Descriptions.Item label="Updated At">{new Date(project.updatedAt).toLocaleString()}</Descriptions.Item>
        </Descriptions>

        <Space>
          <Button type="primary">
            <Link to={`/projects/${projectId}/graph`}>View Graph</Link>
          </Button>
          <Button>
            <Link to={`/projects/${projectId}/plan`}>View Plan</Link>
          </Button>
        </Space>

        <Tabs
          defaultActiveKey="1"
          items={[
            {
              key: '1',
              label: 'Graph Overview',
              children: (
                <Card size="small" title="Graph Summary">
                  <p>Nodes: {project.graph?.nodes.length || 0}</p>
                  <p>Edges: {project.graph?.edges.length || 0}</p>
                  <p>Layers: {project.graph?.layers.length || 0}</p>
                  <Button type="link">
                    <Link to={`/projects/${projectId}/graph`}>View Full Graph</Link>
                  </Button>
                </Card>
              ),
            },
            {
              key: '2',
              label: 'Plan Overview',
              children: (
                <Card size="small" title="Plan Summary">
                  <p>Plan Name: {project.plan?.meta?.name || 'Unnamed Plan'}</p>
                  <p>Import Profiles: {project.plan?.import.profiles.length || 0}</p>
                  <p>Export Profiles: {project.plan?.export.profiles.length || 0}</p>
                  <Button type="link">
                    <Link to={`/projects/${projectId}/plan`}>View Full Plan</Link>
                  </Button>
                </Card>
              ),
            },
          ]}
        />
      </Space>
    </Card>
  );
};

export default ProjectDetail;
