import { memo, useState, useRef } from 'react'
import { NodeProps } from 'reactflow'
import { useMutation } from '@apollo/client/react'
import { Text, Group, ActionIcon, Tooltip, Badge, Stack, Modal, Textarea, ScrollArea, Button } from '@mantine/core'
import { IconDownload, IconEye, IconCopy, IconSelect } from '@tabler/icons-react'
import { PlanDagNodeType, OutputNodeConfig } from '../../../../types/plan-dag'
import { isNodeConfigured } from '../../../../utils/planDagValidation'
import { EXPORT_NODE_OUTPUT, ExportNodeOutputResult } from '../../../../graphql/export'
import { BaseNode } from './BaseNode'
import { showErrorNotification, showSuccessNotification } from '../../../../utils/notifications'

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
  const [previewOpen, setPreviewOpen] = useState(false)
  const [previewContent, setPreviewContent] = useState('')
  const [previewLoading, setPreviewLoading] = useState(false)
  const textareaRef = useRef<HTMLTextAreaElement>(null)

  const config = data.config as OutputNodeConfig

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
          showSuccessNotification('Download Complete', result.filename)
        } catch (error) {
          console.error('Failed to decode and download:', error)
          showErrorNotification('Download Failed', 'Failed to decode export content')
        }
      } else {
        showErrorNotification('Export Failed', result.message)
      }
      setDownloading(false)
    },
    onError: (error: any) => {
      console.error('Export failed:', error.message)
      showErrorNotification('Export Failed', error.message)
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

  // Preview mutation (separate from download)
  const [exportForPreview] = useMutation(EXPORT_NODE_OUTPUT, {
    onCompleted: (data: any) => {
      const result = data.exportNodeOutput as ExportNodeOutputResult
      if (result.success) {
        // Decode base64 content and display as text
        try {
          const decodedContent = atob(result.content)
          setPreviewContent(decodedContent)
          setPreviewOpen(true)
        } catch (error) {
          console.error('Failed to decode content:', error)
          showErrorNotification('Preview Failed', 'Failed to decode export content')
          setPreviewContent('Error: Failed to decode content')
        }
      } else {
        showErrorNotification('Preview Failed', result.message)
        setPreviewContent(`Error: ${result.message}`)
      }
      setPreviewLoading(false)
    },
    onError: (error: any) => {
      console.error('Export failed:', error.message)
      showErrorNotification('Preview Failed', error.message)
      setPreviewContent(`Error: ${error.message}`)
      setPreviewLoading(false)
    },
  })

  const handlePreview = async () => {
    if (!projectId || !isConfigured) return

    setPreviewLoading(true)
    setPreviewContent('')
    exportForPreview({
      variables: {
        projectId,
        nodeId: props.id,
      },
    })
  }

  const handleSelectAll = () => {
    if (textareaRef.current) {
      textareaRef.current.select()
    }
  }

  const handleCopyToClipboard = async () => {
    try {
      await navigator.clipboard.writeText(previewContent)
      showSuccessNotification('Copied', 'Content copied to clipboard')
    } catch (error) {
      console.error('Failed to copy to clipboard:', error)
      showErrorNotification('Copy Failed', 'Failed to copy content to clipboard')
    }
  }

  // Custom label badges for output node
  const labelBadges = !isConfigured ? (
    <Badge variant="outline" size="xs" color="orange">
      Not Configured
    </Badge>
  ) : null

  // Override metadata to use renderTarget as label if available
  const displayMetadata = config.renderTarget
    ? { ...data.metadata, label: config.renderTarget }
    : data.metadata

  return (
    <>
      <BaseNode
        {...props}
        nodeType={PlanDagNodeType.OUTPUT}
        config={config}
        metadata={displayMetadata}
        onEdit={() => onEdit?.(props.id)}
        onDelete={() => onDelete?.(props.id)}
        readonly={readonly}
        edges={edges}
        hasValidConfig={hasValidConfig}
        labelBadges={labelBadges}
      >
        <Stack gap="xs">
          {/* Download and preview buttons */}
          {!readonly && isConfigured && (
            <Group justify="center" gap="xs">
              <Tooltip label="Preview export">
                <ActionIcon
                  size="lg"
                  variant="light"
                  color="gray"
                  radius="xl"
                  data-action-icon="preview"
                  loading={previewLoading}
                  onMouseDown={(e) => {
                    e.stopPropagation()
                    e.preventDefault()
                    handlePreview()
                  }}
                >
                  <IconEye size="0.75rem" />
                </ActionIcon>
              </Tooltip>
              <Tooltip label="Download export">
                <ActionIcon
                  size="lg"
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
                  <IconDownload size="0.75rem" />
                </ActionIcon>
              </Tooltip>
            </Group>
          )}

          {/* Output metadata */}
          {config.outputPath && (
            <Text size="xs" c="dimmed" ff="monospace" lineClamp={1}>
              {config.outputPath}
            </Text>
          )}
        </Stack>
      </BaseNode>

      {/* Preview Dialog - Rendered outside BaseNode to avoid ReactFlow node clipping */}
      <Modal
        opened={previewOpen}
        onClose={() => setPreviewOpen(false)}
        title={`Export Preview: ${config.renderTarget || 'Output'}`}
        size="xl"
      >
        <Stack gap="md">
          <Group gap="xs">
            <Button
              size="xs"
              variant="light"
              leftSection={<IconSelect size={16} />}
              onClick={handleSelectAll}
            >
              Select All
            </Button>
            <Button
              size="xs"
              variant="light"
              leftSection={<IconCopy size={16} />}
              onClick={handleCopyToClipboard}
            >
              Copy to Clipboard
            </Button>
          </Group>
          <ScrollArea h={600}>
            <Textarea
              ref={textareaRef}
              value={previewContent}
              readOnly
              minRows={30}
              autosize
              styles={{
                input: {
                  fontFamily: 'monospace',
                  fontSize: '0.875rem',
                },
              }}
            />
          </ScrollArea>
        </Stack>
      </Modal>
    </>
  )
})

OutputNode.displayName = 'OutputNode'
