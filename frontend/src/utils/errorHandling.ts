const GRAPHQL_PREFIX = /^GraphQL error:\s*/i

const extractFirstGraphQLError = (error: any) => {
  if (!error) return undefined

  if (Array.isArray(error.graphQLErrors) && error.graphQLErrors.length > 0) {
    return error.graphQLErrors[0]
  }

  const networkGraphQLErrors = error.networkError?.result?.errors
  if (Array.isArray(networkGraphQLErrors) && networkGraphQLErrors.length > 0) {
    return networkGraphQLErrors[0]
  }

  return undefined
}

export const isGraphQLError = (error: unknown): boolean => {
  if (!error || typeof error !== 'object') {
    return false
  }

  const typed = error as any
  if (Array.isArray(typed.graphQLErrors) && typed.graphQLErrors.length > 0) {
    return true
  }

  const networkGraphQLErrors = typed.networkError?.result?.errors
  if (Array.isArray(networkGraphQLErrors) && networkGraphQLErrors.length > 0) {
    return true
  }

  return false
}

export const extractGraphQLErrorMessage = (error: unknown): string => {
  if (!error) {
    return 'An unknown error occurred.'
  }

  const typed = error as { message?: string }
  if (typed.message && typed.message.trim().length > 0) {
    return typed.message.replace(GRAPHQL_PREFIX, '').trim()
  }

  const graphQLError = extractFirstGraphQLError(error)
  if (graphQLError?.message) {
    const message = graphQLError.message.replace(GRAPHQL_PREFIX, '').trim()

    if (graphQLError.extensions?.field) {
      return `${graphQLError.extensions.field}: ${message}`
    }

    const pathLabel = Array.isArray(graphQLError.path)
      ? graphQLError.path.join('.')
      : null

    if (pathLabel) {
      return `${pathLabel}: ${message}`
    }

    return message
  }

  if (typeof error === 'string' && error.trim().length > 0) {
    return error.trim()
  }

  return 'An unknown error occurred.'
}
