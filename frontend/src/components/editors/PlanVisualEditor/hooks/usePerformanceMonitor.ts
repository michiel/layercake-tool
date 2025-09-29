import { useState, useRef, useCallback, useEffect } from 'react'

interface PerformanceMetrics {
  renderCount: number
  averageRenderTime: number
  lastRenderTime: number
  eventCounts: {
    nodeChanges: number
    edgeChanges: number
    validations: number
    websocketMessages: number
    positionUpdates: number
  }
  memoryUsage: {
    jsHeapSizeLimit: number
    totalJSHeapSize: number
    usedJSHeapSize: number
  } | null
  performanceBudgets: {
    maxRenderTime: number
    maxRendersPerSecond: number
    maxEventFrequency: number
  }
  violations: string[]
}

interface PerformanceViolation {
  type: 'render_time' | 'render_frequency' | 'event_frequency' | 'memory_usage'
  message: string
  value: number
  threshold: number
  timestamp: Date
}

interface UsePerformanceMonitorOptions {
  enabled?: boolean
  maxRenderTime?: number
  maxRendersPerSecond?: number
  maxEventFrequency?: number
  memoryWarningThreshold?: number // MB
}

export const usePerformanceMonitor = (options: UsePerformanceMonitorOptions = {}) => {
  const {
    enabled = true,
    maxRenderTime = 16, // 16ms for 60fps
    maxRendersPerSecond = 60,
    maxEventFrequency = 10, // events per second
    memoryWarningThreshold = 100 // MB
  } = options

  const [metrics, setMetrics] = useState<PerformanceMetrics>({
    renderCount: 0,
    averageRenderTime: 0,
    lastRenderTime: 0,
    eventCounts: {
      nodeChanges: 0,
      edgeChanges: 0,
      validations: 0,
      websocketMessages: 0,
      positionUpdates: 0,
    },
    memoryUsage: null,
    performanceBudgets: {
      maxRenderTime,
      maxRendersPerSecond,
      maxEventFrequency,
    },
    violations: [],
  })

  const renderTimesRef = useRef<number[]>([])
  const renderCountRef = useRef(0)
  const lastRenderTimeRef = useRef(0)
  const eventTimestampsRef = useRef<Record<string, Date[]>>({
    nodeChanges: [],
    edgeChanges: [],
    validations: [],
    websocketMessages: [],
    positionUpdates: [],
  })
  const violationsRef = useRef<PerformanceViolation[]>([])

  // Track component render performance
  const trackRender = useCallback(() => {
    if (!enabled) return

    const renderTime = performance.now() - lastRenderTimeRef.current
    renderCountRef.current++

    // Track render times (keep last 100 renders)
    renderTimesRef.current.push(renderTime)
    if (renderTimesRef.current.length > 100) {
      renderTimesRef.current.shift()
    }

    // Calculate average render time
    const averageRenderTime = renderTimesRef.current.reduce((a, b) => a + b, 0) / renderTimesRef.current.length

    // Check for render time violations
    if (renderTime > maxRenderTime) {
      const violation: PerformanceViolation = {
        type: 'render_time',
        message: `Render time exceeded budget: ${renderTime.toFixed(2)}ms > ${maxRenderTime}ms`,
        value: renderTime,
        threshold: maxRenderTime,
        timestamp: new Date(),
      }
      violationsRef.current.push(violation)
      console.warn('Performance violation:', violation.message)
    }

    // Check for render frequency violations
    const now = Date.now()
    const oneSecondAgo = now - 1000
    const recentRenders = renderTimesRef.current.filter((_, index) => {
      const renderTimestamp = now - (renderTimesRef.current.length - 1 - index) * 16 // Approximate
      return renderTimestamp > oneSecondAgo
    })

    if (recentRenders.length > maxRendersPerSecond) {
      const violation: PerformanceViolation = {
        type: 'render_frequency',
        message: `Render frequency exceeded budget: ${recentRenders.length} renders/sec > ${maxRendersPerSecond}`,
        value: recentRenders.length,
        threshold: maxRendersPerSecond,
        timestamp: new Date(),
      }
      violationsRef.current.push(violation)
      console.warn('Performance violation:', violation.message)
    }

    // Update metrics
    setMetrics(prev => ({
      ...prev,
      renderCount: renderCountRef.current,
      averageRenderTime,
      lastRenderTime: renderTime,
      violations: violationsRef.current.slice(-10).map(v => v.message), // Keep last 10 violations
    }))

    lastRenderTimeRef.current = performance.now()
  }, [enabled, maxRenderTime, maxRendersPerSecond])

  // Track specific events
  const trackEvent = useCallback((eventType: keyof PerformanceMetrics['eventCounts']) => {
    if (!enabled) return

    const now = new Date()
    eventTimestampsRef.current[eventType].push(now)

    // Clean old timestamps (older than 1 second)
    const oneSecondAgo = new Date(now.getTime() - 1000)
    eventTimestampsRef.current[eventType] = eventTimestampsRef.current[eventType].filter(
      timestamp => timestamp > oneSecondAgo
    )

    // Check for event frequency violations
    const eventCount = eventTimestampsRef.current[eventType].length
    if (eventCount > maxEventFrequency) {
      const violation: PerformanceViolation = {
        type: 'event_frequency',
        message: `${eventType} frequency exceeded budget: ${eventCount} events/sec > ${maxEventFrequency}`,
        value: eventCount,
        threshold: maxEventFrequency,
        timestamp: now,
      }
      violationsRef.current.push(violation)
      console.warn('Performance violation:', violation.message)
    }

    // Update event counts
    setMetrics(prev => ({
      ...prev,
      eventCounts: {
        ...prev.eventCounts,
        [eventType]: prev.eventCounts[eventType] + 1,
      },
    }))
  }, [enabled, maxEventFrequency])

  // Monitor memory usage
  const updateMemoryUsage = useCallback(() => {
    if (!enabled || !(performance as any).memory) return

    const memory = (performance as any).memory
    const memoryUsage = {
      jsHeapSizeLimit: memory.jsHeapSizeLimit,
      totalJSHeapSize: memory.totalJSHeapSize,
      usedJSHeapSize: memory.usedJSHeapSize,
    }

    // Check for memory usage violations
    const usedMB = memoryUsage.usedJSHeapSize / 1024 / 1024
    if (usedMB > memoryWarningThreshold) {
      const violation: PerformanceViolation = {
        type: 'memory_usage',
        message: `Memory usage exceeded threshold: ${usedMB.toFixed(1)}MB > ${memoryWarningThreshold}MB`,
        value: usedMB,
        threshold: memoryWarningThreshold,
        timestamp: new Date(),
      }
      violationsRef.current.push(violation)
      console.warn('Performance violation:', violation.message)
    }

    setMetrics(prev => ({
      ...prev,
      memoryUsage,
    }))
  }, [enabled, memoryWarningThreshold])

  // Reset all metrics
  const resetMetrics = useCallback(() => {
    renderTimesRef.current = []
    renderCountRef.current = 0
    eventTimestampsRef.current = {
      nodeChanges: [],
      edgeChanges: [],
      validations: [],
      websocketMessages: [],
      positionUpdates: [],
    }
    violationsRef.current = []

    setMetrics({
      renderCount: 0,
      averageRenderTime: 0,
      lastRenderTime: 0,
      eventCounts: {
        nodeChanges: 0,
        edgeChanges: 0,
        validations: 0,
        websocketMessages: 0,
        positionUpdates: 0,
      },
      memoryUsage: null,
      performanceBudgets: {
        maxRenderTime,
        maxRendersPerSecond,
        maxEventFrequency,
      },
      violations: [],
    })
  }, [maxRenderTime, maxRendersPerSecond, maxEventFrequency])

  // Get performance summary
  const getPerformanceSummary = useCallback(() => {
    const recentViolations = violationsRef.current.filter(
      v => v.timestamp.getTime() > Date.now() - 60000 // Last minute
    )

    return {
      isHealthy: recentViolations.length === 0,
      recentViolations: recentViolations.length,
      averageRenderTime: metrics.averageRenderTime,
      totalRenders: metrics.renderCount,
      memoryUsageMB: metrics.memoryUsage ? metrics.memoryUsage.usedJSHeapSize / 1024 / 1024 : 0,
      recommendations: generateRecommendations(recentViolations, metrics),
    }
  }, [metrics])

  // Periodic memory monitoring
  useEffect(() => {
    if (!enabled) return

    const interval = setInterval(updateMemoryUsage, 5000) // Every 5 seconds
    return () => clearInterval(interval)
  }, [enabled, updateMemoryUsage])

  // Track renders automatically
  useEffect(() => {
    if (enabled) {
      trackRender()
    }
  })

  return {
    // Metrics
    metrics,

    // Tracking functions
    trackRender,
    trackEvent,
    updateMemoryUsage,

    // Control functions
    resetMetrics,
    getPerformanceSummary,

    // Configuration
    isEnabled: enabled,
  }
}

