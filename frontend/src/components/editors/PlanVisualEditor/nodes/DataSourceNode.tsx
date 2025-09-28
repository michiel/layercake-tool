import { memo, useState, useEffect } from 'react'
import { NodeProps, Handle, Position } from 'reactflow'
import { useQuery } from '@apollo/client/react'
import { Paper, Text, Group, ActionIcon, Tooltip, Badge, Stack, Loader } from '@mantine/core'
import { IconSettings, IconTrash, IconFile, IconAlertCircle, IconCheck, IconClock, IconX } from '@tabler/icons-react'
import { PlanDagNodeType, DataSourceNodeConfig } from '../../../../types/plan-dag'
import { getNodeTypeColor } from '../../../../utils/planDagValidation'
import { GET_DATASOURCE, DataSource, getDataSourceTypeDisplayName, formatFileSize, getStatusColor } from '../../../../graphql/datasources'

// Helper function to get data freshness information
const getDataFreshness = (dataSource: DataSource) => {
  if (!dataSource.processedAt) {
    return { text: 'Never processed', color: 'red', priority: 'high' }
  }

  const processed = new Date(dataSource.processedAt)
  const now = new Date()
  const diffMs = now.getTime() - processed.getTime()
  const diffHours = diffMs / (1000 * 60 * 60)
  const diffDays = diffMs / (1000 * 60 * 60 * 24)

  if (diffHours < 1) {
    return { text: 'Just processed', color: 'green', priority: 'low' }
  } else if (diffHours < 24) {
    return { text: `${Math.round(diffHours)}h ago`, color: 'green', priority: 'low' }
  } else if (diffDays < 7) {
    return { text: `${Math.round(diffDays)}d ago`, color: 'blue', priority: 'medium' }
  } else if (diffDays < 30) {
    return { text: `${Math.round(diffDays)}d ago`, color: 'orange', priority: 'medium' }
  } else {
    return { text: `${Math.round(diffDays)}d ago`, color: 'red', priority: 'high' }
  }
}

interface DataSourceNodeProps extends NodeProps {
  onEdit?: (nodeId: string) => void
  onDelete?: (nodeId: string) => void
  readonly?: boolean
}

