import React, { useState } from 'react'
import { useQuery } from '@apollo/client/react'
import {
  Modal,
  Title,
  Button,
  Group,
  Stack,
  Text,
  Badge,
  Card,
  ScrollArea,
  TextInput,
  Alert,
  LoadingOverlay,
  Tooltip,
  ActionIcon
} from '@mantine/core'
import {
  IconSearch,
  IconFile,
  IconAlertCircle,
  IconCheck,
  IconClock,
  IconX,
  IconRefresh
} from '@tabler/icons-react'
import {
  GET_DATASOURCES,
  DataSource,
  getFileFormatDisplayName,
  getDataTypeDisplayName,
  formatFileSize,
  getStatusColor
} from '../../../../graphql/datasources'

interface DataSourceSelectionDialogProps {
  opened: boolean
  onClose: () => void
  onSelect: (dataSource: DataSource) => void
  currentDataSourceId?: number
  projectId: number
}

export const DataSourceSelectionDialog: React.FC<DataSourceSelectionDialogProps> = ({
  opened,
  onClose,
  onSelect,
  currentDataSourceId,
  projectId
}) => {
  const [searchQuery, setSearchQuery] = useState('')

  // Query for DataSources in current project
  const {
    data: dataSourcesData,
    loading: dataSourcesLoading,
    error: dataSourcesError,
    refetch: refetchDataSources
  } = useQuery(GET_DATASOURCES, {
    variables: { projectId },
    skip: !projectId || projectId === 0,
    errorPolicy: 'all',
    fetchPolicy: 'cache-and-network'
  })

  const dataSources: DataSource[] = (dataSourcesData as any)?.dataSources || []

  // Filter DataSources based on search query
  const filteredDataSources = dataSources.filter(ds =>
    ds.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    ds.filename.toLowerCase().includes(searchQuery.toLowerCase()) ||
    (ds.description || '').toLowerCase().includes(searchQuery.toLowerCase())
  )

  const handleSelect = (dataSource: DataSource) => {
    onSelect(dataSource)
    onClose()
  }

  const getStatusIcon = (status: DataSource['status']) => {
    switch (status) {
      case 'active':
        return <IconCheck size={16} />
      case 'processing':
        return <IconClock size={16} />
      case 'error':
        return <IconX size={16} />
      default:
        return null
    }
  }

  return (
    <Modal
      opened={opened}
      onClose={onClose}
      title={<Title order={4}>Select Data Source</Title>}
      size="lg"
    >
      <Stack gap="md">
        <Group>
          <TextInput
            placeholder="Search data sources..."
            leftSection={<IconSearch size={16} />}
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.currentTarget.value)}
            style={{ flex: 1 }}
          />
          <Tooltip label="Refresh data sources">
            <ActionIcon variant="light" onClick={() => refetchDataSources()}>
              <IconRefresh size={16} />
            </ActionIcon>
          </Tooltip>
        </Group>

        {dataSourcesError && (
          <Alert
            icon={<IconAlertCircle size={16} />}
            title="Error Loading Data Sources"
            color="red"
          >
            <div>
              <Text size="sm">{dataSourcesError.message}</Text>
              <Text size="xs" mt="xs" c="dimmed">
                Project ID: {projectId}, Query Variables: {JSON.stringify({ projectId })}
              </Text>
            </div>
          </Alert>
        )}

        {/* Show loading state when projectId is invalid */}
        {(!projectId || projectId === 0) && !dataSourcesError && (
          <Alert
            icon={<IconAlertCircle size={16} />}
            title="Invalid Project"
            color="orange"
          >
            <Text size="sm">
              No valid project ID provided. Project ID: {projectId}
            </Text>
          </Alert>
        )}

        <ScrollArea h={400} style={{ position: 'relative' }}>
          <LoadingOverlay visible={dataSourcesLoading} />

          {filteredDataSources.length === 0 && !dataSourcesLoading ? (
            <Stack align="center" py="xl" gap="md">
              <IconFile size={48} color="gray" />
              <div style={{ textAlign: 'center' }}>
                <Title order={4}>No Data Sources Found</Title>
                <Text c="dimmed" size="sm">
                  {searchQuery
                    ? 'No data sources match your search criteria.'
                    : 'Create data sources to use them in your Plan DAG.'
                  }
                </Text>
              </div>
            </Stack>
          ) : (
            <Stack gap="sm">
              {filteredDataSources.map((dataSource) => (
                <Card
                  key={dataSource.id}
                  withBorder
                  style={{
                    cursor: dataSource.status === 'active' ? 'pointer' : 'not-allowed',
                    opacity: dataSource.status === 'active' ? 1 : 0.7,
                    border: currentDataSourceId === dataSource.id ? '2px solid var(--mantine-color-blue-filled)' : undefined,
                    backgroundColor: currentDataSourceId === dataSource.id ? 'var(--mantine-color-blue-0)' : undefined
                  }}
                  onClick={() => dataSource.status === 'active' && handleSelect(dataSource)}
                  p="sm"
                >
                  <Group justify="space-between" align="flex-start">
                    <div style={{ flex: 1 }}>
                      <Group gap="sm" mb="xs">
                        <Text fw={500}>{dataSource.name}</Text>
                        {currentDataSourceId === dataSource.id && (
                          <Badge color="blue" size="sm">
                            Current
                          </Badge>
                        )}
                      </Group>

                      {dataSource.description && (
                        <Text size="sm" c="dimmed" mb="xs">
                          {dataSource.description}
                        </Text>
                      )}

                      <Group gap="xs" mb="xs">
                        <Badge
                          variant="light"
                          color={getStatusColor(dataSource.status)}
                          leftSection={getStatusIcon(dataSource.status)}
                          size="sm"
                        >
                          {dataSource.status}
                        </Badge>
                        <Badge variant="outline" size="sm" color="blue">
                          {getFileFormatDisplayName(dataSource.fileFormat)}
                        </Badge>
                        <Badge variant="outline" size="sm" color="green">
                          {getDataTypeDisplayName(dataSource.dataType)}
                        </Badge>
                      </Group>

                      <Group gap="sm" align="center">
                        <Text size="xs" c="dimmed" ff="monospace">
                          {dataSource.filename}
                        </Text>
                        <Text size="xs" c="dimmed">
                          •
                        </Text>
                        <Text size="xs" c="dimmed">
                          {formatFileSize(dataSource.fileSize)}
                        </Text>
                        <Text size="xs" c="dimmed">
                          •
                        </Text>
                        <Text size="xs" c="dimmed">
                          {new Date(dataSource.updatedAt).toLocaleDateString()}
                        </Text>
                      </Group>

                      {dataSource.status === 'error' && dataSource.errorMessage && (
                        <Group gap="xs" mt="xs">
                          <IconAlertCircle size={14} color="red" />
                          <Text size="xs" c="red" lineClamp={1}>
                            {dataSource.errorMessage}
                          </Text>
                        </Group>
                      )}
                    </div>

                    {dataSource.status === 'active' && (
                      <Button
                        size="xs"
                        variant="light"
                        onClick={(e) => {
                          e.stopPropagation()
                          handleSelect(dataSource)
                        }}
                      >
                        Select
                      </Button>
                    )}
                  </Group>
                </Card>
              ))}
            </Stack>
          )}
        </ScrollArea>

        <Group justify="flex-end">
          <Button variant="light" onClick={onClose}>
            Cancel
          </Button>
        </Group>
      </Stack>
    </Modal>
  )
}