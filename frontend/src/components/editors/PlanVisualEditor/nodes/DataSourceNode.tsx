import { memo, useState, useEffect } from 'react'
import { NodeProps } from 'reactflow'
import { useQuery } from '@apollo/client/react'
import { Text, Group, ActionIcon, Tooltip, Badge, Loader } from '@mantine/core'
import { IconAlertCircle, IconTable } from '@tabler/icons-react'
import { PlanDagNodeType, DataSourceNodeConfig } from '../../../../types/plan-dag'
import { isNodeConfigured } from '../../../../utils/planDagValidation'
import { GET_DATASOURCE, DataSource } from '../../../../graphql/datasources'
import { getExecutionStateLabel, getExecutionStateColor, isExecutionComplete, isExecutionInProgress } from '../../../../graphql/preview'
import { DataSourceDataDialog } from '../dialogs/DataSourceDataDialog'
import { BaseNode } from './BaseNode'

interface DataSourceNodeProps extends NodeProps {
  onEdit?: (nodeId: string) => void
  onDelete?: (nodeId: string) => void
  readonly?: boolean
}

export const DataSourceNode = memo((props: DataSourceNodeProps) => {
  const { data, onEdit, onDelete, readonly = false } = props
  const [dataSourceInfo, setDataSourceInfo] = useState<DataSource | null>(null)
  const [showDataDialog, setShowDataDialog] = useState(false)

  const config = data.config as DataSourceNodeConfig

  // Check if node is configured
  const edges = data.edges || []
  const hasValidConfig = data.hasValidConfig !== false
  const isConfigured = isNodeConfigured(PlanDagNodeType.DATA_SOURCE, props.id, edges, hasValidConfig)

  // Use inline execution metadata from PlanDAG query, only query if not available
  const datasourceExecution = data.datasourceExecution
  const needsQuery = !datasourceExecution && config.dataSourceId

  // Fallback query only if inline data not available
  const { data: dataSourceData } = useQuery(GET_DATASOURCE, {
    variables: { id: config.dataSourceId || 0 },
    skip: !needsQuery,
    errorPolicy: 'ignore'
  })

  useEffect(() => {
    if ((dataSourceData as any)?.dataSource) {
      setDataSourceInfo((dataSourceData as any).dataSource as DataSource)
    }
  }, [dataSourceData])

  // Custom label badges for data source node
  const labelBadges = (
    <>
      {!isConfigured && (
        <Badge variant="outline" size="xs" color="orange">
          Not Configured
        </Badge>
      )}
      {datasourceExecution && (
        <Badge
          variant={isExecutionComplete(datasourceExecution.executionState) ? 'light' : 'filled'}
          color={getExecutionStateColor(datasourceExecution.executionState)}
          size="xs"
          leftSection={isExecutionInProgress(datasourceExecution.executionState) ? <Loader size={10} /> : undefined}
        >
          {getExecutionStateLabel(datasourceExecution.executionState)}
        </Badge>
      )}
    </>
  )

  return (
    <>
      <BaseNode
        {...props}
        nodeType={PlanDagNodeType.DATA_SOURCE}
        config={config}
        metadata={data.metadata}
        onEdit={() => onEdit?.(props.id)}
        onDelete={() => onDelete?.(props.id)}
        readonly={readonly}
        edges={edges}
        hasValidConfig={hasValidConfig}
        labelBadges={labelBadges}
      >
        {/* View data button - only show if configured and active */}
        {!readonly && isConfigured && (datasourceExecution?.status === 'active' || dataSourceInfo?.status === 'active') && (
          <Group justify="center">
            <Tooltip label="View datasource data (nodes, edges, layers)">
              <ActionIcon
                size="xl"
                variant="light"
                color="teal"
                radius="xl"
                data-action-icon="data"
                onMouseDown={(e) => {
                  e.stopPropagation()
                  e.preventDefault()
                  setShowDataDialog(true)
                }}
              >
                <IconTable size="1.5rem" />
              </ActionIcon>
            </Tooltip>
          </Group>
        )}

        {/* Show error message if there's an error */}
        {datasourceExecution?.status === 'error' && datasourceExecution.errorMessage && (
          <Group gap="xs">
            <IconAlertCircle size={12} color="red" />
            <Text size="xs" c="red" lineClamp={1} title={datasourceExecution.errorMessage}>
              Error processing
            </Text>
          </Group>
        )}
        {dataSourceInfo?.status === 'error' && dataSourceInfo.errorMessage && (
          <Group gap="xs">
            <IconAlertCircle size={12} color="red" />
            <Text size="xs" c="red" lineClamp={1} title={dataSourceInfo.errorMessage}>
              Error processing
            </Text>
          </Group>
        )}
      </BaseNode>

      {/* Data Source Data Dialog */}
      <DataSourceDataDialog
        opened={showDataDialog}
        onClose={() => setShowDataDialog(false)}
        dataSourceId={config.dataSourceId || null}
        title={`Data Source: ${data.metadata.label}`}
      />
    </>
  )
})

DataSourceNode.displayName = 'DataSourceNode'
