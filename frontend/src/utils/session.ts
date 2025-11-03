const SESSION_STORAGE_KEY = 'layercake_session_id'

export function getOrCreateSessionId(): string {
  if (typeof window === 'undefined') {
    return 'session-server'
  }

  try {
    const existing = localStorage.getItem(SESSION_STORAGE_KEY)
    if (existing && existing.length > 0) {
      return existing
    }
  } catch (error) {
    console.warn('[Session] Failed to read session id from storage:', error)
  }

  const sessionId = generateSessionId()

  try {
    localStorage.setItem(SESSION_STORAGE_KEY, sessionId)
  } catch (error) {
    console.warn('[Session] Failed to persist session id:', error)
  }

  return sessionId
}

function generateSessionId(): string {
  if (typeof crypto !== 'undefined' && 'randomUUID' in crypto) {
    try {
      return `user-${crypto.randomUUID()}`
    } catch {
      // fall through to timestamp-based id
    }
  }

  const random = Math.random().toString(36).substring(2, 11)
  return `user-${Date.now()}-${random}`
}
