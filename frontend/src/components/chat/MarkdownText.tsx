import '@assistant-ui/react-markdown/styles/dot.css'

import { MarkdownTextPrimitive, useIsMarkdownCodeBlock } from '@assistant-ui/react-markdown'
import remarkGfm from 'remark-gfm'
import { memo, useState, useRef } from 'react'
import { parseMessageWithCitations } from '../../utils/citations'
import { CitationFooter } from './CitationFooter'
import type { Citation } from '../../utils/citations'

export const MarkdownText = memo(function MarkdownText() {
  const [parsedCitations, setParsedCitations] = useState<Citation[]>([])
  const lastProcessedTextRef = useRef<string>('')

  // Preprocess function to extract citations and clean content
  const preprocessMarkdown = (text: string) => {
    const { content, citations } = parseMessageWithCitations(text)

    // Only update state if the text has changed (avoid infinite render loop)
    if (text !== lastProcessedTextRef.current) {
      lastProcessedTextRef.current = text

      // Use setTimeout to update state after render completes
      setTimeout(() => {
        setParsedCitations(citations)
      }, 0)
    }

    return content
  }

  return (
    <>
      <MarkdownTextPrimitive
        remarkPlugins={[remarkGfm]}
        preprocess={preprocessMarkdown}
        components={{
          pre: ({ className, ...props }) => (
            <pre className={`overflow-x-auto rounded-lg bg-slate-900 p-4 text-white ${className ?? ''}`} {...props} />
          ),
          code: ({ className, ...props }) => {
            const isBlock = useIsMarkdownCodeBlock()
            return (
              <code
                className={
                  !isBlock
                    ? `rounded bg-muted px-1 py-0.5 font-mono text-sm ${className ?? ''}`
                    : className
                }
                {...props}
              />
            )
          },
        }}
        className="prose prose-sm dark:prose-invert"
      />
      {parsedCitations.length > 0 && <CitationFooter citations={parsedCitations} />}
    </>
  )
})
