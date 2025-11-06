import { useEffect } from 'react'

import { useChatContextStore } from '../state/chatContextStore'

export const useRegisterChatContext = (
  context: string | null,
  projectId?: number,
) => {
  const setPageContext = useChatContextStore((state) => state.setPageContext)
  const clearPageContext = useChatContextStore((state) => state.clearPageContext)
  const setProjectId = useChatContextStore((state) => state.setProjectId)

  useEffect(() => {
    if (projectId !== undefined) {
      setProjectId(projectId)
    }
  }, [projectId, setProjectId])

  useEffect(() => {
    if (context && context.trim().length > 0) {
      setPageContext(context.trim())
      return () => clearPageContext()
    }
    clearPageContext()
    return () => {
      clearPageContext()
    }
  }, [context, setPageContext, clearPageContext])
}
