import { memo, useMemo, useState, type MouseEvent, type PointerEvent } from 'react'
import { NodeProps } from 'reactflow'
import { useMutation } from '@apollo/client/react'
import { gql } from '@apollo/client'
import { IconPresentation, IconExternalLink, IconDownload } from '@tabler/icons-react'
import { PlanDagNodeType, ProjectionNodeConfig } from '../../../../types/plan-dag'
import { isNodeConfigured } from '../../../../utils/planDagValidation'
import { BaseNode } from './BaseNode'
import { Badge } from '@/components/ui/badge'
import { Stack } from '@/components/layout-primitives'
import { Button } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { showErrorNotification, showSuccessNotification } from '@/utils/notifications'
import { createApolloClientForEndpoint } from '@/graphql/client'
import { getServerInfo } from '@/utils/tauri'

const EXPORT_PROJECTION = gql`
  mutation ExportProjection($id: ID!) {
    exportProjection(id: $id) {
      filename
      contentBase64
    }
  }
`

interface ExtendedNodeProps extends NodeProps {
  onEdit?: (nodeId: string) => void
  onDelete?: (nodeId: string) => void
  readonly?: boolean
}

export const ProjectionNode = memo((props: ExtendedNodeProps) => {
  const { data, onEdit, onDelete, readonly = false } = props

  const config = data.config as ProjectionNodeConfig
  const edges = data.edges || []
  const hasValidConfig = data.hasValidConfig !== false && !!config?.projectionId
  const [downloading, setDownloading] = useState(false)

  const projectionsClient = useMemo(
    () =>
      createApolloClientForEndpoint({
        httpPath: '/projections/graphql',
        wsPath: '/projections/graphql/ws',
      }),
    []
  )

  const [exportProjection] = useMutation(EXPORT_PROJECTION, {
    client: projectionsClient,
  })

  const isConfigured = isNodeConfigured(
    PlanDagNodeType.PROJECTION,
    props.id,
    edges,
    hasValidConfig
  )

  const stopPointerInteraction = (event: PointerEvent<HTMLButtonElement>) => {
    event.stopPropagation()
    event.preventDefault()
  }

  const handleActionClick =
    (action: () => void | Promise<void>) => async (event: MouseEvent<HTMLButtonElement>) => {
      event.stopPropagation()
      event.preventDefault()
      await action()
    }

  const handleOpenProjection = async () => {
    if (!config.projectionId) return
    if ((window as any).__TAURI__) {
      try {
        const serverInfo = await getServerInfo()
        if (!serverInfo) {
          showErrorNotification('Open failed', 'Unable to get server information')
          return
        }
        const url = `${serverInfo.url}/projections/viewer/${config.projectionId}?apiBase=${encodeURIComponent(serverInfo.url)}`
        console.log('Creating Tauri window with URL:', url)
        const { WebviewWindow } = await import('@tauri-apps/api/webviewWindow')
        const label = `projection-${config.projectionId}-${Date.now()}`
        const win = new WebviewWindow(label, {
          url,
          maximized: true,
          title: `Projection #${config.projectionId}`
        })
        win.once('tauri://created', () => {
          console.log('Tauri window created successfully')
          showSuccessNotification('Projection opened', 'New window created.')
        })
        win.once('tauri://error', (e: unknown) => {
          console.error('Tauri window error event:', e)
          const errorMsg = typeof e === 'string' ? e : (e as any)?.message || JSON.stringify(e)
          showErrorNotification('Open failed', errorMsg)
        })
      } catch (err: any) {
        console.error('Failed to open projection window', err)
        showErrorNotification('Open failed', err?.message || 'Unable to open projection viewer')
      }
    } else {
      const apiBase = (import.meta as any).env?.VITE_API_BASE_URL || 'http://localhost:3001'
      const url = `${apiBase.replace(/\/+$/, '')}/projections/viewer/${config.projectionId}`
      window.open(url, '_blank', 'noreferrer')
    }
  }

  const handleDownloadProjection = async () => {
    if (!config.projectionId) return
    setDownloading(true)
    try {
      const { data: exportData } = await exportProjection({ variables: { id: config.projectionId.toString() } })
      const payload = (exportData as any)?.exportProjection
      if (!payload?.contentBase64) {
        throw new Error('No export payload returned')
      }
      const binary = atob(payload.contentBase64)
      const len = binary.length
      const bytes = new Uint8Array(len)
      for (let i = 0; i < len; i += 1) {
        bytes[i] = binary.charCodeAt(i)
      }
      const blob = new Blob([bytes], { type: 'application/zip' })
      const url = URL.createObjectURL(blob)
      const link = document.createElement('a')
      link.href = url
      link.download = payload.filename || `projection-${config.projectionId}.zip`
      document.body.appendChild(link)
      link.click()
      link.remove()
      URL.revokeObjectURL(url)
      showSuccessNotification('Export ready', 'Projection bundle downloaded.')
    } catch (err: any) {
      console.error('Failed to export projection', err)
      showErrorNotification('Export failed', err?.message || 'Unable to export projection')
    } finally {
      setDownloading(false)
    }
  }

  const labelBadges = !isConfigured ? (
    <Badge variant="outline" className="text-xs text-orange-600 border-orange-600">
      Not Configured
    </Badge>
  ) : null

  return (
    <BaseNode
      {...props}
      nodeType={PlanDagNodeType.PROJECTION}
      config={config}
      metadata={data.metadata}
      onEdit={() => onEdit?.(props.id)}
      onDelete={() => onDelete?.(props.id)}
      readonly={readonly}
      edges={edges}
      hasValidConfig={hasValidConfig}
      labelBadges={labelBadges}
      extraToolButtons={
        config.projectionId ? (
          <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    size="icon"
                    variant="ghost"
                    className="h-7 w-7 text-blue-600 hover:text-blue-700 hover:bg-blue-100 nodrag"
                    onPointerDown={stopPointerInteraction}
                    onClick={handleActionClick(handleOpenProjection)}
                  >
                    <IconExternalLink size={13} />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>Open projection</TooltipContent>
              </Tooltip>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    size="icon"
                    variant="ghost"
                    className="h-7 w-7 text-green-600 hover:text-green-700 hover:bg-green-100 nodrag"
                    disabled={downloading}
                    onPointerDown={stopPointerInteraction}
                    onClick={handleActionClick(handleDownloadProjection)}
                  >
                    <IconDownload size={13} />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>Download projection</TooltipContent>
              </Tooltip>
          </TooltipProvider>
        ) : null
      }
    >
      <Stack gap="xs">
        {config.projectionId && (
          <div className="flex items-center gap-2 text-xs text-muted-foreground">
            <IconPresentation size={14} />
            <span>Projection #{config.projectionId}</span>
          </div>
        )}
        {!config.projectionId && (
          <p className="text-xs text-muted-foreground">
            No projection selected
          </p>
        )}
      </Stack>
    </BaseNode>
  )
})

ProjectionNode.displayName = 'ProjectionNode'
