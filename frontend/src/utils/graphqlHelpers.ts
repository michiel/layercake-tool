import { FetchResult } from '@apollo/client'
import { extractGraphQLErrorMessage } from './errorHandling'
import { showErrorNotification } from './notifications'

/**
 * Check if a GraphQL mutation result contains errors and show a notification if so.
 *
 * With errorPolicy: 'all', mutations don't throw errors. This helper checks the result
 * for errors and displays them appropriately.
 *
 * @param result The mutation result from Apollo Client
 * @param errorTitle The title to show in the error notification
 * @returns true if there were errors, false otherwise
 *
 * @example
 * const result = await createProject({ variables: { name: 'Test' } })
 * if (handleMutationErrors(result, 'Project creation failed')) {
 *   return // Don't proceed if there were errors
 * }
 * // Success path...
 */
export function handleMutationErrors<T>(
  result: FetchResult<T>,
  errorTitle: string
): boolean {
  console.log('[handleMutationErrors] Checking result:', result)
  console.log('[handleMutationErrors] Has errors?', result.errors?.length)

  if (result.errors && result.errors.length > 0) {
    const message = extractGraphQLErrorMessage({ graphQLErrors: result.errors })
    console.log('[handleMutationErrors] Extracted message:', message)
    showErrorNotification(errorTitle, message)
    return true
  }
  return false
}
