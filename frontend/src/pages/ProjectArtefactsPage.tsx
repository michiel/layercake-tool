import React, { useMemo, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useMutation, useQuery } from '@apollo/client/react'
import {
  IconHierarchy2,
  IconGraph,
  IconDownload,
  IconEye,
  IconTable,
  IconChartDots,
  IconNetwork,
  IconAlertCircle,
  IconChevronDown,
  IconChevronRight,
} from '@tabler/icons-react'
import { gql } from '@apollo/client'
import { GET_PLAN_DAG } from '../graphql/plan-dag'
import { EXPORT_NODE_OUTPUT } from '../graphql/export'
import { Stack, Group } from '../components/layout-primitives'
import PageContainer from '../components/layout/PageContainer'
import { Breadcrumbs } from '../components/common/Breadcrumbs'
import { Spinner } from '../components/ui/spinner'
import { Alert, AlertDescription, AlertTitle } from '../components/ui/alert'
import { Button } from '../components/ui/button'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '../components/ui/tooltip'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '../components/ui/dialog'
import { Textarea } from '../components/ui/textarea'
import { ScrollArea } from '../components/ui/scroll-area'
import { GraphDataDialog } from '../components/editors/PlanVisualEditor/dialogs/GraphDataDialog'
import { MermaidPreviewDialog, DotPreviewDialog } from '../components/visualization'
import { showErrorNotification, showSuccessNotification } from '../utils/notifications'

const GET_PROJECTS = gql`
  query GetProjects {
    projects {
      id
      name
    }
  }
`

type PlanDagNode = {
  id: string
  nodeType: string
  metadata?: { label?: string | null } | null
  config?: string | null
  graphExecution?: {
    graphId?: number | null
  } | null
}

type PlanDagEdge = {
  id: string
  source: string
  target: string
}

type PlanDagResponse = {
  getPlanDag: {
    nodes: PlanDagNode[]
    edges: PlanDagEdge[]
  }
}

type ArtefactEntry =
  | { type: 'graph'; node: PlanDagNode; depth: number }
  | { type: 'artefact'; node: PlanDagNode; depth: number; config: Record<string, any>; parentGraphId: string }

const parseConfig = (config?: string | null): Record<string, any> => {
  if (!config) return {}
  try {
    return JSON.parse(config)
  } catch (error) {
    console.warn('Failed to parse node config', error)
    return {}
  }
}

const formatRenderTarget = (value?: string): string => {
  if (!value) return 'Unknown'
  return value
    .replace(/([A-Z])/g, ' $1')
    .replace(/^./, (c) => c.toUpperCase())
    .trim()
}

const getRenderOptionLabels = (config: Record<string, any>): string[] => {
  const labels: string[] = []

  // Check for contain_nodes
  if (config.contain_nodes === true) {
    labels.push('contain nodes')
  }

  // Check for theme
  if (config.theme === 'light') {
    labels.push('light')
  } else if (config.theme === 'dark') {
    labels.push('dark')
  }

  // Check for orientation
  if (config.orientation === 'LR' || config.orientation === 'lr') {
    labels.push('lr')
  } else if (config.orientation === 'TD' || config.orientation === 'td') {
    labels.push('td')
  }

  return labels
}

