import React, { useMemo, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useMutation, useQuery } from '@apollo/client/react'
import {
  IconHierarchy2,
  IconGraph,
  IconBook,
  IconDownload,
  IconEye,
  IconTable,
  IconChartDots,
  IconNetwork,
  IconAlertCircle,
  IconChevronDown,
  IconChevronRight,
  IconSettings,
  IconExternalLink,
  IconPresentation,
} from '@tabler/icons-react'
import { gql } from '@apollo/client'
import { GET_PLAN_DAG, UPDATE_PLAN_DAG_NODE } from '../graphql/plan-dag'
import { EXPORT_NODE_OUTPUT } from '../graphql/export'
import { Stack, Group } from '../components/layout-primitives'
import PageContainer from '../components/layout/PageContainer'
import { Breadcrumbs } from '../components/common/Breadcrumbs'
import { Spinner } from '../components/ui/spinner'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../components/ui/select'
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
import { NodeConfigDialog } from '../components/editors/PlanVisualEditor/NodeConfigDialog'
import { MermaidPreviewDialog, DotPreviewDialog } from '../components/visualization'
import { showErrorNotification, showSuccessNotification } from '../utils/notifications'
import { PlanDagNodeType } from '../types/plan-dag'
import { useProjectPlanSelection } from '../hooks/useProjectPlanSelection'
import { cn } from '../lib/utils'

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
  | { type: 'story'; node: PlanDagNode; depth: number }
  | {
      type: 'artefact'
      node: PlanDagNode
      depth: number
      config: Record<string, any>
      parentGraphId?: string
      parentStoryId?: string
    }

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
  const renderConfig = config.renderConfig || {}

  // Check for containNodes
  if (renderConfig.containNodes === true) {
    labels.push('contain nodes')
  }

  // Check for theme
  if (renderConfig.theme === 'Light') {
    labels.push('light')
  } else if (renderConfig.theme === 'Dark') {
    labels.push('dark')
  }

  // Check for orientation
  if (renderConfig.orientation === 'LR') {
    labels.push('lr')
  } else if (renderConfig.orientation === 'TB') {
    labels.push('tb')
  }

  return labels
}

