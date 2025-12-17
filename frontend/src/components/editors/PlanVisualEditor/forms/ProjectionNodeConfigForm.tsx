import React, { useCallback, useEffect, useMemo, useState } from 'react'
import { gql } from '@apollo/client'
import { useLazyQuery, useMutation, useQuery } from '@apollo/client/react'
import { ProjectionNodeConfig } from '../../../../types/plan-dag'
import { Stack, Group } from '@/components/layout-primitives'
import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Switch } from '@/components/ui/switch'
import { Checkbox } from '@/components/ui/checkbox'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Spinner } from '@/components/ui/spinner'
import { showErrorNotification, showSuccessNotification } from '@/utils/notifications'
import { createApolloClientForEndpoint } from '@/graphql/client'
import { LIST_STORIES } from '@/graphql/stories'
import { LIST_SEQUENCES } from '@/graphql/sequences'

interface ProjectionNodeConfigFormProps {
  config: ProjectionNodeConfig
  setConfig: (config: ProjectionNodeConfig) => void
  setIsValid: (isValid: boolean) => void
  projectId: number
  graphIdHint?: number | null
  graphSourceNodeIdHint?: string | null
}

const PROJECTION_EDIT_QUERY = gql`
  query ProjectionEdit($id: ID!) {
    projection(id: $id) {
      id
      name
      projectionType
      projectId
      graphId
    }
    projectionState(id: $id) {
      projectionId
      projectionType
      stateJson
    }
  }
`

const CREATE_PROJECTION = gql`
  mutation CreateProjection($input: CreateProjectionInput!) {
    createProjection(input: $input) {
      id
      name
      projectionType
      graphId
    }
  }
`

const UPDATE_PROJECTION = gql`
  mutation UpdateProjection($id: ID!, $input: UpdateProjectionInput!) {
    updateProjection(id: $id, input: $input) {
      id
      name
      projectionType
    }
  }
`

const GRAPH_DATA_BY_DAG_NODE = gql`
  query GraphDataByDagNode($dagNodeId: String!) {
    graphDataByDagNode(dagNodeId: $dagNodeId) {
      id
      name
    }
  }
`

const SAVE_PROJECTION_STATE = gql`
  mutation SaveProjectionState($id: ID!, $state: JSON!) {
    saveProjectionState(id: $id, state: $state)
  }
`

const VERIFY_PROJECTION_STORY_MATCH = gql`
  mutation VerifyProjectionStoryMatch($projectionId: Int!, $stories: [ProjectionStorySelectionInput!]!) {
    verifyProjectionStoryMatch(projectionId: $projectionId, stories: $stories) {
      success
      sequences {
        storyId
        sequenceId
        missingEdges {
          datasetId
          edgeId
        }
      }
    }
  }
`

type StorySelection = {
  enabled: boolean
  sequences: Record<number, boolean>
}

type SequenceInfo = { id: number; name: string }
type SequencesQueryResult = { sequences: SequenceInfo[] }

const buildSelectionPayload = (selections: Record<number, StorySelection>) =>
  Object.entries(selections)
    .filter(([, sel]) => sel.enabled)
    .map(([storyId, sel]) => ({
      storyId: Number(storyId),
      enabledSequenceIds: Object.entries(sel.sequences)
        .filter(([, enabled]) => enabled)
        .map(([seqId]) => Number(seqId)),
    }))

