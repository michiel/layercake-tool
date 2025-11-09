# GraphQL Error Handling

## Issue

With Apollo Client's `errorPolicy: 'all'` configuration, GraphQL mutations don't throw errors when they encounter validation or server errors. Instead, errors are returned in the `result.errors` array. This means traditional try-catch blocks won't catch these errors, and users won't see error notifications.

## Solution

### Using the `handleMutationErrors` Helper

A utility function `handleMutationErrors` has been created to properly handle GraphQL errors with `errorPolicy: 'all'`.

**Location**: `frontend/src/utils/graphqlHelpers.ts`

### Usage Pattern

**Before (Incorrect - won't show errors):**
```typescript
try {
  await mutation({
    variables: { ... }
  })
  showSuccessNotification('Success!')
} catch (error) {
  // This never executes with errorPolicy: 'all'
  showErrorNotification('Failed', extractGraphQLErrorMessage(error))
}
```

**After (Correct):**
```typescript
import { handleMutationErrors } from '../utils/graphqlHelpers'

const result = await mutation({
  variables: { ... }
})

if (handleMutationErrors(result, 'Operation failed')) {
  return // Error was shown to user
}

// Success path
showSuccessNotification('Success!')
```

### How It Works

The helper function:
1. Checks if `result.errors` exists and contains errors
2. Extracts the error message using `extractGraphQLErrorMessage`
3. Displays an error notification using `showErrorNotification`
4. Returns `true` if there were errors, `false` otherwise

### Files Updated

The following files have been updated to use this pattern:
- `frontend/src/pages/DataAcquisitionPage.tsx`
- `frontend/src/pages/SourceManagementPage.tsx`

### Why `errorPolicy: 'all'`?

This policy allows partial data to be returned alongside errors, which is useful for:
- Displaying partial results when some fields fail
- Better user experience in complex queries
- Maintaining consistency across the application

However, it requires explicit error checking in mutation handlers rather than relying on try-catch blocks.

## Guidelines for New Code

When writing GraphQL mutations:

1. **Always** check mutation results for errors using `handleMutationErrors`
2. **Don't** rely on try-catch blocks for GraphQL errors (they won't fire)
3. **Do** use try-catch for non-GraphQL errors (file reading, network timeouts, etc.)

### Example

```typescript
import { useMutation } from '@apollo/client/react'
import { handleMutationErrors } from '../utils/graphqlHelpers'
import { showSuccessNotification } from '../utils/notifications'

const MyComponent = () => {
  const [doSomething] = useMutation(DO_SOMETHING_MUTATION)

  const handleAction = async () => {
    const result = await doSomething({
      variables: { id: 123 }
    })

    if (handleMutationErrors(result, 'Failed to perform action')) {
      return
    }

    showSuccessNotification('Action completed successfully')
  }

  return <Button onClick={handleAction}>Do Something</Button>
}
```
