import { memo } from 'react'
import { NodeProps } from 'reactflow'
import { useNavigate } from 'react-router-dom'
import { useQuery } from '@apollo/client/react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { Stack, Group } from '@/components/layout-primitives'
import { IconSettings, IconExternalLink, IconTrash } from '@tabler/icons-react'
import { PlanDagNodeType, StoryNodeConfig } from '../../../../types/plan-dag'
import { BaseNode } from './BaseNode'
import { resolveNodeHandlers } from './nodeHandlers'
import { GET_STORY, Story } from '../../../../graphql/stories'

interface ExtendedNodeProps extends NodeProps {
  onEdit?: (nodeId: string) => void
  onDelete?: (nodeId: string) => void
  readonly?: boolean
}

export const StoryNode = memo((props: ExtendedNodeProps) => {
  const { data, readonly = false } = props
  const { onEdit: resolvedOnEdit, onDelete: resolvedOnDelete } = resolveNodeHandlers(props)
  const navigate = useNavigate()

  const config = data.config as StoryNodeConfig
  const storyId = config?.storyId
  const hasValidConfig = !!storyId && storyId > 0
  const projectId = data.projectId as number | undefined

  // Fetch story details if configured
  const { data: storyData, loading: storyLoading } = useQuery(GET_STORY, {
    variables: { id: storyId },
    skip: !storyId || storyId <= 0,
  })

  const story: Story | undefined = (storyData as any)?.story

  const handleOpenStoryEditor = (e: React.MouseEvent) => {
    e.stopPropagation()
    e.preventDefault()
    if (projectId && storyId) {
      navigate(`/projects/${projectId}/stories/${storyId}`)
    }
  }

  const labelBadges = !hasValidConfig ? (
    <Badge variant="outline" className="text-xs text-orange-600 border-orange-600">
      Not Configured
    </Badge>
  ) : null

  const displayMetadata = story
    ? { ...(data.metadata ?? {}), label: story.name }
    : data.metadata

  const handleActionClick = (event: React.MouseEvent, callback?: () => void) => {
    event.stopPropagation()
    event.preventDefault()
    if (event.button !== 0) return
    callback?.()
  }

  const storyToolButtons = readonly ? null : (
    <TooltipProvider>
      {hasValidConfig ? (
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              size="icon"
              variant="ghost"
              className="h-7 w-7 text-blue-600 hover:text-blue-700 hover:bg-blue-100"
              data-action-icon="open-story"
              onMouseDown={handleOpenStoryEditor}
            >
              <IconExternalLink size={13} />
            </Button>
          </TooltipTrigger>
          <TooltipContent>Open Story Editor</TooltipContent>
        </Tooltip>
      ) : (
        <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  size="icon"
                  variant="ghost"
                  className="h-7 w-7 text-blue-600 hover:text-blue-700 hover:bg-blue-100"
                  data-action-icon="configure-story"
                  onMouseDown={(event) => handleActionClick(event, () => resolvedOnEdit?.(props.id))}
                >
                  <IconSettings size={13} />
                </Button>
              </TooltipTrigger>
          <TooltipContent>Configure Story</TooltipContent>
        </Tooltip>
      )}
      <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  size="icon"
                  variant="ghost"
                  className="h-7 w-7 text-gray-600 hover:text-gray-700 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-700/50"
                  data-action-icon="edit"
                  onMouseDown={(event) => handleActionClick(event, () => resolvedOnEdit?.(props.id))}
                >
                  <IconSettings size={13} />
                </Button>
              </TooltipTrigger>
        <TooltipContent>Edit node</TooltipContent>
      </Tooltip>
      <Tooltip>
            <TooltipTrigger asChild>
              <Button
                size="icon"
                variant="ghost"
                className="h-7 w-7 text-red-600 hover:text-red-700 hover:bg-red-100 dark:text-red-400 dark:hover:bg-red-900/50"
                data-action-icon="delete"
                onMouseDown={(event) => handleActionClick(event, () => resolvedOnDelete?.(props.id))}
              >
                <IconTrash size={13} />
              </Button>
            </TooltipTrigger>
        <TooltipContent>Delete node</TooltipContent>
      </Tooltip>
    </TooltipProvider>
  )

  return (
    <BaseNode
      {...props}
      nodeType={PlanDagNodeType.STORY}
      config={config}
      metadata={displayMetadata}
      onEdit={() => resolvedOnEdit?.(props.id)}
      onDelete={() => resolvedOnDelete?.(props.id)}
      readonly={readonly}
      edges={data.edges || []}
      hasValidConfig={hasValidConfig}
      labelBadges={labelBadges}
      toolButtons={storyToolButtons}
    >
      <Stack gap="xs">
        {storyLoading && (
          <p className="text-xs text-muted-foreground">Loading...</p>
        )}
        {story && (
          <>
            {story.description && (
              <p className="text-xs text-muted-foreground line-clamp-2">
                {story.description}
              </p>
            )}
            <Group gap="xs">
              <Badge variant="secondary" className="text-xs">
                {story.sequenceCount} sequence{story.sequenceCount !== 1 ? 's' : ''}
              </Badge>
            </Group>
          </>
        )}
        {!story && hasValidConfig && !storyLoading && (
          <p className="text-xs text-red-500">Story not found</p>
        )}
      </Stack>
    </BaseNode>
  )
})

StoryNode.displayName = 'StoryNode'
