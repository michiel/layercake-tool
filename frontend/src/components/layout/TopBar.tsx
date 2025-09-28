import React from 'react';
import { Group, Text, ActionIcon, useMantineColorScheme } from '@mantine/core';
import { IconSun, IconMoon, IconWifi, IconWifiOff, IconGraph } from '@tabler/icons-react';
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
  const isDark = colorScheme === 'dark';
  const isOnline = connectionState === ConnectionState.CONNECTED;

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

        {/* Online status indicator */}
        <ActionIcon
          variant="subtle"
          size="lg"
          color={isOnline ? "green" : "red"}
          title={isOnline ? "Online" : "Offline"}
        >
          {isOnline ? <IconWifi size="1.2rem" /> : <IconWifiOff size="1.2rem" />}
        </ActionIcon>
      </Group>
    </Group>
  );
};