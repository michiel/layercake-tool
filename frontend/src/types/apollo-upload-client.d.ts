declare module 'apollo-upload-client/createUploadLink.mjs' {
  import { ApolloLink } from '@apollo/client'
  import { HttpOptions } from '@apollo/client/link/http'

  export type UploadLinkOptions = HttpOptions & {
    fetch?: typeof fetch
  }

  export function createUploadLink(options?: UploadLinkOptions): ApolloLink
}
