import { memo, useState, useRef, type MouseEvent, type PointerEvent } from 'react'
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
import { IconDownload, IconEye, IconCopy, IconSelect, IconChartDots, IconNetwork } from '@tabler/icons-react'
import { PlanDagNodeType, GraphArtefactNodeConfig, TreeArtefactNodeConfig } from '../../../../types/plan-dag'
import { isNodeConfigured } from '../../../../utils/planDagValidation'
import { EXPORT_NODE_OUTPUT, ExportNodeOutputResult } from '../../../../graphql/export'
import { BaseNode } from './BaseNode'
import { resolveNodeHandlers } from './nodeHandlers'
import { showErrorNotification, showSuccessNotification } from '../../../../utils/notifications'
import { MermaidPreviewDialog, DotPreviewDialog } from '../../../visualization'
import { usePlanVisualEditorContext } from '../context'

type ArtefactConfig = GraphArtefactNodeConfig | TreeArtefactNodeConfig

const normalizeRenderConfigForGraphQL = (renderConfig?: GraphArtefactNodeConfig['renderConfig']) => {
  if (!renderConfig) return undefined

  const mapOrientation = (value?: string) => {
    if (!value) return undefined
    const upper = value.toUpperCase()
    if (upper === 'LR') return 'LR'
    if (upper === 'TB') return 'TB'
    return undefined
  }

  const mapNotePosition = (value?: string) => {
    if (!value) return undefined
    return value.charAt(0).toUpperCase() + value.slice(1)
  }

  const mapBuiltInStyles = (value?: string) => (value ? value.toUpperCase() : undefined)

  const mapGraphvizLayout = (layout?: string) => {
    switch (layout) {
      case 'dot':
        return 'DOT'
      case 'neato':
        return 'NEATO'
      case 'fdp':
        return 'FDP'
      case 'circo':
        return 'CIRCO'
      default:
        return undefined
    }
  }

  const mapMermaidLook = (look?: string) => {
    switch (look) {
      case 'handDrawn':
        return 'HAND_DRAWN'
      case 'default':
      default:
        return 'DEFAULT'
    }
  }

  const mapMermaidDisplay = (display?: string) => {
    switch (display) {
      case 'compact':
        return 'COMPACT'
      case 'full':
      default:
        return 'FULL'
    }
  }

  return {
    containNodes: renderConfig.containNodes,
    orientation: mapOrientation(renderConfig.orientation),
    applyLayers: renderConfig.applyLayers,
    useNodeWeight: renderConfig.useNodeWeight,
    useEdgeWeight: renderConfig.useEdgeWeight,
    builtInStyles: mapBuiltInStyles(renderConfig.builtInStyles),
    targetOptions: renderConfig.targetOptions && {
      graphviz: renderConfig.targetOptions.graphviz && {
        layout: mapGraphvizLayout(renderConfig.targetOptions.graphviz.layout),
        overlap: renderConfig.targetOptions.graphviz.overlap,
        splines: renderConfig.targetOptions.graphviz.splines,
        nodesep: renderConfig.targetOptions.graphviz.nodesep,
        ranksep: renderConfig.targetOptions.graphviz.ranksep,
        commentStyle: renderConfig.targetOptions.graphviz.commentStyle
          ? renderConfig.targetOptions.graphviz.commentStyle.toUpperCase()
          : undefined,
      },
      mermaid: renderConfig.targetOptions.mermaid && {
        look: mapMermaidLook(renderConfig.targetOptions.mermaid.look),
        display: mapMermaidDisplay(renderConfig.targetOptions.mermaid.display),
      },
    },
    addNodeCommentsAsNotes: renderConfig.addNodeCommentsAsNotes,
    notePosition: mapNotePosition(renderConfig.notePosition),
    layerSourceStyles: renderConfig.layerSourceStyles?.map(style => ({
      sourceDatasetId: style.sourceDatasetId ?? null,
      mode: style.mode ? style.mode.toUpperCase() : undefined,
    })),
  }
}

interface ExtendedNodeProps extends NodeProps {
  onEdit?: (nodeId: string) => void
  onDelete?: (nodeId: string) => void
  readonly?: boolean
}

