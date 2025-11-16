import React, { useState } from 'react';
import { IconSun, IconMoon, IconWifi, IconWifiOff, IconGraph, IconLoader, IconRefresh, IconX, IconTag } from '@tabler/icons-react';
import { useApolloClient } from '@apollo/client/react';
import { useTheme } from 'next-themes';
import { Group } from '@/components/layout-primitives';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { UserPresenceIndicator } from '../collaboration/UserPresenceIndicator';
import { ConnectionState } from '../../types/websocket';
import { useTagsFilter } from '../../hooks/useTagsFilter';

interface TopBarProps {
  projectId?: number;
  connectionState?: ConnectionState;
  users?: any[]; // Use existing UserPresenceData type
  currentUserId?: string;
  onNavigateHome?: () => void;
}

export const TopBar: React.FC<TopBarProps> = ({
  projectId,
  connectionState = ConnectionState.DISCONNECTED,
  users = [],
  currentUserId,
  onNavigateHome
}) => {
  const { theme, setTheme } = useTheme();
  const apolloClient = useApolloClient();
  const { activeTags, setActiveTags, clearTags } = useTagsFilter();
  const [tagsInput, setTagsInput] = useState('');
  const isDark = theme === 'dark';
  const isOnline = connectionState === ConnectionState.CONNECTED;
  const isConnecting = connectionState === ConnectionState.CONNECTING;

  const handleTagsKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' && tagsInput.trim()) {
      const newTags = tagsInput
        .split(',')
        .map((t) => t.trim().toLowerCase())
        .filter((t) => t.length > 0);
      if (newTags.length > 0) {
        setActiveTags([...new Set([...activeTags, ...newTags])]);
        setTagsInput('');
      }
    }
  };

  const removeTag = (tagToRemove: string) => {
    const normalized = tagToRemove.toLowerCase();
    setActiveTags(activeTags.filter((tag) => tag !== normalized));
  };

  const handleClearCache = () => {
    // Clear Apollo cache
    apolloClient.clearStore().catch((error) => {
      console.error('Failed to clear Apollo cache:', error);
    });

    // Clear localStorage
    localStorage.clear();

    // Clear sessionStorage
    sessionStorage.clear();

    // Reload the page
    window.location.reload();
  };

  return (
    <Group
      className="h-[60px] px-4 border-b border-border"
      justify="between"
    >
      {/* Left side - Logo and title */}
      <Group gap="sm" className="cursor-pointer" onClick={onNavigateHome}>
        <IconGraph size={28} />
        <h1 className="text-2xl font-bold">Layercake</h1>
      </Group>

      {/* Center - Tags filter */}
      <Group gap="sm" className="flex-1 max-w-lg mx-4">
        <div className="flex items-center gap-2 w-full">
          <IconTag size={16} className="text-muted-foreground" />
          <Input
            placeholder="Filter by tags (press Enter)"
            value={tagsInput}
            onChange={(e) => setTagsInput(e.target.value)}
            onKeyDown={handleTagsKeyDown}
            className="h-8"
          />
          {activeTags.length > 0 && (
            <Button
              variant="ghost"
              size="icon"
              onClick={clearTags}
              title="Clear all tags"
              className="h-8 w-8"
            >
              <IconX size={16} />
            </Button>
          )}
        </div>
        {activeTags.length > 0 && (
          <Group gap="xs" className="flex-wrap">
            {activeTags.map((tag) => (
              <Badge key={tag} variant="secondary" className="gap-1">
                {tag}
                <button
                  onClick={() => removeTag(tag)}
                  className="ml-1 hover:text-destructive"
                >
                  <IconX size={12} />
                </button>
              </Badge>
            ))}
          </Group>
        )}
      </Group>

      {/* Right side - Controls */}
      <Group gap="sm">
        {/* User presence indicator (only show if in a project) */}
        {projectId && (
          <UserPresenceIndicator
            users={users}
            connectionState={connectionState}
            currentUserId={currentUserId}
          />
        )}

        {/* Theme toggle */}
        <Button
          variant="ghost"
          size="icon"
          onClick={() => setTheme(isDark ? 'light' : 'dark')}
          title={isDark ? "Switch to Light Mode" : "Switch to Dark Mode"}
        >
          {isDark ? <IconSun size={20} /> : <IconMoon size={20} />}
        </Button>

        {/* Clear cache and reload */}
        <Button
          variant="ghost"
          size="icon"
          onClick={handleClearCache}
          title="Clear cache and reload"
        >
          <IconRefresh size={20} />
        </Button>

        {/* Connection status indicator */}
        <Button
          variant="ghost"
          size="icon"
          className={
            isOnline ? "text-green-600" :
            isConnecting ? "text-yellow-600" :
            "text-red-600"
          }
          title={
            isOnline ? "Connected to backend" :
            isConnecting ? "Connecting to backend..." :
            "Disconnected from backend"
          }
        >
          {isOnline ? (
            <IconWifi size={20} />
          ) : isConnecting ? (
            <IconLoader size={20} className="animate-spin" />
          ) : (
            <IconWifiOff size={20} />
          )}
        </Button>
      </Group>
    </Group>
  );
};
