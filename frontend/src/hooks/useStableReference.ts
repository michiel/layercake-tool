import React, { useRef, useCallback, useMemo } from 'react'

/**
 * Creates a stable callback reference that doesn't change between renders
 * while still accessing the latest values from the closure
 */
export const useStableCallback = <T extends (...args: any[]) => any>(fn: T): T => {
  const ref = useRef<T>(fn)
  ref.current = fn
  return useCallback((...args: any[]) => ref.current(...args), []) as T
}

/**
 * Creates a stable object reference using deep equality comparison
 * Only recreates the object when its content actually changes
 */
export const useStableObject = <T extends object>(obj: T): T => {
  const ref = useRef<T>(obj)
  const previousStringified = useRef<string>(JSON.stringify(obj))

  const currentStringified = JSON.stringify(obj)

  // Only update if the object content has actually changed
  if (currentStringified !== previousStringified.current) {
    ref.current = obj
    previousStringified.current = currentStringified
  }

  return ref.current
}

/**
 * Creates a stable array reference using shallow equality comparison
 * Only recreates the array when its items change
 */
export const useStableArray = <T>(arr: T[]): T[] => {
  const ref = useRef<T[]>(arr)

  const isEqual = useMemo(() => {
    if (arr.length !== ref.current.length) return false
    return arr.every((item, index) => item === ref.current[index])
  }, [arr])

  if (!isEqual) {
    ref.current = arr
  }

  return ref.current
}

/**
 * Creates a stable memoized value that only updates when dependencies actually change
 * Uses deep equality for dependency comparison
 */
export const useStableMemo = <T>(
  factory: () => T,
  deps: any[]
): T => {
  const depsRef = useRef<any[]>(deps)
  const valueRef = useRef<T | undefined>(undefined)
  const hasValueRef = useRef<boolean>(false)

  const depsChanged = useMemo(() => {
    if (!hasValueRef.current) return true
    if (deps.length !== depsRef.current.length) return true

    return deps.some((dep, index) => {
      const prevDep = depsRef.current[index]
      if (typeof dep === 'object' && typeof prevDep === 'object') {
        return JSON.stringify(dep) !== JSON.stringify(prevDep)
      }
      return dep !== prevDep
    })
  }, [deps])

  if (depsChanged) {
    valueRef.current = factory()
    depsRef.current = deps
    hasValueRef.current = true
  }

  return valueRef.current!
}

/**
 * Utility to check if an external data source has changed
 * Used to prevent circular dependencies in useEffect
 */
export const useExternalDataChangeDetector = <T>(data: T) => {
  const previousDataRef = useRef<T>(data)
  const changeIdRef = useRef<number>(0)

  const hasChanged = useMemo(() => {
    if (typeof data === 'object' && typeof previousDataRef.current === 'object') {
      return JSON.stringify(data) !== JSON.stringify(previousDataRef.current)
    }
    return data !== previousDataRef.current
  }, [data])

  if (hasChanged) {
    previousDataRef.current = data
    changeIdRef.current += 1
  }

  return {
    hasChanged,
    changeId: changeIdRef.current,
    previousData: previousDataRef.current
  }
}

/**
 * Stable version of useEffect that only runs when dependencies actually change
 * Prevents unnecessary effect executions due to reference instability
 */
export const useStableEffect = (
  effect: React.EffectCallback,
  deps: any[]
): void => {
  const stableDeps = useStableArray(deps)

  React.useEffect(effect, stableDeps)
}