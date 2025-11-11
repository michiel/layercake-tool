import {
  memo,
  ReactNode,
  useState,
  useCallback,
  useRef,
  type MouseEvent as ReactMouseEvent,
  type TouchEvent as ReactTouchEvent
} from 'react'
import { Handle, Position, NodeProps } from 'reactflow'
import { IconSettings, IconTrash, IconCheck, IconX, IconArrowRight } from '@tabler/icons-react'
import { PlanDagNodeType, NodeConfig, NodeMetadata } from '../../../../types/plan-dag'
import { getRequiredInputCount, canHaveMultipleOutputs, isNodeConfigured } from '../../../../utils/planDagValidation'
import { getNodeColor, getNodeIcon } from '../../../../utils/nodeStyles'
import { Group } from '@/components/layout-primitives'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'

interface BaseNodeProps extends NodeProps {
  nodeType: PlanDagNodeType
  config: NodeConfig
  metadata: NodeMetadata
  onEdit?: () => void
  onDelete?: () => void
  onLabelChange?: (newLabel: string) => void
  readonly?: boolean
  edges?: any[]
  hasValidConfig?: boolean
  children?: ReactNode
  toolButtons?: ReactNode
  labelBadges?: ReactNode
  footerContent?: ReactNode
  editableLabel?: boolean
}

// Helper function to map Mantine badge colors to shadcn badge classes
const getBadgeClasses = (color: string, variant: 'filled' | 'light' | 'outline') => {
  const colorMap: Record<string, { filled: string; light: string; outline: string }> = {
    orange: {
      filled: 'bg-orange-600 text-white border-orange-600',
      light: 'bg-orange-100 text-orange-800 border-orange-200 dark:bg-orange-900/50 dark:text-orange-300 dark:border-orange-800',
      outline: 'text-orange-600 border-orange-600 dark:text-orange-400 dark:border-orange-500',
    },
    red: {
      filled: 'bg-red-600 text-white border-red-600',
      light: 'bg-red-100 text-red-800 border-red-200 dark:bg-red-900/50 dark:text-red-300 dark:border-red-800',
      outline: 'text-red-600 border-red-600 dark:text-red-400 dark:border-red-500',
    },
    blue: {
      filled: 'bg-blue-600 text-white border-blue-600',
      light: 'bg-blue-100 text-blue-800 border-blue-200 dark:bg-blue-900/50 dark:text-blue-300 dark:border-blue-800',
      outline: 'text-blue-600 border-blue-600 dark:text-blue-400 dark:border-blue-500',
    },
    green: {
      filled: 'bg-green-600 text-white border-green-600',
      light: 'bg-green-100 text-green-800 border-green-200 dark:bg-green-900/50 dark:text-green-300 dark:border-green-800',
      outline: 'text-green-600 border-green-600 dark:text-green-400 dark:border-green-500',
    },
    gray: {
      filled: 'bg-gray-600 text-white border-gray-600',
      light: 'bg-gray-100 text-gray-800 border-gray-200 dark:bg-gray-700/50 dark:text-gray-300 dark:border-gray-600',
      outline: 'text-gray-600 border-gray-600 dark:text-gray-400 dark:border-gray-500',
    },
  }
  return colorMap[color]?.[variant] || colorMap.gray[variant]
}

