# User Query Consolidation Migration Guide

**Date**: 2025-10-26
**Status**: Implemented (Phase 3.1)

## Overview

The four separate user query methods (`me`, `user`, `user_by_username`, `user_by_email`) have been consolidated into a single flexible `find_user` query that accepts a `UserFilter` input type.

## Motivation

**Problem**: Query duplication with four separate methods doing similar work:
- `me(session_id: String)` - Get current user from session
- `user(id: Int)` - Get user by ID
- `user_by_username(username: String)` - Get user by username
- `user_by_email(email: String)` - Get user by email

**Benefits of consolidation**:
- Single query method with flexible filtering
- Easier to extend with new filter criteria
- Consistent pattern for future query consolidations
- Better GraphQL API discoverability
- Reduced code duplication

## What Changed

### New Query

```graphql
type Query {
  # New consolidated query
  find_user(filter: UserFilter!): User
}

input UserFilter {
  id: Int
  email: String
  username: String
  session_id: String
}
```

### Deprecated Queries

All old query methods are now deprecated but remain functional:

```graphql
type Query {
  me(session_id: String!): User @deprecated(reason: "Use find_user(filter: { session_id: \"...\" }) instead")
  user(id: Int!): User @deprecated(reason: "Use find_user(filter: { id: ... }) instead")
  user_by_username(username: String!): User @deprecated(reason: "Use find_user(filter: { username: \"...\" }) instead")
  user_by_email(email: String!): User @deprecated(reason: "Use find_user(filter: { email: \"...\" }) instead")
}
```

## Migration Examples

### Before and After

#### Get Current User (me query)

**Before:**
```graphql
query GetCurrentUser($sessionId: String!) {
  me(session_id: $sessionId) {
    id
    email
    username
    display_name
  }
}
```

**After:**
```graphql
query GetCurrentUser($sessionId: String!) {
  find_user(filter: { session_id: $sessionId }) {
    id
    email
    username
    display_name
  }
}
```

#### Get User by ID

**Before:**
```graphql
query GetUser($id: Int!) {
  user(id: $id) {
    id
    username
    display_name
  }
}
```

**After:**
```graphql
query GetUser($id: Int!) {
  find_user(filter: { id: $id }) {
    id
    username
    display_name
  }
}
```

#### Get User by Username

**Before:**
```graphql
query GetUserByUsername($username: String!) {
  user_by_username(username: $username) {
    id
    email
    display_name
  }
}
```

**After:**
```graphql
query GetUserByUsername($username: String!) {
  find_user(filter: { username: $username }) {
    id
    email
    display_name
  }
}
```

#### Get User by Email

**Before:**
```graphql
query GetUserByEmail($email: String!) {
  user_by_email(email: $email) {
    id
    username
    display_name
  }
}
```

**After:**
```graphql
query GetUserByEmail($email: String!) {
  find_user(filter: { email: $email }) {
    id
    username
    display_name
  }
}
```

## React/TypeScript Migration

### Apollo Client Hooks

**Before:**
```typescript
// Get current user
const { data } = useQuery(gql`
  query GetCurrentUser($sessionId: String!) {
    me(session_id: $sessionId) {
      id
      email
      username
    }
  }
`, { variables: { sessionId } });

// Get user by ID
const { data } = useQuery(gql`
  query GetUser($id: Int!) {
    user(id: $id) {
      id
      username
    }
  }
`, { variables: { id } });
```

**After:**
```typescript
// Get current user
const { data } = useQuery(gql`
  query GetCurrentUser($sessionId: String!) {
    find_user(filter: { session_id: $sessionId }) {
      id
      email
      username
    }
  }
`, { variables: { sessionId } });

// Get user by ID
const { data } = useQuery(gql`
  query GetUser($id: Int!) {
    find_user(filter: { id: $id }) {
      id
      username
    }
  }
`, { variables: { id } });
```

### Generated Types

After migration, regenerate TypeScript types:

```bash
npm run codegen
```

The generated `UserFilter` type will be:

```typescript
export type UserFilter = {
  id?: number | null;
  email?: string | null;
  username?: string | null;
  session_id?: string | null;
};
```

## Implementation Details

### Backend Implementation

The new `find_user` query in `layercake-core/src/graphql/queries/mod.rs`:

