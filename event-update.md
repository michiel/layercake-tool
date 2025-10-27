# WebSocket and GraphQL Subscription Coordination Review

**Date:** 2025-10-27
**Reviewer:** Architecture Analysis
**Focus:** Event update, websocket, and GraphQL subscription coordination and broadcast mechanisms

## Executive Summary

The layercake-tool implements a dual-channel real-time collaboration system using both WebSocket connections (for user presence/cursors) and GraphQL subscriptions (for Plan DAG updates). The architecture demonstrates solid patterns with actor-based concurrency and broadcast channels, but has several areas of concern regarding duplication, missing cleanup, and potential resource leaks.

---

## Architecture Overview

### System Components

1. **WebSocket Layer** (`server/websocket/`)
   - Real-time user presence and cursor updates
   - Actor-based coordination via `CollaborationCoordinator` and `ProjectActor`
   - Direct mpsc channels for client-to-server communication

2. **GraphQL Subscriptions** (`graphql/subscriptions/`)
   - Plan DAG updates (nodes, edges, metadata)
   - Execution status events
   - Delta events (JSON Patch format)
   - Uses global `EventBroadcaster` with tokio broadcast channels

3. **Event Broadcasting** (`utils/event_broadcaster.rs`)
   - Generic pub/sub broadcaster using tokio broadcast channels
   - Thread-safe with RwLock
   - Automatic channel creation
   - Manual cleanup via `cleanup_idle()`

4. **Collaboration Actors** (`collaboration/`)
   - Single-threaded coordinator managing multiple project actors
   - Each project runs in isolated actor with HashMap state
   - Command-based messaging with oneshot response channels

---

## Critical Issues Identified

### 🔴 HIGH PRIORITY

#### 1. **Duplicate Collaboration Systems**

**Location:** `session.rs` vs `project_actor.rs` + `coordinator.rs`

**Issue:** Two complete, parallel implementations of user presence management:

- **SessionManager** (`session.rs`, 418 lines): DashMap-based, appears unused (`#[allow(dead_code)]`)
- **ProjectActor + CollaborationCoordinator** (`project_actor.rs` + `coordinator.rs`, ~650 lines): Actor-based, actively used

**Evidence:**
```rust
// session.rs - Line 12 (appears unused)
pub struct SessionManager {
    state: CollaborationState,
}

// project_actor.rs - Line 14 (actively used)
pub struct ProjectActor {
    project_id: i32,
    command_tx: mpsc::Sender<ProjectCommand>,
    task_handle: tokio::task::JoinHandle<()>,
}
```

**Impact:**
- ~1000 lines of duplicate code
- Maintenance burden
- Confusion about which system is canonical
- Dead code in production builds

**Recommendation:** Remove `session.rs` entirely or document why both exist.

---

#### 2. **Missing Periodic Cleanup Tasks**

**Location:** `event_broadcaster.rs`, `project_actor.rs`, `session.rs`

**Issue:** Cleanup methods exist but are never scheduled:

- `EventBroadcaster::cleanup_idle()` - removes empty broadcast channels (line 133)
- `SessionManager::cleanup_inactive_sessions()` - removes stale users (line 348)
- No scheduled cleanup task in server initialization

**Evidence:**
```rust
// event_broadcaster.rs:133
/// Remove all channels with no active receivers.
/// This should be called periodically (e.g., every minute)
pub async fn cleanup_idle(&self) -> usize { ... }
```

**Impact:**
- Memory leaks from abandoned broadcast channels
- Stale user presence data accumulating
- Performance degradation over time

**Recommendation:** Add background cleanup task in `server/app.rs`.

---

#### 3. **Inconsistent Error Handling in Broadcasts**

**Location:** `subscriptions/mod.rs`, `project_actor.rs`

**Issue:** Mixed patterns for handling broadcast failures:

- GraphQL subscriptions: Log and continue on `RecvError::Lagged` (lines 237-246)
- WebSocket broadcasts: Silent `let _ =` error suppression (line 424)
- Dead connection detection inconsistent

