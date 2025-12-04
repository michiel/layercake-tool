import { useMemo } from 'react'
import { useQuery } from '@apollo/client/react'
import { MermaidPreviewDialog } from '@/components/visualization/MermaidPreviewDialog'
import { GET_DATASOURCES, DataSet } from '@/graphql/datasets'
import { Sequence, SequenceEdgeRef } from '@/graphql/sequences'

interface GraphNode {
  id: string
  label?: string
  name?: string
  layer?: string
  belongs_to?: string
  is_partition?: boolean | string
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

  // Helper to find a node across all enabled datasets
  const findNode = (nodeId: string): GraphNode | null => {
    if (!sequence) return null
    for (const dsId of sequence.enabledDatasetIds) {
      const graphData = datasetGraphData[dsId]
      if (!graphData) continue
      const node = graphData.nodes.find((n) => n.id === nodeId)
      if (node) return node
    }
    return null
  }

  // Helper to get node label - searches across all enabled datasets
  const getNodeLabel = (nodeId: string): string => {
    if (!sequence) return nodeId
    for (const dsId of sequence.enabledDatasetIds) {
      const graphData = datasetGraphData[dsId]
      if (!graphData) continue
      const node = graphData.nodes.find((n) => n.id === nodeId)
      if (node) {
        const label = node.label || node.name || node.attrs?.label || node.attrs?.name
        return label && String(label).trim() ? String(label) : nodeId
      }
    }
    return nodeId
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

    const participantOrder: string[] = []
    const participantLabels: Map<string, string> = new Map()
    const participantPartitions: Map<string, string | null> = new Map()

    // First pass: collect participants in order of first appearance
    for (const ref of sequence.edgeOrder) {
      const edgeInfo = getEdgeInfo(ref)
      if (!edgeInfo) continue

      for (const nodeId of [edgeInfo.source, edgeInfo.target]) {
        if (!participantLabels.has(nodeId)) {
          participantOrder.push(nodeId)
          participantLabels.set(nodeId, getNodeLabel(nodeId))
          // Get partition (belongs_to) for this node
          const node = findNode(nodeId)
          const belongsTo = node?.belongs_to || node?.attrs?.belongs_to || null
          participantPartitions.set(nodeId, belongsTo)
        }
      }
    }

    // Group participants by partition
    const partitionGroups: Map<string | null, string[]> = new Map()
    for (const nodeId of participantOrder) {
      const partition = participantPartitions.get(nodeId) || null
      if (!partitionGroups.has(partition)) {
        partitionGroups.set(partition, [])
      }
      partitionGroups.get(partition)!.push(nodeId)
    }

    const lines: string[] = ['sequenceDiagram']

    // Add participant declarations
    for (const nodeId of participantOrder) {
      const label = participantLabels.get(nodeId) || nodeId
      const participantId = makeParticipantId(nodeId)
      lines.push(`    participant ${participantId} as "${escapeLabel(label)}"`)
    }

    // Add edges as messages
    for (let i = 0; i < sequence.edgeOrder.length; i++) {
      const ref = sequence.edgeOrder[i]
      const edgeInfo = getEdgeInfo(ref)
      if (!edgeInfo) continue

      const sourceId = makeParticipantId(edgeInfo.source)
      const targetId = makeParticipantId(edgeInfo.target)

      // Add note before the connection if present
      if (ref.note) {
        const noteText = escapeLabel(ref.note)
        const position = ref.notePosition || 'Both'
        if (position === 'Both') {
          lines.push(`    Note over ${sourceId},${targetId}: ${noteText}`)
        } else if (position === 'Source') {
          lines.push(`    Note over ${sourceId}: ${noteText}`)
        } else if (position === 'Target') {
          lines.push(`    Note over ${targetId}: ${noteText}`)
        }
      }

      const orderNum = i + 1
      const parts: string[] = [String(orderNum)]
      if (edgeInfo.label) parts.push(escapeLabel(edgeInfo.label))
      if (edgeInfo.comments) parts.push(escapeLabel(edgeInfo.comments))
      const message = parts.join(': ')
      lines.push(`    ${sourceId}->>${targetId}: ${message}`)
    }

    return lines.join('\n')
  }, [sequence, datasetGraphData])

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
