import React from 'react';
import { useMantineColorScheme } from '@mantine/core'; // TODO: Replace with next-themes in Stage 8
import { IconSun, IconMoon, IconWifi, IconWifiOff, IconGraph, IconLoader, IconRefresh } from '@tabler/icons-react';
import { useApolloClient } from '@apollo/client/react';
import { Group } from '@/components/layout-primitives';
import { Button } from '@/components/ui/button';
import { UserPresenceIndicator } from '../collaboration/UserPresenceIndicator';
import { ConnectionState } from '../../types/websocket';

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
  const { colorScheme, toggleColorScheme } = useMantineColorScheme();
  const apolloClient = useApolloClient();
  const isDark = colorScheme === 'dark';
  const isOnline = connectionState === ConnectionState.CONNECTED;
  const isConnecting = connectionState === ConnectionState.CONNECTING;

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
          onClick={() => toggleColorScheme()}
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