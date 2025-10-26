# Deprecated Code Tracking

**Date**: 2025-10-26
**Status**: Active Tracking (Phase 3.5)

## Overview

This document tracks all deprecated GraphQL API endpoints, their replacement, deprecation date, and planned removal date. This ensures a smooth migration path for clients.

## Deprecation Policy

**Migration Period**: 6 months minimum between deprecation and removal

**Process**:
1. Mark item with `#[graphql(deprecation = "...")]` directive
2. Document in this file with deprecation date
3. Update migration guides
4. Monitor usage metrics (if available)
5. After migration period, plan removal in major version
6. Remove and document as breaking change

## Currently Deprecated

### Mutations

#### updatePlanDag (Deprecated: 2025-10-26)

**Status**: 🟡 Deprecated
**Deprecated In**: Phase 1.1
**Removal Target**: v2.0.0 (6+ months)

**Reason**: Bulk replace operation conflicts with delta-based real-time updates and collaboration features.

**Replacement**:
Use individual delta operations:
- `addPlanDagNode`
- `updatePlanDagNode`
- `deletePlanDagNode`
- `addPlanDagEdge`
- `deletePlanDagEdge`
- `updatePlanDagEdge`
- `movePlanDagNode`
- `batchMovePlanDagNodes`

**Migration Guide**: `docs/DEPRECATED_MUTATIONS_MIGRATION.md`

**Usage**: Unknown (no metrics yet)

**Action Items**:
- [ ] Add usage metrics to track how often this mutation is called
- [ ] Create frontend migration PR
- [ ] Test with all clients
- [ ] Set firm removal date after 6 months

---

### Queries

#### me(sessionId: String)

**Status**: 🟡 Deprecated
**Deprecated In**: Phase 3.1
**Removal Target**: v2.0.0 (6+ months)

**Reason**: Query consolidation - multiple queries doing similar work

**Replacement**:
```graphql
findUser(filter: { sessionId: "..." })
```

**Migration Guide**: `docs/USER_QUERY_MIGRATION.md`

**Usage**: Unknown (no metrics yet)

---

#### user(id: Int)

**Status**: 🟡 Deprecated
**Deprecated In**: Phase 3.1
**Removal Target**: v2.0.0 (6+ months)

**Reason**: Query consolidation

**Replacement**:
```graphql
findUser(filter: { id: 123 })
```

**Migration Guide**: `docs/USER_QUERY_MIGRATION.md`

**Usage**: Unknown (no metrics yet)

---

#### userByUsername(username: String)

**Status**: 🟡 Deprecated
**Deprecated In**: Phase 3.1
**Removal Target**: v2.0.0 (6+ months)

**Reason**: Query consolidation

**Replacement**:
```graphql
findUser(filter: { username: "john" })
```

**Migration Guide**: `docs/USER_QUERY_MIGRATION.md`

**Usage**: Unknown (no metrics yet)

---

#### userByEmail(email: String)

**Status**: 🟡 Deprecated
**Deprecated In**: Phase 3.1
**Removal Target**: v2.0.0 (6+ months)

**Reason**: Query consolidation

**Replacement**:
```graphql
findUser(filter: { email: "john@example.com" })
```

**Migration Guide**: `docs/USER_QUERY_MIGRATION.md`

**Usage**: Unknown (no metrics yet)

---

## Previously Removed

### User Presence System (Removed: 2025-10-XX)

**Status**: ✅ Removed
**Removed In**: Pre-Phase 1

**What Was Removed**:
- Mutations:
  - `updateUserPresence`
  - `userOffline`
  - `presenceHeartbeat`
  - `updateCursorPosition`
- Queries:
  - `projectOnlineUsers`
  - `userPresence`
- Subscriptions:
  - `userPresenceChanged`
- Types:
  - `UserPresence`
  - `UserPresenceInfo`
  - `CursorPosition`
  - `UpdateUserPresenceInput`

**Reason**: Migrated to WebSocket-only implementation for better real-time performance

**Replacement**: WebSocket collaboration system at `/ws/collaboration`

**Migration**: Automatic - frontend switched to WebSocket connections

---

### Dead Code (Removed: 2025-10-XX)

**Status**: ✅ Removed
**Removed In**: Pre-Phase 1

**What Was Removed**:
- `types/node.rs` module (unused GraphQL types)
- `types/edge.rs` module (unused GraphQL types)

**Reason**: Code was never used in production

---

## Removal Checklist

When removing deprecated code after migration period:

