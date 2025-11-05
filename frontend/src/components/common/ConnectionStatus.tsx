import React from 'react'
import { Badge } from '@/components/ui/badge'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { Spinner } from '@/components/ui/spinner'
import { IconWifi, IconWifiOff } from '@tabler/icons-react'
import { useQuery } from '@apollo/client/react'
import { gql } from '@apollo/client'

// Health check query to verify backend connectivity
const HEALTH_CHECK = gql`
  query HealthCheck {
    projects {
      id
      name
    }
  }
`

export const ConnectionStatus: React.FC = () => {
  // Health check query with polling to monitor backend connectivity
  const { loading, error } = useQuery(HEALTH_CHECK, {
    pollInterval: 5000, // Poll every 5 seconds
    errorPolicy: 'all', // Show partial data even with errors
  })

  const getStatusConfig = () => {
    if (loading) {
      return {
        variant: 'secondary' as const,
        className: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200',
        icon: <Spinner size="xs" />,
        label: 'Connecting...',
        tooltip: 'Connecting to backend server',
      }
    }

    if (error) {
      return {
        variant: 'destructive' as const,
        className: '',
        icon: <IconWifiOff size={12} />,
        label: 'Offline',
        tooltip: `Backend connection failed: ${error.message}`,
      }
    }

    return {
      variant: 'secondary' as const,
      className: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
      icon: <IconWifi size={12} />,
      label: 'Online',
      tooltip: 'Connected to backend server',
    }
  }

  const status = getStatusConfig()

  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <Badge variant={status.variant} className={status.className}>
            <span className="flex items-center gap-1.5">
              {status.icon}
              <span className="text-xs">{status.label}</span>
            </span>
          </Badge>
        </TooltipTrigger>
        <TooltipContent>
          <p>{status.tooltip}</p>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  )
}