import { useParams, Link } from 'react-router-dom';
import { useQuery } from '@apollo/client';
import { Card, Typography, Spin, Alert, Descriptions, List, Space, Button, Collapse, Tag, Tabs, Empty } from 'antd';
import { GET_PLAN } from '../graphql/queries';
import { ImportProfile, ExportProfileItem } from '../types';

const { Title, Text, Paragraph } = Typography;
const { Panel } = Collapse;

const PlanDetail = () => {
  const { projectId } = useParams<{ projectId: string }>();
  const { loading, error, data } = useQuery(GET_PLAN, {
    variables: { projectId },
    skip: !projectId
  });

  if (loading) return <Spin size="large" />;
  if (error) return <Alert message="Error loading plan" description={error.message} type="error" showIcon />;
  if (!data || !data.plan) return <Alert message="Plan not found" type="warning" showIcon />;

  const { plan } = data;

  // Plan Configuration Tab Content
  const planConfigContent = (
    <Space direction="vertical" style={{ width: '100%' }} size="large">
      <Descriptions title="Plan Metadata" bordered>
        <Descriptions.Item label="Name">{plan.meta?.name || 'Unnamed Plan'}</Descriptions.Item>
      </Descriptions>

      <Collapse defaultActiveKey={['1', '2']}>
        <Panel header="Import Configuration" key="1">
          <List
            header={<div>Import Profiles</div>}
            bordered
            dataSource={plan.import.profiles}
            renderItem={(profile: ImportProfile, index: number) => (
              <List.Item>
                <Space direction="vertical" style={{ width: '100%' }}>
                  <Text strong>Profile #{index + 1}</Text>
                  <Descriptions bordered size="small" column={2}>
                    <Descriptions.Item label="Filename">{profile.filename}</Descriptions.Item>
                    <Descriptions.Item label="File Type">{profile.filetype}</Descriptions.Item>
                  </Descriptions>
                </Space>
              </List.Item>
            )}
          />
        </Panel>

        <Panel header="Export Configuration" key="2">
          <List
            header={<div>Export Profiles</div>}
            bordered
            dataSource={plan.export.profiles}
            renderItem={(profile: ExportProfileItem, index: number) => (
              <List.Item>
                <Space direction="vertical" style={{ width: '100%' }}>
                  <Text strong>Profile #{index + 1}</Text>
                  <Descriptions bordered size="small" column={2}>
                    <Descriptions.Item label="Filename">{profile.filename}</Descriptions.Item>
                    <Descriptions.Item label="Exporter">
                      <Tag color="blue">{profile.exporter}</Tag>
                    </Descriptions.Item>
                  </Descriptions>
                    
                  {profile.graphConfig && (
                    <Collapse size="small">
                      <Panel header="Graph Configuration" key="1">
                        <Descriptions bordered size="small" column={2}>
                          {profile.graphConfig.generateHierarchy !== undefined && (
                            <Descriptions.Item label="Generate Hierarchy">
                              {profile.graphConfig.generateHierarchy ? 'Yes' : 'No'}
                            </Descriptions.Item>
                          )}
                          {profile.graphConfig.maxPartitionDepth !== undefined && (
                            <Descriptions.Item label="Max Partition Depth">
                              {profile.graphConfig.maxPartitionDepth}
                            </Descriptions.Item>
                          )}
                          {profile.graphConfig.maxPartitionWidth !== undefined && (
                            <Descriptions.Item label="Max Partition Width">
                              {profile.graphConfig.maxPartitionWidth}
                            </Descriptions.Item>
                          )}
                          {profile.graphConfig.invertGraph !== undefined && (
                            <Descriptions.Item label="Invert Graph">
                              {profile.graphConfig.invertGraph ? 'Yes' : 'No'}
                            </Descriptions.Item>
                          )}
                          {profile.graphConfig.nodeLabelMaxLength !== undefined && (
                            <Descriptions.Item label="Node Label Max Length">
                              {profile.graphConfig.nodeLabelMaxLength}
                            </Descriptions.Item>
                          )}
                          {profile.graphConfig.nodeLabelInsertNewlinesAt !== undefined && (
                            <Descriptions.Item label="Node Label Insert Newlines At">
                              {profile.graphConfig.nodeLabelInsertNewlinesAt}
                            </Descriptions.Item>
                          )}
                          {profile.graphConfig.edgeLabelMaxLength !== undefined && (
                            <Descriptions.Item label="Edge Label Max Length">
                              {profile.graphConfig.edgeLabelMaxLength}
                            </Descriptions.Item>
                          )}
                          {profile.graphConfig.edgeLabelInsertNewlinesAt !== undefined && (
                            <Descriptions.Item label="Edge Label Insert Newlines At">
                              {profile.graphConfig.edgeLabelInsertNewlinesAt}
                            </Descriptions.Item>
                          )}
                        </Descriptions>
                      </Panel>
                    </Collapse>
                  )}
                </Space>
              </List.Item>
            )}
          />
        </Panel>
      </Collapse>
    </Space>
  );

  // Plan Visualization Tab Content - Placeholder
  const planVisualizationContent = (
    <Card style={{ marginTop: 16 }}>
      <Empty 
        image={Empty.PRESENTED_IMAGE_SIMPLE}
        description={
          <Space direction="vertical" align="center">
            <Text>Plan Visualization Placeholder</Text>
            <Paragraph type="secondary">
              This section will contain interactive visualizations for the plan structure, 
              showing relationships between import and export configurations.
            </Paragraph>
          </Space>
        }
      >
        <Button type="primary" disabled>Coming Soon</Button>
      </Empty>
    </Card>
  );

  // Main render
  return (
    <Card>
      <Space direction="vertical" style={{ width: '100%' }} size="large">
        <Space style={{ justifyContent: 'space-between', width: '100%' }}>
          <Title level={2}>Plan Details</Title>
          <Button type="primary">
            <Link to={`/projects/${projectId}`}>Back to Project</Link>
          </Button>
        </Space>

        <Tabs defaultActiveKey="config">
          <Tabs.TabPane tab="Configuration" key="config">
            {planConfigContent}
          </Tabs.TabPane>
          <Tabs.TabPane tab="Visualization" key="visualization">
            {planVisualizationContent}
          </Tabs.TabPane>
        </Tabs>
      </Space>
    </Card>
  );
};

export default PlanDetail;
