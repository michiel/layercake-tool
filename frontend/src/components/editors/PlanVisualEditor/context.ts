import React from 'react'

export interface PlanVisualEditorContextValue {
  projectId: number
  planId?: number
}

export const PlanVisualEditorContext = React.createContext<PlanVisualEditorContextValue | null>(
  null
)

export const usePlanVisualEditorContext = () => React.useContext(PlanVisualEditorContext)