**Evidence:**
```rust
// subscriptions/mod.rs:237 - Good pattern
Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
    tracing::warn!("Broadcast receiver lagged for plan {}, skipped {} messages", ...);
    continue;
}

// project_actor.rs:424 - Silent failure
if let Err(_) = connection.send(message.clone()).await {
    warn!("Dead connection detected..."); // warns but doesn't clean up
}
```

**Impact:**
- Silent message loss
- Inconsistent client experience
- Difficult debugging

**Recommendation:** Standardise error handling pattern across both systems.

---

### ⚠️ MEDIUM PRIORITY

#### 4. **Lack of Backpressure Mechanisms**

**Location:** `handler.rs`, `project_actor.rs`

**Issue:** Fixed-size channels without backpressure handling:

- WebSocket handler: 100-message bounded channel (line 50)
- Rate limiter: 20 msg/sec but no coordinated backpressure (line 85)
- No guidance for slow consumers

**Evidence:**
```rust
// handler.rs:50
let (tx, mut rx) = mpsc::channel::<ServerMessage>(100); // Fixed buffer

// handler.rs:85
let mut rate_limiter = RateLimiter::new(20, std::time::Duration::from_secs(1));
```

**Impact:**
- Clients can be silently dropped when buffers fill
- No flow control feedback to publishers
- Unpredictable behaviour under load

**Recommendation:** Implement bounded queue monitoring and adaptive backpressure.

---

#### 5. **GraphQL Subscription Lag Detection Without Recovery**

**Location:** `subscriptions/mod.rs`

**Issue:** Lag detection warns but doesn't help clients recover:

- Detects when subscribers fall behind (lines 237-246, 281-286, 320-325)
- No mechanism to request missing events
- Clients don't know they're out of sync

**Evidence:**
```rust
// subscriptions/mod.rs:237-243
Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
    tracing::warn!("Broadcast receiver lagged for plan {}, skipped {} messages", ...);
    continue; // Just skip and hope
}
```

**Impact:**
- Clients silently lose events during high load
- No recovery mechanism
- Data inconsistency between clients

**Recommendation:** Add sequence numbers and resync protocol.

---

#### 6. **Actor Lifecycle Management Gaps**

**Location:** `coordinator.rs`, `project_actor.rs`

**Issue:** ProjectActors spawned but cleanup is passive:

- Projects removed only when explicitly empty (line 65-68)
- No timeout for inactive projects
- No maximum lifetime limits
- Potential for long-lived actors consuming resources

**Evidence:**
```rust
// coordinator.rs:65-68
if project.is_empty().await {
    debug!("Project {} is empty, removing", project_id);
    self.projects.remove(&project_id);
}
```

**Impact:**
- Resource accumulation for rarely-accessed projects
- No predictable lifecycle

**Recommendation:** Add idle timeout and maximum lifetime policies.

---

### ℹ️ LOW PRIORITY

#### 7. **Broadcasting to Single User**

**Location:** `project_actor.rs:453`

**Issue:** Cursor updates broadcast a single-user vector:

```rust
// project_actor.rs:477-482
let message = ServerMessage::DocumentActivity {
    data: DocumentActivityData {
        document_id: document_id.to_string(),
        active_users: vec![doc_user], // Single user in vector
    },
};
```

**Impact:** Minor inefficiency, semantically confusing.

**Recommendation:** Consider dedicated `CursorUpdate` message type.

---

#### 8. **Missing Metrics and Observability**

**Location:** Throughout

**Issue:** No instrumentation for:

- Message throughput
- Lag frequency/duration
- Connection lifetime statistics
- Channel buffer utilisation
- Error rates

**Impact:** Difficult to diagnose production issues.

**Recommendation:** Add structured metrics using `tracing` spans.

---

#### 9. **SessionManager Historical Artifact**

**Location:** `session.rs`

**Issue:** Previous DashMap-based implementation with deadlock fixes (lines 176-185, 244-253):

```rust
// session.rs:176-185
// CRITICAL FIX: Collect all connections into a Vec FIRST to avoid holding
// DashMap iterator while performing async operations.
```

This suggests a prior architecture that had serious concurrency issues, now superseded by actor model.

