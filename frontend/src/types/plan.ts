export interface Plan {
  id: number
  projectId: number
  name: string
  description?: string | null
  tags: string[]
  status: string
  version: number
  yamlContent: string
  dependencies?: number[] | null
  createdAt: string
  updatedAt: string
}
