import { useQuery } from '@apollo/client';
import { Link } from 'react-router-dom';
import { Table, Card, Typography, Spin, Alert, Space } from 'antd';
import { GET_PROJECTS } from '../graphql/queries';
import { Project } from '../types';

const { Title } = Typography;

const ProjectsList = () => {
  const { loading, error, data } = useQuery(GET_PROJECTS);

  const columns = [
    {
      title: 'Name',
      dataIndex: 'name',
      key: 'name',
      render: (text: string, record: Project) => <Link to={`/projects/${record.id}`}>{text}</Link>,
    },
    {
      title: 'Description',
      dataIndex: 'description',
      key: 'description',
    },
    {
      title: 'Created At',
      dataIndex: 'createdAt',
      key: 'createdAt',
      render: (text: string) => new Date(text).toLocaleString(),
    },
    {
      title: 'Actions',
      key: 'actions',
      render: (_: any, record: Project) => (
        <Space size="middle">
          <Link to={`/projects/${record.id}`}>View</Link>
          <Link to={`/projects/${record.id}/graph`}>Graph</Link>
          <Link to={`/projects/${record.id}/plan`}>Plan</Link>
        </Space>
      ),
    },
  ];

  if (loading) return <Spin size="large" />;
  if (error) return <Alert message="Error loading projects" description={error.message} type="error" showIcon />;

  return (
    <Card>
      <Title level={2}>Projects</Title>
      <Table 
        dataSource={data.projects} 
        columns={columns} 
        rowKey="id"
        pagination={{ pageSize: 10 }}
      />
    </Card>
  );
};

export default ProjectsList;
