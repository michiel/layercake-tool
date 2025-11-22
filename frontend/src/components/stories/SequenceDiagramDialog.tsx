import { useMemo } from 'react'
import { useQuery } from '@apollo/client/react'
import { MermaidPreviewDialog } from '@/components/visualization/MermaidPreviewDialog'
import { GET_DATASOURCES, DataSet } from '@/graphql/datasets'
import { GET_PROJECT_LAYERS, ProjectLayer } from '@/graphql/layers'
import { Sequence, SequenceEdgeRef } from '@/graphql/sequences'

interface GraphNode {
  id: string
  label?: string
  name?: string
  layer?: string
  attrs?: Record<string, any>
}

interface GraphEdge {
  id: string
  source: string
  target: string
  label?: string
  comments?: string
}

interface GraphData {
  nodes: GraphNode[]
  edges: GraphEdge[]
}

interface SequenceDiagramDialogProps {
  open: boolean
  onClose: () => void
  sequence: Sequence | null
  projectId: number
}

// Helper to escape Mermaid special characters in labels
const escapeLabel = (label: string): string => {
  return label.replace(/"/g, '\\"').replace(/\n/g, ' ')
}

// Helper to make valid Mermaid participant IDs (alphanumeric + underscore only)
const makeParticipantId = (nodeId: string): string => {
  return nodeId.replace(/[^a-zA-Z0-9_]/g, '_')
}

export const SequenceDiagramDialog = ({
  open,
  onClose,
  sequence,
  projectId,
}: SequenceDiagramDialogProps) => {
  // Fetch datasets for the project
  const { data: datasetsData } = useQuery(GET_DATASOURCES, {
    variables: { projectId },
    skip: !projectId || !open,
  })
  const allDatasets: DataSet[] = (datasetsData as any)?.dataSets || []

  // Fetch project layers
  const { data: layersData } = useQuery(GET_PROJECT_LAYERS, {
    variables: { projectId },
    skip: !projectId || !open,
  })
  const projectLayers: ProjectLayer[] = (layersData as any)?.projectLayers || []

  // Parse graph data from enabled datasets
  const datasetGraphData = useMemo(() => {
    if (!sequence) return {}
    const result: Record<number, GraphData> = {}
    const enabledIds = new Set(sequence.enabledDatasetIds)

    for (const ds of allDatasets) {
      if (!enabledIds.has(ds.id)) continue
      try {
        const data = JSON.parse(ds.graphJson)
        const nodes = data.nodes || []
        const edges = data.edges || data.links || []
        result[ds.id] = { nodes, edges }
      } catch (e) {
        console.error(`Failed to parse graphJson for dataset ${ds.id}:`, e)
        result[ds.id] = { nodes: [], edges: [] }
      }
    }
    return result
  }, [allDatasets, sequence])

  // Helper to get node info including layer - searches across all enabled datasets
  const getNodeInfo = (nodeId: string): { label: string; layer?: string } => {
    if (!sequence) return { label: nodeId }
    for (const dsId of sequence.enabledDatasetIds) {
      const graphData = datasetGraphData[dsId]
      if (!graphData) continue
      const node = graphData.nodes.find((n) => n.id === nodeId)
      if (node) {
        const label = node.label || node.name || node.attrs?.label || node.attrs?.name
        return {
          label: label && String(label).trim() ? String(label) : nodeId,
          layer: node.layer,
        }
      }
    }
    return { label: nodeId }
  }

  // Helper to get layer colors from project layers
  const getLayerColor = (layerId?: string): string | null => {
    if (!layerId) return null
    const layer = projectLayers.find((l) => l.layerId === layerId && l.enabled)
    if (!layer) return null
    return layer.backgroundColor || null
  }

  // Helper to get edge info
  const getEdgeInfo = (ref: SequenceEdgeRef): { source: string; target: string; label?: string; comments?: string } | null => {
    const graphData = datasetGraphData[ref.datasetId]
    const edge = graphData?.edges.find((e) => e.id === ref.edgeId)
    if (!edge) return null
    return {
      source: edge.source,
      target: edge.target,
      label: edge.label,
      comments: edge.comments,
    }
  }

  // Generate Mermaid sequence diagram
  const mermaidDiagram = useMemo(() => {
    if (!sequence || !sequence.edgeOrder.length) {
      return 'sequenceDiagram\n    Note over A: No edges in sequence'
    }

    const lines: string[] = ['sequenceDiagram']
    const participantOrder: string[] = []
    const participantInfo: Map<string, { label: string; color: string | null }> = new Map()

    // First pass: collect participants in order of first appearance
    for (const ref of sequence.edgeOrder) {
      const edgeInfo = getEdgeInfo(ref)
      if (!edgeInfo) continue

      if (!participantInfo.has(edgeInfo.source)) {
        participantOrder.push(edgeInfo.source)
        const nodeInfo = getNodeInfo(edgeInfo.source)
        participantInfo.set(edgeInfo.source, {
          label: nodeInfo.label,
          color: getLayerColor(nodeInfo.layer),
        })
      }
      if (!participantInfo.has(edgeInfo.target)) {
        participantOrder.push(edgeInfo.target)
        const nodeInfo = getNodeInfo(edgeInfo.target)
        participantInfo.set(edgeInfo.target, {
          label: nodeInfo.label,
          color: getLayerColor(nodeInfo.layer),
        })
      }
    }

    // Add participant declarations with colored boxes
    for (const nodeId of participantOrder) {
      const info = participantInfo.get(nodeId)
      if (!info) continue
      const participantId = makeParticipantId(nodeId)
      const color = info.color
      if (color) {
        lines.push(`    box ${color}`)
        lines.push(`        participant ${participantId} as "${escapeLabel(info.label)}"`)
        lines.push(`    end`)
      } else {
        lines.push(`    participant ${participantId} as "${escapeLabel(info.label)}"`)
      }
    }

    // Add edges as messages
    for (let i = 0; i < sequence.edgeOrder.length; i++) {
      const ref = sequence.edgeOrder[i]
      const edgeInfo = getEdgeInfo(ref)
      if (!edgeInfo) continue

      const sourceId = makeParticipantId(edgeInfo.source)
      const targetId = makeParticipantId(edgeInfo.target)
      const orderNum = i + 1
      const parts: string[] = [String(orderNum)]
      if (edgeInfo.label) parts.push(escapeLabel(edgeInfo.label))
      if (edgeInfo.comments) parts.push(escapeLabel(edgeInfo.comments))
      const message = parts.join(': ')
      lines.push(`    ${sourceId}->>${targetId}: ${message}`)
    }

    return lines.join('\n')
  }, [sequence, datasetGraphData, projectLayers])

  return (
    <MermaidPreviewDialog
      open={open}
      onClose={onClose}
      diagram={mermaidDiagram}
      title={sequence ? `Sequence Diagram: ${sequence.name}` : 'Sequence Diagram'}
    />
  )
}

export default SequenceDiagramDialog
