import { memo, useRef, useState, type MouseEvent, type PointerEvent } from 'react'
import { NodeProps } from 'reactflow'
import { useMutation } from '@apollo/client/react'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { Stack, Group } from '@/components/layout-primitives'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Textarea } from '@/components/ui/textarea'
import { IconDownload, IconChartDots, IconEye, IconCopy, IconSelect } from '@tabler/icons-react'
import { PlanDagNodeType, SequenceArtefactNodeConfig } from '../../../../types/plan-dag'
import { isNodeConfigured } from '../../../../utils/planDagValidation'
import { EXPORT_NODE_OUTPUT, ExportNodeOutputResult } from '../../../../graphql/export'
import { BaseNode } from './BaseNode'
import { resolveNodeHandlers } from './nodeHandlers'
import { showErrorNotification, showSuccessNotification } from '../../../../utils/notifications'
import { MermaidPreviewDialog } from '../../../visualization'

interface ExtendedNodeProps extends NodeProps {
  onEdit?: (nodeId: string) => void
  onDelete?: (nodeId: string) => void
  readonly?: boolean
}

export const SequenceArtefactNode = memo((props: ExtendedNodeProps) => {
  const { data, readonly = false } = props
  const { onEdit: resolvedOnEdit, onDelete: resolvedOnDelete } = resolveNodeHandlers(props)
  const [downloading, setDownloading] = useState(false)
  const [mermaidOpen, setMermaidOpen] = useState(false)
  const [mermaidContent, setMermaidContent] = useState('')
  const [previewOpen, setPreviewOpen] = useState(false)
  const [previewContent, setPreviewContent] = useState('')
  const [previewTarget, setPreviewTarget] = useState<'text' | 'mermaid' | null>(null)
  const [loadingTarget, setLoadingTarget] = useState<'text' | 'mermaid' | null>(null)
  const textareaRef = useRef<HTMLTextAreaElement>(null)

  const config = data.config as SequenceArtefactNodeConfig

  const edges = data.edges || []
  const hasValidConfig = data.hasValidConfig !== false
  const isConfiguredNode = isNodeConfigured(PlanDagNodeType.SEQUENCE_ARTEFACT, props.id, edges, hasValidConfig)
  const normalizedTarget = config.renderTarget?.toLowerCase()
  const isMermaid = normalizedTarget?.includes('mermaid')

  const projectId = data.projectId as number | undefined
  const planId = data.planId as number | undefined

  const [exportNodeOutput] = useMutation(EXPORT_NODE_OUTPUT, {
    onCompleted: (data: any) => {
      const result = data.exportNodeOutput as ExportNodeOutputResult
      if (result.success) {
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
    if (!projectId || !planId || !isConfiguredNode) return

    setDownloading(true)
    exportNodeOutput({
      variables: {
        projectId,
        planId,
        nodeId: props.id,
      },
    })
  }

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
            setMermaidContent('Error: Failed to decode content')
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
      setLoadingTarget(null)
      setPreviewTarget(null)
    },
  })

  const triggerPreview = (target: 'text' | 'mermaid') => {
    if (!projectId || !planId || !isConfiguredNode) return
    if (target === 'text') {
      setPreviewContent('')
    } else {
      setMermaidContent('')
    }
    setPreviewTarget(target)
    setLoadingTarget(target)
    exportForPreview({
      variables: {
        projectId,
        planId,
        nodeId: props.id,
      },
    })
  }

  const handlePreviewExport = () => {
    triggerPreview('text')
  }

  const handleMermaidPreview = () => {
    triggerPreview('mermaid')
  }

  const stopPointerInteraction = (event: PointerEvent<HTMLButtonElement>) => {
    event.stopPropagation()
    event.preventDefault()
  }

  const handleActionClick =
    (action: () => void) => (event: MouseEvent<HTMLButtonElement>) => {
      event.stopPropagation()
      event.preventDefault()
      action()
    }

  const handleCopyPreview = () => {
    if (textareaRef.current) {
      textareaRef.current.select()
      document.execCommand('copy')
      showSuccessNotification('Copied', 'Preview copied to clipboard')
    }
  }

  const handleSelectPreview = () => {
    textareaRef.current?.select()
  }

  const labelBadges = !isConfiguredNode ? (
    <Badge variant="outline" className="text-xs text-orange-600 border-orange-600">
      Not Configured
    </Badge>
  ) : null

  const displayMetadata = config.renderTarget
    ? { ...(data.metadata ?? {}), label: config.renderTarget }
    : data.metadata

  return (
    <>
      <BaseNode
        {...props}
        nodeType={PlanDagNodeType.SEQUENCE_ARTEFACT}
        config={config}
        metadata={displayMetadata}
        onEdit={() => resolvedOnEdit?.(props.id)}
        onDelete={() => resolvedOnDelete?.(props.id)}
        readonly={readonly}
        edges={edges}
        hasValidConfig={hasValidConfig}
        labelBadges={labelBadges}
      >
        <Stack gap="xs">
          {!readonly && isConfiguredNode && (
            <Group justify="center" gap="xs">
              <TooltipProvider>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      size="icon"
                      variant="ghost"
                      className="h-9 w-9 rounded-full text-emerald-600 nodrag"
                      data-action-icon="preview-export"
                      disabled={loadingTarget === 'text'}
                      onPointerDown={stopPointerInteraction}
                      onClick={handleActionClick(handlePreviewExport)}
                    >
                      <IconEye size={12} />
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>Preview export</TooltipContent>
                </Tooltip>
                {isMermaid && (
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                      size="icon"
                      variant="ghost"
                      className="h-9 w-9 rounded-full text-purple-600 nodrag"
                      data-action-icon="mermaid-preview"
                      disabled={loadingTarget === 'mermaid'}
                      onPointerDown={stopPointerInteraction}
                      onClick={handleActionClick(handleMermaidPreview)}
                    >
                        <IconChartDots size={12} />
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>Preview Mermaid sequence diagram</TooltipContent>
                  </Tooltip>
                )}
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      size="icon"
                      variant="ghost"
                      className="h-9 w-9 rounded-full text-blue-600 nodrag"
                      data-action-icon="download"
                      disabled={downloading}
                      onPointerDown={stopPointerInteraction}
                      onClick={handleActionClick(handleDownload)}
                    >
                      <IconDownload size={12} />
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>Download export</TooltipContent>
                </Tooltip>
              </TooltipProvider>
            </Group>
          )}

          {config.outputPath && (
            <p className="text-xs text-muted-foreground font-mono line-clamp-1">
              {config.outputPath}
            </p>
          )}
        </Stack>
      </BaseNode>
      <Dialog open={previewOpen} onOpenChange={(open) => setPreviewOpen(open)}>
        <DialogContent className="sm:max-w-[600px]">
          <DialogHeader>
            <DialogTitle>Export preview</DialogTitle>
          </DialogHeader>
          <Stack gap="sm">
            <Group gap="xs">
              <Button size="sm" variant="outline" onClick={handleCopyPreview}>
                <IconCopy size={14} className="mr-1" />
                Copy
              </Button>
              <Button size="sm" variant="outline" onClick={handleSelectPreview}>
                <IconSelect size={14} className="mr-1" />
                Select
              </Button>
            </Group>
            <ScrollArea className="max-h-[400px]">
              <Textarea
                ref={textareaRef}
                value={previewContent}
                readOnly
                rows={12}
                className="font-mono text-xs"
              />
            </ScrollArea>
          </Stack>
        </DialogContent>
      </Dialog>
      <MermaidPreviewDialog
        open={mermaidOpen}
        onClose={() => setMermaidOpen(false)}
        diagram={mermaidContent}
        title={`Sequence Diagram Preview: ${data.metadata?.label || config.renderTarget || 'Artefact'}`}
      />
    </>
  )
})

SequenceArtefactNode.displayName = 'SequenceArtefactNode'
