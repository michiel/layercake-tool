import React, { useEffect, useState } from 'react';
import { useQuery, useMutation } from '@apollo/client/react';
import { StoryNodeConfig, NodeMetadata } from '../../../../types/plan-dag';
import { Stack } from '@/components/layout-primitives';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/dialog';
import { Spinner } from '@/components/ui/spinner';
import { IconPlus } from '@tabler/icons-react';
import { LIST_STORIES, CREATE_STORY, Story } from '@/graphql/stories';

interface StoryNodeConfigFormProps {
  config: StoryNodeConfig;
  setConfig: (config: StoryNodeConfig) => void;
  setIsValid: (isValid: boolean) => void;
  projectId: number;
  metadata: NodeMetadata;
  setMetadata: (metadata: NodeMetadata) => void;
}

export const StoryNodeConfigForm: React.FC<StoryNodeConfigFormProps> = ({
  config,
  setConfig,
  setIsValid,
  projectId,
  metadata,
  setMetadata,
}) => {
  const [localConfig, setLocalConfig] = useState<StoryNodeConfig>({
    storyId: config.storyId ?? undefined,
  });
  const [createModalOpen, setCreateModalOpen] = useState(false);
  const [newStoryName, setNewStoryName] = useState('');
  const [newStoryDescription, setNewStoryDescription] = useState('');

  const { data: storiesData, loading: storiesLoading, refetch } = useQuery(LIST_STORIES, {
    variables: { projectId },
    skip: !projectId,
  });
  const stories: Story[] = (storiesData as any)?.stories || [];

  const [createStory, { loading: createLoading }] = useMutation(CREATE_STORY, {
    onCompleted: (data) => {
      const newStory = (data as any)?.createStory;
      if (newStory) {
        setLocalConfig(prev => ({ ...prev, storyId: newStory.id }));
        setMetadata({ ...metadata, label: newStory.name });
        setCreateModalOpen(false);
        setNewStoryName('');
        setNewStoryDescription('');
        refetch();
      }
    },
    onError: (error) => {
      console.error('Failed to create story:', error);
      alert(`Failed to create story: ${error.message}`);
    },
  });

  useEffect(() => {
    setConfig(localConfig);
  }, [localConfig, setConfig]);

  useEffect(() => {
    // Valid when a story is selected
    setIsValid(!!localConfig.storyId && localConfig.storyId > 0);
  }, [localConfig, setIsValid]);

  const handleStoryChange = (value: string) => {
    const storyId = parseInt(value, 10);
    setLocalConfig(prev => ({ ...prev, storyId }));

    // Update the node label to match the story name
    const selectedStory = stories.find(s => s.id === storyId);
    if (selectedStory) {
      setMetadata({ ...metadata, label: selectedStory.name });
    }
  };

  const handleCreateStory = async () => {
    if (!newStoryName.trim()) {
      alert('Please enter a story name');
      return;
    }

    await createStory({
      variables: {
        input: {
          projectId,
          name: newStoryName.trim(),
          description: newStoryDescription.trim() || null,
        },
      },
    });
  };

  return (
    <Stack gap="md">
      <div className="space-y-2">
        <Label htmlFor="story-select">Story</Label>
        {storiesLoading ? (
          <div className="flex items-center gap-2">
            <Spinner className="h-4 w-4" />
            <span className="text-sm text-muted-foreground">Loading stories...</span>
          </div>
        ) : (
          <div className="flex gap-2">
            <Select
              value={localConfig.storyId?.toString() || ''}
              onValueChange={handleStoryChange}
            >
              <SelectTrigger id="story-select" className="flex-1">
                <SelectValue placeholder="Select a story" />
              </SelectTrigger>
              <SelectContent>
                {stories.length === 0 ? (
                  <SelectItem value="none" disabled>
                    No stories available
                  </SelectItem>
                ) : (
                  stories.map((story) => (
                    <SelectItem key={story.id} value={story.id.toString()}>
                      {story.name} ({story.sequenceCount} sequences)
                    </SelectItem>
                  ))
                )}
              </SelectContent>
            </Select>
            <Button
              variant="outline"
              size="icon"
              onClick={() => setCreateModalOpen(true)}
              title="Create new story"
            >
              <IconPlus size={16} />
            </Button>
          </div>
        )}
        <p className="text-sm text-muted-foreground">
          Select an existing story or create a new one. The story provides sequence data to connected nodes.
        </p>
      </div>

      {localConfig.storyId && localConfig.storyId > 0 && (
        <div className="rounded-md border p-3 bg-muted/50">
          {(() => {
            const selectedStory = stories.find(s => s.id === localConfig.storyId);
            if (!selectedStory) return <p className="text-sm text-muted-foreground">Story not found</p>;
            return (
              <Stack gap="xs">
                <p className="font-medium">{selectedStory.name}</p>
                {selectedStory.description && (
                  <p className="text-sm text-muted-foreground">{selectedStory.description}</p>
                )}
                <p className="text-sm text-muted-foreground">
                  {selectedStory.sequenceCount} sequence{selectedStory.sequenceCount !== 1 ? 's' : ''}
                </p>
              </Stack>
            );
          })()}
        </div>
      )}

      {/* Create Story Modal */}
      <Dialog open={createModalOpen} onOpenChange={setCreateModalOpen}>
        <DialogContent className="sm:max-w-[500px]">
          <DialogHeader>
            <DialogTitle>Create Story</DialogTitle>
          </DialogHeader>
          <Stack gap="md" className="py-4">
            <div className="space-y-2">
              <Label htmlFor="new-story-name">Name</Label>
              <Input
                id="new-story-name"
                value={newStoryName}
                onChange={(e) => setNewStoryName(e.target.value)}
                placeholder="Enter story name"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="new-story-description">Description</Label>
              <Textarea
                id="new-story-description"
                value={newStoryDescription}
                onChange={(e) => setNewStoryDescription(e.target.value)}
                placeholder="Optional description"
                rows={3}
              />
            </div>
          </Stack>
          <DialogFooter>
            <Button variant="ghost" onClick={() => setCreateModalOpen(false)} disabled={createLoading}>
              Cancel
            </Button>
            <Button onClick={handleCreateStory} disabled={createLoading || !newStoryName.trim()}>
              {createLoading && <Spinner className="mr-2 h-4 w-4" />}
              Create Story
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </Stack>
  );
};
