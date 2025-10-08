import React from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Container, Title, Alert, LoadingOverlay, Button, Stack } from '@mantine/core';
import { IconAlertCircle, IconArrowLeft } from '@tabler/icons-react';
import { useQuery } from '@apollo/client/react';
import { gql } from '@apollo/client';
import { Breadcrumbs } from '../components/common/Breadcrumbs';
import { LayercakeGraphEditor } from '../components/graphs/LayercakeGraphEditor';
import { ReactFlowProvider } from 'reactflow';
import { Graph } from '@/graphql/graphs';

const GET_PROJECTS = gql`
  query GetProjects {
    projects {
      id
      name
    }
  }
`;

const GET_GRAPH_DETAILS = gql`
  query GetGraphDetails($id: Int!) {
    graph(id: $id) {
      id
      name
      nodeId
      executionState
      nodeCount
      edgeCount
      createdAt
      updatedAt
      layers {
        id
        name
        color
      }
      graphNodes {
        id
        label
        layer
        weight
        isPartition
        attrs
      }
      graphEdges {
        id
        source
        target
        label
        layer
        weight
        attrs
      }
    }
  }
`;

interface GraphEditorPageProps {}

export const GraphEditorPage: React.FC<GraphEditorPageProps> = () => {
  const navigate = useNavigate();
  const { projectId, graphId } = useParams<{ projectId: string; graphId: string }>();

  const { data: projectsData } = useQuery<{ projects: Array<{ id: number; name: string }> }>(GET_PROJECTS);
  const selectedProject = projectsData?.projects.find((p: { id: number; name: string }) => p.id === parseInt(projectId || '0'));

  const { data: graphData, loading: graphLoading, error: graphError } = useQuery<{ graph: Graph }, { id: number }>(GET_GRAPH_DETAILS, {
    variables: { id: parseInt(graphId || '0') },
    skip: !graphId,
  });

  const graph: Graph | null = graphData?.graph || null;

  const handleNavigate = (route: string) => {
    navigate(route);
  };

  const handleBack = () => {
    navigate(`/projects/${projectId}/graphs`);
  };

  if (!selectedProject) {
    return (
      <Container size="xl">
        <Title order={1}>Project Not Found</Title>
        <Button onClick={() => navigate('/projects')} mt="md">
          Back to Projects
        </Button>
      </Container>
    );
  }

  if (graphLoading) {
    return (
      <Container size="xl">
        <LoadingOverlay visible />
        <div style={{ height: '400px' }} />
      </Container>
    );
  }

  if (graphError || !graph) {
    return (
      <Container size="xl">
        <Alert
          icon={<IconAlertCircle size={16} />}
          title="Error Loading Graph"
          color="red"
          mb="md"
        >
          {graphError?.message || 'Graph not found'}
        </Alert>
        <Button onClick={handleBack} leftSection={<IconArrowLeft size={16} />}>
          Back to Graphs
        </Button>
      </Container>
    );
  }

  return (
    <Stack h="100%" gap={0}>
      <div style={{ padding: '16px', borderBottom: '1px solid #e9ecef' }}>
        <Breadcrumbs
          projectName={selectedProject.name}
          projectId={selectedProject.id}
          currentPage={`Graphs > ${graph.name}`}
          onNavigate={handleNavigate}
        />
      </div>

      <div style={{ flex: 1, overflow: 'hidden', position: 'relative' }}>
        <ReactFlowProvider>
          <LayercakeGraphEditor graph={graph} />
        </ReactFlowProvider>
      </div>

      {/* Add save/cancel buttons here later */}
    </Stack>
  );
};
