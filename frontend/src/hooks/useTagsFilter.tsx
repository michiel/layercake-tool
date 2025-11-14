import { createContext, useContext, useState, ReactNode } from 'react'

interface TagsFilterContextValue {
  activeTags: string[]
  setActiveTags: (tags: string[]) => void
  clearTags: () => void
}

const TagsFilterContext = createContext<TagsFilterContextValue | null>(null)

const STORAGE_KEY = 'layercake-tags-filter'

export const TagsFilterProvider = ({ children }: { children: ReactNode }) => {
  const [activeTags, setActiveTagsState] = useState<string[]>(() => {
    // Load from localStorage on mount
    try {
      const stored = localStorage.getItem(STORAGE_KEY)
      return stored ? JSON.parse(stored) : []
    } catch {
      return []
    }
  })

  const setActiveTags = (tags: string[]) => {
    setActiveTagsState(tags)
    // Persist to localStorage
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(tags))
    } catch (error) {
      console.error('Failed to save tags filter to localStorage:', error)
    }
  }

  const clearTags = () => {
    setActiveTags([])
  }

  return (
    <TagsFilterContext.Provider value={{ activeTags, setActiveTags, clearTags }}>
      {children}
    </TagsFilterContext.Provider>
  )
}

export const useTagsFilter = () => {
  const context = useContext(TagsFilterContext)
  if (!context) {
    throw new Error('useTagsFilter must be used within TagsFilterProvider')
  }
  return context
}
