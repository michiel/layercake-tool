import { useCallback, useEffect, useMemo, useState } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { gql } from '@apollo/client'
import { useLazyQuery, useMutation, useQuery } from '@apollo/client/react'
import PageContainer from '@/components/layout/PageContainer'
import { Breadcrumbs } from '@/components/common/Breadcrumbs'
import { Group, Stack } from '@/components/layout-primitives'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Spinner } from '@/components/ui/spinner'
import { Switch } from '@/components/ui/switch'
import { Checkbox } from '@/components/ui/checkbox'
import { Badge } from '@/components/ui/badge'
import { showErrorNotification, showSuccessNotification } from '@/utils/notifications'
import { createApolloClientForEndpoint } from '@/graphql/client'
import { LIST_STORIES } from '@/graphql/stories'
import { LIST_SEQUENCES } from '@/graphql/sequences'

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

const buildSelectionPayload = (selections: Record<number, StorySelection>) =>
  Object.entries(selections)
    .filter(([, sel]) => sel.enabled)
    .map(([storyId, sel]) => ({
      storyId: Number(storyId),
      enabledSequenceIds: Object.entries(sel.sequences)
        .filter(([, enabled]) => enabled)
        .map(([seqId]) => Number(seqId)),
    }))

export const ProjectionEditPage = () => {
  const navigate = useNavigate()
  const { projectId, projectionId } = useParams<{ projectId: string; projectionId: string }>()
  const projectIdNum = Number(projectId || 0)
  const projectionIdNum = Number(projectionId || 0)

  const projectionsClient = useMemo(
    () =>
      createApolloClientForEndpoint({
        httpPath: '/projections/graphql',
        wsPath: '/projections/graphql/ws',
      }),
    []
  )

  const { data: projectionData, loading: loadingProjection } = useQuery(PROJECTION_EDIT_QUERY, {
    variables: { id: projectionId },
    skip: !projectionId,
    client: projectionsClient,
    fetchPolicy: 'cache-and-network',
  })

  const { data: storiesData, loading: loadingStories } = useQuery(LIST_STORIES, {
    variables: { projectId: projectIdNum },
    skip: !projectIdNum,
  })

  const [loadSequences, { loading: loadingSeq }] = useLazyQuery(LIST_SEQUENCES, {
    fetchPolicy: 'cache-first',
  })

  const [saveState, { loading: saving }] = useMutation(SAVE_PROJECTION_STATE, {
    client: projectionsClient,
  })
  const [verifyStories, { loading: verifying }] = useMutation(VERIFY_PROJECTION_STORY_MATCH)

  const projection = (projectionData as any)?.projection
  const projectionState = (projectionData as any)?.projectionState
  const stories = (storiesData as any)?.stories ?? []

  const [storyModeEnabled, setStoryModeEnabled] = useState(false)
  const [storySelections, setStorySelections] = useState<Record<number, StorySelection>>({})
  const [sequenceCache, setSequenceCache] = useState<Record<number, SequenceInfo[]>>({})
  const [verificationResult, setVerificationResult] = useState<{
    success: boolean
    missing: { storyId: number; sequenceId: number; missingEdges: { datasetId: number; edgeId: string }[] }[]
  } | null>(null)
  const [hydrated, setHydrated] = useState(false)

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

  // Hydrate from saved state once
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
    const payload = {
      ...(projectionState?.stateJson ?? {}),
      storyMode: {
        enabled: storyModeEnabled,
        stories: buildSelectionPayload(storySelections),
      },
    }
    try {
      await saveState({ variables: { id: projectionId, state: payload } })
      showSuccessNotification('Projection updated', 'Story mode settings saved.')
    } catch (err: any) {
      showErrorNotification('Save failed', err?.message || 'Unable to save projection state')
    }
  }

  const handleVerify = async () => {
    if (!projectionIdNum) return
    const selection = buildSelectionPayload(storySelections)
    if (!storyModeEnabled || selection.length === 0) {
      showErrorNotification('Nothing to verify', 'Enable story mode and select at least one story.')
      return
    }
    try {
      const { data } = await verifyStories({
        variables: { projectionId: projectionIdNum, stories: selection },
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

  if (loadingProjection || loadingStories) {
    return (
      <PageContainer>
        <Group gap="sm" align="center">
          <Spinner className="h-4 w-4" />
          <span>Loading projection...</span>
        </Group>
      </PageContainer>
    )
  }

  if (!projection) {
    return (
      <PageContainer>
        <h1 className="text-2xl font-bold">Projection not found</h1>
      </PageContainer>
    )
  }

  return (
    <PageContainer>
      <Breadcrumbs
        items={[
          { label: 'Projects', href: '/projects' },
          { label: `Project ${projectId}`, href: `/projects/${projectId}/workbench/projections` },
          { label: projection.name, href: `/projects/${projectId}/workbench/projections/${projectionId}/edit` },
        ]}
      />
      <Group justify="between" className="mb-4">
        <div>
          <h1 className="text-3xl font-bold">{projection.name}</h1>
          <p className="text-muted-foreground">
            Projection type: {projection.projectionType} · Graph {projection.graphId}
          </p>
        </div>
        <Group gap="sm">
          <Button variant="outline" onClick={() => navigate(-1)}>
            Back
          </Button>
          <Button onClick={handleSave} disabled={saving}>
            {saving && <Spinner className="mr-2 h-4 w-4" />}
            Save
          </Button>
        </Group>
      </Group>

      <Stack gap="md">
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
              <Switch checked={storyModeEnabled} onCheckedChange={setStoryModeEnabled} />
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
                  <Button variant="outline" onClick={handleVerify} disabled={verifying}>
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
                            disabled={!storyModeEnabled}
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
    </PageContainer>
  )
}

export default ProjectionEditPage
