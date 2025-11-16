import { memo, useState, useEffect } from 'react'
import { NodeProps } from 'reactflow'
import { useQuery } from '@apollo/client/react'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { Spinner } from '@/components/ui/spinner'
import { Group } from '@/components/layout-primitives'
import { IconAlertCircle, IconTable } from '@tabler/icons-react'
import { PlanDagNodeType, DataSetNodeConfig } from '../../../../types/plan-dag'
import { isNodeConfigured } from '../../../../utils/planDagValidation'
import { GET_DATASOURCE, DataSet } from '../../../../graphql/datasets'
import { getExecutionStateLabel, getExecutionStateColor, isExecutionComplete, isExecutionInProgress } from '../../../../graphql/preview'
import { DataSetDataDialog } from '../dialogs/DataSetDataDialog'
import { BaseNode } from './BaseNode'

interface DataSetNodeProps extends NodeProps {
  onEdit?: (nodeId: string) => void
  onDelete?: (nodeId: string) => void
  readonly?: boolean
}

export const DataSetNode = memo((props: DataSetNodeProps) => {
  const { data, onEdit, onDelete, readonly = false } = props
  const [dataSourceInfo, setDataSetInfo] = useState<DataSet | null>(null)
  const [showDataDialog, setShowDataDialog] = useState(false)

  const config = data.config as DataSetNodeConfig

  // Check if node is configured
  const edges = data.edges || []
  const hasValidConfig = data.hasValidConfig !== false
  const isConfigured = isNodeConfigured(PlanDagNodeType.DATA_SOURCE, props.id, edges, hasValidConfig)

  // Use inline execution metadata from PlanDAG query, only query if not available
  const datasetExecution = data.datasetExecution
  const needsQuery = !datasetExecution && config.dataSetId

  // Fallback query only if inline data not available
  const { data: dataSourceData } = useQuery(GET_DATASOURCE, {
    variables: { id: config.dataSetId || 0 },
    skip: !needsQuery,
    errorPolicy: 'ignore'
  })

  useEffect(() => {
    if ((dataSourceData as any)?.dataSet) {
      setDataSetInfo((dataSourceData as any).dataSet as DataSet)
    }
  }, [dataSourceData])

  // Helper to get badge classes based on Mantine color
  const getBadgeClasses = (color: string, variant: 'filled' | 'light' | 'outline') => {
    const colorMap: Record<string, { filled: string; light: string; outline: string }> = {
      orange: {
        filled: 'bg-orange-600 text-white border-orange-600',
        light: 'bg-orange-100 text-orange-800 border-orange-200',
        outline: 'text-orange-600 border-orange-600',
      },
      blue: {
        filled: 'bg-blue-600 text-white border-blue-600',
        light: 'bg-blue-100 text-blue-800 border-blue-200',
        outline: 'text-blue-600 border-blue-600',
      },
      green: {
        filled: 'bg-green-600 text-white border-green-600',
        light: 'bg-green-100 text-green-800 border-green-200',
        outline: 'text-green-600 border-green-600',
      },
      yellow: {
        filled: 'bg-yellow-600 text-white border-yellow-600',
        light: 'bg-yellow-100 text-yellow-800 border-yellow-200',
        outline: 'text-yellow-600 border-yellow-600',
      },
      red: {
        filled: 'bg-red-600 text-white border-red-600',
        light: 'bg-red-100 text-red-800 border-red-200',
        outline: 'text-red-600 border-red-600',
      },
      gray: {
        filled: 'bg-gray-600 text-white border-gray-600',
        light: 'bg-gray-100 text-gray-800 border-gray-200',
        outline: 'text-gray-600 border-gray-600',
      },
    }
    return colorMap[color]?.[variant] || colorMap.gray[variant]
  }

  // Custom label badges for data source node
  const hasBadges = !isConfigured || (datasetExecution && !isExecutionComplete(datasetExecution.executionState))
  const labelBadges = hasBadges ? (
    <>
      {!isConfigured && (
        <Badge variant="outline" className={`text-xs ${getBadgeClasses('orange', 'outline')}`}>
          Not Configured
        </Badge>
      )}
      {datasetExecution && !isExecutionComplete(datasetExecution.executionState) && (
        <Badge
          variant={isExecutionComplete(datasetExecution.executionState) ? 'secondary' : 'default'}
          className={`text-xs ${getBadgeClasses(
            getExecutionStateColor(datasetExecution.executionState),
            isExecutionComplete(datasetExecution.executionState) ? 'light' : 'filled'
          )}`}
        >
          <span className="flex items-center gap-1">
            {isExecutionInProgress(datasetExecution.executionState) && <Spinner size="xs" />}
            {getExecutionStateLabel(datasetExecution.executionState)}
          </span>
        </Badge>
      )}
    </>
  ) : null

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
        {!readonly && isConfigured && (datasetExecution?.status === 'active' || dataSourceInfo?.status === 'active') && (
          <Group justify="center">
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                <Button
                  size="icon"
                  variant="ghost"
                  className="h-9 w-9 rounded-full text-teal-600"
                  data-action-icon="data"
                  onMouseDown={(e: React.MouseEvent) => {
                    e.stopPropagation()
                    e.preventDefault()
                    setShowDataDialog(true)
                  }}
                >
                    <IconTable size={12} />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>View dataset data (nodes, edges, layers)</TooltipContent>
              </Tooltip>
            </TooltipProvider>
          </Group>
        )}

        {/* Show error message if there's an error */}
        {datasetExecution?.status === 'error' && datasetExecution.errorMessage && (
          <Group gap="xs">
            <IconAlertCircle size={12} className="text-red-600" />
            <p className="text-xs text-red-600 line-clamp-1" title={datasetExecution.errorMessage}>
              Error processing
            </p>
          </Group>
        )}
        {dataSourceInfo?.status === 'error' && dataSourceInfo.errorMessage && (
          <Group gap="xs">
            <IconAlertCircle size={12} className="text-red-600" />
            <p className="text-xs text-red-600 line-clamp-1" title={dataSourceInfo.errorMessage}>
              Error processing
            </p>
          </Group>
        )}
      </BaseNode>

      {/* Data Source Data Dialog */}
      <DataSetDataDialog
        opened={showDataDialog}
        onClose={() => setShowDataDialog(false)}
        dataSetId={config.dataSetId || null}
        title={`Data Source: ${data.metadata.label}`}
      />
    </>
  )
})

DataSetNode.displayName = 'DataSetNode'
