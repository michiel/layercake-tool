declare module 'apollo-upload-client/UploadHttpLink.mjs' {
  import { ApolloLink } from '@apollo/client'
  import { HttpLink } from '@apollo/client/link/http'

  export interface UploadLinkOptions extends HttpLink.Options {
    fetch?: typeof fetch
  }

  export default class UploadHttpLink extends ApolloLink {
    constructor(options?: UploadLinkOptions)
  }
}