// Generate performance recommendations based on violations and metrics
const generateRecommendations = (
  violations: PerformanceViolation[],
  metrics: PerformanceMetrics
): string[] => {
  const recommendations: string[] = []

  const renderViolations = violations.filter(v => v.type === 'render_time')
  const frequencyViolations = violations.filter(v => v.type === 'render_frequency')
  const eventViolations = violations.filter(v => v.type === 'event_frequency')
  const memoryViolations = violations.filter(v => v.type === 'memory_usage')

  if (renderViolations.length > 0) {
    recommendations.push('Consider optimising component renders with React.memo or useMemo')
    recommendations.push('Reduce complex calculations in render functions')
  }

  if (frequencyViolations.length > 0) {
    recommendations.push('Implement proper event debouncing and throttling')
    recommendations.push('Use stable references for callback functions')
  }

  if (eventViolations.length > 0) {
    recommendations.push('Reduce event frequency with better throttling')
    recommendations.push('Batch similar events to reduce processing overhead')
  }

  if (memoryViolations.length > 0) {
    recommendations.push('Check for memory leaks in event listeners or subscriptions')
    recommendations.push('Clear unused references and optimize data structures')
  }

  if (metrics.averageRenderTime > 10) {
    recommendations.push('Render time is above optimal - consider component optimisation')
  }

  return recommendations
}