import { memo, useState } from 'react'
import { NodeProps } from 'reactflow'
import { useMutation } from '@apollo/client/react'
import { Text, Group, ActionIcon, Tooltip, Badge, Stack } from '@mantine/core'
import { IconDownload } from '@tabler/icons-react'
import { PlanDagNodeType, OutputNodeConfig } from '../../../../types/plan-dag'
import { isNodeConfigured } from '../../../../utils/planDagValidation'
import { getNodeColor } from '../../../../utils/nodeStyles'
import { EXPORT_NODE_OUTPUT, ExportNodeOutputResult } from '../../../../graphql/export'
import { BaseNode } from './BaseNode'

interface OutputNodeProps extends NodeProps {
  onEdit?: (nodeId: string) => void
  onDelete?: (nodeId: string) => void
  readonly?: boolean
}

// Map render targets to file extensions (currently unused - backend generates filename)
// const getRenderTargetExtension = (renderTarget: string): string => {
//   const extensionMap: Record<string, string> = {
//     'DOT': 'dot',
//     'GraphML': 'graphml',
//     'GML': 'gml',
//     'JSON': 'json',
//     'CSV': 'csv',
//     'PNG': 'png',
//     'SVG': 'svg',
//     'PlantUML': 'puml',
//     'Mermaid': 'mermaid',
//   }
//   return extensionMap[renderTarget] || 'txt'
// }

export const OutputNode = memo((props: OutputNodeProps) => {
  const { data, onEdit, onDelete, readonly = false } = props
  const [downloading, setDownloading] = useState(false)

  const config = data.config as OutputNodeConfig
  const color = getNodeColor(PlanDagNodeType.OUTPUT)

  // Check if node is configured
  const edges = data.edges || []
  const hasValidConfig = data.hasValidConfig !== false
  const isConfigured = isNodeConfigured(PlanDagNodeType.OUTPUT, props.id, edges, hasValidConfig)

  // Get project ID from context
  const projectId = data.projectId as number | undefined

  // Export mutation
  const [exportNodeOutput] = useMutation(EXPORT_NODE_OUTPUT, {
    onCompleted: (data: any) => {
      const result = data.exportNodeOutput as ExportNodeOutputResult
      if (result.success) {
        // Decode base64 content and trigger download
        try {
          const binaryString = atob(result.content)
          const bytes = new Uint8Array(binaryString.length)
          for (let i = 0; i < binaryString.length; i++) {
            bytes[i] = binaryString.charCodeAt(i)
          }
          const blob = new Blob([bytes], { type: result.mimeType })
          const url = window.URL.createObjectURL(blob)
          const link = document.createElement('a')
          link.href = url
          link.download = result.filename
          document.body.appendChild(link)
          link.click()
          document.body.removeChild(link)
          window.URL.revokeObjectURL(url)
          console.log('Download completed:', result.filename)
        } catch (error) {
          console.error('Failed to decode and download:', error)
        }
      } else {
        console.error('Export failed:', result.message)
      }
      setDownloading(false)
    },
    onError: (error: any) => {
      console.error('Export failed:', error.message)
      setDownloading(false)
    },
  })

  const handleDownload = async () => {
    if (!projectId || !isConfigured) return

    setDownloading(true)
    exportNodeOutput({
      variables: {
        projectId,
        nodeId: props.id,
      },
    })
  }

  // Custom label badges for output node
  const labelBadges = (
    <>
      <Badge
        variant="light"
        color={color}
        size="xs"
        style={{ textTransform: 'none' }}
      >
        Output
      </Badge>
      {!isConfigured && (
        <Badge variant="outline" size="xs" color="orange">
          Not Configured
        </Badge>
      )}
    </>
  )

  return (
    <BaseNode
      {...props}
      nodeType={PlanDagNodeType.OUTPUT}
      config={config}
      metadata={data.metadata}
      onEdit={() => onEdit?.(props.id)}
      onDelete={() => onDelete?.(props.id)}
      readonly={readonly}
      edges={edges}
      hasValidConfig={hasValidConfig}
      labelBadges={labelBadges}
    >
      <Stack gap="xs">
        {/* Download button */}
        {!readonly && isConfigured && (
          <Group justify="center">
            <Tooltip label="Download export">
              <ActionIcon
                size="xl"
                variant="light"
                color="blue"
                radius="xl"
                data-action-icon="download"
                loading={downloading}
                onMouseDown={(e) => {
                  e.stopPropagation()
                  e.preventDefault()
                  handleDownload()
                }}
              >
                <IconDownload size="1.5rem" />
              </ActionIcon>
            </Tooltip>
          </Group>
        )}

        {/* Output metadata */}
        {config.renderTarget && (
          <Text size="xs" c="dimmed">
            Format: {config.renderTarget}
          </Text>
        )}

        {config.outputPath && (
          <Text size="xs" c="dimmed" ff="monospace" lineClamp={1}>
            {config.outputPath}
          </Text>
        )}
      </Stack>
    </BaseNode>
  )
})

OutputNode.displayName = 'OutputNode'
