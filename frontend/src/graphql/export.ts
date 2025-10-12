import { gql } from '@apollo/client';

export const EXPORT_NODE_OUTPUT = gql`
  mutation ExportNodeOutput($projectId: Int!, $nodeId: String!) {
    exportNodeOutput(projectId: $projectId, nodeId: $nodeId) {
      success
      message
      content
      filename
      mimeType
    }
  }
`;

export interface ExportNodeOutputResult {
  success: boolean;
  message: string;
  content: string; // Base64 encoded content
  filename: string;
  mimeType: string;
}
