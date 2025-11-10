/**
 * Utilities for parsing and handling RAG citations in chat messages
 */

export interface Citation {
  index: number
  source: string
}

export interface ParsedMessage {
  content: string
  citations: Citation[]
}

/**
 * Parse a message that may contain citations in the format:
 *
 * Main message content...
 *
 * ---
 * **Sources:**
 * - [1] source1.txt
 * - [2] source2.txt
 */
export function parseMessageWithCitations(message: string): ParsedMessage {
  // Check if message contains citation separator
  const citationSeparator = '\n---\n**Sources:**\n'
  const parts = message.split(citationSeparator)

  if (parts.length < 2) {
    // No citations found
    return {
      content: message,
      citations: []
    }
  }

  const content = parts[0].trim()
  const citationsText = parts[1].trim()

  // Parse individual citations
  const citations: Citation[] = []
  const citationLines = citationsText.split('\n')

  for (const line of citationLines) {
    // Match pattern: - [1] filename
    const match = line.match(/^-\s*\[(\d+)\]\s*(.+)$/)
    if (match) {
      const index = parseInt(match[1], 10)
      const source = match[2].trim()
      citations.push({ index, source })
    }
  }

  return {
    content,
    citations
  }
}

/**
 * Check if a message contains citations
 */
export function hasCitations(message: string): boolean {
  return message.includes('\n---\n**Sources:**\n')
}
