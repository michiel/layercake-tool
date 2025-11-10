import { IconFileText } from '@tabler/icons-react'
import { Badge } from '../ui/badge'
import { Card } from '../ui/card'
import type { Citation } from '../../utils/citations'

interface CitationFooterProps {
  citations: Citation[]
}

/**
 * Displays a list of sources/citations for RAG-enhanced messages
 */
export function CitationFooter({ citations }: CitationFooterProps) {
  if (citations.length === 0) {
    return null
  }

  return (
    <Card className="mt-4 border-l-4 border-l-blue-500 bg-slate-50 dark:bg-slate-900">
      <div className="p-3">
        <div className="mb-2 flex items-center gap-2">
          <IconFileText className="h-4 w-4 text-blue-600 dark:text-blue-400" />
          <span className="text-sm font-semibold text-slate-700 dark:text-slate-300">
            Sources
          </span>
          <Badge variant="secondary" className="text-xs">
            📚 Knowledge Base
          </Badge>
        </div>
        <ul className="space-y-1.5">
          {citations.map((citation) => (
            <li
              key={citation.index}
              className="flex items-start gap-2 text-sm text-slate-600 dark:text-slate-400"
            >
              <span className="mt-0.5 flex h-5 w-5 flex-shrink-0 items-center justify-center rounded-full bg-blue-100 text-xs font-medium text-blue-700 dark:bg-blue-900 dark:text-blue-300">
                {citation.index}
              </span>
              <span className="break-all font-mono">{citation.source}</span>
            </li>
          ))}
        </ul>
      </div>
    </Card>
  )
}
