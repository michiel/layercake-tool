/**
 * Tauri utility functions for desktop application
 */

import { invoke } from '@tauri-apps/api/core'

export interface ServerInfo {
  port: number
  secret: string
  url: string
}

/**
 * Check if the application is running in Tauri (desktop mode)
 */
export function isTauriApp(): boolean {
  return '__TAURI__' in window
}

/**
 * Get the embedded server connection information
 * This is only available in Tauri desktop mode
 */
export async function getServerInfo(): Promise<ServerInfo | null> {
  if (!isTauriApp()) {
    return null
  }

  try {
    const info = await invoke<ServerInfo>('get_server_info')
    console.log('[Tauri] Server info received:', { port: info.port, url: info.url })
    return info
  } catch (error) {
    console.error('[Tauri] Failed to get server info:', error)
    return null
  }
}

/**
 * Check if the embedded server is healthy
 */
export async function checkServerStatus(): Promise<boolean> {
  if (!isTauriApp()) {
    return false
  }

  try {
    const status = await invoke<boolean>('check_server_status')
    return status
  } catch (error) {
    console.error('[Tauri] Failed to check server status:', error)
    return false
  }
}

/**
 * Wait for the embedded server to be ready
 * Polls the server status until it's healthy or times out
 */
export async function waitForServer(
  maxAttempts: number = 30,
  delayMs: number = 1000
): Promise<boolean> {
  if (!isTauriApp()) {
    return false
  }

  console.log('[Tauri] Waiting for embedded server to be ready...')

  for (let attempt = 0; attempt < maxAttempts; attempt++) {
    const isHealthy = await checkServerStatus()
    if (isHealthy) {
      console.log('[Tauri] Embedded server is ready')
      return true
    }

    console.log(`[Tauri] Server not ready yet (attempt ${attempt + 1}/${maxAttempts})`)
    await new Promise((resolve) => setTimeout(resolve, delayMs))
  }

  console.error('[Tauri] Server failed to become ready within timeout')
  return false
}
