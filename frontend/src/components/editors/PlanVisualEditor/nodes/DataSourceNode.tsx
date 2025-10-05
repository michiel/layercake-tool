import { memo, useState, useEffect } from 'react'
import { NodeProps, Handle, Position } from 'reactflow'
import { useQuery } from '@apollo/client/react'
import { Paper, Text, Group, ActionIcon, Tooltip, Badge, Stack, Loader } from '@mantine/core'
import { IconSettings, IconTrash, IconAlertCircle, IconCheck, IconClock, IconX, IconTable } from '@tabler/icons-react'
import { PlanDagNodeType, DataSourceNodeConfig } from '../../../../types/plan-dag'
import { isNodeConfigured } from '../../../../utils/planDagValidation'
import { getNodeColor, getNodeIcon, getNodeTypeLabel } from '../../../../utils/nodeStyles'
import { GET_DATASOURCE, DataSource, getFileFormatDisplayName, getDataTypeDisplayName, formatFileSize, getStatusColor } from '../../../../graphql/datasources'
import { useDataSourcePreview } from '../../../../hooks/usePreview'
import { getExecutionStateLabel, getExecutionStateColor, isExecutionComplete, isExecutionInProgress } from '../../../../graphql/preview'
import { DataPreviewDialog } from '../../../visualization/DataPreviewDialog'

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
  const [showPreview, setShowPreview] = useState(false)

  const config = data.config as DataSourceNodeConfig
  const color = getNodeColor(PlanDagNodeType.DATA_SOURCE)

  // Check if node is configured
  const edges = data.edges || []
  const hasValidConfig = data.hasValidConfig !== false
  const isConfigured = isNodeConfigured(PlanDagNodeType.DATA_SOURCE, props.id, edges, hasValidConfig)

  // Get project ID from context
  const projectId = data.projectId as number | undefined

  // Query DataSource details if dataSourceId is available
  const { data: dataSourceData, loading: dataSourceLoading } = useQuery(GET_DATASOURCE, {
    variables: { id: config.dataSourceId || 0 },
    skip: !config.dataSourceId,
    errorPolicy: 'ignore'
  })

  // Query data source preview (for table data)
  const { preview: dataPreview, loading: previewLoading, error: previewError } = useDataSourcePreview(
    projectId || 0,
    props.id,
    { skip: !showPreview || !projectId }
  )

  // Query execution state (always fetch to show status)
  const { preview: executionPreview } = useDataSourcePreview(
    projectId || 0,
    props.id,
    { skip: !projectId, limit: 0 }
  )

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
          <Group gap="xs" wrap="wrap">
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
            <Badge variant="outline" size="xs" color="blue">
              {getFileFormatDisplayName(dataSourceInfo.fileFormat)}
            </Badge>
            <Badge variant="outline" size="xs" color="green">
              {getDataTypeDisplayName(dataSourceInfo.dataType)}
            </Badge>
          </Group>

          <Text size="xs" c="dimmed" ff="monospace" lineClamp={1}>
            {dataSourceInfo.filename}
          </Text>

          <Group gap="xs" justify="space-between" wrap="wrap">
            <Text size="xs" c="dimmed">
              {formatFileSize(dataSourceInfo.fileSize)}
            </Text>
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
      <Text size="xs" c="dimmed">
        Click edit to select a data source
      </Text>
    )
  }

  return (
    <>
      {/* Output Handles - DataSource nodes only have outputs, no inputs */}
      <Handle
        type="source"
        position={Position.Right}
        id="output-right"
        style={{
          background: '#fff',
          border: `2px solid ${color}`,
          width: 12,
          height: 12,
          borderRadius: '0',
        }}
      />
      <Handle
        type="source"
        position={Position.Bottom}
        id="output-bottom"
        style={{
          background: '#fff',
          border: `2px solid ${color}`,
          width: 12,
          height: 12,
          borderRadius: '0',
        }}
      />

      {/* Node Content */}
      <Paper
        shadow={selected ? "md" : "sm"}
        p="md"
        style={{
          border: `2px solid ${color}`,
          borderRadius: 8,
          minWidth: 200,
          maxWidth: 280,
          background: '#fff',
          cursor: 'default',
          pointerEvents: 'all',
        }}
      >
        {/* Top right: Edit and Delete icons */}
        {!readonly && (
          <Group gap={4} style={{ position: 'absolute', top: 8, right: 8, pointerEvents: 'auto', zIndex: 10 }}>
            <Tooltip label="Edit data source">
              <ActionIcon
                size="sm"
                variant="subtle"
                color="gray"
                data-action-icon="edit"
                onMouseDown={(e) => {
                  e.stopPropagation()
                  e.preventDefault()
                  console.log('Edit icon mousedown for data source, calling onEdit')
                  onEdit?.(props.id)
                }}
              >
                <IconSettings size="0.8rem" />
              </ActionIcon>
            </Tooltip>
            <Tooltip label="Delete node">
              <ActionIcon
                size="sm"
                variant="subtle"
                color="red"
                data-action-icon="delete"
                onMouseDown={(e) => {
                  e.stopPropagation()
                  e.preventDefault()
                  console.log('Delete icon mousedown for data source, calling onDelete')
                  onDelete?.(props.id)
                }}
              >
                <IconTrash size="0.8rem" />
              </ActionIcon>
            </Tooltip>
          </Group>
        )}

        {/* Middle: Icon and Label */}
        <Group gap="sm" mb="sm" wrap="nowrap" className="node-header" style={{ paddingRight: !readonly ? 60 : 0, cursor: 'grab' }}>
          <div style={{
            color,
            display: 'flex',
            alignItems: 'center',
            flexShrink: 0
          }}>
            {getNodeIcon(PlanDagNodeType.DATA_SOURCE, '1.4rem')}
          </div>
          <Text size="sm" fw={600} lineClamp={2} style={{ wordBreak: 'break-word', flex: 1, minWidth: 0 }}>
            {data.metadata.label}
          </Text>
        </Group>

        {/* Center: Table icon for data preview */}
        {!readonly && isConfigured && (
          <Group justify="center" mb="md">
            <Tooltip label="Preview data">
              <ActionIcon
                size="xl"
                variant="light"
                color="blue"
                radius="xl"
                data-action-icon="preview"
                onMouseDown={(e) => {
                  e.stopPropagation()
                  e.preventDefault()
                  setShowPreview(true)
                }}
              >
                <IconTable size="1.5rem" />
              </ActionIcon>
            </Tooltip>
          </Group>
        )}

        {/* Bottom: Labels and data source details */}
        <Stack gap="xs">
          <Group gap="xs" wrap="wrap">
            <Badge
              variant="light"
              color={color}
              size="xs"
              style={{ textTransform: 'none' }}
            >
              {getNodeTypeLabel(PlanDagNodeType.DATA_SOURCE)}
            </Badge>
            {!isConfigured && (
              <Badge variant="outline" size="xs" color="orange">
                Not Configured
              </Badge>
            )}
            {executionPreview && (
              <Badge
                variant={isExecutionComplete(executionPreview.executionState) ? 'light' : 'filled'}
                color={getExecutionStateColor(executionPreview.executionState)}
                size="xs"
                leftSection={isExecutionInProgress(executionPreview.executionState) ? <Loader size={10} /> : undefined}
              >
                {getExecutionStateLabel(executionPreview.executionState)}
              </Badge>
            )}
          </Group>

          {/* DataSource-specific content */}
          {renderDataSourceContent()}
        </Stack>
      </Paper>

      {/* Data Preview Dialog */}
      <DataPreviewDialog
        opened={showPreview}
        onClose={() => setShowPreview(false)}
        preview={dataPreview || null}
        loading={previewLoading}
        error={previewError}
        title={`Data Preview: ${data.metadata.label}`}
      />
    </>
  )
})

DataSourceNode.displayName = 'DataSourceNode'
