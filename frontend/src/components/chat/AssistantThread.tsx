import {
  ComposerPrimitive,
  MessagePrimitive,
  ThreadPrimitive,
  type ToolCallMessagePartComponent,
} from '@assistant-ui/react'
import { IconCircleX, IconSend } from '@tabler/icons-react'

import { Button } from '../ui/button'

const ToolCallPart: ToolCallMessagePartComponent<unknown, unknown> = ({
  toolName,
  result,
}) => (
  <div className="space-y-1 text-sm text-muted-foreground">
    <p className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
      {toolName}
    </p>
    {typeof result === 'string' ? (
      <p className="whitespace-pre-wrap break-words">{result}</p>
    ) : (
      <pre className="whitespace-pre-wrap break-words rounded-md bg-muted/60 p-2 text-xs">
        {JSON.stringify(result, null, 2)}
      </pre>
    )}
  </div>
)

const UserMessage = () => (
  <MessagePrimitive.Root className="grid w-full max-w-3xl auto-rows-auto grid-cols-[minmax(48px,1fr)_auto] gap-y-2 self-end py-3 [&>*]:col-start-2">
    <div className="rounded-2xl bg-primary px-4 py-3 text-sm text-primary-foreground shadow">
      <MessagePrimitive.Parts />
    </div>
  </MessagePrimitive.Root>
)

const AssistantMessage = () => (
  <MessagePrimitive.Root className="grid w-full max-w-3xl auto-rows-auto grid-cols-[auto_minmax(48px,1fr)] gap-y-2 self-start py-3 [&>*]:col-start-1">
    <div className="rounded-2xl border border-border bg-muted px-4 py-3 text-sm shadow-sm">
      <MessagePrimitive.Parts components={{ tools: { Override: ToolCallPart } }} />
    </div>
  </MessagePrimitive.Root>
)

const SystemMessage = () => (
  <MessagePrimitive.Root className="mx-auto max-w-2xl py-2 text-xs text-muted-foreground">
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

const ThreadComposer = ({ disabled, placeholder }: { disabled: boolean; placeholder?: string }) => (
  <ComposerPrimitive.Root className="mx-auto flex w-full max-w-3xl items-end gap-3 rounded-xl border border-border bg-background px-4 py-3 shadow-sm">
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

export const AssistantThread = ({
  suggestions = [],
  composerDisabled = false,
  showSuggestions = true,
  composerPlaceholder,
}: {
  suggestions?: string[]
  composerDisabled?: boolean
  showSuggestions?: boolean
  composerPlaceholder?: string
}) => (
  <ThreadPrimitive.Root className="relative flex h-full flex-col items-center">
    <ThreadPrimitive.Viewport className="flex-1 w-full max-w-3xl space-y-3 overflow-y-auto px-1 pb-4">
      <ThreadPrimitive.Empty>
        <div className="flex flex-col items-center gap-6 py-12">
          <div className="w-full rounded-xl border border-dashed border-border bg-background p-6">
            <h3 className="text-base font-semibold">Start a conversation</h3>
            <p className="mt-2 text-sm text-muted-foreground">
              Ask a question about this project. The assistant can run Layercake tools whenever additional context is helpful.
            </p>
          </div>
          {showSuggestions && suggestions.length > 0 && (
            <div className="flex max-w-3xl flex-wrap items-stretch gap-3">
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
          SystemMessage,
        }}
      />

    </ThreadPrimitive.Viewport>

    <div className="sticky bottom-0 w-full border-t border-border bg-background/95 px-1 py-3 backdrop-blur supports-[backdrop-filter]:bg-background/80">
      <ThreadComposer disabled={composerDisabled} placeholder={composerPlaceholder} />
    </div>
  </ThreadPrimitive.Root>
)
