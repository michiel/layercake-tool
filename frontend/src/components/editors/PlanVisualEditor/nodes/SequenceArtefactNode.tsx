import { memo, useState } from 'react'
import { NodeProps } from 'reactflow'
import { useMutation } from '@apollo/client/react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { Stack, Group } from '@/components/layout-primitives'
import { IconDownload, IconChartDots } from '@tabler/icons-react'
import { PlanDagNodeType, SequenceArtefactNodeConfig } from '../../../../types/plan-dag'
import { isNodeConfigured } from '../../../../utils/planDagValidation'
import { EXPORT_NODE_OUTPUT, ExportNodeOutputResult } from '../../../../graphql/export'
import { BaseNode } from './BaseNode'
import { showErrorNotification, showSuccessNotification } from '../../../../utils/notifications'
import { MermaidPreviewDialog } from '../../../visualization'

interface ExtendedNodeProps extends NodeProps {
  onEdit?: (nodeId: string) => void
  onDelete?: (nodeId: string) => void
  readonly?: boolean
}

export const SequenceArtefactNode = memo((props: ExtendedNodeProps) => {
  const { data, onEdit, onDelete, readonly = false } = props
  const [downloading, setDownloading] = useState(false)
  const [mermaidOpen, setMermaidOpen] = useState(false)
  const [mermaidContent, setMermaidContent] = useState('')
  const [loadingPreview, setLoadingPreview] = useState(false)

  const config = data.config as SequenceArtefactNodeConfig

  const edges = data.edges || []
  const hasValidConfig = data.hasValidConfig !== false
  const isConfiguredNode = isNodeConfigured(PlanDagNodeType.SEQUENCE_ARTEFACT, props.id, edges, hasValidConfig)
  const normalizedTarget = config.renderTarget?.toLowerCase()
  const isMermaid = normalizedTarget?.includes('mermaid')

  const projectId = data.projectId as number | undefined

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
    if (!projectId || !isConfiguredNode) return

    setDownloading(true)
    exportNodeOutput({
      variables: {
        projectId,
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
          setMermaidContent(decodedContent)
          setMermaidOpen(true)
        } catch (error) {
          console.error('Failed to decode content:', error)
          showErrorNotification('Preview Failed', 'Failed to decode export content')
          setMermaidContent('Error: Failed to decode content')
        }
      } else {
        showErrorNotification('Preview Failed', result.message)
        setMermaidContent(`Error: ${result.message}`)
      }
      setLoadingPreview(false)
    },
    onError: (error: any) => {
      console.error('Export failed:', error.message)
      showErrorNotification('Preview Failed', error.message)
      setMermaidContent(`Error: ${error.message}`)
      setLoadingPreview(false)
    },
  })

  const handleMermaidPreview = async () => {
    if (!projectId || !isConfiguredNode) return

    setLoadingPreview(true)
    setMermaidContent('')
    exportForPreview({
      variables: {
        projectId,
        nodeId: props.id,
      },
    })
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
        onEdit={() => onEdit?.(props.id)}
        onDelete={() => onDelete?.(props.id)}
        readonly={readonly}
        edges={edges}
        hasValidConfig={hasValidConfig}
        labelBadges={labelBadges}
      >
        <Stack gap="xs">
          {!readonly && isConfiguredNode && (
            <Group justify="center" gap="xs">
              <TooltipProvider>
                {isMermaid && (
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        size="icon"
                        variant="ghost"
                        className="h-9 w-9 rounded-full text-purple-600"
                        data-action-icon="mermaid-preview"
                        disabled={loadingPreview}
                        onMouseDown={(e: React.MouseEvent) => {
                          e.stopPropagation()
                          e.preventDefault()
                          handleMermaidPreview()
                        }}
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

          {config.outputPath && (
            <p className="text-xs text-muted-foreground font-mono line-clamp-1">
              {config.outputPath}
            </p>
          )}
        </Stack>
      </BaseNode>

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
