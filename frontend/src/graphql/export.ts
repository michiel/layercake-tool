import { gql, type TypedDocumentNode } from '@apollo/client'

export const EXPORT_NODE_OUTPUT: TypedDocumentNode<
  { exportNodeOutput: ExportNodeOutputResult },
  {
    projectId: number
    nodeId: string
    planId?: number | null
    renderConfig?: Record<string, unknown> | null
  }
> = gql`
  mutation ExportNodeOutput(
    $projectId: Int!
    $nodeId: String!
    $planId: Int
    $renderConfig: RenderConfigInput
  ) {
    exportNodeOutput(
      projectId: $projectId
      planId: $planId
      nodeId: $nodeId
      renderConfigOverride: $renderConfig
    ) {
      success
      message
      content
      filename
      mimeType
    }
  }
`

export interface ExportNodeOutputResult {
  success: boolean
  message: string
  content: string // Base64 encoded content
  filename: string
  mimeType: string
}
