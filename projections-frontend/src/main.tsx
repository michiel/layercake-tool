import './polyfills/webgpu'
import React from 'react'
import ReactDOM from 'react-dom/client'
import { ApolloClient, InMemoryCache, split } from '@apollo/client/core'
import { ApolloProvider } from '@apollo/client/react'
import { HttpLink } from '@apollo/client/link/http'
import { GraphQLWsLink } from '@apollo/client/link/subscriptions'
import { getMainDefinition } from '@apollo/client/utilities'
import { createClient } from 'graphql-ws'
import App from './app'

// Resolve the API base URL. The viewer is normally served same-origin by the
// layercake binary, so it defaults to window.location.origin. An explicit
// `?apiBase=` query param or VITE_API_BASE_URL (dev) overrides that.
const urlParams = new URLSearchParams(window.location.search)
const apiBaseFromUrl = urlParams.get('apiBase')
const baseUrl =
  apiBaseFromUrl ||
  (import.meta.env.VITE_API_BASE_URL as string | undefined) ||
  window.location.origin
console.log('[Projections Viewer] Using API base URL:', baseUrl)

const httpUrl = `${baseUrl}/projections/graphql`
const wsUrl = `${baseUrl.replace('http', 'ws')}/projections/graphql/ws`

const wsLink = new GraphQLWsLink(
  createClient({
    url: wsUrl,
  })
)

const httpLink = new HttpLink({
  uri: httpUrl,
})

const splitLink = split(
  ({ query }) => {
    const definition = getMainDefinition(query)
    return definition.kind === 'OperationDefinition' && definition.operation === 'subscription'
  },
  wsLink,
  httpLink
)

const client = new ApolloClient({
  link: splitLink,
  cache: new InMemoryCache(),
})

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <ApolloProvider client={client}>
      <App />
    </ApolloProvider>
  </React.StrictMode>
)
