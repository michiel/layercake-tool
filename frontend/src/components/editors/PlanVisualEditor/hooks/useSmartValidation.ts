import { useState, useCallback, useRef, useMemo } from 'react'
import { PlanDag } from '../../../../types/plan-dag'
import { usePlanDagValidation } from '../../../../hooks/usePlanDag'

interface UseSmartValidationOptions {
  enabled?: boolean
  debounceMs?: number
  maxValidationRate?: number // Max validations per minute
}

interface ValidationState {
  isValidating: boolean
  lastValidation: Date | null
  validationCount: number
  errors: any[]
  isValid: boolean | null
}

interface StructuralChangeDetection {
  hasStructuralChange: (previous: PlanDag | null, current: PlanDag | null) => boolean
  shouldValidate: (changeType: 'structural' | 'cosmetic' | 'transient') => boolean
}

// Detect if changes are structural (require validation) vs cosmetic (don't require validation)
const createStructuralChangeDetector = (): StructuralChangeDetection => {
  const hasStructuralChange = useCallback((previous: PlanDag | null, current: PlanDag | null): boolean => {
    if (!previous || !current) return true

    // Quick structural checks
    if (previous.nodes.length !== current.nodes.length) return true
    if (previous.edges.length !== current.edges.length) return true

    // Check for node type changes, config changes, or new/removed nodes
    for (let i = 0; i < current.nodes.length; i++) {
      const prevNode = previous.nodes.find(n => n.id === current.nodes[i].id)
      const currNode = current.nodes[i]

      if (!prevNode) return true // New node
      if (prevNode.nodeType !== currNode.nodeType) return true // Type change
      if (JSON.stringify(prevNode.config) !== JSON.stringify(currNode.config)) return true // Config change
    }

    // Check for edge changes (any edge change is structural)
    for (let i = 0; i < current.edges.length; i++) {
      const prevEdge = previous.edges.find(e => e.id === current.edges[i].id)
      const currEdge = current.edges[i]

      if (!prevEdge) return true // New edge
      if (prevEdge.source !== currEdge.source || prevEdge.target !== currEdge.target) return true // Connection change
      if (JSON.stringify(prevEdge.metadata) !== JSON.stringify(currEdge.metadata)) return true // Metadata change
    }

    // No structural changes detected
    return false
  }, [])

  const shouldValidate = useCallback((changeType: 'structural' | 'cosmetic' | 'transient'): boolean => {
    return changeType === 'structural'
  }, [])

  return { hasStructuralChange, shouldValidate }
}

export const useSmartValidation = (options: UseSmartValidationOptions = {}) => {
  const {
    enabled = true,
    debounceMs = 1500, // Increased from 500ms to reduce frequency
    maxValidationRate = 10 // Max 10 validations per minute
  } = options

  const [state, setState] = useState<ValidationState>({
    isValidating: false,
    lastValidation: null,
    validationCount: 0,
    errors: [],
    isValid: null,
  })

  const { validate: graphqlValidate, validationResult, loading } = usePlanDagValidation()
  const validationTimeoutRef = useRef<NodeJS.Timeout | null>(null)
  const lastPlanDagRef = useRef<PlanDag | null>(null)
  const validationHistoryRef = useRef<Date[]>([])

  // Structural change detection
  const detector = useMemo(() => createStructuralChangeDetector(), [])

  // Rate limiting logic
  const isRateLimited = useCallback((): boolean => {
    const now = new Date()
    const oneMinuteAgo = new Date(now.getTime() - 60000)

    // Clean old validation timestamps
    validationHistoryRef.current = validationHistoryRef.current.filter(
      timestamp => timestamp > oneMinuteAgo
    )

    return validationHistoryRef.current.length >= maxValidationRate
  }, [maxValidationRate])

  // Clear validation timeout
  const clearValidationTimeout = useCallback(() => {
    if (validationTimeoutRef.current) {
      clearTimeout(validationTimeoutRef.current)
      validationTimeoutRef.current = null
    }
  }, [])

  // Execute validation
  const executeValidation = useCallback(async (planDag: PlanDag): Promise<void> => {
    if (!enabled || loading || isRateLimited()) {
      return
    }

    setState(prev => ({ ...prev, isValidating: true }))

    try {
      const result = await graphqlValidate(planDag)
      const validation = result.data?.validatePlanDag

      if (validation) {
        setState(prev => ({
          ...prev,
          isValidating: false,
          lastValidation: new Date(),
          validationCount: prev.validationCount + 1,
          errors: validation.errors || [],
          isValid: validation.isValid,
        }))

        // Track validation timestamp for rate limiting
        validationHistoryRef.current.push(new Date())

        console.log(`Smart validation completed - Valid: ${validation.isValid}, Errors: ${validation.errors?.length || 0}`)
      }
    } catch (error) {
      console.error('Smart validation failed:', error)
      setState(prev => ({
        ...prev,
        isValidating: false,
        errors: [{ message: 'Validation service unavailable' }],
        isValid: false,
      }))
    }
  }, [enabled, loading, isRateLimited, graphqlValidate])

  // Schedule validation with debouncing and smart detection
  const scheduleValidation = useCallback((
    planDag: PlanDag,
    changeType: 'structural' | 'cosmetic' | 'transient' = 'structural'
  ): void => {
    if (!enabled) return

    // Check if validation is needed based on change type
    if (!detector.shouldValidate(changeType)) {
      console.log(`Smart validation skipped - ${changeType} change detected`)
      return
    }

    // Check if structural changes actually occurred
    const hasStructuralChanges = detector.hasStructuralChange(lastPlanDagRef.current, planDag)
    if (!hasStructuralChanges) {
      console.log('Smart validation skipped - no structural changes detected')
      return
    }

    // Update reference for future comparisons
    lastPlanDagRef.current = planDag

    // Clear existing timeout
    clearValidationTimeout()

    // Schedule new validation
    validationTimeoutRef.current = setTimeout(() => {
      executeValidation(planDag)
    }, debounceMs)

    console.log(`Smart validation scheduled in ${debounceMs}ms for ${changeType} change`)
  }, [enabled, detector, debounceMs, clearValidationTimeout, executeValidation])

  // Manual validation trigger
  const validateNow = useCallback(async (planDag: PlanDag): Promise<void> => {
    clearValidationTimeout()
    await executeValidation(planDag)
  }, [clearValidationTimeout, executeValidation])

  // Clear all validation state
  const clearValidation = useCallback(() => {
    clearValidationTimeout()
    setState({
      isValidating: false,
      lastValidation: null,
      validationCount: 0,
      errors: [],
      isValid: null,
    })
    validationHistoryRef.current = []
    lastPlanDagRef.current = null
  }, [clearValidationTimeout])

  // Update state when GraphQL validation result changes
  React.useEffect(() => {
    if (validationResult) {
      setState(prev => ({
        ...prev,
        errors: validationResult.errors || [],
        isValid: validationResult.isValid,
      }))
    }
  }, [validationResult])

  // Cleanup on unmount
  React.useEffect(() => {
    return () => {
      clearValidationTimeout()
    }
  }, [clearValidationTimeout])

  return {
    // State
    ...state,

    // Actions
    scheduleValidation,
    validateNow,
    clearValidation,

    // Configuration
    isEnabled: enabled,
    isRateLimited: isRateLimited(),

    // Statistics
    validationRate: validationHistoryRef.current.length, // validations in last minute
    maxValidationRate,
  }
}

// React import for useEffect
import React from 'react'