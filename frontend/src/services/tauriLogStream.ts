import type { UnlistenFn } from '@tauri-apps/api/event'
import { attachLogger, LogLevel } from '@tauri-apps/plugin-log'

export type AppLogLevel = 'TRACE' | 'DEBUG' | 'INFO' | 'WARN' | 'ERROR'

export interface AppLogEntry {
  level: AppLogLevel
  message: string
  target?: string | null
  metadata?: {
    modulePath?: string | null
    file?: string | null
    line?: number | null
  }
  timestamp: Date
}

const isTauri = typeof window !== 'undefined' && '__TAURI__' in window

const MAX_BUFFER_SIZE = 2000
const subscribers = new Set<(entries: AppLogEntry[]) => void>()
const buffer: AppLogEntry[] = []

let started = false
let unsubscribe: UnlistenFn | null = null

const emitSnapshot = () => {
  const snapshot = [...buffer]
  subscribers.forEach((listener) => listener(snapshot))
}

export async function startLogStream(): Promise<void> {
  if (!isTauri || started) {
    return
  }

  started = true
  try {
    unsubscribe = await attachLogger(({ level, message }) => {
      const formatted: AppLogEntry = {
        level: mapLogLevel(level),
        message,
        timestamp: new Date(),
      }
      buffer.push(formatted)
      if (buffer.length > MAX_BUFFER_SIZE) {
        buffer.splice(0, buffer.length - MAX_BUFFER_SIZE)
      }
      emitSnapshot()
    })
  } catch (error) {
    console.error('[Tauri] Failed to attach log listener:', error)
  }
}

export function getLogEntries(): AppLogEntry[] {
  return [...buffer]
}

export function subscribeToLogStream(listener: (entries: AppLogEntry[]) => void): () => void {
  subscribers.add(listener)
  listener([...buffer])
  return () => {
    subscribers.delete(listener)
  }
}

export function clearLogBuffer(): void {
  buffer.length = 0
  emitSnapshot()
}

export function isTauriRuntime(): boolean {
  return isTauri
}

export function stopLogStream(): void {
  if (unsubscribe) {
    unsubscribe()
    unsubscribe = null
  }
  started = false
}

function mapLogLevel(level: LogLevel): AppLogLevel {
  switch (level) {
    case LogLevel.Trace:
      return 'TRACE'
    case LogLevel.Debug:
      return 'DEBUG'
    case LogLevel.Warn:
      return 'WARN'
    case LogLevel.Error:
      return 'ERROR'
    case LogLevel.Info:
    default:
      return 'INFO'
  }
}
