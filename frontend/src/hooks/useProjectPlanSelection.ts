import { useCallback, useEffect, useMemo } from 'react'
import { useLocation, useNavigate } from 'react-router-dom'
import { useQuery } from '@apollo/client/react'
import { LIST_PLANS } from '@/graphql/plans'
import { Plan } from '@/types/plan'

interface UseProjectPlanSelectionResult {
  plans: Plan[]
  selectedPlanId: number | null
  selectedPlan: Plan | null
  loading: boolean
  error?: Error
  selectPlan: (planId: number) => void
  refreshPlans: () => void
}

export const useProjectPlanSelection = (projectId: number): UseProjectPlanSelectionResult => {
  const navigate = useNavigate()
  const location = useLocation()

  const normalizedSearch = useMemo(() => {
    return location.search.startsWith('?') ? location.search.substring(1) : location.search
  }, [location.search])

  const planIdFromQuery = useMemo(() => {
    if (!normalizedSearch) {
      return null
    }
    const params = new URLSearchParams(normalizedSearch)
    const raw = params.get('planId')
    if (!raw) {
      return null
    }
    const parsed = Number(raw)
    return Number.isFinite(parsed) ? parsed : null
  }, [normalizedSearch])

  const {
    data,
    loading,
    error,
    refetch,
  } = useQuery<{ plans: Plan[] }>(LIST_PLANS, {
    variables: { projectId },
    skip: !projectId,
    fetchPolicy: 'cache-and-network',
  })

  const plans: Plan[] = data?.plans ?? []

  const selectedPlanId = useMemo(() => {
    if (planIdFromQuery && plans.some(plan => plan.id === planIdFromQuery)) {
      return planIdFromQuery
    }
    if (plans.length > 0) {
      return plans[0].id
    }
    return null
  }, [planIdFromQuery, plans])

  useEffect(() => {
    if (!projectId || !plans.length || !selectedPlanId) {
      return
    }
    if (planIdFromQuery === selectedPlanId) {
      return
    }
    const params = new URLSearchParams(normalizedSearch)
    params.set('planId', selectedPlanId.toString())
    const nextSearch = params.toString()
    const currentSearch = normalizedSearch
    if (nextSearch === currentSearch) {
      return
    }
    navigate({ pathname: location.pathname, search: nextSearch }, { replace: true })
  }, [
    projectId,
    plans.length,
    selectedPlanId,
    planIdFromQuery,
    normalizedSearch,
    navigate,
    location.pathname,
  ])

  const selectPlan = useCallback(
    (planId: number) => {
      const params = new URLSearchParams(normalizedSearch)
      params.set('planId', planId.toString())
      navigate({ pathname: location.pathname, search: params.toString() }, { replace: false })
    },
    [navigate, normalizedSearch, location.pathname],
  )

  const selectedPlan = useMemo(
    () => plans.find(plan => plan.id === selectedPlanId) ?? null,
    [plans, selectedPlanId],
  )

  return {
    plans,
    selectedPlanId,
    selectedPlan,
    loading,
    error,
    selectPlan,
    refreshPlans: () => {
      void refetch()
    },
  }
}
