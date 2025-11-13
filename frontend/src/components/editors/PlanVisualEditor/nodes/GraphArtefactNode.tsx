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
import { IconDownload, IconEye, IconCopy, IconSelect, IconChartDots, IconNetwork } from '@tabler/icons-react'
import { PlanDagNodeType, GraphArtefactNodeConfig, TreeArtefactNodeConfig } from '../../../../types/plan-dag'
import { isNodeConfigured } from '../../../../utils/planDagValidation'
import { EXPORT_NODE_OUTPUT, ExportNodeOutputResult } from '../../../../graphql/export'
import { BaseNode } from './BaseNode'
import { showErrorNotification, showSuccessNotification } from '../../../../utils/notifications'
import { MermaidPreviewDialog, DotPreviewDialog } from '../../../visualization'

type ArtefactConfig = GraphArtefactNodeConfig | TreeArtefactNodeConfig

interface ExtendedNodeProps extends NodeProps {
  onEdit?: (nodeId: string) => void
  onDelete?: (nodeId: string) => void
  readonly?: boolean
}

interface ArtefactNodeProps extends ExtendedNodeProps {
  kind: 'graph' | 'tree'
}

const ArtefactNodeBase = memo((props: ArtefactNodeProps) => {
  const { data, onEdit, onDelete, readonly = false, kind } = props
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
    if (!projectId || !isConfigured) return

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

  const handleDotPreview = async () => {
    if (!projectId || !isConfigured) return

    setPreviewTarget('dot')
    setLoadingTarget('dot')
    setDotContent('')
    exportForPreview({
      variables: {
        projectId,
        nodeId: props.id,
      },
    })
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

  const allowsMermaidPreview = config.renderTarget === 'Mermaid' || config.renderTarget === 'MermaidMindmap'
  const allowsDotPreview = config.renderTarget === 'DOT'

  return (
    <>
      <BaseNode
        {...props}
        onEdit={readonly ? undefined : onEdit}
        onDelete={readonly ? undefined : onDelete}
        readonly={readonly}
      >
        <Stack gap="sm">
          <Group gap="xs" className="flex-wrap">
            <Badge variant="outline">
              {config.renderTarget || (kind === 'tree' ? 'Tree Artefact' : 'Graph Artefact')}
            </Badge>
            {config.outputPath && (
              <Badge variant="secondary" className="font-mono">
                {config.outputPath}
              </Badge>
            )}
          </Group>

          <Group gap="xs" className="flex-wrap">
            <Button
              size="sm"
              variant="secondary"
              onClick={handlePreview}
              disabled={previewTarget !== null || !isConfigured}
            >
              {isTextPreviewLoading ? 'Rendering…' : (
                <Group gap="xs">
                  <IconEye size={16} />
                  <span>Preview</span>
                </Group>
              )}
            </Button>

            {kind === 'graph' && (
              <>
                <Button
                  size="sm"
                  variant="secondary"
                  onClick={handleMermaidPreview}
                  disabled={!allowsMermaidPreview || previewTarget !== null || !isConfigured}
                >
                  {isMermaidLoading ? 'Rendering…' : (
                    <Group gap="xs">
                      <IconChartDots size={16} />
                      <span>Mermaid</span>
                    </Group>
                  )}
                </Button>

                <Button
                  size="sm"
                  variant="secondary"
                  onClick={handleDotPreview}
                  disabled={!allowsDotPreview || previewTarget !== null || !isConfigured}
                >
                  {isDotLoading ? 'Rendering…' : (
                    <Group gap="xs">
                      <IconNetwork size={16} />
                      <span>Graphviz</span>
                    </Group>
                  )}
                </Button>
              </>
            )}

            <Button
              size="sm"
              variant="default"
              onClick={handleDownload}
              disabled={downloading || !isConfigured}
            >
              {downloading ? 'Downloading…' : (
                <Group gap="xs">
                  <IconDownload size={16} />
                  <span>Download</span>
                </Group>
              )}
            </Button>
          </Group>
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
