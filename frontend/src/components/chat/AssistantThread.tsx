import { useMemo, useState } from 'react'
import {
  ComposerPrimitive,
  MessagePrimitive,
  ThreadPrimitive,
  useMessage,
  type ToolCallMessagePartComponent,
  type ThreadAssistantMessagePart,
} from '@assistant-ui/react'
import { MarkdownText } from './MarkdownText'
import { IconCircleX, IconClipboard, IconCode, IconSend, IconTable } from '@tabler/icons-react'

import { Button } from '../ui/button'
import { cn } from '../../lib/utils'
import { showErrorNotification, showSuccessNotification } from '../../utils/notifications'

const formatValue = (value: unknown) => {
  if (value == null) {
    return ''
  }
  if (typeof value === 'string') {
    return value
  }
  if (typeof value === 'number' || typeof value === 'boolean') {
    return String(value)
  }
  try {
    return JSON.stringify(value, null, 2)
  } catch (error) {
    console.warn('Unable to stringify value for copy', error)
    return String(value)
  }
}

const getToolCallOutputText = (toolName: string, result: unknown, fallback?: string) => {
  const text = formatValue(result)
  if (text) {
    return `Tool ${toolName} Result\n\n${text}`
  }
  if (fallback) {
    return `Tool ${toolName}\n\n${fallback}`
  }
  return `Tool ${toolName}`
}

const copyTextToClipboard = async (text: string) => {
  if (!text) return
  if (typeof navigator !== 'undefined' && navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text)
    return
  }

  const textarea = document.createElement('textarea')
  textarea.value = text
  textarea.style.position = 'fixed'
  textarea.style.opacity = '0'
  document.body.appendChild(textarea)
  textarea.focus()
  textarea.select()
  document.execCommand('copy')
  document.body.removeChild(textarea)
}

const ToolCallPart: ToolCallMessagePartComponent<unknown, unknown> = ({
  toolName,
  result,
  argsText,
  args,
}) => {
  const outputText = getToolCallOutputText(toolName, result, argsText || formatValue(args))

  const handleCopy = async () => {
    try {
      await copyTextToClipboard(outputText)
      showSuccessNotification('Copied tool output')
    } catch (error) {
      showErrorNotification('Could not copy tool output')
      console.error('Failed to copy tool output', error)
    }
  }

  return (
    <div className="space-y-2 text-sm text-muted-foreground">
      <div className="flex flex-wrap items-center justify-between gap-3 text-xs font-medium uppercase tracking-wide text-muted-foreground">
        <span>{toolName}</span>
        <Button variant="ghost" size="sm" className="h-7 px-2 text-xs" onClick={handleCopy}>
          <IconClipboard className="h-3.5 w-3.5" /> Copy output
        </Button>
      </div>
      {typeof result === 'string' ? (
        <p className="whitespace-pre-wrap break-words">{result}</p>
      ) : result != null ? (
        <pre className="whitespace-pre-wrap break-words rounded-md bg-muted/60 p-2 text-xs">
          {formatValue(result)}
        </pre>
      ) : (
        <pre className="whitespace-pre-wrap break-words rounded-md bg-muted/40 p-2 text-xs text-muted-foreground">
          {argsText || formatValue(args)}
        </pre>
      )}
    </div>
  )
}

const buildMessageRawText = (parts: readonly ThreadAssistantMessagePart[]) =>
  parts
    .map((part) => {
      switch (part.type) {
        case 'text':
        case 'reasoning':
          return part.text
        case 'tool-call':
          return getToolCallOutputText(
            part.toolName,
            part.result,
            part.argsText || formatValue(part.args),
          )
        case 'source':
          return part.title
            ? `${part.title} (${part.url})`
            : `Source: ${part.url}`
        case 'file':
          return `File: ${part.filename ?? 'attachment'} (${part.mimeType})`
        case 'image':
          return part.filename ? `Image: ${part.filename}` : 'Image attachment'
        default:
          return ''
      }
    })
    .filter(Boolean)
    .join('\n\n')

