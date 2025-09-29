import { useState, useCallback, useRef, useEffect } from 'react'
import { PlanDag } from '../../../../types/plan-dag'

interface UpdateOperation {
  id: string
  type: 'structural' | 'cosmetic' | 'transient'
  priority: 'immediate' | 'debounced' | 'throttled'
  operation: () => Promise<void> | void
  timestamp: number
}

interface UseUnifiedUpdateManagerOptions {
  onValidationNeeded?: (planDag: PlanDag) => void
  onPersistenceNeeded?: (planDag: PlanDag) => Promise<void>
  debounceMs?: number
  throttleMs?: number
  maxQueueSize?: number
}

interface UpdateManagerState {
  isProcessing: boolean
  queueSize: number
  lastUpdate: Date | null
  pendingOperations: UpdateOperation[]
}

export const useUnifiedUpdateManager = (options: UseUnifiedUpdateManagerOptions = {}) => {
  const {
    onValidationNeeded,
    onPersistenceNeeded,
    debounceMs = 500,
    throttleMs = 1000,
    maxQueueSize = 20
  } = options

  const [state, setState] = useState<UpdateManagerState>({
    isProcessing: false,
    queueSize: 0,
    lastUpdate: null,
    pendingOperations: []
  })

  const operationQueueRef = useRef<UpdateOperation[]>([])
  const processingRef = useRef(false)
  const timersRef = useRef<{
    debounce: NodeJS.Timeout | null
    throttle: NodeJS.Timeout | null
  }>({ debounce: null, throttle: null })

  // Performance monitoring
  const metricsRef = useRef({
    operationsProcessed: 0,
    averageProcessingTime: 0,
    lastOperationTime: 0
  })

  const clearTimers = useCallback(() => {
    if (timersRef.current.debounce) {
      clearTimeout(timersRef.current.debounce)
      timersRef.current.debounce = null
    }
    if (timersRef.current.throttle) {
      clearTimeout(timersRef.current.throttle)
      timersRef.current.throttle = null
    }
  }, [])

  const updateState = useCallback(() => {
    setState({
      isProcessing: processingRef.current,
      queueSize: operationQueueRef.current.length,
      lastUpdate: new Date(),
      pendingOperations: [...operationQueueRef.current]
    })
  }, [])

  const processQueue = useCallback(async () => {
    if (processingRef.current || operationQueueRef.current.length === 0) {
      return
    }

    processingRef.current = true
    updateState()

    const startTime = performance.now()

    try {
      // Sort operations by priority and timestamp
      const sortedOps = [...operationQueueRef.current].sort((a, b) => {
        const priorityOrder = { immediate: 0, throttled: 1, debounced: 2 }
        const priorityDiff = priorityOrder[a.priority] - priorityOrder[b.priority]
        if (priorityDiff !== 0) return priorityDiff
        return a.timestamp - b.timestamp
      })

      // Process immediate operations first
      const immediateOps = sortedOps.filter(op => op.priority === 'immediate')
      const deferredOps = sortedOps.filter(op => op.priority !== 'immediate')

      // Execute immediate operations
      for (const op of immediateOps) {
        try {
          await op.operation()
          operationQueueRef.current = operationQueueRef.current.filter(o => o.id !== op.id)
          metricsRef.current.operationsProcessed++
        } catch (error) {
          console.error(`Failed to execute operation ${op.id}:`, error)
          // Remove failed operation from queue
          operationQueueRef.current = operationQueueRef.current.filter(o => o.id !== op.id)
        }
      }

      // Execute one deferred operation to prevent queue buildup
      if (deferredOps.length > 0) {
        const nextOp = deferredOps[0]
        try {
          await nextOp.operation()
          operationQueueRef.current = operationQueueRef.current.filter(o => o.id !== nextOp.id)
          metricsRef.current.operationsProcessed++
        } catch (error) {
          console.error(`Failed to execute operation ${nextOp.id}:`, error)
          operationQueueRef.current = operationQueueRef.current.filter(o => o.id !== nextOp.id)
        }
      }

      const processingTime = performance.now() - startTime
      metricsRef.current.averageProcessingTime =
        (metricsRef.current.averageProcessingTime + processingTime) / 2
      metricsRef.current.lastOperationTime = processingTime

    } finally {
      processingRef.current = false
      updateState()

      // Schedule next processing if queue not empty
      if (operationQueueRef.current.length > 0) {
        setTimeout(processQueue, 10) // Small delay to prevent blocking
      }
    }
  }, [updateState])

  const scheduleOperation = useCallback((operation: Omit<UpdateOperation, 'timestamp'>) => {
    const newOperation: UpdateOperation = {
      ...operation,
      timestamp: Date.now()
    }

    // Prevent queue overflow
    if (operationQueueRef.current.length >= maxQueueSize) {
      console.warn('Update queue is full, dropping oldest operation')
      operationQueueRef.current.shift()
    }

    // Remove duplicate operations of the same ID
    operationQueueRef.current = operationQueueRef.current.filter(op => op.id !== operation.id)
    operationQueueRef.current.push(newOperation)

    updateState()

    // Schedule processing based on priority
    switch (operation.priority) {
      case 'immediate':
        processQueue()
        break
      case 'throttled':
        if (!timersRef.current.throttle) {
          timersRef.current.throttle = setTimeout(() => {
            timersRef.current.throttle = null
            processQueue()
          }, throttleMs)
        }
        break
      case 'debounced':
        clearTimers()
        timersRef.current.debounce = setTimeout(() => {
          timersRef.current.debounce = null
          processQueue()
        }, debounceMs)
        break
    }
  }, [processQueue, maxQueueSize, throttleMs, debounceMs, clearTimers, updateState])

  // Specific update methods for different operation types
  const scheduleStructuralUpdate = useCallback((planDag: PlanDag, operationName: string) => {
    scheduleOperation({
      id: `structural-${operationName}-${Date.now()}`,
      type: 'structural',
      priority: 'immediate',
      operation: async () => {
        if (onPersistenceNeeded) {
          await onPersistenceNeeded(planDag)
        }
        if (onValidationNeeded) {
          onValidationNeeded(planDag)
        }
      }
    })
  }, [scheduleOperation, onPersistenceNeeded, onValidationNeeded])

  const scheduleCosmeticUpdate = useCallback((planDag: PlanDag, operationName: string) => {
    scheduleOperation({
      id: `cosmetic-${operationName}`,
      type: 'cosmetic',
      priority: 'debounced',
      operation: async () => {
        if (onPersistenceNeeded) {
          await onPersistenceNeeded(planDag)
        }
      }
    })
  }, [scheduleOperation, onPersistenceNeeded])

  const scheduleTransientUpdate = useCallback((operationName: string, operation: () => void) => {
    scheduleOperation({
      id: `transient-${operationName}`,
      type: 'transient',
      priority: 'throttled',
      operation
    })
  }, [scheduleOperation])

  // Cancel all pending operations
  const cancelPendingOperations = useCallback(() => {
    operationQueueRef.current = []
    clearTimers()
    updateState()
  }, [clearTimers, updateState])

  // Force process queue immediately
  const flushOperations = useCallback(async () => {
    clearTimers()
    await processQueue()
  }, [clearTimers, processQueue])

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      clearTimers()
      operationQueueRef.current = []
    }
  }, [clearTimers])

  return {
    // State
    ...state,

    // Metrics
    metrics: metricsRef.current,

    // Update scheduling methods
    scheduleStructuralUpdate,
    scheduleCosmeticUpdate,
    scheduleTransientUpdate,

    // Control methods
    cancelPendingOperations,
    flushOperations,

    // Raw scheduling for custom operations
    scheduleOperation,
  }
}