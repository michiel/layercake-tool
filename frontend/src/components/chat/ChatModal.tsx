import { AssistantModalPrimitive, AssistantRuntimeProvider } from '@assistant-ui/react'
import { IconMessageDots } from '@tabler/icons-react'

import { Button } from '../ui/button'
import { useChat } from './ChatProvider'
import { AssistantThread } from './AssistantThread'

export const ChatModal = () => {
  const { runtime, provider, isAwaitingAssistant, restart, session } = useChat()

  return (
    <AssistantModalPrimitive.Root>
      <AssistantModalPrimitive.Anchor className="pointer-events-none fixed bottom-4 right-4 z-50 size-12">
        <AssistantModalPrimitive.Trigger asChild>
          <Button
            size="icon"
            className="pointer-events-auto h-12 w-12 rounded-full shadow-lg"
            variant="default"
          >
            <IconMessageDots className="h-5 w-5" />
          </Button>
        </AssistantModalPrimitive.Trigger>
      </AssistantModalPrimitive.Anchor>
      <AssistantModalPrimitive.Content className="flex h-[560px] w-[420px] flex-col overflow-hidden rounded-2xl border border-border bg-background shadow-2xl">
        <div className="flex items-center justify-between border-b border-border px-4 py-3">
          <div>
            <p className="text-sm font-semibold">Assistant</p>
            <p className="text-xs text-muted-foreground">
              {session?.model ? `Model: ${session.model}` : provider}
            </p>
          </div>
          <div className="flex items-center gap-2">
            <Button
              size="sm"
              variant="ghost"
              onClick={() => restart()}
              disabled={isAwaitingAssistant}
            >
              Restart
            </Button>
          </div>
        </div>
        <AssistantRuntimeProvider runtime={runtime}>
          <AssistantThread
            suggestions={[
              'Summarize the latest project updates.',
              'List recent tool invocations for this project.',
              'What are the open tasks for this project?',
            ]}
          />
        </AssistantRuntimeProvider>
      </AssistantModalPrimitive.Content>
    </AssistantModalPrimitive.Root>
  )
}
