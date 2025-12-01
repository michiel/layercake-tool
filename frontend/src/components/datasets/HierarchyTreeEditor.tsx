import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { Tree, NodeRendererProps, TreeApi } from 'react-arborist'
import {
  IconHierarchy2,
  IconPlus,
  IconTrash,
  IconCornerDownRight,
  IconDeviceFloppy,
  IconRefresh,
  IconAlertCircle,
  IconChevronRight,
  IconChevronDown,
  IconArrowsMaximize,
  IconArrowsMinimize,
  IconGripVertical,
} from '@tabler/icons-react'
import { GraphData, GraphNode, GraphEdge, GraphLayer } from '@/components/editors/GraphSpreadsheetEditor'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Badge } from '@/components/ui/badge'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Group } from '@/components/layout-primitives'

type HierarchyNode = GraphNode & {
  belongs_to?: string | null
}

interface DatasetHierarchyTreeEditorProps {
  graphData: GraphData | null
  onSave: (next: GraphData) => Promise<void>
}

interface TreeItem {
  id: string
  name: string
  node: HierarchyNode
  children?: TreeItem[]
}

const generateNodeId = (existing: Set<string>) => {
  let counter = existing.size + 1
  let candidate = `node_${counter}`
  while (existing.has(candidate)) {
    counter += 1
    candidate = `node_${counter}`
  }
  return candidate
}

const normalizeLayerId = (layerId?: string | null) => {
  if (!layerId) return ''
  return layerId
}

const UNASSIGNED_LAYER_VALUE = '__unassigned__'

