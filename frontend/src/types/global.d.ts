declare namespace NodeJS {
  interface Timeout {}
}

export interface CollaborationEvent {
  eventId: string
  eventType: 'NODE_CREATED' | 'NODE_UPDATED' | 'NODE_DELETED' | 'EDGE_CREATED' | 'EDGE_DELETED'
  userId: string
  timestamp: string
  data: {
    nodeEvent?: {
      node: {
        id: string
        nodeType: string
        position: { x: number; y: number }
        metadata: { label: string; description?: string }
      }
    }
    edgeEvent?: {
      edge: {
        id: string
        source: string
        target: string
      }
    }
  }
}