const ProjectArtefactsPage: React.FC = () => {
  const navigate = useNavigate()
  const { projectId } = useParams<{ projectId: string }>()
  const numericProjectId = parseInt(projectId || '0', 10)

  const { data: projectsData } = useQuery<{ projects: Array<{ id: number; name: string }> }>(GET_PROJECTS)
  const selectedProject = projectsData?.projects.find((p: { id: number; name: string }) => p.id === numericProjectId)

  const { data, loading, error } = useQuery<PlanDagResponse>(GET_PLAN_DAG, {
    variables: { projectId: numericProjectId },
    fetchPolicy: 'cache-and-network',
  })

  const [graphDataDialog, setGraphDataDialog] = useState<{ open: boolean; graphId: number | null }>({
    open: false,
    graphId: null,
  })
  const [textPreview, setTextPreview] = useState<{ open: boolean; content: string; title: string }>({
    open: false,
    content: '',
    title: '',
  })
  const [mermaidPreview, setMermaidPreview] = useState<{ open: boolean; content: string; title: string }>({
    open: false,
    content: '',
    title: '',
  })
  const [dotPreview, setDotPreview] = useState<{ open: boolean; content: string; title: string }>({
    open: false,
    content: '',
    title: '',
  })
  const [previewLoading, setPreviewLoading] = useState<{ nodeId: string; kind: 'text' | 'mermaid' | 'dot' } | null>(
    null,
  )
  const [downloadingNodeId, setDownloadingNodeId] = useState<string | null>(null)
  const [collapsedGraphs, setCollapsedGraphs] = useState<Set<string>>(new Set())

  const toggleGraphCollapse = (graphId: string) => {
    setCollapsedGraphs((prev) => {
      const next = new Set(prev)
      if (next.has(graphId)) {
        next.delete(graphId)
      } else {
        next.add(graphId)
      }
      return next
    })
  }

  const [exportForPreview] = useMutation(EXPORT_NODE_OUTPUT, {
    onCompleted: (response: any) => {
      const result = response.exportNodeOutput
      const target = previewLoading?.kind ?? 'text'
      const nodeId = previewLoading?.nodeId
      if (result?.success) {
        try {
          const decoded = atob(result.content)
          const title = `Preview: ${result.filename}`
          if (target === 'mermaid') {
            setMermaidPreview({ open: true, content: decoded, title })
          } else if (target === 'dot') {
            setDotPreview({ open: true, content: decoded, title })
          } else {
            setTextPreview({ open: true, content: decoded, title })
          }
        } catch (err) {
          console.error('Failed to decode preview content', err)
          showErrorNotification('Preview Failed', 'Unable to decode preview content')
        }
      } else if (nodeId) {
        showErrorNotification('Preview Failed', result?.message ?? 'Unknown error')
      }
      setPreviewLoading(null)
    },
    onError: (err) => {
      console.error('Preview failed:', err)
      showErrorNotification('Preview Failed', err.message)
      setPreviewLoading(null)
    },
  })

  const [exportNodeOutput] = useMutation(EXPORT_NODE_OUTPUT, {
    onCompleted: (response: any) => {
      const result = response.exportNodeOutput
      if (result?.success) {
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
          console.error('Failed to decode and download export', error)
          showErrorNotification('Download Failed', 'Unable to decode export content')
        }
      } else {
        showErrorNotification('Download Failed', result?.message ?? 'Unknown error')
      }
      setDownloadingNodeId(null)
    },
    onError: (error) => {
      console.error('Download failed:', error)
      showErrorNotification('Download Failed', error.message)
      setDownloadingNodeId(null)
    },
  })

  const entries = useMemo<ArtefactEntry[]>(() => {
    const nodes = data?.getPlanDag?.nodes ?? []
    const edges = data?.getPlanDag?.edges ?? []
    if (!nodes.length) return []

    const nodeMap = new Map(nodes.map((node) => [node.id, node]))
    const graphNodes = nodes.filter((node) => node.nodeType === 'GraphNode')
    const graphIds = new Set(graphNodes.map((node) => node.id))
    const outgoing = new Map<string, string[]>()
    const incomingGraphCount = new Map<string, number>()

    edges.forEach((edge) => {
      outgoing.set(edge.source, [...(outgoing.get(edge.source) ?? []), edge.target])
      if (graphIds.has(edge.source) && graphIds.has(edge.target)) {
        incomingGraphCount.set(edge.target, (incomingGraphCount.get(edge.target) ?? 0) + 1)
      }
    })

    let roots = graphNodes.filter((node) => (incomingGraphCount.get(node.id) ?? 0) === 0)
    if (!roots.length) {
      roots = graphNodes
    }

    const visited = new Set<string>()
    const entries: ArtefactEntry[] = []

    const traverseGraph = (nodeId: string, depth: number) => {
      if (visited.has(nodeId)) return
      visited.add(nodeId)

      const node = nodeMap.get(nodeId)
      if (!node) return
      entries.push({ type: 'graph', node, depth })

      const children = outgoing.get(nodeId) ?? []
      const graphChildren = children.filter((id) => graphIds.has(id))
      const artefactChildren = children
        .map((id) => nodeMap.get(id))
        .filter(
          (child): child is PlanDagNode =>
            !!child && (child.nodeType === 'GraphArtefactNode' || child.nodeType === 'TreeArtefactNode'),
        )

      graphChildren.forEach((childId) => traverseGraph(childId, depth + 1))

      artefactChildren.forEach((child) => {
        entries.push({
          type: 'artefact',
          node: child,
          depth: depth + 1,
          config: parseConfig(child.config),
          parentGraphId: nodeId,
        })
      })
    }

    roots.forEach((root) => traverseGraph(root.id, 0))
    graphNodes.forEach((node) => {
      if (!visited.has(node.id)) {
        traverseGraph(node.id, 0)
      }
    })

    return entries
  }, [data])

  const handleGraphData = (graphId?: number | null) => {
    if (!graphId) {
      showErrorNotification('Graph preview unavailable', 'Run the graph to generate data before previewing.')
      return
    }
    setGraphDataDialog({ open: true, graphId })
  }

  const handlePreview = (nodeId: string, target: 'text' | 'mermaid' | 'dot') => {
    if (!numericProjectId) return
    setPreviewLoading({ nodeId, kind: target })
    exportForPreview({
      variables: { projectId: numericProjectId, nodeId },
    })
  }

  const handleDownload = (nodeId: string) => {
    if (!numericProjectId) return
    setDownloadingNodeId(nodeId)
    exportNodeOutput({
      variables: { projectId: numericProjectId, nodeId },
    })
  }

  const handleNavigate = (route: string) => {
    navigate(route)
  }

  const renderEntry = (entry: ArtefactEntry, index: number) => {
    if (entry.type === 'graph') {
      const label = entry.node.metadata?.label || entry.node.id
      const graphId = entry.node.graphExecution?.graphId ?? null
      const isCollapsed = collapsedGraphs.has(entry.node.id)
      const hasChildren = index < entries.length - 1 && entries[index + 1].type === 'artefact'

      return (
        <div
          key={entry.node.id}
          className="flex items-center justify-between py-4 px-4 border-b last:border-b-0 bg-muted/30"
          style={{ paddingLeft: `${16 + entry.depth * 24}px` }}
        >
          <Group gap="sm" className="text-sm">
            {hasChildren && (
              <Button
                size="icon"
                variant="ghost"
                className="h-6 w-6 -ml-2"
                onClick={(e) => {
                  e.stopPropagation()
                  toggleGraphCollapse(entry.node.id)
                }}
              >
                {isCollapsed ? <IconChevronRight size={16} /> : <IconChevronDown size={16} />}
              </Button>
            )}
            {!hasChildren && <div className="w-6" />}
            <IconGraph className="h-4 w-4 text-muted-foreground" />
            <div>
              <p className="font-medium">{label}</p>
              <p className="text-xs text-muted-foreground">Graph Node</p>
            </div>
          </Group>
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  size="icon"
                  variant="ghost"
                  className="h-8 w-8"
                  disabled={!graphId}
                  onClick={() => handleGraphData(graphId)}
                >
                  <IconTable size={16} />
                </Button>
              </TooltipTrigger>
              <TooltipContent>Preview graph data</TooltipContent>
            </Tooltip>
          </TooltipProvider>
        </div>
      )
    }

    // Check if parent graph is collapsed
    if (collapsedGraphs.has(entry.parentGraphId)) {
      return null
    }

    const label = entry.node.metadata?.label || entry.node.id
    const renderTarget = entry.config?.renderTarget
    const renderTargetLabel = formatRenderTarget(renderTarget)
    const normalizedTarget = typeof renderTarget === 'string' ? renderTarget.toLowerCase() : ''
    const isMermaidTreemap = normalizedTarget === 'mermaidtreemap'
    const supportsMermaidPreview = normalizedTarget.includes('mermaid') && !isMermaidTreemap
    const supportsDotPreview = renderTarget === 'DOT'
    const renderOptionLabels = getRenderOptionLabels(entry.config)

    const isPreviewLoading = previewLoading?.nodeId === entry.node.id
    const isMermaidLoading = isPreviewLoading && previewLoading?.kind === 'mermaid'
    const isDotLoading = isPreviewLoading && previewLoading?.kind === 'dot'
    const isTextLoading = isPreviewLoading && previewLoading?.kind === 'text'

    return (
      <div
        key={entry.node.id}
        className="flex items-center justify-between py-4 px-4 border-b last:border-b-0"
        style={{ paddingLeft: `${16 + entry.depth * 24}px` }}
      >
        <Group gap="sm" className="text-sm">
          <IconHierarchy2 className="h-4 w-4 text-muted-foreground" />
          <div>
            <p className="font-medium">{label}</p>
            <p className="text-xs text-muted-foreground">{renderTargetLabel}</p>
          </div>
        </Group>
        {renderOptionLabels.length > 0 && (
          <Group gap="xs" className="flex-1 justify-center">
            {renderOptionLabels.map((label) => (
              <span key={label} className="text-xs px-2 py-1 rounded bg-muted text-muted-foreground">
                {label}
              </span>
            ))}
          </Group>
        )}
        <Group gap="xs">
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  size="icon"
                  variant="ghost"
                  className="h-8 w-8"
                  disabled={isTextLoading}
                  onClick={() => handlePreview(entry.node.id, 'text')}
                >
                  <IconEye size={16} />
                </Button>
              </TooltipTrigger>
              <TooltipContent>Preview export</TooltipContent>
            </Tooltip>
            {supportsMermaidPreview && (
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    size="icon"
                    variant="ghost"
                    className="h-8 w-8 text-purple-600"
                    disabled={isMermaidLoading}
                    onClick={() => handlePreview(entry.node.id, 'mermaid')}
                  >
                    <IconChartDots size={16} />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>Visual preview</TooltipContent>
              </Tooltip>
            )}
            {supportsDotPreview && (
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    size="icon"
                    variant="ghost"
                    className="h-8 w-8 text-green-600"
                    disabled={isDotLoading}
                    onClick={() => handlePreview(entry.node.id, 'dot')}
                  >
                    <IconNetwork size={16} />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>Graphviz preview</TooltipContent>
              </Tooltip>
            )}
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  size="icon"
                  variant="ghost"
                  className="h-8 w-8 text-blue-600"
                  disabled={downloadingNodeId === entry.node.id}
                  onClick={() => handleDownload(entry.node.id)}
                >
                  <IconDownload size={16} />
                </Button>
              </TooltipTrigger>
              <TooltipContent>Download artefact</TooltipContent>
            </Tooltip>
          </TooltipProvider>
        </Group>
      </div>
    )
  }

  if (!selectedProject) {
    return (
      <PageContainer>
        <h1 className="text-3xl font-bold">Project Not Found</h1>
        <Button onClick={() => navigate('/projects')} className="mt-4">
          Back to Projects
        </Button>
      </PageContainer>
    )
  }

  return (
    <>
      <PageContainer>
        <Breadcrumbs
          projectName={selectedProject.name}
          projectId={selectedProject.id}
          sections={[{ title: 'Graph creation', href: `/projects/${selectedProject.id}/plan` }]}
          currentPage="Artefacts"
          onNavigate={handleNavigate}
        />

        <Group justify="between" className="mb-4">
          <div>
            <h1 className="text-3xl font-bold">Artefacts</h1>
            <p className="text-sm text-muted-foreground mt-1">
              Browse graphs and their exported artefacts
            </p>
          </div>
        </Group>

        {error && (
          <Alert variant="destructive" className="mb-4">
            <IconAlertCircle className="h-4 w-4" />
            <AlertTitle>Error</AlertTitle>
            <AlertDescription>
              {error.message}
            </AlertDescription>
          </Alert>
        )}

        {loading && (
          <div className="flex items-center justify-center py-16">
            <Spinner />
          </div>
        )}
        {!loading && !entries.length && (
          <Alert className="max-w-4xl">
            <AlertTitle>No graphs detected</AlertTitle>
            <AlertDescription>Create graphs in the plan to see their artefacts here.</AlertDescription>
          </Alert>
        )}
        {!!entries.length && (
          <div className="max-w-4xl">
            <div className="rounded-lg border bg-background shadow-sm">
              {entries.map((entry, index) => renderEntry(entry, index))}
            </div>
          </div>
        )}
      </PageContainer>

      <GraphDataDialog
        opened={graphDataDialog.open}
        graphId={graphDataDialog.graphId}
        onClose={() => setGraphDataDialog({ open: false, graphId: null })}
        title="Graph Data"
      />

      <Dialog open={textPreview.open} onOpenChange={(open) => !open && setTextPreview((prev) => ({ ...prev, open: false }))}>
        <DialogContent className="sm:max-w-[700px] max-h-[90vh] flex flex-col">
          <DialogHeader>
            <DialogTitle>{textPreview.title}</DialogTitle>
          </DialogHeader>
          <div className="flex-1 overflow-hidden">
            <Stack gap="md">
              <ScrollArea className="h-[500px]">
                <Textarea value={textPreview.content} readOnly rows={30} className="font-mono text-sm resize-none" />
              </ScrollArea>
            </Stack>
          </div>
        </DialogContent>
      </Dialog>

      <MermaidPreviewDialog
        open={mermaidPreview.open}
        diagram={mermaidPreview.content}
        title={mermaidPreview.title}
        onClose={() => setMermaidPreview({ open: false, content: '', title: '' })}
      />
      <DotPreviewDialog
        open={dotPreview.open}
        diagram={dotPreview.content}
        title={dotPreview.title}
        onClose={() => setDotPreview({ open: false, content: '', title: '' })}
      />
    </>
  )
}

export { ProjectArtefactsPage }
