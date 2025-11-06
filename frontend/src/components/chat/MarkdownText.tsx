import '@assistant-ui/react-markdown/styles/dot.css'

import { MarkdownTextPrimitive, useIsMarkdownCodeBlock } from '@assistant-ui/react-markdown'
import remarkGfm from 'remark-gfm'
import { memo } from 'react'

export const MarkdownText = memo(function MarkdownText() {
  return (
    <MarkdownTextPrimitive
      remarkPlugins={[remarkGfm]}
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
  )
})
