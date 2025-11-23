import { memo } from 'react'
import { NodeProps } from 'reactflow'
import { useNavigate } from 'react-router-dom'
import { useQuery } from '@apollo/client/react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { Stack, Group } from '@/components/layout-primitives'
import { IconSettings, IconExternalLink } from '@tabler/icons-react'
import { PlanDagNodeType, StoryNodeConfig } from '../../../../types/plan-dag'
import { BaseNode } from './BaseNode'
import { GET_STORY, Story } from '../../../../graphql/stories'

interface ExtendedNodeProps extends NodeProps {
  onEdit?: (nodeId: string) => void
  onDelete?: (nodeId: string) => void
  readonly?: boolean
}

export const StoryNode = memo((props: ExtendedNodeProps) => {
  const { data, onEdit, onDelete, readonly = false } = props
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

  const customToolButtons = !readonly && (
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
              className="h-7 w-7 text-gray-600 hover:text-gray-700 hover:bg-gray-100"
              data-action-icon="edit"
              onMouseDown={(e) => {
                e.stopPropagation()
                e.preventDefault()
                onEdit?.(props.id)
              }}
            >
              <IconSettings size={13} />
            </Button>
          </TooltipTrigger>
          <TooltipContent>Configure Story</TooltipContent>
        </Tooltip>
      )}
    </TooltipProvider>
  )

  return (
    <BaseNode
      {...props}
      nodeType={PlanDagNodeType.STORY}
      config={config}
      metadata={displayMetadata}
      onEdit={() => onEdit?.(props.id)}
      onDelete={() => onDelete?.(props.id)}
      readonly={readonly}
      edges={data.edges || []}
      hasValidConfig={hasValidConfig}
      labelBadges={labelBadges}
      toolButtons={customToolButtons}
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
