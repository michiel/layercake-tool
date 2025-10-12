import { memo, useState } from 'react'
import { NodeProps, Handle, Position } from 'reactflow'
import { useMutation } from '@apollo/client/react'
import { Paper, Text, Group, ActionIcon, Tooltip, Badge, Stack } from '@mantine/core'
import { IconSettings, IconTrash, IconDownload } from '@tabler/icons-react'
import { PlanDagNodeType, OutputNodeConfig } from '../../../../types/plan-dag'
import { isNodeConfigured } from '../../../../utils/planDagValidation'
import { getNodeColor, getNodeIcon, getNodeTypeLabel } from '../../../../utils/nodeStyles'
import { EXPORT_NODE_OUTPUT, ExportNodeOutputResult } from '../../../../graphql/export'

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
  const { data, selected, onEdit, onDelete, readonly = false } = props
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

  return (
    <>
      {/* Input Handles */}
      <Handle
        type="target"
        position={Position.Left}
        id="input-left"
        style={{
          background: '#fff',
          border: `2px solid ${color}`,
          width: 12,
          height: 12,
          borderRadius: '0',
        }}
      />
      <Handle
        type="target"
        position={Position.Top}
        id="input-top"
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
            <Tooltip label="Edit output node">
              <ActionIcon
                size="sm"
                variant="subtle"
                color="gray"
                data-action-icon="edit"
                onMouseDown={(e) => {
                  e.stopPropagation()
                  e.preventDefault()
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
            {getNodeIcon(PlanDagNodeType.OUTPUT, '1.4rem')}
          </div>
          <Text size="sm" fw={600} lineClamp={2} style={{ wordBreak: 'break-word', flex: 1, minWidth: 0 }}>
            {data.metadata.label}
          </Text>
        </Group>

        {/* Center: Download button */}
        {!readonly && isConfigured && (
          <Group justify="center" mb="md">
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

        {/* Bottom: Labels and metadata */}
        <Stack gap="xs">
          <Group gap="xs" wrap="wrap">
            <Badge
              variant="light"
              color={color}
              size="xs"
              style={{ textTransform: 'none' }}
            >
              {getNodeTypeLabel(PlanDagNodeType.OUTPUT)}
            </Badge>
            {!isConfigured && (
              <Badge variant="outline" size="xs" color="orange">
                Not Configured
              </Badge>
            )}
          </Group>

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
      </Paper>
    </>
  )
})

OutputNode.displayName = 'OutputNode'