**Impact:** Code archaeology overhead.

**Recommendation:** Remove or move to `examples/` as reference.

---

## Efficiency Concerns

### Positive Patterns

1. **Actor Model**: Single-threaded actors eliminate lock contention
2. **Double-Checked Locking**: EventBroadcaster uses optimal read/write lock pattern (line 153)
3. **Lag Detection**: Proactive monitoring of subscriber health
4. **Bounded Channels**: Prevents runaway memory growth

### Inefficiencies

1. **Clone-Heavy Broadcasting**: Messages cloned for each recipient
   - `project_actor.rs:424, 449, 488, 524` - multiple `.clone()` calls per broadcast

2. **Redundant Plan ID Filtering**: Subscribers filter by plan_id after receiving all events
   - `subscriptions/mod.rs:126, 233` - filter in subscriber instead of channel-per-plan

3. **Instant Formatting**: Repeated conversions in hot paths
   - `project_actor.rs:529` - formats on every broadcast

4. **HashMap Iterations**: Collecting presence data iterates multiple times
   - `project_actor.rs:355-384` - could be cached/memoised

---

## Gaps and Missing Features

### 1. **Reconnection Support**
- No session persistence across disconnects
- No resumption tokens or sequence numbers
- Clients must re-establish full state

### 2. **Authentication Integration**
- WebSocket handler has TODO for JWT validation (line 27)
- No authorisation checks on subscriptions
- Project membership not verified

### 3. **Rate Limiting Coordination**
- Per-connection rate limiter doesn't coordinate with global load
- No circuit breakers
- No adaptive throttling

### 4. **Connection Draining**
- Shutdown sends error but doesn't wait for graceful close
- No connection draining period

### 5. **Health Checks**
- `ProjectHealthReport` exists but not exposed via API
- No readiness/liveness probes for actor system

### 6. **Document Type Handling**
- Document type defaults to Canvas (line 260)
- No validation of document type transitions
- Cursor position validation is type-specific but not enforced at creation

---

## Risks

### Concurrency Risks

1. **EventBroadcaster Leak**: Without periodic cleanup, abandoned channels accumulate
   - **Severity:** High
   - **Likelihood:** Certain over time

2. **Actor Task Leak**: ProjectActor tasks might not terminate cleanly
   - **Severity:** Medium
   - **Likelihood:** Low (oneshot cleanup exists)

3. **Broadcast Lag Cascade**: One slow subscriber can fill broadcast buffers, causing others to lag
   - **Severity:** Medium
   - **Likelihood:** High under load

### Data Consistency Risks

1. **Event Loss**: Lagged subscribers silently skip events
   - **Severity:** High
   - **Likelihood:** Medium during bursts

2. **Partial State**: Joining clients get current presence but not full DAG history
   - **Severity:** Low (expected for stateless join)
   - **Likelihood:** N/A

### Security Risks

1. **Unauthenticated WebSockets**: TODO indicates missing authentication
   - **Severity:** High
   - **Likelihood:** Current state

2. **Plan ID Spoofing**: No verification that user can access plan_id
   - **Severity:** High
   - **Likelihood:** Easy to exploit

3. **Resource Exhaustion**: No global connection limits
   - **Severity:** Medium
   - **Likelihood:** Medium

---

## Recommendations and Implementation Plan

### Phase 1: Critical Fixes (1-2 weeks)

#### 1.1 Remove Duplicate Code
- **Action:** Delete `session.rs` and `websocket/types.rs:CollaborationState`
- **Files:** `layercake-core/src/server/websocket/session.rs`
- **Testing:** Ensure no imports remain; run full test suite
- **Risk:** Low (marked as `dead_code`)

#### 1.2 Implement Periodic Cleanup
- **Action:** Add background cleanup task to `server/app.rs`
- **Code:**
  ```rust
  // In create_app():
  tokio::spawn(async move {
      let mut interval = tokio::time::interval(Duration::from_secs(60));
      loop {
          interval.tick().await;
          let cleaned = COLLABORATION_EVENTS.cleanup_idle().await;
          let cleaned_delta = DELTA_EVENTS.cleanup_idle().await;
          let cleaned_exec = EXECUTION_STATUS_EVENTS.cleanup_idle().await;
          if cleaned + cleaned_delta + cleaned_exec > 0 {
              tracing::info!("Cleaned {} idle broadcast channels", cleaned + cleaned_delta + cleaned_exec);
          }
      }
  });
  ```
