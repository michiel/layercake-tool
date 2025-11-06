import React, { useState } from 'react'
import { useQuery, useMutation, useApolloClient } from '@apollo/client/react'
import {
  IconHistory,
  IconPlayerPlay,
  IconTrash,
  IconX,
  IconAlertCircle,
  IconClock
} from '@tabler/icons-react'
import { Stack, Group } from '../layout-primitives'
import { Alert, AlertDescription } from '../ui/alert'
import { Badge } from '../ui/badge'
import { Button } from '../ui/button'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '../ui/dialog'
import { ScrollArea } from '../ui/scroll-area'
import { Spinner } from '../ui/spinner'
import { Tooltip, TooltipContent, TooltipTrigger, TooltipProvider } from '../ui/tooltip'
import {
  GET_GRAPH_EDITS,
  REPLAY_GRAPH_EDITS,
  CLEAR_GRAPH_EDITS,
  GraphEdit
} from '../../graphql/graphs'

interface EditHistoryModalProps {
  opened: boolean
  onClose: () => void
  graphId: number
  graphName: string
  onApplyEdits: (edits: GraphEdit[]) => void
}

const EditHistoryModal: React.FC<EditHistoryModalProps> = ({
  opened,
  onClose,
  graphId,
  graphName,
  onApplyEdits
}) => {
  const [showAppliedEdits, setShowAppliedEdits] = useState(false)
  const client = useApolloClient()

  const { data, loading, error, refetch } = useQuery(GET_GRAPH_EDITS, {
    variables: {
      graphId,
      unappliedOnly: !showAppliedEdits
    },
    skip: !opened
  })

  const [replayEdits, { loading: replayLoading }] = useMutation(REPLAY_GRAPH_EDITS, {
    onCompleted: (data: any) => {
      const summary = data.replayGraphEdits
      alert(`Replay Complete: Applied ${summary.applied}, Skipped ${summary.skipped}, Failed ${summary.failed}`)
      refetch()
    },
    onError: (error: any) => {
      alert(`Replay Failed: ${error.message}`)
    }
  })

  const [clearEdits, { loading: clearLoading }] = useMutation(CLEAR_GRAPH_EDITS, {
    onCompleted: () => {
      alert('Edits Cleared: All edit history has been removed')
      refetch()
      onClose()
    },
    onError: (error: any) => {
      alert(`Clear Failed: ${error.message}`)
    }
  })

  const handleReplay = async () => {
    if (!window.confirm('This will replay all unapplied edits on the current graph data. Edits that can\'t be applied will be skipped. Continue?')) {
      return
    }

    try {
      // Fetch unapplied edits to apply them optimistically
      const { data: editsData } = await client.query({
        query: GET_GRAPH_EDITS,
        variables: { graphId, unappliedOnly: true },
        fetchPolicy: 'network-only',
      })

      const unappliedEdits = (editsData as any)?.graphEdits || []

      // Apply edits optimistically to canvas
      if (unappliedEdits.length > 0) {
        onApplyEdits(unappliedEdits)
      }

      // Now trigger the backend replay
      await replayEdits({ variables: { graphId } })
    } catch (err: any) {
      alert(`Replay Failed: ${err.message}`)
    }
  }

  const handleClear = () => {
    if (window.confirm('This will permanently delete all edit history for this graph. This action cannot be undone. Continue?')) {
      clearEdits({ variables: { graphId } })
    }
  }

  const formatValue = (value: any): string => {
    if (value === null || value === undefined) return 'null'
    if (typeof value === 'object') return JSON.stringify(value, null, 2)
    return String(value)
  }

  const getTargetTypeIcon = (targetType: string) => {
    switch (targetType) {
      case 'node': return '⬢'
      case 'edge': return '→'
      case 'layer': return '▦'
      default: return '•'
    }
  }

  const edits = (data as any)?.graphEdits || []

  return (
    <Dialog open={opened} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="sm:max-w-[800px]">
        <DialogHeader>
          <DialogTitle>
            <Group gap="xs">
              <IconHistory className="h-5 w-5" />
              <span>Edit History: {graphName}</span>
            </Group>
          </DialogTitle>
        </DialogHeader>

        <TooltipProvider>
          <Stack gap="md" className="py-4">
            {/* Controls */}
            <Group justify="between">
              <Group gap="xs">
                <Button
                  size="sm"
                  variant={showAppliedEdits ? 'secondary' : 'default'}
                  onClick={() => setShowAppliedEdits(!showAppliedEdits)}
                >
                  {showAppliedEdits ? 'Show Unapplied Only' : 'Show All Edits'}
                </Button>
                <Badge variant="secondary">
                  {edits.length} {edits.length === 1 ? 'edit' : 'edits'}
                </Badge>
              </Group>

              <Group gap="xs">
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      variant="secondary"
                      size="icon"
                      onClick={handleReplay}
                      disabled={replayLoading || edits.filter((e: GraphEdit) => !e.applied).length === 0}
                    >
                      {replayLoading ? <Spinner className="h-4 w-4" /> : <IconPlayerPlay className="h-4 w-4" />}
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>Replay unapplied edits</TooltipContent>
                </Tooltip>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      variant="secondary"
                      size="icon"
                      onClick={handleClear}
                      disabled={clearLoading || edits.length === 0}
                      className="text-red-600"
                    >
                      {clearLoading ? <Spinner className="h-4 w-4" /> : <IconTrash className="h-4 w-4" />}
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>Clear all edits</TooltipContent>
                </Tooltip>
              </Group>
            </Group>

            {/* Loading state */}
            {loading && (
              <Group justify="center" className="py-12">
                <Spinner className="h-5 w-5" />
                <p className="text-sm text-muted-foreground">Loading edit history...</p>
              </Group>
            )}

            {/* Error state */}
            {error && (
              <Alert variant="destructive">
                <IconX className="h-4 w-4" />
                <AlertDescription>
                  Failed to load edit history: {error.message}
                </AlertDescription>
              </Alert>
            )}

            {/* Empty state */}
            {!loading && !error && edits.length === 0 && (
              <Alert>
                <IconAlertCircle className="h-4 w-4" />
                <AlertDescription>
                  {showAppliedEdits
                    ? 'No edit history found for this graph.'
                    : 'No unapplied edits. All changes have been applied.'}
                </AlertDescription>
              </Alert>
            )}

            {/* Edit timeline */}
            {!loading && !error && edits.length > 0 && (
              <ScrollArea className="h-[500px]">
                <div className="relative space-y-4 before:absolute before:inset-0 before:left-4 before:h-full before:w-0.5 before:bg-border">
                  {edits.map((edit: GraphEdit) => (
                    <div key={edit.id} className="relative pl-10">
                      {/* Timeline bullet */}
                      <div className="absolute left-0 flex h-8 w-8 items-center justify-center rounded-full border-2 border-border bg-background text-sm">
                        {getTargetTypeIcon(edit.targetType)}
                      </div>

                      {/* Timeline content */}
                      <div className="space-y-2">
                        <Group gap="xs" className="flex-wrap">
                          <Badge
                            variant="secondary"
                            className={
                              edit.operation === 'create'
                                ? 'bg-green-100 text-green-900'
                                : edit.operation === 'update'
                                  ? 'bg-blue-100 text-blue-900'
                                  : edit.operation === 'delete'
                                    ? 'bg-red-100 text-red-900'
                                    : ''
                            }
                          >
                            {edit.operation}
                          </Badge>
                          <Badge variant="secondary">
                            {edit.targetType}
                          </Badge>
                          <span className="text-sm font-medium">
                            {edit.targetId}
                          </span>
                          {edit.applied && (
                            <Badge variant="secondary" className="bg-green-100 text-green-900 text-xs">
                              Applied
                            </Badge>
                          )}
                        </Group>

                        <Stack gap="xs" className="mt-2">
                          {edit.fieldName && (
                            <p className="text-xs text-muted-foreground">
                              Field: <code className="px-1 py-0.5 rounded bg-muted text-xs">{edit.fieldName}</code>
                            </p>
                          )}

                          {edit.operation === 'update' && (
                            <div>
                              <p className="text-xs text-muted-foreground mb-1">Old value:</p>
                              <pre className="text-[11px] max-h-[100px] overflow-auto bg-muted p-2 rounded">
                                <code>{formatValue(edit.oldValue)}</code>
                              </pre>
                              <p className="text-xs text-muted-foreground mt-2 mb-1">New value:</p>
                              <pre className="text-[11px] max-h-[100px] overflow-auto bg-muted p-2 rounded">
                                <code>{formatValue(edit.newValue)}</code>
                              </pre>
                            </div>
                          )}

                          {edit.operation === 'create' && edit.newValue && (
                            <div>
                              <p className="text-xs text-muted-foreground mb-1">Created with:</p>
                              <pre className="text-[11px] max-h-[150px] overflow-auto bg-muted p-2 rounded">
                                <code>{formatValue(edit.newValue)}</code>
                              </pre>
                            </div>
                          )}

                          {edit.operation === 'delete' && edit.oldValue && (
                            <div>
                              <p className="text-xs text-muted-foreground mb-1">Deleted:</p>
                              <pre className="text-[11px] max-h-[150px] overflow-auto bg-muted p-2 rounded">
                                <code>{formatValue(edit.oldValue)}</code>
                              </pre>
                            </div>
                          )}

                          <Group gap="xs">
                            <IconClock className="h-3 w-3" />
                            <p className="text-xs text-muted-foreground">
                              {new Date(edit.createdAt).toLocaleString()}
                            </p>
                            <p className="text-xs text-muted-foreground">
                              • Sequence #{edit.sequenceNumber}
                            </p>
                          </Group>
                        </Stack>
                      </div>
                    </div>
                  ))}
                </div>
              </ScrollArea>
            )}
          </Stack>
        </TooltipProvider>
      </DialogContent>
    </Dialog>
  )
}

export default EditHistoryModal