export const DataSourceNode = memo((props: DataSourceNodeProps) => {
  const { data, selected, onEdit, onDelete, readonly = false } = props
  const [dataSourceInfo, setDataSourceInfo] = useState<DataSource | null>(null)

  const config = data.config as DataSourceNodeConfig
  const color = getNodeTypeColor(PlanDagNodeType.DATA_SOURCE)

  // Query DataSource details if dataSourceId is available
  const { data: dataSourceData, loading: dataSourceLoading } = useQuery(GET_DATASOURCE, {
    variables: { id: config.dataSourceId || 0 },
    skip: !config.dataSourceId,
    errorPolicy: 'ignore'
  })

  useEffect(() => {
    if ((dataSourceData as any)?.dataSource) {
      setDataSourceInfo((dataSourceData as any).dataSource as DataSource)
    }
  }, [dataSourceData])

  const getStatusIcon = (status: DataSource['status']) => {
    switch (status) {
      case 'active':
        return <IconCheck size={12} />
      case 'processing':
        return <IconClock size={12} />
      case 'error':
        return <IconX size={12} />
      default:
        return null
    }
  }

  const renderDataSourceContent = () => {
    if (dataSourceLoading) {
      return (
        <Group gap="xs">
          <Loader size="xs" />
          <Text size="xs" c="dimmed">Loading...</Text>
        </Group>
      )
    }

    if (dataSourceInfo) {
      const freshness = getDataFreshness(dataSourceInfo)

      return (
        <Stack gap="xs">
          <Group gap="xs" justify="space-between">
            <Group gap="xs">
              <Badge
                size="xs"
                color={getStatusColor(dataSourceInfo.status)}
                leftSection={getStatusIcon(dataSourceInfo.status)}
                style={{
                  animation: dataSourceInfo.status === 'processing' ? 'pulse 2s infinite' : undefined
                }}
              >
                {dataSourceInfo.status}
              </Badge>
              <Badge variant="outline" size="xs">
                {getDataSourceTypeDisplayName(dataSourceInfo.sourceType)}
              </Badge>
            </Group>

            {/* Data freshness indicator */}
            <Tooltip label={`Last processed: ${dataSourceInfo.processedAt ? new Date(dataSourceInfo.processedAt).toLocaleString() : 'Never'}`}>
              <Badge
                size="xs"
                color={freshness.color}
                variant={freshness.priority === 'high' ? 'filled' : 'light'}
                style={{ cursor: 'help' }}
              >
                {freshness.text}
              </Badge>
            </Tooltip>
          </Group>

          <Text size="xs" c="dimmed" ff="monospace" lineClamp={1}>
            {dataSourceInfo.filename}
          </Text>

          <Group gap="xs" justify="space-between">
            <Text size="xs" c="dimmed">
              {formatFileSize(dataSourceInfo.fileSize)}
            </Text>
            <Text size="xs" c="dimmed">
              {new Date(dataSourceInfo.updatedAt).toLocaleDateString()}
            </Text>
          </Group>

          {dataSourceInfo.status === 'error' && dataSourceInfo.errorMessage && (
            <Group gap="xs">
              <IconAlertCircle size={12} color="red" />
              <Text size="xs" c="red" lineClamp={1} title={dataSourceInfo.errorMessage}>
                Error processing
              </Text>
            </Group>
          )}
        </Stack>
      )
    }

    // Fallback for legacy or missing DataSource
    if (config.source || config.dataType) {
      return (
        <Stack gap="xs">
          <Badge variant="outline" size="xs" color="gray">
            Legacy
          </Badge>
          <Text size="xs" c="dimmed">
            {config.dataType}: {config.source}
          </Text>
        </Stack>
      )
    }

    // No DataSource configured
    return (
      <Stack gap="xs">
        <Badge variant="outline" size="xs" color="orange">
          Not Configured
        </Badge>
        <Text size="xs" c="dimmed">
          Click to select DataSource
        </Text>
      </Stack>
    )
  }

  return (
    <>
      {/* Output Handles - DataSource nodes only have outputs, no inputs */}
      {/* Right Output Handle */}
      <Handle
        type="source"
        position={Position.Right}
        id="output-right"
        style={{
          background: '#fff',
          border: `2px solid ${color}`,
          width: 12,
          height: 12,
          borderRadius: '0', // Square for outputs
        }}
      />

      {/* Bottom Output Handle */}
      <Handle
        type="source"
        position={Position.Bottom}
        id="output-bottom"
        style={{
          background: '#fff',
          border: `2px solid ${color}`,
          width: 12,
          height: 12,
          borderRadius: '0', // Square for outputs
        }}
      />

      {/* Node Content */}
      <Paper
        shadow={selected ? "md" : "sm"}
        p="sm"
        style={{
          border: selected ? `2px solid ${color}` : `1px solid #e9ecef`,
          borderRadius: 8,
          minWidth: 200,
          maxWidth: 280,
          background: '#fff',
          cursor: 'default', // Remove pointer cursor since we don't want click-to-edit
        }}
      >
        <Group justify="space-between" mb="xs">
          <Group gap="xs">
            <IconFile size={14} />
            <Badge
              color={color}
              variant="light"
              size="sm"
            >
              Data Source
            </Badge>
          </Group>

          {!readonly && (
            <Group gap="xs">
              <Tooltip label="Edit DataSource">
                <ActionIcon
                  size="sm"
                  variant="subtle"
                  color="gray"
                  onClick={() => onEdit?.(props.id)}
                >
                  <IconSettings size="0.8rem" />
                </ActionIcon>
              </Tooltip>
              <Tooltip label="Delete node">
                <ActionIcon
                  size="sm"
                  variant="subtle"
                  color="red"
                  onClick={() => onDelete?.(props.id)}
                >
                  <IconTrash size="0.8rem" />
                </ActionIcon>
              </Tooltip>
            </Group>
          )}
        </Group>

        <Text size="sm" fw={500} mb="xs">
          {data.metadata.label}
        </Text>

        {data.metadata.description && (
          <Text size="xs" c="dimmed" lineClamp={2} mb="xs">
            {data.metadata.description}
          </Text>
        )}

        {/* DataSource-specific content */}
        <div style={{ marginTop: 8 }}>
          {renderDataSourceContent()}
        </div>
      </Paper>
    </>
  )
})

DataSourceNode.displayName = 'DataSourceNode'