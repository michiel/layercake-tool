import React from 'react';
import { Group, Text, ActionIcon, useMantineColorScheme } from '@mantine/core';
import { IconSun, IconMoon, IconWifi, IconWifiOff, IconGraph, IconLoader, IconRefresh } from '@tabler/icons-react';
import { useApolloClient } from '@apollo/client/react';
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
    <Group h={60} px="md" justify="space-between"
           style={{ borderBottom: '1px solid var(--mantine-color-gray-3)' }}>
      {/* Left side - Logo and title */}
      <Group gap="sm" style={{ cursor: 'pointer' }} onClick={onNavigateHome}>
        <IconGraph size={28} />
        <Text size="xl" fw={700}>Layercake</Text>
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
        <ActionIcon
          variant="subtle"
          size="lg"
          onClick={() => toggleColorScheme()}
          title={isDark ? "Switch to Light Mode" : "Switch to Dark Mode"}
        >
          {isDark ? <IconSun size="1.2rem" /> : <IconMoon size="1.2rem" />}
        </ActionIcon>

        {/* Clear cache and reload */}
        <ActionIcon
          variant="subtle"
          size="lg"
          onClick={handleClearCache}
          title="Clear cache and reload"
          color="gray"
        >
          <IconRefresh size="1.2rem" />
        </ActionIcon>

        {/* Connection status indicator */}
        <ActionIcon
          variant="subtle"
          size="lg"
          color={isOnline ? "green" : isConnecting ? "yellow" : "red"}
          title={
            isOnline ? "Connected to backend" :
            isConnecting ? "Connecting to backend..." :
            "Disconnected from backend"
          }
        >
          {isOnline ? (
            <IconWifi size="1.2rem" />
          ) : isConnecting ? (
            <div style={{ animation: 'spin 1s linear infinite' }}>
              <IconLoader size="1.2rem" />
              <style>
                {`
                  @keyframes spin {
                    0% { transform: rotate(0deg); }
                    100% { transform: rotate(360deg); }
                  }
                `}
              </style>
            </div>
          ) : (
            <IconWifiOff size="1.2rem" />
          )}
        </ActionIcon>
      </Group>
    </Group>
  );
};