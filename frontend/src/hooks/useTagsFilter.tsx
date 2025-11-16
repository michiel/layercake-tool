import { createContext, useContext, useState, ReactNode } from 'react'

interface TagsFilterContextValue {
  activeTags: string[]
  setActiveTags: (tags: string[]) => void
  clearTags: () => void
}

const TagsFilterContext = createContext<TagsFilterContextValue | null>(null)

const STORAGE_KEY = 'layercake-tags-filter'

const normalizeTags = (tags: string[]) => {
  const seen = new Set<string>()
  const normalized: string[] = []
  tags.forEach((t) => {
    const tag = t.trim().toLowerCase()
    if (tag.length === 0) return
    if (!seen.has(tag)) {
      seen.add(tag)
      normalized.push(tag)
    }
  })
  return normalized
}

export const TagsFilterProvider = ({ children }: { children: ReactNode }) => {
  const [activeTags, setActiveTagsState] = useState<string[]>(() => {
    // Load from localStorage on mount
    try {
      const stored = localStorage.getItem(STORAGE_KEY)
      if (!stored) return []
      const parsed = JSON.parse(stored)
      return Array.isArray(parsed) ? normalizeTags(parsed) : []
    } catch {
      return []
    }
  })

  const setActiveTags = (tags: string[]) => {
    const normalized = normalizeTags(tags)
    setActiveTagsState(normalized)
    // Persist to localStorage
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(normalized))
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