export const BaseNode = memo(({
  nodeType,
  metadata,
  selected,
  onEdit,
  onDelete,
  onLabelChange,
  readonly = false,
  edges = [],
  hasValidConfig = true,
  id,
  children,
  toolButtons,
  labelBadges,
  footerContent,
  editableLabel = false,
  sourcePosition
}: BaseNodeProps) => {
  const color = getNodeColor(nodeType)
  const requiredInputs = getRequiredInputCount(nodeType)
  const canHaveOutputs = canHaveMultipleOutputs(nodeType)

  // Check if node is configured
  const isConfigured = isNodeConfigured(nodeType, id || '', edges, hasValidConfig)

  // Label editing state
  const [isEditingLabel, setIsEditingLabel] = useState(false)
  const [labelValue, setLabelValue] = useState(metadata.label || '')
  const [edgeTriggerHovered, setEdgeTriggerHovered] = useState(false)
  const [edgeDropTargetHovered, setEdgeDropTargetHovered] = useState(false)

  const handleRefs = useRef<Record<string, HTMLDivElement | null>>({})

  const registerHandleRef = useCallback(
    (id: string) => (el: HTMLDivElement | null) => {
      if (el) {
        handleRefs.current[id] = el
      } else {
        delete handleRefs.current[id]
      }
    },
    []
  )

  type ConnectionStartEvent = ReactMouseEvent<HTMLDivElement> | ReactTouchEvent<HTMLDivElement>

  const getPointerDetails = useCallback((event: ConnectionStartEvent) => {
    const nativeEvent = event.nativeEvent as MouseEvent | TouchEvent

    if ('touches' in nativeEvent && nativeEvent.touches.length > 0) {
      const touch = nativeEvent.touches[0]
      return {
        clientX: touch.clientX,
        clientY: touch.clientY,
        pointerType: 'touch' as const
      }
    }

    if ('changedTouches' in nativeEvent && nativeEvent.changedTouches.length > 0) {
      const touch = nativeEvent.changedTouches[0]
      return {
        clientX: touch.clientX,
        clientY: touch.clientY,
        pointerType: 'touch' as const
      }
    }

    return {
      clientX: (nativeEvent as MouseEvent).clientX,
      clientY: (nativeEvent as MouseEvent).clientY,
      pointerType: 'mouse' as const
    }
  }, [])

  const dispatchHandlePointerDown = useCallback(
    (handleElement: HTMLDivElement, clientX: number, clientY: number, pointerType: 'mouse' | 'touch') => {
      const commonInit = {
        bubbles: true,
        cancelable: true,
        button: 0,
        buttons: 1,
        clientX,
        clientY
      }

      if (typeof PointerEvent !== 'undefined') {
        try {
          handleElement.dispatchEvent(
            new PointerEvent('pointerdown', {
              ...commonInit,
              pointerId: Date.now(),
              pointerType
            })
          )
        } catch {
          // Safari might not support PointerEvent; fall back to MouseEvent
        }
      }

      handleElement.dispatchEvent(
        new MouseEvent('mousedown', {
          ...commonInit,
          view: window
        })
      )
    },
    []
  )

  const handleStartConnection = useCallback(
    (event: ConnectionStartEvent) => {
      if (readonly) return

      const preferredHandleId = sourcePosition === Position.Bottom ? 'output-bottom' : 'output-right'
      const handleElement =
        handleRefs.current[preferredHandleId] ??
        handleRefs.current['output-right'] ??
        handleRefs.current['output-bottom']

      if (!handleElement) {
        console.warn('[BaseNode] Unable to start connection - no output handle available for node', id)
        return
      }

      event.stopPropagation()
      event.preventDefault()

      const { clientX, clientY, pointerType } = getPointerDetails(event)
      const handleRect = handleElement.getBoundingClientRect()
      const resolvedClientX = Number.isFinite(clientX) ? clientX : handleRect.left + handleRect.width / 2
      const resolvedClientY = Number.isFinite(clientY) ? clientY : handleRect.top + handleRect.height / 2

      dispatchHandlePointerDown(handleElement, resolvedClientX, resolvedClientY, pointerType)
    },
    [readonly, sourcePosition, getPointerDetails, dispatchHandlePointerDown, id]
  )

  const handleLabelSave = () => {
    if (labelValue.trim() && labelValue !== metadata.label) {
      onLabelChange?.(labelValue.trim())
    }
    setIsEditingLabel(false)
  }

  const handleLabelCancel = () => {
    setLabelValue(metadata.label || '')
    setIsEditingLabel(false)
  }

  // Default tool buttons if not provided
  const defaultToolButtons = !readonly && (
    <TooltipProvider>
      {editableLabel && (
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              size="icon"
              variant="ghost"
              className="h-7 w-7 text-blue-600 hover:text-blue-700 hover:bg-blue-100 dark:text-blue-400 dark:hover:bg-blue-900/50"
              data-action-icon="edit-label"
              onMouseDown={(e) => {
                e.stopPropagation()
                e.preventDefault()
                setIsEditingLabel(true)
              }}
            >
              <IconSettings size={13} />
            </Button>
          </TooltipTrigger>
          <TooltipContent>Edit label</TooltipContent>
        </Tooltip>
      )}
      {!editableLabel && (
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              size="icon"
              variant="ghost"
              className="h-7 w-7 text-gray-600 hover:text-gray-700 hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-700/50"
              data-action-icon="edit"
              onMouseDown={(e) => {
                e.stopPropagation()
                e.preventDefault()
                onEdit?.()
              }}
            >
              <IconSettings size={13} />
            </Button>
          </TooltipTrigger>
          <TooltipContent>Edit node</TooltipContent>
        </Tooltip>
      )}
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            size="icon"
            variant="ghost"
            className="h-7 w-7 text-red-600 hover:text-red-700 hover:bg-red-100 dark:text-red-400 dark:hover:bg-red-900/50"
            data-action-icon="delete"
            onMouseDown={(e) => {
              e.stopPropagation()
              e.preventDefault()
              onDelete?.()
            }}
          >
            <IconTrash size={13} />
          </Button>
        </TooltipTrigger>
        <TooltipContent>Delete node</TooltipContent>
      </Tooltip>
    </TooltipProvider>
  )

  // Default label badges if not provided
  const defaultLabelBadges = (
    <>
      {!isConfigured && (
        <Badge variant="outline" className={`text-xs ${getBadgeClasses('orange', 'outline')}`}>
          Not Configured
        </Badge>
      )}
    </>
  )

  return (
    <>
      {/* Input Handles - Hidden for floating edges but functional */}
      {requiredInputs > 0 && (
        <>
          <Handle
            type="target"
            position={Position.Left}
            id="input-left"
            isConnectable={!readonly}
            style={{
              opacity: 0,
              pointerEvents: 'auto',
              width: 36,
              height: 36,
              transform: 'translate(-18px, -50%)',
            }}
          />
          <Handle
            type="target"
            position={Position.Top}
            id="input-top"
            isConnectable={!readonly}
            style={{
              opacity: 0,
              pointerEvents: 'auto',
              width: 36,
              height: 36,
              transform: 'translate(-50%, -18px)',
            }}
          />
        </>
      )}

      {/* Node Content */}
      <div
        className={`${selected ? 'shadow-lg' : 'shadow-sm'} bg-card text-card-foreground rounded-lg flex flex-col relative`}
        style={{
          border: `2px solid ${color}`,
          minWidth: 200,
          maxWidth: 280,
          cursor: 'default',
          pointerEvents: 'all',
        }}
      >
        {requiredInputs > 0 && !readonly && (
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger asChild>
                <div
                  style={{
                    position: 'absolute',
                    top: '50%',
                    left: -14,
                    width: 32,
                    height: 32,
                    transform: 'translateY(-50%)',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    borderRadius: '50%',
                    background: edgeDropTargetHovered ? color : 'hsl(var(--background))',
                    color: edgeDropTargetHovered ? '#fff' : color,
                    border: `1px solid ${color}`,
                    boxShadow: '0 1px 2px rgba(0,0,0,0.05)',
                  }}
                  onMouseEnter={() => setEdgeDropTargetHovered(true)}
                  onMouseLeave={() => setEdgeDropTargetHovered(false)}
                  className="nodrag"
                >
                  <IconArrowRight size={16} style={{ pointerEvents: 'none' }} />
                </div>
              </TooltipTrigger>
              <TooltipContent side="right">Edge drop target</TooltipContent>
            </Tooltip>
          </TooltipProvider>
        )}
        {canHaveOutputs && !readonly && (
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger asChild>
                <div
                  style={{
                    position: 'absolute',
                    top: '50%',
                    right: -14,
                    width: 32,
                    height: 32,
                    transform: 'translateY(-50%)',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    cursor: 'pointer',
                    borderRadius: '50%',
                    background: edgeTriggerHovered ? color : 'hsl(var(--background))',
                    color: edgeTriggerHovered ? '#fff' : color,
                    border: `1px solid ${color}`,
                    boxShadow: '0 1px 2px rgba(0,0,0,0.05)',
                  }}
                  onMouseEnter={() => setEdgeTriggerHovered(true)}
                  onMouseLeave={() => setEdgeTriggerHovered(false)}
                  onMouseDown={(event) => handleStartConnection(event)}
                  onTouchStart={(event) => handleStartConnection(event)}
                  className="nodrag"
                >
                  <IconArrowRight size={16} style={{ pointerEvents: 'none' }} />
                </div>
              </TooltipTrigger>
              <TooltipContent side="left">Create edge</TooltipContent>
            </Tooltip>
          </TooltipProvider>
        )}
        {/* Top Row: Icon and Label */}
        <Group
          gap="sm"
          wrap={false}
          className="node-header p-4 pb-2"
          style={{ cursor: isEditingLabel ? 'default' : 'grab', flex: '0 0 auto' }}
        >
          <div style={{
            color,
            display: 'flex',
            alignItems: 'center',
            flexShrink: 0
          }}>
            {getNodeIcon(nodeType, '1.4rem')}
          </div>
          {isEditingLabel ? (
            <Group gap="xs" style={{ flex: 1, minWidth: 0 }} wrap={false}>
              <Input
                className="h-8 text-sm flex-1"
                value={labelValue}
                onChange={(e) => setLabelValue(e.currentTarget.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') {
                    handleLabelSave()
                  } else if (e.key === 'Escape') {
                    handleLabelCancel()
                  }
                }}
                autoFocus
                onMouseDown={(e) => e.stopPropagation()}
                onClick={(e) => e.stopPropagation()}
              />
              <Button
                size="icon"
                className="h-7 w-7 bg-green-600 hover:bg-green-700"
                onMouseDown={(e) => {
                  e.stopPropagation()
                  e.preventDefault()
                  handleLabelSave()
                }}
              >
                <IconCheck size={13} />
              </Button>
              <Button
                size="icon"
                className="h-7 w-7 bg-red-600 hover:bg-red-700"
                onMouseDown={(e) => {
                  e.stopPropagation()
                  e.preventDefault()
                  handleLabelCancel()
                }}
              >
                <IconX size={13} />
              </Button>
            </Group>
          ) : (
            <p className="text-sm font-semibold line-clamp-2" style={{ wordBreak: 'break-word', flex: 1, minWidth: 0 }}>
              {metadata.label}
            </p>
          )}
        </Group>

        {/* Middle: Node-specific content */}
        {children && (
          <div className="px-4 pb-2" style={{ flex: '1 1 auto' }}>
            {children}
          </div>
        )}

        {/* Bottom Section 1: Labels (narrow horizontal section) */}
        {(labelBadges !== undefined ? labelBadges !== null : !isConfigured) && (
          <div
            className="px-4 py-2 border-t border-border"
            style={{
              flex: '0 0 auto',
            }}
          >
            <Group gap="xs" wrap={true}>
              {labelBadges ?? defaultLabelBadges}
            </Group>
          </div>
        )}

        {/* Bottom Section 2: Tool buttons and footer content (narrow horizontal section) */}
        {(!readonly || footerContent) && (
          <Group
            gap="sm"
            justify="between"
            className="border-t border-border px-4 py-2"
            style={{
              flex: '0 0 auto',
              pointerEvents: 'auto',
              minHeight: 36,
            }}
          >
            {footerContent && <div style={{ flex: 1, minWidth: 0 }}>{footerContent}</div>}
            {!readonly && (
              <>
                <Group gap="xs" wrap={false}>
                  {toolButtons || defaultToolButtons}
                </Group>
                <div style={{ flex: 1 }} />
              </>
            )}
          </Group>
        )}
      </div>

      {/* Output Handles - Hidden for floating edges but functional */}
      {canHaveOutputs && (
        <>
          <Handle
            type="source"
            position={Position.Right}
            id="output-right"
            isConnectable={!readonly}
            ref={registerHandleRef('output-right')}
            className="nodrag"
            style={{
              opacity: 0,
              pointerEvents: 'auto',
              width: 28,
              height: 28,
              transform: 'translate(14px, -50%)',
            }}
          />
          <Handle
            type="source"
            position={Position.Bottom}
            id="output-bottom"
            isConnectable={!readonly}
            ref={registerHandleRef('output-bottom')}
            style={{
              opacity: 0,
              pointerEvents: 'auto',
            }}
          />
        </>
      )}
    </>
  )
}, (prevProps, nextProps) => {
  // PERFORMANCE FIX (Phase 1.3): Custom memo comparison to prevent unnecessary re-renders
  // Return true if props are equal (skip re-render), false if different (allow re-render)

  // Always re-render if these critical props change
  if (prevProps.id !== nextProps.id) return false
  if (prevProps.selected !== nextProps.selected) return false
  if (prevProps.nodeType !== nextProps.nodeType) return false

  // Check if data object reference changed
  // With Phase 1.1 fix, data should be stable during drag
  if (prevProps.data === nextProps.data) {
    // Data reference is same, props are equal
    return true
  }

  // Data reference changed, do deep comparison of important fields
  const prevData = prevProps.data || {}
  const nextData = nextProps.data || {}

  // Compare critical data fields
  if (prevData.isUnconfigured !== nextData.isUnconfigured) return false
  if (prevData.hasValidConfig !== nextData.hasValidConfig) return false
  if (prevData.config !== nextData.config) return false
  if (prevData.metadata !== nextData.metadata) return false

  // Props are effectively equal, skip re-render
  return true
})

BaseNode.displayName = 'BaseNode'
