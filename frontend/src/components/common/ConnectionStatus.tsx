import React from 'react'
import { Badge, Tooltip } from '@mantine/core'
import { IconWifi, IconWifiOff, IconLoader } from '@tabler/icons-react'
import { useQuery } from '@apollo/client/react'
import { gql } from '@apollo/client'

// Mock health check - will be replaced with actual schema
const HEALTH_CHECK = gql`
  query HealthCheck {
    health {
      status
    }
  }
`

export const ConnectionStatus: React.FC = () => {
  // Disabled for frontend-only development
  const { loading, error } = { loading: false, error: null } // useQuery(HEALTH_CHECK, {
  //   pollInterval: 5000, // Poll every 5 seconds
  //   errorPolicy: 'ignore', // Don't crash on connection errors
  // })

  const getStatusConfig = () => {
    if (loading) {
      return {
        color: 'yellow',
        icon: <IconLoader size={12} />,
        label: 'Connecting...',
        tooltip: 'Connecting to backend server',
      }
    }

    if (error) {
      return {
        color: 'red',
        icon: <IconWifiOff size={12} />,
        label: 'Offline',
        tooltip: `Backend connection failed: ${error.message}`,
      }
    }

    return {
      color: 'green',
      icon: <IconWifi size={12} />,
      label: 'Online',
      tooltip: 'Connected to backend server',
    }
  }

  const status = getStatusConfig()

  return (
    <Tooltip label={status.tooltip}>
      <Badge
        color={status.color}
        variant="light"
        leftSection={status.icon}
        size="sm"
      >
        {status.label}
      </Badge>
    </Tooltip>
  )
}