- **Testing:** Monitor metrics over 24 hours in staging
- **Risk:** Low

#### 1.3 Standardise Broadcast Error Handling
- **Action:** Create helper method for broadcast with dead connection cleanup
- **Location:** `project_actor.rs`
- **Code:**
  ```rust
  async fn broadcast_with_cleanup(
      &mut self,
      message: ServerMessage,
      exclude_user: Option<&str>,
  ) -> Vec<String> {
      let mut dead_connections = Vec::new();
      for (user_id, connection) in &self.connections {
          if exclude_user.is_some_and(|u| u == user_id) {
              continue;
          }
          if let Err(_) = connection.send(message.clone()).await {
              tracing::warn!("Dead connection for user {}", user_id);
              dead_connections.push(user_id.clone());
          }
      }
      // Clean up after iteration complete
      for user_id in &dead_connections {
          self.connections.remove(user_id);
          self.users.remove(user_id);
      }
      dead_connections
  }
  ```
- **Testing:** Integration tests with forced disconnections
- **Risk:** Low

### Phase 2: Security and Robustness (2-3 weeks)

#### 2.1 Implement WebSocket Authentication
- **Action:** Complete JWT validation in `handler.rs:27`
- **Dependencies:** JWT library selection and key management
- **Testing:** Unit tests for token validation, integration tests for auth flow
- **Risk:** Medium (authentication changes)

#### 2.2 Add Subscription Authorisation
- **Action:** Verify plan/project membership in subscription resolvers
- **Files:** `graphql/subscriptions/mod.rs:113, 217, 262, 301`
- **Testing:** Authorisation matrix tests
- **Risk:** Medium

#### 2.3 Implement Backpressure Monitoring
- **Action:** Add channel fullness metrics and slow consumer detection
- **Code:**
  ```rust
  // In handler.rs, monitor send failures:
  match tx.try_send(msg) {
      Ok(_) => {},
      Err(TrySendError::Full(_)) => {
          tracing::warn!("Client {} receive buffer full", user_id);
          metrics::increment_counter!("ws.buffer_full");
      }
      Err(TrySendError::Closed(_)) => break,
  }
  ```
- **Testing:** Load testing with slow consumers
- **Risk:** Low

### Phase 3: Reliability Improvements (3-4 weeks)

#### 3.1 Add Sequence Numbers and Resync
- **Action:** Add `sequence_id` to all subscription events
- **Protocol:**
  - Server includes `seq_id` in each event
  - Client requests resync with `last_received_seq`
  - Server replays missing events or sends full state
- **Files:** `graphql/subscriptions/mod.rs`, `graphql/types/*.rs`
- **Testing:** Chaos testing with forced lags
- **Risk:** High (protocol change)

#### 3.2 Implement Actor Lifecycle Policies
- **Action:** Add idle timeout (30 min) and max lifetime (24h) for ProjectActors
- **Code:**
  ```rust
  // In ProjectState::run():
  let mut idle_timer = tokio::time::interval(Duration::from_secs(1800)); // 30 min
  let max_lifetime = Instant::now() + Duration::from_secs(86400); // 24h

  loop {
      tokio::select! {
          Some(cmd) = command_rx.recv() => { /* handle */ }
          _ = idle_timer.tick() => {
              if self.connections.is_empty() {
                  tracing::info!("Project {} idle, shutting down", self.project_id);
                  break;
              }
          }
          _ = tokio::time::sleep_until(max_lifetime.into()) => {
              tracing::info!("Project {} reached max lifetime", self.project_id);
              break;
          }
      }
  }
  ```
- **Testing:** Time-accelerated tests
- **Risk:** Medium