**Pre-Removal**:
- [ ] Verify migration period has passed (6+ months minimum)
- [ ] Check usage metrics (confirm <5% usage)
- [ ] Announce removal in release notes
- [ ] Update CHANGELOG.md with breaking changes
- [ ] Coordinate with frontend team

**Removal**:
- [ ] Remove `#[graphql(deprecation)]` attribute
- [ ] Delete deprecated function/type
- [ ] Update tests
- [ ] Update documentation
- [ ] Move entry from "Currently Deprecated" to "Previously Removed" in this file

**Post-Removal**:
- [ ] Verify schema introspection doesn't show removed items
- [ ] Test with GraphQL clients
- [ ] Monitor error logs for attempts to use removed endpoints
- [ ] Update API documentation website

## Versioning Strategy

**Current Version**: v1.x

**Deprecation Rules**:
- Deprecate in minor versions (v1.1, v1.2, etc.)
- Remove in major versions (v2.0, v3.0, etc.)
- Maintain backwards compatibility within major versions

**Version Plan**:
- **v1.x**: Current version with deprecated endpoints still functional
- **v2.0**: Remove all items deprecated before 2025-04-26
  - Remove `updatePlanDag` mutation
  - Remove old user queries (`me`, `user`, `userByUsername`, `userByEmail`)
- **v3.0**: Future breaking changes TBD

## Usage Monitoring

### Recommended Implementation

Add GraphQL middleware to track deprecated endpoint usage:

```rust
use async_graphql::*;

struct DeprecationMetrics;

impl Extension for DeprecationMetrics {
    async fn parse_query(&self, ctx: &ExtensionContext<'_>, ...） {
        // Track which queries/mutations are used
        // Log usage of deprecated endpoints
        // Send metrics to monitoring system
    }
}
```

**Metrics to Track**:
- Request count per deprecated endpoint
- Unique clients using deprecated endpoints
- Trend over time (increasing or decreasing usage)

**Alerts**:
- Alert if deprecated endpoint usage increases
- Alert if removal date approaching and usage still > 5%

## Client Migration Tracking

### Known Clients

1. **Frontend (React)**
   - Repository: `frontend/`
   - Owner: Frontend Team
   - Migration Status:
     - updatePlanDag: ⏳ Not Started
     - Old user queries: ⏳ Not Started

2. **Desktop (Tauri)**
   - Repository: `src-tauri/`
   - Owner: Desktop Team
   - Migration Status:
     - updatePlanDag: ⏳ Not Started
     - Old user queries: ⏳ Not Started

3. **MCP Integration**
   - Repository: `external-modules/axum-mcp/`
   - Owner: Backend Team
   - Migration Status: ⏳ Not Reviewed

### Migration Coordination

**Process**:
1. Create GitHub issues for each client
2. Assign to team owners
3. Link to migration guides
4. Set deadline (3 months before removal)
5. Track progress in this document
6. Review PRs together

## Communication Plan

### When Deprecating

**Channels**:
- CHANGELOG.md entry
- GitHub release notes
- API documentation update
- Email to API users (if available)
- Slack notification (internal teams)

**Message Template**:
```
⚠️ DEPRECATION NOTICE

The following endpoints are deprecated as of v1.X:

- `me()` query → Use `findUser(filter: { sessionId })` instead
- See migration guide: docs/USER_QUERY_MIGRATION.md

These endpoints will be removed in v2.0 (estimated: YYYY-MM-DD)
Please migrate your code before then.
```

### Before Removing

**Timeline**: 1 month before removal

**Channels**:
- CHANGELOG.md with breaking changes section
- GitHub issue tracking removal
- Direct email to active API users
- Slack announcement
- API documentation prominent warning

**Message Template**:
```
🚨 BREAKING CHANGE IN v2.0

The following deprecated endpoints will be REMOVED in v2.0 (releasing: YYYY-MM-DD):

- `me()` query
- `user()` query
- `userByUsername()` query
- `userByEmail()` query

Action Required: Migrate to `findUser(filter)` before upgrading to v2.0

Migration guide: docs/USER_QUERY_MIGRATION.md
```

## Future Deprecations (Planned)

### Phase 4: After Authorization Implementation

**Candidates for Deprecation**:
- Any unprotected queries/mutations that should require auth
- Overly permissive queries without proper filtering

**Timeline**: TBD after Phase 4 completion

---

**Last Updated**: 2025-10-26
**Next Review**: 2025-11-26
**Maintainer**: Backend Team
