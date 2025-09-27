import { useState, useCallback, useRef } from 'react'

interface UseUpdateManagementOptions {
  throttleMs?: number
  debounceMs?: number
  maxPendingUpdates?: number
}

export const useUpdateManagement = (options: UseUpdateManagementOptions = {}) => {
  const {
    throttleMs = 1000,
    debounceMs = 500,
    maxPendingUpdates = 10
  } = options

  const [updatesPaused, setUpdatesPaused] = useState(false)
  const [pendingUpdates, setPendingUpdates] = useState(0)
  const lastUpdateTimeRef = useRef<number>(0)
  const updateThrottleRef = useRef<number | null>(null)
  const updateDebounceRef = useRef<number | null>(null)

  const throttledUpdate = useCallback((updateFn: () => void) => {
    const now = Date.now()
    const timeSinceLastUpdate = now - lastUpdateTimeRef.current

    if (updatesPaused) {
      setPendingUpdates(prev => Math.min(prev + 1, maxPendingUpdates))
      return
    }

    if (timeSinceLastUpdate < throttleMs) {
      // Throttle: delay update until minimum time has passed
      if (updateThrottleRef.current) {
        clearTimeout(updateThrottleRef.current)
      }

      const delay = throttleMs - timeSinceLastUpdate
      updateThrottleRef.current = setTimeout(() => {
        lastUpdateTimeRef.current = Date.now()
        updateFn()
      }, delay)
    } else {
      // Can update immediately
      lastUpdateTimeRef.current = now
      updateFn()
    }
  }, [updatesPaused, throttleMs, maxPendingUpdates])

  const debouncedUpdate = useCallback((updateFn: () => void) => {
    if (updateDebounceRef.current) {
      clearTimeout(updateDebounceRef.current)
    }

    updateDebounceRef.current = setTimeout(() => {
      if (!updatesPaused) {
        updateFn()
      } else {
        setPendingUpdates(prev => Math.min(prev + 1, maxPendingUpdates))
      }
    }, debounceMs)
  }, [updatesPaused, debounceMs, maxPendingUpdates])

  const pauseUpdates = useCallback(() => {
    setUpdatesPaused(true)
  }, [])

  const resumeUpdates = useCallback(() => {
    setUpdatesPaused(false)
    setPendingUpdates(0)
  }, [pendingUpdates])

  // Cleanup timeouts
  const cleanup = useCallback(() => {
    if (updateThrottleRef.current) {
      clearTimeout(updateThrottleRef.current)
    }
    if (updateDebounceRef.current) {
      clearTimeout(updateDebounceRef.current)
    }
  }, [])

  return {
    updatesPaused,
    pendingUpdates,
    throttledUpdate,
    debouncedUpdate,
    pauseUpdates,
    resumeUpdates,
    cleanup
  }
}