export const HierarchyTreeEditor = ({ graphData, onSave }: DatasetHierarchyTreeEditorProps) => {
  const [nodes, setNodes] = useState<HierarchyNode[]>(graphData?.nodes || [])
  const [edges, setEdges] = useState<GraphEdge[]>(graphData?.edges || [])
  const [layers, setLayers] = useState<GraphLayer[]>(graphData?.layers || [])
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const [dirty, setDirty] = useState(false)
  const [saving, setSaving] = useState(false)
  const [idError, setIdError] = useState<string | null>(null)
  const treeRef = useRef<TreeApi<TreeItem> | null>(null)

  useEffect(() => {
    setNodes(graphData?.nodes || [])
    setEdges(graphData?.edges || [])
    setLayers(graphData?.layers || [])
    setSelectedId(null)
    setDirty(false)
    setIdError(null)
  }, [graphData])

  const nodeIdSet = useMemo(() => new Set(nodes.map(n => n.id)), [nodes])

  const parentMap = useMemo(() => {
    const map = new Map<string | null, HierarchyNode[]>()
    const validIds = new Set(nodes.map(n => n.id))
    nodes.forEach(node => {
      const parent = node.belongs_to && validIds.has(node.belongs_to) ? node.belongs_to : null
      const bucket = map.get(parent) || []
      bucket.push(node)
      map.set(parent, bucket)
    })
    return map
  }, [nodes])

  const buildTreeItems = useCallback(
    (parent: string | null): TreeItem[] => {
      const children = parentMap.get(parent) || []
      return children
        .slice()
        .sort((a, b) => {
          const aLabel = a.label || a.id
          const bLabel = b.label || b.id
          return aLabel.localeCompare(bLabel)
        })
        .map(child => {
          const nestedChildren = buildTreeItems(child.id)
          return {
            id: child.id,
            name: child.label || child.id,
            node: child,
            children: nestedChildren.length ? nestedChildren : undefined,
          }
        })
    },
    [parentMap]
  )

  const treeData = useMemo(() => buildTreeItems(null), [buildTreeItems])
  const selectedNode = nodes.find(n => n.id === selectedId) || null
  const layerMap = useMemo(() => {
    const map = new Map<string, GraphLayer>()
    layers.forEach(layer => {
      if (layer.id) {
        map.set(layer.id, layer)
      }
    })
    return map
  }, [layers])

  const markDirty = () => {
    if (!dirty) setDirty(true)
  }

  const handleAddNode = (parentId: string | null) => {
    const newId = generateNodeId(nodeIdSet)
    const newNode: HierarchyNode = {
      id: newId,
      label: 'New node',
      layer: selectedNode?.layer || '',
      is_partition: false,
      belongs_to: parentId ?? undefined,
      comment: '',
    }
    setNodes(prev => [...prev, newNode])
    setSelectedId(newId)
    setDirty(true)
  }

  const collectDescendants = useCallback(
    (id: string): Set<string> => {
      const toDelete = new Set<string>([id])
      let changed = true
      while (changed) {
        changed = false
        nodes.forEach(node => {
          if (node.belongs_to && toDelete.has(node.belongs_to) && !toDelete.has(node.id)) {
            toDelete.add(node.id)
            changed = true
          }
        })
      }
      return toDelete
    },
    [nodes]
  )

  const handleDeleteNode = () => {
    if (!selectedId) return
    const targetNode = nodes.find(n => n.id === selectedId)
    if (!targetNode) return
    const descendants = collectDescendants(selectedId)
    if (descendants.size === 0) return
    if (!window.confirm('Delete selected node and all of its descendants?')) {
      return
    }
    setNodes(prev => prev.filter(node => !descendants.has(node.id)))
    setEdges(prev => prev.filter(edge => !descendants.has(edge.source) && !descendants.has(edge.target)))
    if (descendants.has(selectedId)) {
      setSelectedId(null)
    }
    setDirty(true)
  }

  const handleMove = async ({
    dragIds,
    parentId,
  }: {
    dragIds: string[]
    parentId: string | null
  }) => {
    if (!dragIds.length) return
    const normalizedParent = parentId || null
    if (dragIds.includes(normalizedParent || '')) {
      return
    }
    setNodes(prev =>
      prev.map(node =>
        dragIds.includes(node.id)
          ? {
              ...node,
              belongs_to: normalizedParent ?? undefined,
            }
          : node
      )
    )
    markDirty()
  }

  const updateNodeField = (nodeId: string, field: keyof HierarchyNode, value: string) => {
    setNodes(prev =>
      prev.map(node => (node.id === nodeId ? { ...node, [field]: value } : node))
    )
    markDirty()
  }

  const handleLayerChange = (layerId: string) => {
    if (!selectedNode) return
    const normalized = layerId === UNASSIGNED_LAYER_VALUE ? '' : layerId
    updateNodeField(selectedNode.id, 'layer', normalized)
  }

  const handleIdChange = (value: string) => {
    if (!selectedNode) return
    const trimmed = value.trim()
    if (!trimmed) {
      setIdError('ID is required')
      return
    }
    if (trimmed !== selectedNode.id && nodes.some(node => node.id === trimmed)) {
      setIdError('ID must be unique')
      return
    }
    if (trimmed === selectedNode.id) {
      setIdError(null)
      updateNodeField(selectedNode.id, 'id', trimmed)
      return
    }

    setNodes(prev =>
      prev.map(node => {
        if (node.id === selectedNode.id) {
          return { ...node, id: trimmed }
        }
        if (node.belongs_to === selectedNode.id) {
          return { ...node, belongs_to: trimmed }
        }
        return node
      })
    )
    setEdges(prev =>
      prev.map(edge => {
        let updated = edge
        if (edge.source === selectedNode.id) {
          updated = { ...updated, source: trimmed }
        }
        if (edge.target === selectedNode.id) {
          updated = { ...updated, target: trimmed }
        }
        return updated
      })
    )
    setIdError(null)
    setSelectedId(trimmed)
    markDirty()
  }

  const handleSave = async () => {
    if (!graphData) return
    setSaving(true)
    try {
      const validIds = new Set(nodes.map(n => n.id))
      const sanitizedEdges = edges.filter(edge => validIds.has(edge.source) && validIds.has(edge.target))
      const nextGraph: GraphData = {
        nodes,
        edges: sanitizedEdges,
        layers,
      }
      await onSave(nextGraph)
      setDirty(false)
    } finally {
      setSaving(false)
    }
  }

  const handleDiscard = () => {
    if (!graphData) return
    setNodes(graphData.nodes || [])
    setEdges(graphData.edges || [])
    setLayers(graphData.layers || [])
    setSelectedId(null)
    setDirty(false)
    setIdError(null)
  }

  if (!graphData) {
    return (
      <Alert>
        <IconAlertCircle className="h-4 w-4" />
        <AlertTitle>No graph data found</AlertTitle>
        <AlertDescription>Upload or generate graph nodes before editing the hierarchy.</AlertDescription>
      </Alert>
    )
  }

  const NodeRow = ({ node, style, dragHandle }: NodeRendererProps<TreeItem>) => {
    const current = node.data.node
    const layerId = normalizeLayerId(current.layer)
    const layer = layerId ? layerMap.get(layerId) : undefined
    const isSelected = selectedId === current.id
    const isPartition =
      current.is_partition === true ||
      (typeof current.is_partition === 'string' && current.is_partition === 'true')
    const layerLabel = layer?.label || layerId || 'Unassigned'
    const layerBg = layer?.background_color || '#CBD5F5'
    const layerText = layer?.text_color || '#1F2937'
    const layerBorder = layer?.border_color || '#94A3B8'
    return (
      <div
        style={{ ...style, width: '100%' }}
        className={`flex w-full cursor-pointer items-center gap-2 rounded-md px-2 py-1 text-sm ${
          isSelected ? 'bg-primary/10 text-primary' : 'hover:bg-muted'
        }`}
        onClick={() => setSelectedId(current.id)}
        onDoubleClick={() => {
          if (node.isInternal) {
            node.toggle()
          }
        }}
      >
        {node.isInternal ? (
          <button
            type="button"
            className="flex h-5 w-5 items-center justify-center rounded hover:bg-muted-foreground/20"
            onClick={event => {
              event.stopPropagation()
              node.toggle()
            }}
          >
            {node.isOpen ? (
              <IconChevronDown className="h-4 w-4" />
            ) : (
              <IconChevronRight className="h-4 w-4" />
            )}
          </button>
        ) : (
          <span className="h-5 w-5" />
        )}
        <span
          className="h-2 w-2 rounded-full"
          style={{
            backgroundColor: layerBg,
            color: layerText,
          }}
        />
        <div className="flex min-w-0 flex-1 items-center justify-between gap-2">
          <span className="truncate">{current.label || current.id}</span>
          <div className="flex items-center gap-2 text-xs">
            <span
              className="rounded-md px-2 py-0.5"
              style={{
                backgroundColor: layerBg,
                color: layerText,
                border: `1px solid ${layerBorder}`,
              }}
            >
              {layerLabel}
            </span>
            {isPartition && <Badge variant="outline">Partition</Badge>}
          </div>
        </div>
        <span
          ref={dragHandle}
          className="ml-2 flex h-5 w-5 cursor-grab items-center justify-center text-muted-foreground"
          title="Drag to move"
        >
          <IconGripVertical className="h-4 w-4" />
        </span>
      </div>
    )
  }

  return (
    <Card className="border">
      <CardHeader className="flex flex-row items-center justify-between gap-4">
        <CardTitle className="flex items-center gap-2">
          <IconHierarchy2 className="h-4 w-4" />
          Dataset Hierarchy
        </CardTitle>
        <Group gap="sm">
          <Button
            variant="outline"
            size="sm"
            onClick={handleDiscard}
            disabled={!dirty || saving}
          >
            <IconRefresh className="mr-2 h-4 w-4" />
            Discard
          </Button>
          <Button
            size="sm"
            onClick={handleSave}
            disabled={!dirty || saving}
          >
            {saving && <IconDeviceFloppy className="mr-2 h-4 w-4 animate-spin" />}
            {!saving && <IconDeviceFloppy className="mr-2 h-4 w-4" />}
            Save changes
          </Button>
        </Group>
      </CardHeader>
      <CardContent className="p-0">
        {dirty && (
          <div className="border-b border-amber-200 bg-amber-50 px-4 py-2 text-sm text-amber-900">
            Unsaved hierarchy changes
          </div>
        )}
        <div className="flex h-[620px]">
          <div className="flex basis-4/5 flex-col border-r pr-2">
            <div className="flex items-center gap-2 border-b px-4 py-2">
              <Button size="sm" variant="secondary" onClick={() => handleAddNode(null)}>
                <IconPlus className="mr-1 h-4 w-4" />
                Add root
              </Button>
              <Button
                size="sm"
                variant="secondary"
                onClick={() => handleAddNode(selectedId)}
                disabled={!selectedId}
              >
                <IconCornerDownRight className="mr-1 h-4 w-4" />
                Add child
              </Button>
              <Button
                size="sm"
                variant="outline"
                onClick={handleDeleteNode}
                disabled={!selectedId}
              >
                <IconTrash className="mr-1 h-4 w-4" />
                Delete
              </Button>
              <div className="ml-auto flex items-center gap-2">
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() => treeRef.current?.closeAll()}
                >
                  <IconArrowsMinimize className="mr-1 h-4 w-4" />
                  Collapse all
                </Button>
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() => treeRef.current?.openAll()}
                >
                  <IconArrowsMaximize className="mr-1 h-4 w-4" />
                  Expand all
                </Button>
              </div>
            </div>
            <div className="flex-1 px-2 py-2">
              <Tree<TreeItem>
                data={treeData}
                openByDefault
                onMove={handleMove}
                selection={selectedId ?? undefined}
                className="h-full"
                rowHeight={32}
                width="100%"
                ref={treeRef}
              >
                {NodeRow}
              </Tree>
            </div>
          </div>
          <div className="basis-1/5 pl-2">
            <ScrollArea className="h-full">
              <div className="space-y-4 px-4 py-4">
                {selectedNode ? (
                  <>
                    <div>
                      <Label htmlFor="node-id">Node ID</Label>
                      <Input
                        id="node-id"
                        value={selectedNode.id}
                        onChange={e => handleIdChange(e.target.value)}
                      />
                      {idError && <p className="mt-1 text-xs text-destructive">{idError}</p>}
                    </div>
                    <div>
                      <Label htmlFor="node-label">Label</Label>
                      <Input
                        id="node-label"
                        value={selectedNode.label || ''}
                        onChange={e => updateNodeField(selectedNode.id, 'label', e.target.value)}
                      />
                    </div>
                    <div>
                      <Label htmlFor="node-comment">Comment</Label>
                      <Textarea
                        id="node-comment"
                        value={selectedNode.comment || ''}
                        rows={4}
                        onChange={e => updateNodeField(selectedNode.id, 'comment', e.target.value)}
                      />
                    </div>
                    <div>
                      <Label>Layer</Label>
                      <Select
                        value={normalizeLayerId(selectedNode.layer) || UNASSIGNED_LAYER_VALUE}
                        onValueChange={handleLayerChange}
                        disabled={layers.length === 0}
                      >
                        <SelectTrigger>
                          <SelectValue
                            placeholder={
                              layers.length === 0 ? 'No layers available' : 'Select layer'
                            }
                          />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value={UNASSIGNED_LAYER_VALUE}>Unassigned</SelectItem>
                          {layers.map(layer => (
                            <SelectItem key={layer.id} value={layer.id}>
                              {layer.label || layer.id}
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                      {layers.length === 0 && (
                        <p className="mt-1 text-xs text-muted-foreground">
                          Define dataset layers to enable layer selection.
                        </p>
                      )}
                      {selectedNode.layer && (
                        <div className="mt-2 flex items-center gap-2 text-xs text-muted-foreground">
                          <span>Preview:</span>
                          <span
                            className="h-4 w-4 rounded"
                            style={{
                              backgroundColor:
                                layerMap.get(normalizeLayerId(selectedNode.layer || ''))?.background_color ||
                                '#CBD5F5',
                              border: '1px solid rgba(0,0,0,0.1)',
                            }}
                          />
                        </div>
                      )}
                    </div>
                    <div>
                      <Label>Parent</Label>
                      <p className="text-sm text-muted-foreground">
                        {selectedNode.belongs_to || 'Root'}
                      </p>
                    </div>
                  </>
                ) : (
                  <p className="text-sm text-muted-foreground">Select a node to edit its properties.</p>
                )}
              </div>
            </ScrollArea>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