const ProjectArtefactsPage: React.FC = () => {
  const navigate = useNavigate()
  const { projectId } = useParams<{ projectId: string }>()
  const projectIdNum = Number(projectId || 0)
  const projectIdParam = projectId ?? (projectIdNum ? String(projectIdNum) : '')

  const { data: projectsData } = useQuery<{ projects: Array<{ id: number; name: string }> }>(GET_PROJECTS)
  const selectedProject = projectsData?.projects.find((p: { id: number; name: string }) => p.id === projectIdNum)

  const {
    plans,
    selectedPlanId,
    selectedPlan,
    loading: plansLoading,
    selectPlan,
  } = useProjectPlanSelection(projectIdNum)
  const planQuerySuffix = selectedPlanId ? `?planId=${selectedPlanId}` : ''
  const handleManagePlans = () => {
    if (!selectedProject) {
      return
    }
    navigate(`/projects/${selectedProject.id}/plans`)
  }

  const { data, loading: planDagLoading, error } = useQuery<PlanDagResponse>(GET_PLAN_DAG, {
    variables: { projectId: projectIdNum, planId: selectedPlanId },
    fetchPolicy: 'cache-and-network',
    skip: !selectedPlanId,
  })
  const loading = planDagLoading || plansLoading

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
  const [collapsedStories, setCollapsedStories] = useState<Set<string>>(new Set())
  const [editNodeDialog, setEditNodeDialog] = useState<{
    open: boolean
    nodeId: string | null
    nodeType: PlanDagNodeType | null
    config: any
    metadata: any
  }>({
    open: false,
    nodeId: null,
    nodeType: null,
    config: null,
    metadata: null,
  })

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

  const toggleStoryCollapse = (storyId: string) => {
    setCollapsedStories((prev) => {
      const next = new Set(prev)
      if (next.has(storyId)) {
        next.delete(storyId)
      } else {
        next.add(storyId)
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

  const [updatePlanDagNode] = useMutation(UPDATE_PLAN_DAG_NODE, {
    refetchQueries: [{ query: GET_PLAN_DAG, variables: { projectId: projectIdNum, planId: selectedPlanId } }],
    onCompleted: () => {
      showSuccessNotification('Node Updated', 'Artefact node configuration saved')
    },
    onError: (error) => {
      console.error('Update failed:', error)
      showErrorNotification('Update Failed', error.message)
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
    const storyNodes = nodes.filter((node) => node.nodeType === 'StoryNode')
    const storyIds = new Set(storyNodes.map((node) => node.id))
    const incomingStoryCount = new Map<string, number>()

    edges.forEach((edge) => {
      outgoing.set(edge.source, [...(outgoing.get(edge.source) ?? []), edge.target])
      if (graphIds.has(edge.source) && graphIds.has(edge.target)) {
        incomingGraphCount.set(edge.target, (incomingGraphCount.get(edge.target) ?? 0) + 1)
      }
      if (storyIds.has(edge.source) && storyIds.has(edge.target)) {
        incomingStoryCount.set(edge.target, (incomingStoryCount.get(edge.target) ?? 0) + 1)
      }
    })

    let graphRoots = graphNodes.filter((node) => (incomingGraphCount.get(node.id) ?? 0) === 0)
    if (!graphRoots.length) {
      graphRoots = graphNodes
    }

    const visitedGraphs = new Set<string>()
    const graphEntries: ArtefactEntry[] = []

    const traverseGraph = (nodeId: string, depth: number) => {
      if (visitedGraphs.has(nodeId)) return
      visitedGraphs.add(nodeId)

      const node = nodeMap.get(nodeId)
      if (!node) return
      graphEntries.push({ type: 'graph', node, depth })

      const children = outgoing.get(nodeId) ?? []
      const graphChildren = children.filter((id) => graphIds.has(id))
      const artefactChildren = children
        .map((id) => nodeMap.get(id))
        .filter(
          (child): child is PlanDagNode =>
            !!child && (child.nodeType === 'GraphArtefactNode' || child.nodeType === 'TreeArtefactNode' || child.nodeType === 'ProjectionNode'),
        )

      graphChildren.forEach((childId) => traverseGraph(childId, depth + 1))

      artefactChildren.forEach((child) => {
        graphEntries.push({
          type: 'artefact',
          node: child,
          depth: depth + 1,
          config: parseConfig(child.config),
          parentGraphId: nodeId,
        })
      })
    }

    graphRoots.forEach((root) => traverseGraph(root.id, 0))
    graphNodes.forEach((node) => {
      if (!visitedGraphs.has(node.id)) {
        traverseGraph(node.id, 0)
      }
    })

    let storyRoots = storyNodes.filter((node) => (incomingStoryCount.get(node.id) ?? 0) === 0)
    if (!storyRoots.length) {
      storyRoots = storyNodes
    }

    const visitedStories = new Set<string>()
    const storyEntries: ArtefactEntry[] = []

    const traverseStory = (nodeId: string, depth: number) => {
      if (visitedStories.has(nodeId)) return
      visitedStories.add(nodeId)

      const node = nodeMap.get(nodeId)
      if (!node) return
      storyEntries.push({ type: 'story', node, depth })

      const children = outgoing.get(nodeId) ?? []
      const storyChildren = children.filter((id) => storyIds.has(id))
      const sequenceChildren = children
        .map((id) => nodeMap.get(id))
        .filter((child): child is PlanDagNode => !!child && child.nodeType === 'SequenceArtefactNode')

      storyChildren.forEach((childId) => traverseStory(childId, depth + 1))

      sequenceChildren.forEach((child) => {
        storyEntries.push({
          type: 'artefact',
          node: child,
          depth: depth + 1,
          config: parseConfig(child.config),
          parentStoryId: nodeId,
        })
      })
    }

    storyRoots.forEach((root) => traverseStory(root.id, 0))
    storyNodes.forEach((node) => {
      if (!visitedStories.has(node.id)) {
        traverseStory(node.id, 0)
      }
    })

    return [...graphEntries, ...storyEntries]
  }, [data])

  const handleGraphData = (graphId?: number | null) => {
    if (!graphId) {
      showErrorNotification('Graph preview unavailable', 'Run the graph to generate data before previewing.')
      return
    }
    setGraphDataDialog({ open: true, graphId })
  }

  const handlePreview = (nodeId: string, target: 'text' | 'mermaid' | 'dot') => {
    if (!projectIdNum || !selectedPlanId) return
    setPreviewLoading({ nodeId, kind: target })
    exportForPreview({
      variables: { projectId: projectIdNum, planId: selectedPlanId, nodeId },
    })
  }

  const handleDownload = (nodeId: string) => {
    if (!projectIdNum || !selectedPlanId) return
    setDownloadingNodeId(nodeId)
    exportNodeOutput({
      variables: { projectId: projectIdNum, planId: selectedPlanId, nodeId },
    })
  }


  const handleNavigate = (route: string) => {
    navigate(route)
  }

  const handleEditNode = (nodeId: string, nodeType: string, config: any, metadata: any) => {
    let dagNodeType: PlanDagNodeType
    if (nodeType === 'GraphArtefactNode') {
      dagNodeType = PlanDagNodeType.GRAPH_ARTEFACT
    } else if (nodeType === 'TreeArtefactNode') {
      dagNodeType = PlanDagNodeType.TREE_ARTEFACT
    } else if (nodeType === 'ProjectionNode') {
      dagNodeType = PlanDagNodeType.PROJECTION
    } else {
      dagNodeType = PlanDagNodeType.TREE_ARTEFACT // fallback
    }
    setEditNodeDialog({
      open: true,
      nodeId,
      nodeType: dagNodeType,
      config: parseConfig(config),
      metadata: metadata || {},
    })
  }

  const handleSaveNode = async (nodeId: string, config: any, metadata: any) => {
    await updatePlanDagNode({
      variables: {
        projectId: projectIdNum,
        nodeId,
        updates: {
          config: JSON.stringify(config),
          metadata,
        },
      },
    })
  }

  const renderEntry = (entry: ArtefactEntry, index: number) => {
    if (entry.type === 'graph') {
      const label = entry.node.metadata?.label || entry.node.id
      const graphId = entry.node.graphExecution?.graphDataId ?? entry.node.graphExecution?.graphId ?? null
      const legacyGraphId = entry.node.graphExecution?.graphId
      const isLegacy = !!legacyGraphId && legacyGraphId !== graphId
      const isCollapsed = collapsedGraphs.has(entry.node.id)
      const hasChildren = entries.some(
        (child, childIndex) =>
          childIndex > index && child.type === 'artefact' && child.parentGraphId === entry.node.id,
      )

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
              <p className="text-xs text-muted-foreground">
                Graph Node {isLegacy ? '(legacy data)' : ''}
              </p>
            </div>
          </Group>
          <Group gap="xs">
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
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    size="icon"
                    variant="ghost"
                    className="h-8 w-8"
                    disabled={!graphId}
                    onClick={() => navigate(`/projects/${projectId}/graph/${graphId}/edit${planQuerySuffix}`)}
                  >
                    <IconExternalLink size={16} />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>Open graph editor</TooltipContent>
              </Tooltip>
            </TooltipProvider>
          </Group>
        </div>
      )
    }

    if (entry.type === 'story') {
      const label = entry.node.metadata?.label || entry.node.id
      const isCollapsed = collapsedStories.has(entry.node.id)
      const hasChildren = entries.some(
        (child, childIndex) =>
          childIndex > index && child.type === 'artefact' && child.parentStoryId === entry.node.id,
      )
      const storyConfig = parseConfig(entry.node.config)
      const storyId = storyConfig.storyId
      const storyLink = storyId && projectIdParam ? `/projects/${projectIdParam}/stories/${storyId}` : null
      const sequenceLink = storyLink ? `${storyLink}?tab=sequences` : null

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
                  toggleStoryCollapse(entry.node.id)
                }}
              >
                {isCollapsed ? <IconChevronRight size={16} /> : <IconChevronDown size={16} />}
              </Button>
            )}
            {!hasChildren && <div className="w-6" />}
            <IconBook className="h-4 w-4 text-muted-foreground" />
            <div>
              <p className="font-medium">{label}</p>
              <p className="text-xs text-muted-foreground">Story Node</p>
            </div>
          </Group>
          <Group gap="xs">
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    size="icon"
                    variant="ghost"
                    className="h-8 w-8"
                    disabled={!storyLink}
                    onClick={() => storyLink && navigate(storyLink)}
                  >
                    <IconTable size={16} />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>Open story</TooltipContent>
              </Tooltip>
            </TooltipProvider>
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    size="icon"
                    variant="ghost"
                    className="h-8 w-8"
                    disabled={!sequenceLink}
                    onClick={() => sequenceLink && navigate(sequenceLink)}
                  >
                    <IconExternalLink size={16} />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>Open sequence editor</TooltipContent>
              </Tooltip>
            </TooltipProvider>
          </Group>
        </div>
      )
    }

    // Check if parent graph is collapsed
    if (entry.parentGraphId && collapsedGraphs.has(entry.parentGraphId)) {
      return null
    }

    if (entry.parentStoryId && collapsedStories.has(entry.parentStoryId)) {
      return null
    }

    // Handle ProjectionNode specially
    if (entry.node.nodeType === 'ProjectionNode') {
      const label = entry.node.metadata?.label || entry.node.id
      const projectionConfig = parseConfig(entry.node.config)
      const projectionId = projectionConfig.projectionId
      const projectionLink = projectionId && projectIdParam ? `/projects/${projectIdParam}/projections/${projectionId}` : null

      return (
        <div
          key={entry.node.id}
          className="flex items-center py-4 px-4 border-b last:border-b-0"
          style={{ paddingLeft: `${16 + entry.depth * 24}px` }}
        >
          <Group gap="sm" className="text-sm min-w-0 flex-shrink-0">
            <IconPresentation className="h-4 w-4 text-orange-600" />
            <div className="min-w-0">
              <p className="font-medium truncate">{label}</p>
              <p className="text-xs text-muted-foreground">Projection</p>
            </div>
          </Group>
          <Group gap="xs" className="ml-auto flex-shrink-0">
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    size="icon"
                    variant="ghost"
                    className="h-8 w-8"
                    disabled={!projectionLink}
                    onClick={() => projectionLink && navigate(projectionLink)}
                  >
                    <IconExternalLink size={16} />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>Open projection viewer</TooltipContent>
              </Tooltip>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    size="icon"
                    variant="ghost"
                    className="h-8 w-8"
                    onClick={() => handleEditNode(entry.node.id, entry.node.nodeType, entry.node.config, entry.node.metadata)}
                  >
                    <IconSettings size={16} />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>Edit properties</TooltipContent>
              </Tooltip>
            </TooltipProvider>
          </Group>
        </div>
      )
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
        className="flex items-center py-4 px-4 border-b last:border-b-0"
        style={{ paddingLeft: `${16 + entry.depth * 24}px` }}
      >
        <Group gap="sm" className="text-sm min-w-0 flex-shrink-0">
          <IconHierarchy2 className="h-4 w-4 text-muted-foreground" />
          <div className="min-w-0">
            <p className="font-medium truncate">{label}</p>
            <p className="text-xs text-muted-foreground">{renderTargetLabel}</p>
          </div>
        </Group>
        {renderOptionLabels.length > 0 && (
          <Group gap="xs" className="flex-1 justify-center px-4">
            {renderOptionLabels.map((optionLabel) => (
              <span key={optionLabel} className="text-xs px-2 py-1 rounded bg-muted text-muted-foreground font-medium">
                {optionLabel}
              </span>
            ))}
          </Group>
        )}
        <Group gap="xs" className="ml-auto flex-shrink-0">
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
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  size="icon"
                  variant="ghost"
                  className="h-8 w-8"
                  onClick={() => handleEditNode(entry.node.id, entry.node.nodeType, entry.node.config, entry.node.metadata)}
                >
                  <IconSettings size={16} />
                </Button>
              </TooltipTrigger>
              <TooltipContent>Edit properties</TooltipContent>
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
          sections={[{ title: 'Workbench', href: `/projects/${selectedProject.id}/workbench${planQuerySuffix}` }]}
          currentPage="Artefacts"
          onNavigate={handleNavigate}
        />

        <Group justify="between" className="mb-4">
          <div>
            <h1 className="text-3xl font-bold">Artefacts</h1>
            <p className="text-sm text-muted-foreground mt-1">
              Browse graphs and exported artefacts for {selectedPlan ? selectedPlan.name : 'this project'}
            </p>
          </div>
          <Group gap="xs" className="flex-wrap justify-end lg:hidden">
            <Select
              value={selectedPlanId ? selectedPlanId.toString() : ''}
              onValueChange={(value) => selectPlan(Number(value))}
              disabled={plansLoading || plans.length === 0}
            >
              <SelectTrigger className="w-[220px]">
                <SelectValue
                  placeholder={
                    plans.length
                      ? 'Select a plan'
                      : plansLoading
                        ? 'Loading plans...'
                        : 'No plans available'
                  }
                />
              </SelectTrigger>
              <SelectContent>
                {plans.map((plan) => (
                  <SelectItem key={plan.id} value={plan.id.toString()}>
                    {plan.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Button variant="secondary" onClick={handleManagePlans}>
              Manage plans
            </Button>
          </Group>
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
        <div className="mt-6">
          <div className="w-full mx-auto max-w-6xl">
            <div className="lg:flex lg:items-start lg:gap-6">
              <div className="hidden lg:block w-64 flex-shrink-0">
                <div className="sticky top-24 space-y-3">
                  <div className="flex items-center justify-between">
                    <p className="text-sm font-semibold text-muted-foreground uppercase tracking-wide">Plans</p>
                    <Button variant="ghost" size="sm" className="h-7 px-2" onClick={handleManagePlans}>
                      Manage
                    </Button>
                  </div>
                  {plansLoading ? (
                    <div className="flex items-center justify-center py-6">
                      <Spinner />
                    </div>
                  ) : plans.length ? (
                    <div className="space-y-2">
                      {plans.map(plan => (
                        <button
                          type="button"
                          key={plan.id}
                          onClick={() => selectPlan(plan.id)}
                          className={cn(
                            'w-full rounded-md border px-3 py-2 text-left transition focus:outline-none focus:ring-2 focus:ring-ring',
                            selectedPlanId === plan.id
                              ? 'border-primary bg-primary/10 text-primary shadow-sm'
                              : 'border-transparent bg-muted/40 text-foreground hover:bg-muted'
                          )}
                        >
                          <p className="font-medium truncate">{plan.name}</p>
                          {plan.description && (
                            <p className="text-xs text-muted-foreground line-clamp-2 mt-1">
                              {plan.description}
                            </p>
                          )}
                        </button>
                      ))}
                    </div>
                  ) : (
                    <p className="text-sm text-muted-foreground">No plans available.</p>
                  )}
                </div>
              </div>
              <div className="flex-1">
                {loading && (
                  <div className="flex items-center justify-center py-16">
                    <Spinner />
                  </div>
                )}
                {!loading && !entries.length && (
                  <Alert className="max-w-4xl mx-auto lg:mx-0">
                    <AlertTitle>No graphs detected</AlertTitle>
                    <AlertDescription>Create graphs in the plan to see their artefacts here.</AlertDescription>
                  </Alert>
                )}
                {!loading && !!entries.length && (
                  <div className="max-w-4xl mx-auto lg:mx-0">
                    <div className="rounded-lg border bg-background shadow-sm">
                      <div className="flex items-center py-3 px-4 border-b bg-muted/50 font-medium text-sm">
                        <div className="flex-shrink-0 min-w-0" style={{ width: '40%' }}>
                          Name
                        </div>
                        <div className="flex-1 text-center">
                          Options
                        </div>
                        <div className="flex-shrink-0 text-right" style={{ width: '180px' }}>
                          Actions
                        </div>
                      </div>
                      {entries.map((entry, index) => renderEntry(entry, index))}
                    </div>
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
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

      {editNodeDialog.nodeId && editNodeDialog.nodeType && (
        <NodeConfigDialog
          opened={editNodeDialog.open}
          onClose={() => setEditNodeDialog({ open: false, nodeId: null, nodeType: null, config: null, metadata: null })}
          nodeType={editNodeDialog.nodeType}
          projectId={projectIdNum}
          onSave={handleSaveNode}
          nodeId={editNodeDialog.nodeId}
          config={editNodeDialog.config}
          metadata={editNodeDialog.metadata}
          graphIdHint={null}
        />
      )}
    </>
  )
}

export { ProjectArtefactsPage }
