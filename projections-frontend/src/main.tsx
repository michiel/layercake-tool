import React from 'react'
import ReactDOM from 'react-dom/client'
import { ApolloClient, InMemoryCache, split } from '@apollo/client/core'
import { ApolloProvider } from '@apollo/client/react'
import { HttpLink } from '@apollo/client/link/http'
import { GraphQLWsLink } from '@apollo/client/link/subscriptions'
import { getMainDefinition } from '@apollo/client/utilities'
import { createClient } from 'graphql-ws'
import App from './app'

const baseUrl = (import.meta.env.VITE_API_BASE_URL as string | undefined) || 'http://localhost:3001'
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