export const ProjectionNodeConfigForm: React.FC<ProjectionNodeConfigFormProps> = ({
  config,
  setConfig,
  setIsValid,
  projectId,
  graphIdHint,
  graphSourceNodeIdHint,
}) => {
  const projectionsClient = useMemo(
    () =>
      createApolloClientForEndpoint({
        httpPath: '/projections/graphql',
        wsPath: '/projections/graphql/ws',
      }),
    []
  )

  const [localConfig, setLocalConfig] = useState<ProjectionNodeConfig>({
    projectionId: config.projectionId,
  })
  const [projectionName, setProjectionName] = useState('')
  const [projectionType, setProjectionType] = useState<'force3d' | 'layer3d'>('force3d')
  const [storyModeEnabled, setStoryModeEnabled] = useState(false)
  const [storySelections, setStorySelections] = useState<Record<number, StorySelection>>({})
  const [sequenceCache, setSequenceCache] = useState<Record<number, SequenceInfo[]>>({})
  const [hydrated, setHydrated] = useState(false)
  const [verificationResult, setVerificationResult] = useState<{
    success: boolean
    missing: { storyId: number; sequenceId: number; missingEdges: { datasetId: number; edgeId: string }[] }[]
  } | null>(null)

  const projectionId = localConfig.projectionId

  const { data: projectionData, loading: loadingProjection, refetch: refetchProjection } = useQuery(
    PROJECTION_EDIT_QUERY,
    {
      variables: { id: projectionId?.toString() ?? '' },
      skip: !projectionId,
      client: projectionsClient,
      fetchPolicy: 'cache-and-network',
    }
  )

  const { data: storiesData, loading: loadingStories } = useQuery(LIST_STORIES, {
    variables: { projectId },
    skip: !projectId,
  })

  const [loadSequences, { loading: loadingSeq }] = useLazyQuery<SequencesQueryResult, { storyId: number }>(
    LIST_SEQUENCES,
    {
      fetchPolicy: 'cache-first',
    }
  )

  const [createProjection, { loading: creatingProjection }] = useMutation(CREATE_PROJECTION, {
    client: projectionsClient,
  })
  const [updateProjection, { loading: updatingProjection }] = useMutation(UPDATE_PROJECTION, {
    client: projectionsClient,
  })
  const [saveState, { loading: savingState }] = useMutation(SAVE_PROJECTION_STATE, {
    client: projectionsClient,
  })
  const [verifyStories, { loading: verifying }] = useMutation(VERIFY_PROJECTION_STORY_MATCH)

  const projection = (projectionData as any)?.projection
  const projectionState = (projectionData as any)?.projectionState
  const stories = (storiesData as any)?.stories ?? []
  const { data: graphDataLookup, loading: graphLookupLoading } = useQuery(GRAPH_DATA_BY_DAG_NODE, {
    variables: { dagNodeId: graphSourceNodeIdHint ?? '' },
    skip: !graphSourceNodeIdHint,
    fetchPolicy: 'cache-first',
  })

  const resolvedGraphDataId: number | null = useMemo(() => {
    const lookupId = (graphDataLookup as any)?.graphDataByDagNode?.id
    if (lookupId !== undefined && lookupId !== null) {
      const parsed = Number(lookupId)
      if (Number.isFinite(parsed)) return parsed
    }
    if (graphIdHint && Number.isFinite(graphIdHint)) return graphIdHint
    return null
  }, [graphIdHint, graphDataLookup])

  // Reset state when switching projections
  useEffect(() => {
    setHydrated(false)
    setStorySelections({})
    setSequenceCache({})
    setStoryModeEnabled(false)
    setVerificationResult(null)
    setProjectionName('')
    setProjectionType('force3d')
  }, [projectionId, resolvedGraphDataId])

  const ensureSequences = useCallback(
    async (storyId: number) => {
      if (sequenceCache[storyId]) return sequenceCache[storyId]
      const { data } = await loadSequences({ variables: { storyId } })
      const seqs: SequenceInfo[] = data?.sequences ?? []
      setSequenceCache((prev) => ({ ...prev, [storyId]: seqs }))
      return seqs
    },
    [loadSequences, sequenceCache]
  )

  // Auto-create a projection for the node if none exists yet
  useEffect(() => {
    const createIfNeeded = async () => {
      if (projectionId || !projectId || !resolvedGraphDataId) return
      try {
        const { data } = await createProjection({
          variables: {
            input: {
              projectId,
              graphId: resolvedGraphDataId,
              name: `Projection ${resolvedGraphDataId}`,
              projectionType: 'force3d',
            },
          },
        })
        const newProjection = (data as any)?.createProjection
        if (newProjection?.id) {
          setLocalConfig({ projectionId: Number(newProjection.id) })
          setProjectionName(newProjection.name)
          setProjectionType((newProjection.projectionType as 'force3d' | 'layer3d') ?? 'force3d')
          setIsValid(true)
          showSuccessNotification('Projection created', 'Projection node is now linked.')
        }
      } catch (err: any) {
        showErrorNotification('Projection creation failed', err?.message || 'Unable to create projection')
      }
    }
    void createIfNeeded()
  }, [projectionId, projectId, resolvedGraphDataId, createProjection, setIsValid])

  // If the linked projection points at a different graph, recreate and relink
  useEffect(() => {
    const relinkIfNeeded = async () => {
      if (!projection || !resolvedGraphDataId) return
      if (projectionId && Number(projection.id) !== projectionId) return
      const projectionGraphId = Number(projection.graphId)
      if (Number.isFinite(projectionGraphId) && projectionGraphId === resolvedGraphDataId) {
        return
      }
      try {
        const { data } = await createProjection({
          variables: {
            input: {
              projectId,
              graphId: resolvedGraphDataId,
              name: projection.name || `Projection ${resolvedGraphDataId}`,
              projectionType: projection.projectionType || 'force3d',
            },
          },
        })
        const newProjection = (data as any)?.createProjection
        if (newProjection?.id) {
          setLocalConfig({ projectionId: Number(newProjection.id) })
          setProjectionName(newProjection.name)
          setProjectionType((newProjection.projectionType as 'force3d' | 'layer3d') ?? 'force3d')
          setIsValid(true)
          showSuccessNotification('Projection relinked', 'Projection now points at the computed graph output.')
        }
      } catch (err: any) {
        console.error('Failed to relink projection to graph data', err)
        showErrorNotification('Projection relink failed', err?.message || 'Unable to recreate projection')
      }
    }
    void relinkIfNeeded()
  }, [projection, resolvedGraphDataId, projectId, createProjection, setIsValid])

  // Keep parent config in sync
  useEffect(() => {
    setConfig(localConfig)
  }, [localConfig, setConfig])

  useEffect(() => {
    setIsValid(!!localConfig.projectionId)
  }, [localConfig, setIsValid])

  useEffect(() => {
    if (!projection || hydrated) return
    setProjectionName(projection.name)
    setProjectionType((projection.projectionType as 'force3d' | 'layer3d') ?? 'force3d')
  }, [projection, hydrated])

  // Hydrate story mode state from saved projection state
  useEffect(() => {
    if (!projectionState || hydrated) return
    const storyMode = (projectionState.stateJson as any)?.storyMode ?? {}
    setStoryModeEnabled(!!storyMode.enabled)
    const nextSelections: Record<number, StorySelection> = {}
    for (const entry of storyMode.stories ?? []) {
      nextSelections[entry.storyId] = {
        enabled: true,
        sequences: Object.fromEntries((entry.enabledSequenceIds ?? []).map((id: number) => [id, true])),
      }
    }
    setStorySelections(nextSelections)
    setHydrated(true)
  }, [projectionState, hydrated])

  useEffect(() => {
    const loadInitialSequences = async () => {
      const enabledStories = Object.entries(storySelections)
        .filter(([, sel]) => sel.enabled)
        .map(([id]) => Number(id))
      for (const storyId of enabledStories) {
        if (sequenceCache[storyId]) continue
        const seqs = await ensureSequences(storyId)
        if (seqs.length && !storySelections[storyId]?.sequences) {
          setStorySelections((prev) => ({
            ...prev,
            [storyId]: {
              enabled: true,
              sequences: Object.fromEntries(seqs.map((s) => [s.id, true])),
            },
          }))
        }
      }
    }
    if (hydrated) {
      void loadInitialSequences()
    }
  }, [ensureSequences, hydrated, sequenceCache, storySelections])

  const handleStoryToggle = async (storyId: number, enabled: boolean) => {
    if (enabled) {
      const seqs = await ensureSequences(storyId)
      setStorySelections((prev) => ({
        ...prev,
        [storyId]: {
          enabled: true,
          sequences: Object.fromEntries(
            seqs.map((s) => [s.id, prev[storyId]?.sequences?.[s.id] ?? true])
          ),
        },
      }))
    } else {
      setStorySelections((prev) => ({
        ...prev,
        [storyId]: { enabled: false, sequences: prev[storyId]?.sequences ?? {} },
      }))
    }
  }

  const handleSequenceToggle = (storyId: number, sequenceId: number, enabled: boolean) => {
    setStorySelections((prev) => ({
      ...prev,
      [storyId]: {
        enabled: prev[storyId]?.enabled ?? false,
        sequences: {
          ...(prev[storyId]?.sequences ?? {}),
          [sequenceId]: enabled,
        },
      },
    }))
  }

  const handleSave = async () => {
    if (!projectionId) return
    try {
      await updateProjection({
        variables: {
          id: projectionId.toString(),
          input: {
            name: projectionName,
            projectionType,
          },
        },
      })
      const payload = {
        ...(projectionState?.stateJson ?? {}),
        storyMode: {
          enabled: storyModeEnabled,
          stories: buildSelectionPayload(storySelections),
        },
      }
      await saveState({ variables: { id: projectionId.toString(), state: payload } })
      showSuccessNotification('Projection saved', 'Projection properties updated.')
      await refetchProjection()
    } catch (err: any) {
      showErrorNotification('Save failed', err?.message || 'Unable to save projection')
    }
  }

  const handleVerify = async () => {
    if (!projectionId) return
    const selection = buildSelectionPayload(storySelections)
    if (!storyModeEnabled || selection.length === 0) {
      showErrorNotification('Nothing to verify', 'Enable story mode and select at least one story.')
      return
    }
    try {
      const { data } = await verifyStories({
        variables: { projectionId: projectionId, stories: selection },
      })
      const result = (data as any)?.verifyProjectionStoryMatch
      if (!result) {
        showErrorNotification('Verification failed', 'No result returned')
        return
      }
      const missing = (result.sequences as any[]) ?? []
      setVerificationResult({ success: result.success, missing })
      if (result.success || missing.length === 0) {
        showSuccessNotification('Verified', 'All selected stories match this projection graph.')
      } else {
        showErrorNotification('Mismatch found', 'Some sequences reference edges not in this projection.')
      }
    } catch (err: any) {
      showErrorNotification('Verification failed', err?.message || 'Unable to verify stories')
    }
  }

  const busy = loadingProjection || loadingStories || creatingProjection

  if (!projectionId && !resolvedGraphDataId) {
    const hasUpstreamHint = !!graphIdHint || !!graphSourceNodeIdHint
    return (
      <div className="text-sm text-muted-foreground flex items-center gap-2">
        {hasUpstreamHint || graphLookupLoading ? (
          <>
            <Spinner className="h-4 w-4" />
            <span>Preparing projection from connected graph…</span>
          </>
        ) : (
          <span>
            Connect this projection node to a graph first so a projection can be created automatically.
          </span>
        )}
      </div>
    )
  }

  return (
    <Stack gap="md">
      {(busy || updatingProjection || savingState) && (
        <Group gap="xs" align="center" className="text-sm text-muted-foreground">
          <Spinner className="h-4 w-4" />
          <span>Loading projection editor…</span>
        </Group>
      )}

      <Card>
        <CardHeader>
          <CardTitle>Projection details</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="space-y-2">
            <Label htmlFor="projection-name">Name</Label>
            <Input
              id="projection-name"
              value={projectionName}
              onChange={(e) => setProjectionName(e.target.value)}
              placeholder="Projection name"
              disabled={busy}
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="projection-type">Projection Type</Label>
            <Select
              value={projectionType}
              onValueChange={(value) => setProjectionType(value as 'force3d' | 'layer3d')}
              disabled={busy}
            >
              <SelectTrigger id="projection-type">
                <SelectValue placeholder="Select type" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="force3d">Force 3D</SelectItem>
                <SelectItem value="layer3d">Layer 3D</SelectItem>
              </SelectContent>
            </Select>
            <p className="text-xs text-muted-foreground">Projection nodes are created with a dedicated projection entity and inherit their input graph.</p>
          </div>
          <Group justify="end" gap="sm">
            <Button variant="outline" onClick={() => refetchProjection()} disabled={busy}>
              Refresh
            </Button>
            <Button onClick={handleSave} disabled={busy || !projectionId}>
              {(savingState || updatingProjection) && <Spinner className="mr-2 h-4 w-4" />}
              Save
            </Button>
          </Group>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Story mode</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <Group justify="between" align="center">
            <div>
              <div className="font-semibold">Enable story mode</div>
              <div className="text-muted-foreground text-sm">Toggle to attach stories to this projection.</div>
            </div>
            <Switch checked={storyModeEnabled} onCheckedChange={setStoryModeEnabled} disabled={busy} />
          </Group>

          <div className="rounded border bg-muted/20 p-3">
            <Group justify="between" align="center" className="mb-3">
              <div>
                <div className="font-semibold">Add stories</div>
                <div className="text-muted-foreground text-sm">
                  Select stories and enable individual sequences to include them.
                </div>
              </div>
              <Group gap="sm">
                <Button variant="outline" onClick={handleVerify} disabled={verifying || busy}>
                  {verifying && <Spinner className="mr-2 h-4 w-4" />}
                  Verify story match
                </Button>
              </Group>
            </Group>
            <div className="space-y-3">
              {stories.length === 0 && (
                <p className="text-sm text-muted-foreground">No stories in this project.</p>
              )}
              {stories.map((story: any) => {
                const selection = storySelections[story.id]
                const enabled = selection?.enabled ?? false
                const sequences = sequenceCache[story.id] ?? []
                return (
                  <div key={story.id} className="rounded border bg-background p-3 shadow-sm">
                    <Group justify="between" align="center">
                      <div>
                        <div className="font-semibold">{story.name}</div>
                        <div className="text-muted-foreground text-xs">Story #{story.id}</div>
                      </div>
                      <Group gap="sm" align="center">
                        <span className="text-xs text-muted-foreground">Enable</span>
                        <Switch
                          checked={enabled}
                          onCheckedChange={(v) => handleStoryToggle(story.id, v)}
                          disabled={!storyModeEnabled || busy}
                        />
                      </Group>
                    </Group>
                    {enabled && (
                      <div className="mt-3 space-y-2">
                        <div className="text-xs uppercase text-muted-foreground">Sequences</div>
                        {loadingSeq && sequences.length === 0 ? (
                          <Group gap="xs" align="center">
                            <Spinner className="h-3 w-3" />
                            <span className="text-xs text-muted-foreground">Loading sequences…</span>
                          </Group>
                        ) : sequences.length === 0 ? (
                          <div className="text-sm text-muted-foreground">No sequences for this story.</div>
                        ) : (
                          <div className="grid gap-2 sm:grid-cols-2">
                            {sequences.map((seq) => {
                              const checked = selection?.sequences?.[seq.id] ?? true
                              return (
                                <label
                                  key={seq.id}
                                  className="flex items-center gap-2 rounded border p-2 text-sm"
                                >
                                  <Checkbox
                                    checked={checked}
                                    onCheckedChange={(v) => handleSequenceToggle(story.id, seq.id, Boolean(v))}
                                  />
                                  <div className="flex flex-col">
                                    <span className="font-medium">{seq.name}</span>
                                    <span className="text-[11px] text-muted-foreground">Sequence #{seq.id}</span>
                                  </div>
                                </label>
                              )
                            })}
                          </div>
                        )}
                      </div>
                    )}
                  </div>
                )
              })}
            </div>
            {verificationResult && (
              <div className="mt-4 rounded border bg-background p-3">
                {verificationResult.success && verificationResult.missing.length === 0 ? (
                  <div className="text-sm text-emerald-500">All selected stories match this projection.</div>
                ) : (
                  <div className="space-y-2">
                    <div className="text-sm font-semibold text-destructive">Missing edges detected</div>
                    {verificationResult.missing.map((seq) => (
                      <div key={`${seq.storyId}-${seq.sequenceId}`} className="text-sm">
                        <div className="mb-1 flex items-center gap-2">
                          <Badge variant="outline">Story {seq.storyId}</Badge>
                          <Badge variant="secondary">Sequence {seq.sequenceId}</Badge>
                        </div>
                        <div className="ml-1 text-xs text-muted-foreground">
                          Missing edges:{' '}
                          {seq.missingEdges.map((e: any) => `${e.datasetId}:${e.edgeId}`).join(', ')}
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )}
          </div>
        </CardContent>
      </Card>
    </Stack>
  )
}