#### 3.3 Add Structured Metrics
- **Action:** Integrate `metrics` crate with existing `tracing`
- **Metrics:**
  - `ws.connections.active` (gauge)
  - `ws.messages.sent` (counter)
  - `ws.buffer_full` (counter)
  - `graphql.subscription.lag` (counter)
  - `broadcast.channels.idle_cleaned` (counter)
- **Testing:** Verify metrics in Prometheus/Grafana
- **Risk:** Low

### Phase 4: Optimisations (2-3 weeks)

#### 4.1 Channel-Per-Plan Broadcasting
- **Action:** Refactor EventBroadcaster to use separate channels per plan
- **Current:** Single channel, subscribers filter by plan_id
- **Proposed:** EventBroadcaster already supports this via key type
- **Impact:** Eliminate redundant message delivery
- **Testing:** Benchmark multi-plan scenarios
- **Risk:** Low (already supported by EventBroadcaster design)

#### 4.2 Message Pooling
- **Action:** Use `Arc<Message>` instead of cloning for broadcasts
- **Code:**
  ```rust
  // In project_actor.rs:
  let message = Arc::new(ServerMessage::DocumentActivity { ... });
  for connection in self.connections.values() {
      let msg_ref = Arc::clone(&message);
      connection.send((*msg_ref).clone()).await; // Only clone on send
  }
  ```
- **Testing:** Memory profiling
- **Risk:** Low

#### 4.3 Presence Data Caching
- **Action:** Cache `collect_presence_data()` result, invalidate on changes
- **Impact:** Reduce redundant iterations for broadcasts
- **Testing:** Benchmark presence broadcasts
- **Risk:** Medium (cache invalidation complexity)

---

## Summary Tables

### Issues by Severity

| Severity | Count | Examples |
|----------|-------|----------|
| High | 3 | Duplicate systems, missing cleanup, security gaps |
| Medium | 4 | Backpressure, lag recovery, actor lifecycle, auth |
| Low | 3 | Inefficiencies, metrics, code cleanup |

### Implementation Effort

| Phase | Duration | Effort | Dependencies |
|-------|----------|--------|--------------|
| Phase 1 | 1-2 weeks | 3 developer-weeks | None |
| Phase 2 | 2-3 weeks | 6 developer-weeks | JWT library selection |
| Phase 3 | 3-4 weeks | 8 developer-weeks | Phase 2 complete |
| Phase 4 | 2-3 weeks | 5 developer-weeks | Phase 3 complete |
| **Total** | **8-12 weeks** | **22 developer-weeks** | |

### Risk Matrix

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| EventBroadcaster leak | High | High | Phase 1.2 cleanup task |
| Unauthenticated access | High | High | Phase 2.1/2.2 auth |
| Event loss during lag | Medium | High | Phase 3.1 resync protocol |
| Actor resource accumulation | Medium | Medium | Phase 3.2 lifecycle |
| Broadcast lag cascade | Medium | Medium | Phase 2.3 backpressure |

---

## Testing Strategy

### Unit Tests
- EventBroadcaster cleanup behaviour
- Rate limiter accuracy
- Actor command handling
- Message serialisation

### Integration Tests
- WebSocket connection lifecycle
- GraphQL subscription lag scenarios
- Dead connection cleanup
- Multi-user collaboration flows

### Load Tests
- 1000 concurrent connections per project
- Burst message patterns (100 msg/sec)
- Slow consumer scenarios
- Memory leak detection over 24h

### Chaos Tests
- Random disconnections
- Network delays
- Actor crashes
- Broadcast channel saturation

---

## Conclusion

The system demonstrates good architectural patterns (actor model, broadcast channels, double-checked locking) but has critical gaps in cleanup, security, and reliability. The dual WebSocket/GraphQL subscription approach is sound but needs cleanup to remove legacy code.

**Priority Actions:**
1. Remove duplicate `SessionManager` code
2. Implement periodic cleanup tasks
3. Add authentication and authorisation
4. Implement lag recovery mechanisms

The proposed 4-phase plan addresses issues in order of criticality, with Phase 1 providing immediate stability improvements and subsequent phases adding robustness and optimisation.

**Estimated Total Effort:** 22 developer-weeks over 8-12 calendar weeks.
