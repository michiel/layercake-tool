import { memo, useState, useRef } from 'react'
import { NodeProps } from 'reactflow'
import { useMutation } from '@apollo/client/react'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Badge } from '@/components/ui/badge'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { Stack, Group } from '@/components/layout-primitives'
import { IconDownload, IconEye, IconCopy, IconSelect, IconChartDots } from '@tabler/icons-react'
import { PlanDagNodeType, OutputNodeConfig } from '../../../../types/plan-dag'
import { isNodeConfigured } from '../../../../utils/planDagValidation'
import { EXPORT_NODE_OUTPUT, ExportNodeOutputResult } from '../../../../graphql/export'
import { BaseNode } from './BaseNode'
import { showErrorNotification, showSuccessNotification } from '../../../../utils/notifications'
import { MermaidPreviewDialog } from '../../../visualization/MermaidPreviewDialog'

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
  const [mermaidOpen, setMermaidOpen] = useState(false)
  const [mermaidContent, setMermaidContent] = useState('')
  const [previewTarget, setPreviewTarget] = useState<'text' | 'mermaid' | null>(null)
  const [loadingTarget, setLoadingTarget] = useState<'text' | 'mermaid' | null>(null)
  const textareaRef = useRef<HTMLTextAreaElement>(null)

  const isTextPreviewLoading = loadingTarget === 'text'
  const isMermaidLoading = loadingTarget === 'mermaid'

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
        try {
          const decodedContent = atob(result.content)
          if (previewTarget === 'mermaid') {
            setMermaidContent(decodedContent)
            setMermaidOpen(true)
          } else {
            setPreviewContent(decodedContent)
            setPreviewOpen(true)
          }
        } catch (error) {
          console.error('Failed to decode content:', error)
          showErrorNotification('Preview Failed', 'Failed to decode export content')
          if (previewTarget === 'mermaid') {
            setMermaidContent('')
          } else {
            setPreviewContent('Error: Failed to decode content')
          }
        }
      } else {
        showErrorNotification('Preview Failed', result.message)
        if (previewTarget === 'mermaid') {
          setMermaidContent(`Error: ${result.message}`)
        } else {
          setPreviewContent(`Error: ${result.message}`)
        }
      }
      setLoadingTarget(null)
      setPreviewTarget(null)
    },
    onError: (error: any) => {
      console.error('Export failed:', error.message)
      showErrorNotification('Preview Failed', error.message)
      if (previewTarget === 'mermaid') {
        setMermaidContent(`Error: ${error.message}`)
      } else {
        setPreviewContent(`Error: ${error.message}`)
      }
      setLoadingTarget(null)
      setPreviewTarget(null)
    },
  })

  const handlePreview = async () => {
    if (!projectId || !isConfigured) return

    setPreviewTarget('text')
    setLoadingTarget('text')
    setPreviewContent('')
    exportForPreview({
      variables: {
        projectId,
        nodeId: props.id,
      },
    })
  }

  const handleMermaidPreview = async () => {
    if (!projectId || !isConfigured) return

    setPreviewTarget('mermaid')
    setLoadingTarget('mermaid')
    setMermaidContent('')
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
    <Badge variant="outline" className="text-xs text-orange-600 border-orange-600">
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
              <TooltipProvider>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      size="icon"
                      variant="ghost"
                      className="h-9 w-9 rounded-full"
                      data-action-icon="preview"
                      disabled={isTextPreviewLoading}
                      onMouseDown={(e: React.MouseEvent) => {
                        e.stopPropagation()
                        e.preventDefault()
                        handlePreview()
                      }}
                    >
                      <IconEye size={12} />
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>Preview export</TooltipContent>
                </Tooltip>
                {config.renderTarget === 'Mermaid' && (
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        size="icon"
                        variant="ghost"
                        className="h-9 w-9 rounded-full text-purple-600"
                        data-action-icon="mermaid"
                        disabled={isMermaidLoading}
                        onMouseDown={(e: React.MouseEvent) => {
                          e.stopPropagation()
                          e.preventDefault()
                          handleMermaidPreview()
                        }}
                      >
                        <IconChartDots size={12} />
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>Render Mermaid preview</TooltipContent>
                  </Tooltip>
                )}
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      size="icon"
                      variant="ghost"
                      className="h-9 w-9 rounded-full text-blue-600"
                      data-action-icon="download"
                      disabled={downloading}
                      onMouseDown={(e: React.MouseEvent) => {
                        e.stopPropagation()
                        e.preventDefault()
                        handleDownload()
                      }}
                    >
                      <IconDownload size={12} />
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>Download export</TooltipContent>
                </Tooltip>
              </TooltipProvider>
            </Group>
          )}

          {/* Output metadata */}
          {config.outputPath && (
            <p className="text-xs text-muted-foreground font-mono line-clamp-1">
              {config.outputPath}
            </p>
          )}
        </Stack>
      </BaseNode>

      {/* Preview Dialog - Rendered outside BaseNode to avoid ReactFlow node clipping */}
      <Dialog open={previewOpen} onOpenChange={(open) => !open && setPreviewOpen(false)}>
        <DialogContent className="sm:max-w-[700px] max-h-[90vh] flex flex-col">
          <DialogHeader>
            <DialogTitle>Export Preview: {config.renderTarget || 'Output'}</DialogTitle>
          </DialogHeader>
          <div className="flex-1 overflow-hidden">
            <Stack gap="md">
              <Group gap="xs">
                <Button
                  size="sm"
                  variant="secondary"
                  onClick={handleSelectAll}
                  className="h-8"
                >
                  <IconSelect size={16} className="mr-2" />
                  Select All
                </Button>
                <Button
                  size="sm"
                  variant="secondary"
                  onClick={handleCopyToClipboard}
                  className="h-8"
                >
                  <IconCopy size={16} className="mr-2" />
                  Copy to Clipboard
                </Button>
              </Group>
              <ScrollArea className="h-[600px]">
                <Textarea
                  ref={textareaRef}
                  value={previewContent}
                  readOnly
                  rows={30}
                  className="font-mono text-sm resize-none"
                />
              </ScrollArea>
            </Stack>
          </div>
      </DialogContent>
    </Dialog>

      <MermaidPreviewDialog
        open={mermaidOpen}
        onClose={() => setMermaidOpen(false)}
        diagram={mermaidContent}
        title={`Mermaid Preview: ${data.metadata?.label || config.renderTarget || 'Output'}`}
      />
    </>
  )
})

OutputNode.displayName = 'OutputNode'