```rust
async fn find_user(&self, ctx: &Context<'_>, filter: UserFilter) -> Result<Option<User>> {
    let context = ctx.data::<GraphQLContext>()?;

    // Handle session_id lookup (equivalent to old 'me' query)
    if let Some(session_id) = filter.session_id {
        let session = user_sessions::Entity::find()
            .filter(user_sessions::Column::SessionId.eq(&session_id))
            .filter(user_sessions::Column::IsActive.eq(true))
            .one(&context.db)
            .await?;

        if let Some(session) = session {
            if session.expires_at > chrono::Utc::now() {
                let user = users::Entity::find_by_id(session.user_id)
                    .one(&context.db)
                    .await?;
                return Ok(user.map(User::from));
            }
        }
        return Ok(None);
    }

    // Build query based on provided filters
    let mut query = users::Entity::find();

    if let Some(id) = filter.id {
        query = query.filter(users::Column::Id.eq(id));
    }
    if let Some(email) = filter.email {
        query = query.filter(users::Column::Email.eq(email));
    }
    if let Some(username) = filter.username {
        query = query.filter(users::Column::Username.eq(username));
    }

    let user = query.one(&context.db).await?;
    Ok(user.map(User::from))
}
```

### Backward Compatibility

Old query methods delegate to `find_user`:

```rust
#[graphql(deprecation = "Use find_user(filter: { id: ... }) instead")]
async fn user(&self, ctx: &Context<'_>, id: i32) -> Result<Option<User>> {
    self.find_user(ctx, UserFilter {
        id: Some(id),
        email: None,
        username: None,
        session_id: None
    }).await
}
```

This ensures:
- No breaking changes for existing clients
- Gradual migration path
- Clear deprecation warnings in GraphQL schema

## Filter Behavior

### Single Filter

Only one filter field should be provided at a time. If multiple filters are provided, they will be combined with AND logic:

```graphql
# ⚠️ This will only match users with BOTH id=42 AND username="john"
query {
  find_user(filter: { id: 42, username: "john" }) {
    id
    username
  }
}
```

**Best practice**: Use only one filter field per query.

### Session ID Priority

If `session_id` is provided, it takes priority over other filters:

```graphql
# This will use session_id lookup, ignoring the id field
query {
  find_user(filter: { session_id: "abc123", id: 42 }) {
    id
  }
}
```

## Migration Timeline

| Phase | Timeline | Tasks |
|-------|----------|-------|
| **Phase 1: Implementation** | Week 1 | ✅ Implement `find_user` query<br>✅ Add deprecation to old queries<br>✅ Update documentation |
| **Phase 2: Frontend Migration** | Week 2-3 | Update GraphQL queries to use `find_user`<br>Regenerate TypeScript types<br>Test all user lookup flows |
| **Phase 3: Monitoring** | Week 4-6 | Monitor usage of deprecated queries<br>Track migration progress<br>Support teams with migration |
| **Phase 4: Cleanup** | Week 7+ | Remove deprecated queries<br>Update final documentation |

## Testing

### Backend Tests

```rust
#[tokio::test]
async fn test_find_user_by_id() {
    let result = schema.execute(
        r#"query { find_user(filter: { id: 1 }) { id username } }"#
    ).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_find_user_by_session_id() {
    let result = schema.execute(
        r#"query { find_user(filter: { session_id: "test-session" }) { id } }"#
    ).await;
    assert!(result.is_ok());
}
```

### Frontend Tests

```typescript
describe('find_user query', () => {
  it('should find user by id', async () => {
    const { data } = await client.query({
      query: FIND_USER_QUERY,
      variables: { filter: { id: 1 } }
    });
    expect(data.find_user).toBeDefined();
  });

  it('should find user by session_id', async () => {
    const { data } = await client.query({
      query: FIND_USER_QUERY,
      variables: { filter: { session_id: 'test-session' } }
    });
    expect(data.find_user).toBeDefined();
  });
});
```

## Future Enhancements

Potential improvements to the filter system:

1. **Multiple Users**: Add a `find_users` query that returns a list
2. **Advanced Filters**: Add filters for `is_active`, `created_after`, etc.
3. **Sorting**: Add sorting options to filter input
4. **Pagination**: Integrate with pagination system when implemented

---

**Status**: Implemented and ready for frontend migration
**Files Modified**:
- `layercake-core/src/graphql/types/user.rs` - Added `UserFilter` input type
- `layercake-core/src/graphql/queries/mod.rs` - Added `find_user` query, deprecated old queries