const extractFirstMarkdownTable = (text: string) => {
  if (!text) return null
  const normalized = text.replace(/\r\n/g, '\n')
  const tableRegex = /(^\|.*\|\s*$\n\|[-:|\s]+\|\s*$\n(?:\|.*\|\s*$\n)+)/gm
  const match = tableRegex.exec(normalized)
  return match ? match[0].trim() : null
}

const SystemMessage = ({ pageLayout }: { pageLayout: boolean }) => (
  <MessagePrimitive.Root
    className={cn(
      'py-2 text-xs text-muted-foreground',
      pageLayout ? 'mx-0 w-full' : 'mx-auto max-w-2xl',
    )}
  >
    <MessagePrimitive.Parts />
  </MessagePrimitive.Root>
)

const Suggestion = ({ prompt }: { prompt: string }) => (
  <ThreadPrimitive.Suggestion
    prompt={prompt}
    method="replace"
    autoSend
    className="flex flex-col gap-1 rounded-xl border border-dashed border-border bg-muted/50 p-3 text-left text-sm transition-colors hover:bg-muted"
  >
    {prompt}
  </ThreadPrimitive.Suggestion>
)

export const AssistantThread = ({
  suggestions = [],
  composerDisabled = false,
  showSuggestions = true,
  composerPlaceholder,
  layout = 'modal',
}: {
  suggestions?: string[]
  composerDisabled?: boolean
  showSuggestions?: boolean
  composerPlaceholder?: string
  layout?: 'modal' | 'page'
}) => {
  const pageLayout = layout === 'page'
  const widthClass = pageLayout ? 'max-w-none' : 'max-w-3xl'
  const viewportPaddingBottom = pageLayout ? 'pb-4' : 'pb-24'

  const UserMessage = () => (
    <MessagePrimitive.Root
      className={cn(
        'grid w-full auto-rows-auto grid-cols-[minmax(48px,1fr)_auto] gap-y-2 self-end py-3 [&>*]:col-start-2',
        widthClass,
      )}
    >
      <div className="rounded-2xl bg-primary px-4 py-3 text-sm text-primary-foreground shadow">
        <MessagePrimitive.Parts />
      </div>
    </MessagePrimitive.Root>
  )

  const AssistantMessage = () => {
    const messageState = useMessage((state) => state)
    const content = (messageState?.content ?? []) as readonly ThreadAssistantMessagePart[]
    const rawText = useMemo(() => buildMessageRawText(content), [content])
    const tableText = useMemo(() => extractFirstMarkdownTable(rawText), [rawText])
    const [showRaw, setShowRaw] = useState(false)

    const handleCopyMessage = async () => {
      if (!rawText) return
      try {
        await copyTextToClipboard(rawText)
        showSuccessNotification('Copied response')
      } catch (error) {
        showErrorNotification('Could not copy response')
        console.error('Failed to copy assistant response', error)
      }
    }

    const hasCopyableContent = Boolean(rawText)
    const hasTable = Boolean(tableText)

    const handleCopyTable = async () => {
      if (!tableText) return
      try {
        await copyTextToClipboard(tableText)
        showSuccessNotification('Copied table')
      } catch (error) {
        showErrorNotification('Could not copy table')
        console.error('Failed to copy table', error)
      }
    }

    return (
      <MessagePrimitive.Root
        className={cn(
          'grid w-full auto-rows-auto grid-cols-[auto_minmax(48px,1fr)] gap-y-2 self-start py-3 [&>*]:col-start-1',
          widthClass,
        )}
      >
        <div className="rounded-2xl border border-border bg-muted px-4 py-3 text-sm shadow-sm">
          {showRaw ? (
            <pre className="whitespace-pre-wrap break-words font-mono text-xs text-muted-foreground">
              {rawText}
            </pre>
          ) : (
            <MessagePrimitive.Parts
              components={{ Text: MarkdownText, tools: { Override: ToolCallPart } }}
            />
          )}
        </div>
        <div className="col-span-2 mt-2 flex flex-wrap justify-end gap-2 text-xs">
          <Button
            variant="ghost"
            size="sm"
            className="h-7 px-2"
            onClick={handleCopyTable}
            disabled={!hasTable}
          >
            <IconTable className="h-3.5 w-3.5" /> Copy table
          </Button>
          <Button
            variant="ghost"
            size="sm"
            className="h-7 px-2"
            onClick={() => setShowRaw((prev) => !prev)}
            disabled={!hasCopyableContent}
          >
            <IconCode className="h-3.5 w-3.5" />
            {showRaw ? 'Hide raw' : 'Show raw'}
          </Button>
          <Button
            variant="ghost"
            size="sm"
            className="h-7 px-2"
            onClick={handleCopyMessage}
            disabled={!hasCopyableContent}
          >
            <IconClipboard className="h-3.5 w-3.5" /> Copy
          </Button>
        </div>
      </MessagePrimitive.Root>
    )
  }

  const ThreadComposer = ({ disabled, placeholder }: { disabled: boolean; placeholder?: string }) => (
    <ComposerPrimitive.Root
      className={cn(
        'flex w-full items-end gap-3 rounded-xl border border-border bg-background px-4 py-3 shadow-sm',
        widthClass,
        pageLayout ? 'mx-0' : 'mx-auto',
      )}
    >
      <ComposerPrimitive.Input
        aria-label="Ask the assistant"
        placeholder={placeholder ?? (disabled ? 'Waiting for response…' : 'Write a message…')}
        className="flex-1 resize-none border-none bg-transparent p-0 text-sm outline-none focus-visible:ring-0 disabled:cursor-not-allowed disabled:text-muted-foreground"
        disabled={disabled}
      />
      <div className="flex items-center gap-2">
        <ThreadPrimitive.If running>
          <ComposerPrimitive.Cancel asChild>
            <Button variant="ghost" size="icon">
              <IconCircleX className="h-4 w-4" />
            </Button>
          </ComposerPrimitive.Cancel>
        </ThreadPrimitive.If>
        <ThreadPrimitive.If running={false}>
          <ComposerPrimitive.Send asChild>
            <Button size="icon" disabled={disabled}>
              <IconSend className="h-4 w-4" />
            </Button>
          </ComposerPrimitive.Send>
        </ThreadPrimitive.If>
      </div>
    </ComposerPrimitive.Root>
  )

  return (
    <ThreadPrimitive.Root
      className={cn(
        'relative flex h-full flex-col',
        pageLayout ? 'items-stretch' : 'items-center',
      )}
    >
      <ThreadPrimitive.Viewport
        className={cn('flex-1 w-full space-y-3 overflow-y-auto px-1', viewportPaddingBottom, widthClass)}
      >
        <ThreadPrimitive.Empty>
          <div className="flex flex-col gap-6 py-12">
            <div className="w-full rounded-xl border border-dashed border-border bg-background p-6">
              <h3 className="text-base font-semibold">Start a conversation</h3>
              <p className="mt-2 text-sm text-muted-foreground">
                Ask a question about this project. The assistant can run Layercake tools whenever additional context is helpful.
              </p>
            </div>
            {showSuggestions && suggestions.length > 0 && (
              <div className={cn('flex flex-wrap items-stretch gap-3', widthClass)}>
                {suggestions.map((prompt) => (
                  <Suggestion key={prompt} prompt={prompt} />
                ))}
              </div>
            )}
          </div>
        </ThreadPrimitive.Empty>

        <ThreadPrimitive.Messages
          components={{
            UserMessage,
            AssistantMessage,
            SystemMessage: () => <SystemMessage pageLayout={pageLayout} />,
          }}
        />
      </ThreadPrimitive.Viewport>

      <div className="sticky bottom-0 w-full border-t border-border bg-background/95 px-1 py-3 backdrop-blur supports-[backdrop-filter]:bg-background/80">
        <ThreadComposer disabled={composerDisabled} placeholder={composerPlaceholder} />
      </div>
    </ThreadPrimitive.Root>
  )
}