interface ArtefactNodeProps extends ExtendedNodeProps {
  kind: 'graph' | 'tree'
}

const ArtefactNodeBase = memo((props: ArtefactNodeProps) => {
  const { data, readonly = false, kind } = props
  const { onEdit: resolvedOnEdit, onDelete: resolvedOnDelete } = resolveNodeHandlers(props)
  const [downloading, setDownloading] = useState(false)
  const [previewOpen, setPreviewOpen] = useState(false)
  const [previewContent, setPreviewContent] = useState('')
  const [mermaidOpen, setMermaidOpen] = useState(false)
  const [mermaidContent, setMermaidContent] = useState('')
  const [dotOpen, setDotOpen] = useState(false)
  const [dotContent, setDotContent] = useState('')
  const [previewTarget, setPreviewTarget] = useState<'text' | 'mermaid' | 'dot' | null>(null)
  const [loadingTarget, setLoadingTarget] = useState<'text' | 'mermaid' | 'dot' | null>(null)
  const textareaRef = useRef<HTMLTextAreaElement>(null)

  const isTextPreviewLoading = loadingTarget === 'text'
  const isMermaidLoading = loadingTarget === 'mermaid'
  const isDotLoading = loadingTarget === 'dot'

  const config = data.config as ArtefactConfig

  const edges = data.edges || []
  const hasValidConfig = data.hasValidConfig !== false
  const dagNodeType = kind === 'tree' ? PlanDagNodeType.TREE_ARTEFACT : PlanDagNodeType.GRAPH_ARTEFACT
  const isConfigured = isNodeConfigured(dagNodeType, props.id, edges, hasValidConfig)
  const normalizedTarget = config.renderTarget?.toLowerCase()
  const isMermaidMindmap = kind === 'tree' && normalizedTarget === 'mermaidmindmap'
  const isMermaidTreemap = kind === 'tree' && normalizedTarget === 'mermaidtreemap'

  const editorContext = usePlanVisualEditorContext()
  const projectId = (data.projectId as number | undefined) ?? editorContext?.projectId
  const planId = (data.planId as number | undefined) ?? editorContext?.planId

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
    if (!projectId || !planId || !isConfigured) return

    setDownloading(true)
    exportNodeOutput({
      variables: {
        projectId,
        planId,
        nodeId: props.id,
        renderConfig: normalizeRenderConfigForGraphQL(config.renderConfig),
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
          } else if (previewTarget === 'dot') {
            setDotContent(decodedContent)
            setDotOpen(true)
          } else {
            setPreviewContent(decodedContent)
            setPreviewOpen(true)
          }
        } catch (error) {
          console.error('Failed to decode content:', error)
          showErrorNotification('Preview Failed', 'Failed to decode export content')
          if (previewTarget === 'mermaid') {
            setMermaidContent('Error: Failed to decode content')
          } else if (previewTarget === 'dot') {
            setDotContent('Error: Failed to decode content')
          } else {
            setPreviewContent('Error: Failed to decode content')
          }
        }
      } else {
        showErrorNotification('Preview Failed', result.message)
        const message = `Error: ${result.message}`
        if (previewTarget === 'mermaid') {
          setMermaidContent(message)
        } else if (previewTarget === 'dot') {
          setDotContent(message)
        } else {
          setPreviewContent(message)
        }
      }
      setLoadingTarget(null)
      setPreviewTarget(null)
    },
    onError: (error: any) => {
      console.error('Export failed:', error.message)
      showErrorNotification('Preview Failed', error.message)
      const message = `Error: ${error.message}`
      if (previewTarget === 'mermaid') {
        setMermaidContent(message)
      } else if (previewTarget === 'dot') {
        setDotContent(message)
      } else {
        setPreviewContent(message)
      }
      setLoadingTarget(null)
      setPreviewTarget(null)
    },
  })

  const requestPreview = async (target: 'text' | 'mermaid' | 'dot') => {
    if (!projectId || !planId || !isConfigured) return

    setPreviewTarget(target)
    setLoadingTarget(target)
    if (target === 'mermaid') {
      setMermaidContent('')
    } else if (target === 'dot') {
      setDotContent('')
    } else {
      setPreviewContent('')
    }
    exportForPreview({
      variables: {
        projectId,
        planId,
        nodeId: props.id,
        renderConfig: normalizeRenderConfigForGraphQL(config.renderConfig),
      },
    })
  }

  const handlePreview = async () => {
    await requestPreview('text')
  }

  const handleMermaidPreview = async () => {
    await requestPreview('mermaid')
  }

  const handleDotPreview = async () => {
    await requestPreview('dot')
  }

  const handleSelectAll = () => {
    textareaRef.current?.focus()
    textareaRef.current?.select()
  }

  const handleCopyToClipboard = async () => {
    if (!previewContent) return
    try {
      await navigator.clipboard.writeText(previewContent)
      showSuccessNotification('Copied', 'Preview copied to clipboard')
    } catch (error) {
      console.error('Failed to copy text:', error)
      showErrorNotification('Copy Failed', 'Unable to copy preview to clipboard')
    }
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

  const labelBadges = !isConfigured ? (
    <Badge variant="outline" className="text-xs text-orange-600 border-orange-600">
      Not Configured
    </Badge>
  ) : null

  const displayMetadata = config.renderTarget
    ? { ...(data.metadata ?? {}), label: config.renderTarget }
    : data.metadata

  const mermaidTooltip = isMermaidMindmap
    ? 'Preview Mermaid mindmap'
    : isMermaidTreemap
      ? 'Preview Mermaid treemap'
      : 'Preview Mermaid render'

  return (
    <>
      <BaseNode
        {...props}
        nodeType={dagNodeType}
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
          {!readonly && isConfigured && (
            <Group justify="center" gap="xs">
              <TooltipProvider>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      size="icon"
                      variant="ghost"
                      className="h-9 w-9 rounded-full nodrag"
                      data-action-icon="preview"
                      disabled={isTextPreviewLoading}
                      onPointerDown={stopPointerInteraction}
                      onClick={handleActionClick(handlePreview)}
                    >
                      <IconEye size={12} />
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>Preview export</TooltipContent>
                </Tooltip>
                {normalizedTarget?.includes('mermaid') && !isMermaidTreemap && (
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        size="icon"
                        variant="ghost"
                        className="h-9 w-9 rounded-full text-purple-600 nodrag"
                        data-action-icon="mermaid-preview"
                        disabled={isMermaidLoading}
                        onPointerDown={stopPointerInteraction}
                        onClick={handleActionClick(handleMermaidPreview)}
                      >
                        <IconChartDots size={12} />
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>{mermaidTooltip}</TooltipContent>
                  </Tooltip>
                )}
                {kind === 'graph' && normalizedTarget === 'dot' && (
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        size="icon"
                        variant="ghost"
                        className="h-9 w-9 rounded-full text-green-600 nodrag"
                        data-action-icon="dot-preview"
                        disabled={isDotLoading}
                        onPointerDown={stopPointerInteraction}
                        onClick={handleActionClick(handleDotPreview)}
                      >
                        <IconNetwork size={12} />
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>Preview Graphviz render</TooltipContent>
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

      <Dialog open={previewOpen} onOpenChange={(open) => !open && setPreviewOpen(false)}>
        <DialogContent className="sm:max-w-[700px] max-h-[90vh] flex flex-col">
          <DialogHeader>
            <DialogTitle>Export Preview: {config.renderTarget || 'Artefact'}</DialogTitle>
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
        title={`Mermaid Preview: ${data.metadata?.label || config.renderTarget || 'Artefact'}`}
      />

      <DotPreviewDialog
        open={dotOpen}
        onClose={() => setDotOpen(false)}
        diagram={dotContent}
        title={`Graphviz Preview: ${data.metadata?.label || config.renderTarget || 'Artefact'}`}
      />
    </>
  )
})

ArtefactNodeBase.displayName = 'ArtefactNodeBase'

export const GraphArtefactNode = memo((props: ExtendedNodeProps) => (
  <ArtefactNodeBase {...props} kind="graph" />
))

export const TreeArtefactNode = memo((props: ExtendedNodeProps) => (
  <ArtefactNodeBase {...props} kind="tree" />
))
