# Security Review: Layercake Tool

**Review Date:** 2026-01-02
**Reviewer:** Security Assessment
**Codebase Version:** v0.3.7
**Scope:** Rust backend (layercake-core, layercake-server, layercake-cli), GraphQL API, authentication/authorization mechanisms

---

## Executive Summary

### Overall Risk Score: **MEDIUM-HIGH**

This security review identified several **CRITICAL** and **HIGH** priority security vulnerabilities in the Layercake application that require immediate attention. The application implements authentication and authorization mechanisms, but contains significant weaknesses that could allow unauthorized access, privilege escalation, and data exposure.

### Key Findings Summary

- **Critical Issues:** 3 findings requiring immediate remediation
- **High-Priority Issues:** 5 findings requiring urgent attention
- **Medium-Priority Issues:** 4 findings for near-term resolution
- **Low-Priority Issues:** 3 findings for long-term improvement
- **Positive Practices:** 6 security-positive observations

### Compliance Status

- **OWASP Top 10 2021:** 6 of 10 categories affected
- **Authentication Maturity:** Partial implementation, not production-ready
- **Authorization Maturity:** Basic RBAC implemented but bypassable
- **API Security:** GraphQL lacks standard protections

### Immediate Action Required

1. **Remove or secure authentication bypass mechanism** (CRITICAL)
2. **Implement CSRF protection** for GraphQL mutations (CRITICAL)
3. **Add GraphQL query complexity limits** and depth restrictions (HIGH)
4. **Disable GraphQL introspection** in production (HIGH)
5. **Implement rate limiting** for authentication endpoints (HIGH)

---

## 1. Critical Findings

### 1.1 Authentication Bypass via Environment Variable (CRITICAL)

**Severity:** CRITICAL
**CWE:** CWE-798 (Use of Hard-coded Credentials), CWE-306 (Missing Authentication for Critical Function)
**OWASP:** A07:2021 - Identification and Authentication Failures

**Location:**
- `/home/michiel/dev/layercake-tool/layercake-server/src/auth/mod.rs:8,51-58`
- `/home/michiel/dev/layercake-tool/layercake-core/src/services/authorization.rs:78,191-199`

**Description:**

The application implements a complete authentication and authorization bypass mechanism controlled by an environment variable `LAYERCAKE_LOCAL_AUTH_BYPASS`. When this variable is set to "1", "true", "yes", or "on", **all authorization checks are completely skipped**, granting unrestricted access to all operations.

```rust
// layercake-server/src/auth/mod.rs
impl Authorizer for DefaultAuthorizer {
    fn authorize(&self, actor: &Actor, action: &str) -> Result<(), CoreError> {
        if local_auth_bypass_enabled() {
            return Ok(());  // ⚠️ Complete bypass
        }
        // ... actual authorization logic
    }
}

fn local_auth_bypass_enabled() -> bool {
    std::env::var("LAYERCAKE_LOCAL_AUTH_BYPASS")
        .ok()
        .map(|value| {
            let normalized = value.trim().to_ascii_lowercase();
            matches!(normalized.as_str(), "1" | "true" | "yes" | "on")
        })
        .unwrap_or(false)
}
```

This bypass affects:
- All GraphQL mutations and queries
- Project access controls
- Data modification operations
- Administrative functions

**Impact:**

- **Confidentiality:** CRITICAL - Complete data exposure
- **Integrity:** CRITICAL - Unrestricted data modification
- **Availability:** HIGH - Potential for data deletion
- **Authentication:** COMPLETE BYPASS - Any user can access any resource
- **Authorization:** COMPLETE BYPASS - All role-based restrictions ignored

**Attack Scenario:**

1. Attacker discovers the environment variable through:
   - Configuration file exposure
   - Environment variable enumeration
   - Documentation/code review
   - Social engineering
2. Attacker sets `LAYERCAKE_LOCAL_AUTH_BYPASS=1` in deployment environment
3. All authentication and authorization checks are bypassed
4. Attacker gains full administrative access to all projects and data

**Remediation:**

**IMMEDIATE (Critical):**
1. **Remove this bypass completely** from production code
2. If development-only bypass is needed:
   - Use a compile-time feature flag: `#[cfg(debug_assertions)]`
   - Add runtime check for development mode with explicit opt-in
   - Log all bypass uses prominently
   - Make it fail-closed (off by default, not on)

```rust
// Recommended approach
fn local_auth_bypass_enabled() -> bool {
    #[cfg(debug_assertions)]
    {
        if std::env::var("LAYERCAKE_DEV_MODE_INSECURE_BYPASS")
            .ok()
            .map(|v| v == "INSECURE_ENABLE_DEV_BYPASS")
            .unwrap_or(false)
        {
            tracing::warn!("⚠️  INSECURE: Auth bypass enabled for development");
            return true;
        }
    }
    false
}
```

3. **Alternative:** Use proper development credentials instead of bypasses
4. Add deployment checks to ensure this variable is never set in production
5. Document that this creates a critical security vulnerability if enabled

---

### 1.2 Missing CSRF Protection on GraphQL Mutations (CRITICAL)

**Severity:** CRITICAL
**CWE:** CWE-352 (Cross-Site Request Forgery)
**OWASP:** A01:2021 - Broken Access Control

**Location:**
- `/home/michiel/dev/layercake-tool/layercake-server/src/server/app.rs:200-223,258-273`
- All GraphQL mutation endpoints

**Description:**

The GraphQL API accepts mutations over HTTP POST without CSRF token validation. The CORS configuration allows credentials but does not implement any CSRF protection mechanism:

```rust
// layercake-server/src/server/app.rs
let cors = match cors_origin {
    Some(origin) => CorsLayer::new()
        .allow_origin(origin.parse()?)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any)
        .allow_credentials(false),  // ⚠️ Credentials disabled but mutations still possible
    None => CorsLayer::new()
        .allow_origin(Any)  // ⚠️ Allows any origin
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any)
        .allow_credentials(false),
};
```

While credentials are set to `false`, session IDs are passed via custom headers (`x-layercake-session`), which can still be exploited via CSRF if the session header is sent automatically.

**Impact:**

An attacker can craft malicious websites that make authenticated requests to the GraphQL API on behalf of logged-in users, causing:
- Unauthorised project deletion
- Data modification or corruption
- Privilege escalation (adding collaborators)
- Configuration changes

**Attack Scenario:**

1. User logs into Layercake application (session stored in browser)
2. User visits attacker-controlled website
3. Attacker's page makes POST request to GraphQL endpoint with malicious mutation
4. If session header is automatically included, mutation executes with user's privileges
5. Attacker successfully modifies data, deletes projects, or changes permissions

**Remediation:**

1. **Implement CSRF tokens** for all state-changing GraphQL mutations:
   ```rust
   // Add CSRF token validation middleware
   async fn validate_csrf_token(headers: &HeaderMap) -> Result<(), Error> {
       let token = headers.get("x-csrf-token")
           .and_then(|v| v.to_str().ok())
           .ok_or_else(|| Error::new("Missing CSRF token"))?;

       // Validate token against session
       validate_token_for_session(token)?;
       Ok(())
   }
   ```

2. **Use SameSite cookie attribute** if switching to cookie-based sessions:
   ```rust
   Cookie::build("session", session_id)
       .same_site(SameSite::Strict)
       .secure(true)
       .http_only(true)
   ```

3. **Require custom headers** for all mutations (current header-based approach is a weak mitigation)

4. **Implement double-submit cookie pattern** as additional defence layer

5. **Update CORS policy** to be more restrictive:
   ```rust
   .allow_credentials(true)  // If using cookies
   .allow_headers(["x-csrf-token", "content-type", "x-layercake-session"])
   ```

---

### 1.3 Weak Session ID Generation (CRITICAL)

**Severity:** CRITICAL
**CWE:** CWE-330 (Use of Insufficiently Random Values)
**OWASP:** A02:2021 - Cryptographic Failures

**Location:**
- `/home/michiel/dev/layercake-tool/layercake-core/src/database/entities/user_sessions.rs:59`
- `/home/michiel/dev/layercake-tool/layercake-core/src/services/auth_service.rs:44-46`

**Description:**

Session IDs are generated using predictable, non-cryptographic methods:

```rust
// user_sessions.rs:59 - PREDICTABLE SESSION ID
pub fn new(user_id: i32, user_name: String, project_id: i32) -> Self {
    let now = chrono::Utc::now();
    let session_id = format!("sess_{}_{}", user_id, now.timestamp_millis());
    // ⚠️ This is HIGHLY PREDICTABLE!
    // Format: sess_<user_id>_<timestamp_milliseconds>
}

// auth_service.rs:44-46 - Uses UUID v4 (better but not used for sessions)
pub fn generate_session_id() -> String {
    Uuid::new_v4().to_string()  // This method exists but isn't being used
}
```

The current session ID generation in `user_sessions.rs` creates IDs like:
- `sess_123_1704225600000`
- `sess_124_1704225601000`

An attacker can easily predict session IDs by:
1. Knowing or guessing a user ID (often sequential: 1, 2, 3...)
2. Estimating the timestamp (within a small window)
3. Brute-forcing a small range of millisecond values

**Impact:**

- **Session Hijacking:** Attacker can predict and steal active sessions
- **Account Takeover:** Full account compromise possible
- **Privilege Escalation:** Can assume identity of admin users
- **Lateral Movement:** Access to all projects the user can access

**Attack Scenario:**

1. Attacker identifies target user ID (e.g., observing user ID in API responses)
2. Attacker estimates when user logged in (or tries recent timestamps)
3. Attacker generates candidate session IDs: `sess_42_1704225600000` through `sess_42_1704225700000`
4. Attacker tests each ID by making GraphQL requests with `x-layercake-session` header
5. Valid session found → Attacker gains full access to user's account

**Remediation:**

**IMMEDIATE:**

1. **Use cryptographically secure random session IDs:**
   ```rust
   use rand::Rng;

   pub fn new(user_id: i32, user_name: String, project_id: i32) -> Self {
       let now = chrono::Utc::now();
       let session_id = generate_secure_session_id();  // Use the better method!
       // ...
   }

   fn generate_secure_session_id() -> String {
       use rand::distributions::Alphanumeric;
       let random_bytes: String = rand::thread_rng()
           .sample_iter(&Alphanumeric)
           .take(32)
           .map(char::from)
           .collect();
       format!("sess_{}", random_bytes)
   }
   ```

2. **Or use UUID v4** (already available in codebase):
   ```rust
   let session_id = AuthService::generate_session_id();
   ```

3. **Minimum session ID entropy:** 128 bits (32 hex characters or equivalent)

4. **Invalidate all existing sessions** after deploying the fix

5. **Add session binding** to additional factors:
   - User-Agent header
   - IP address (with careful consideration for proxies)
   - Client fingerprint

---

## 2. High-Priority Findings

### 2.1 GraphQL Introspection Enabled (HIGH)

**Severity:** HIGH
**CWE:** CWE-200 (Exposure of Sensitive Information to an Unauthorised Actor)
**OWASP:** A01:2021 - Broken Access Control

**Location:**
- `/home/michiel/dev/layercake-tool/layercake-server/src/server/app.rs:176-179`

**Description:**

GraphQL introspection is enabled by default with no mechanism to disable it in production. The schema is built without any introspection controls:

```rust
let schema: Schema<Query, Mutation, Subscription> =
    Schema::build(Query, Mutation::default(), Subscription)
        .data(graphql_context)
        .finish();  // ⚠️ No introspection control
```

This allows any client to query the entire GraphQL schema structure, revealing:
- All available queries, mutations, and subscriptions
- Field names, types, and descriptions
- Internal data structures
- Business logic hints
- Potential attack surface

**Impact:**

- **Information Disclosure:** Complete API schema exposed
- **Attack Surface Mapping:** Attackers can enumerate all endpoints
- **Reconnaissance:** Facilitates targeted attacks
- **Business Logic Exposure:** Internal operations revealed

**Remediation:**

1. **Disable introspection in production:**
   ```rust
   use async_graphql::*;

   let schema = Schema::build(Query, Mutation::default(), Subscription)
       .data(graphql_context)
       .disable_introspection()  // Add this
       .finish();
   ```

2. **Or conditionally enable based on environment:**
   ```rust
   let mut schema_builder = Schema::build(Query, Mutation::default(), Subscription)
       .data(graphql_context);

   if std::env::var("LAYERCAKE_ENV").unwrap_or_default() != "production" {
       schema_builder = schema_builder.enable_introspection();
   }

   let schema = schema_builder.finish();
   ```

3. **Implement authenticated introspection** for development tools:
   - Require admin role for introspection queries
   - Use separate GraphQL endpoint for development

---

### 2.2 No GraphQL Query Complexity Limits (HIGH)

**Severity:** HIGH
**CWE:** CWE-400 (Uncontrolled Resource Consumption)
**OWASP:** A04:2021 - Insecure Design

**Location:**
- `/home/michiel/dev/layercake-tool/layercake-server/src/server/app.rs:176-179`
- All GraphQL query handlers

**Description:**

The GraphQL implementation has no query complexity analysis, depth limits, or cost-based restrictions. Attackers can craft deeply nested or computationally expensive queries:

```graphql
# Example attack query - no limits prevent this
query AttackQuery {
  projects {                          # Level 1
    plans {                           # Level 2
      dataSets {                      # Level 3
        graphData {                   # Level 4
          nodes {                     # Level 5
            # ... could go much deeper
          }
        }
      }
    }
  }
}

# Or batched queries
query {
  p1: projects { ... }
  p2: projects { ... }
  # ... repeat 1000 times
}
```

**Impact:**

- **Denial of Service:** Server resource exhaustion
- **Database Overload:** Excessive database queries
- **Performance Degradation:** System-wide slowdown
- **Cost Amplification:** Increased cloud infrastructure costs

**Remediation:**

1. **Implement query complexity analysis:**
   ```rust
   use async_graphql::*;

   let schema = Schema::build(Query, Mutation::default(), Subscription)
       .data(graphql_context)
       .limit_complexity(100)  // Maximum complexity
       .limit_depth(10)        // Maximum nesting depth
       .finish();
   ```

2. **Add custom complexity calculator:**
   ```rust
   schema_builder.validation_mode(ValidationMode::Strict)
       .complexity_calculator(|ctx| {
           // Custom complexity calculation based on field costs
       });
   ```

3. **Implement query cost analysis** for expensive fields:
   ```rust
   #[Object]
   impl Query {
       #[graphql(complexity = 50)]  // High cost
       async fn expensive_operation(&self) -> Result<Data> {
           // ...
       }
   }
   ```

4. **Set reasonable limits:**
   - Max query depth: 10-15 levels
   - Max complexity: 100-500 points
   - Max field count: 100-200 fields
   - Query timeout: 30 seconds

---

### 2.3 Missing Rate Limiting on Authentication Endpoints (HIGH)

**Severity:** HIGH
**CWE:** CWE-307 (Improper Restriction of Excessive Authentication Attempts)
**OWASP:** A07:2021 - Identification and Authentication Failures

**Location:**
- `/home/michiel/dev/layercake-tool/layercake-server/src/graphql/mutations/auth.rs:91-136`
- `/home/michiel/dev/layercake-tool/layercake-server/src/server/websocket/handler.rs:85-110`

**Description:**

Authentication mutations (`login`, `register`) have no rate limiting. While WebSocket connections implement basic rate limiting (20 messages/second), GraphQL authentication endpoints are completely unprotected:

```rust
// auth.rs - NO RATE LIMITING
async fn login(&self, ctx: &Context<'_>, input: LoginInput) -> Result<LoginResponse> {
    // ⚠️ Unlimited login attempts possible
    let user = users::Entity::find()
        .filter(users::Column::Email.eq(&input.email))
        .one(&context.db)
        .await?;

    let is_valid = AuthService::verify_password(&input.password, &user.password_hash)?;
    // ...
}
```

Only WebSocket has rate limiting:
```rust
// websocket/handler.rs:85 - ONLY for WebSocket, not HTTP
let mut rate_limiter = RateLimiter::new(20, std::time::Duration::from_secs(1));
```

**Impact:**

- **Brute Force Attacks:** Password guessing attacks
- **Credential Stuffing:** Testing leaked credentials
- **Account Enumeration:** Discovering valid user accounts
- **Denial of Service:** API exhaustion via login attempts

**Attack Scenario:**

1. Attacker obtains list of email addresses
2. Attacker sends unlimited GraphQL login mutations:
   ```graphql
   mutation {
     login(input: {email: "user@example.com", password: "guess1"})
   }
   ```
3. Attacker tests thousands of passwords per second
4. No rate limit prevents the attack
5. Attacker eventually guesses weak passwords or causes DoS

**Remediation:**

1. **Implement rate limiting at multiple layers:**

   **a) IP-based rate limiting:**
   ```rust
   use tower::limit::RateLimitLayer;
   use std::time::Duration;

   let rate_limit = ServiceBuilder::new()
       .layer(RateLimitLayer::new(5, Duration::from_secs(60)));  // 5 requests/min

   app.layer(rate_limit);
   ```

   **b) Per-account rate limiting:**
   ```rust
   // Track failed attempts per email
   async fn login(&self, ctx: &Context<'_>, input: LoginInput) -> Result<LoginResponse> {
       let attempts = get_failed_attempts(&input.email).await;
       if attempts > 5 {
           return Err(StructuredError::forbidden(
               "Too many failed attempts. Try again in 15 minutes"
           ));
       }
       // ... proceed with login
   }
   ```

2. **Implement progressive delays:**
   ```rust
   let delay_ms = std::cmp::min(failed_attempts * 2000, 30000);  // Max 30 seconds
   tokio::time::sleep(Duration::from_millis(delay_ms)).await;
   ```

3. **Add account lockout** after repeated failures:
   - 5 failed attempts → 15-minute lockout
   - 10 failed attempts → 1-hour lockout
   - 20 failed attempts → 24-hour lockout

4. **Implement CAPTCHA** after 3 failed attempts

5. **Log and monitor** authentication attempts for security analysis

---

### 2.4 Insecure Session Management (HIGH)

**Severity:** HIGH
**CWE:** CWE-613 (Insufficient Session Expiration)
**OWASP:** A07:2021 - Identification and Authentication Failures

**Location:**
- `/home/michiel/dev/layercake-tool/layercake-core/src/database/entities/user_sessions.rs:75`
- `/home/michiel/dev/layercake-tool/layercake-core/src/services/authorization.rs:38-69`

**Description:**

Several session management weaknesses exist:

1. **Fixed 24-hour session expiration** with no refresh mechanism:
```rust
expires_at: Set(now + chrono::Duration::hours(24)), // ⚠️ Fixed 24 hours
```

2. **No session invalidation** on password change
3. **No concurrent session limits** per user
4. **Session validation only checks expiration**, not revocation:
```rust
if session.expires_at <= Utc::now() {
    return Err(CoreError::unauthorized("Session expired"));
}
// ⚠️ No check for manual revocation or password change
```

5. **Sessions stored with minimal metadata** - no IP binding or fingerprinting

**Impact:**

- **Session Fixation:** Attacker can pre-set session IDs
- **Prolonged Unauthorised Access:** Stolen sessions valid for 24 hours
- **No Session Revocation:** Can't invalidate compromised sessions
- **Concurrent Session Abuse:** Multiple devices using same account

**Remediation:**

1. **Implement session refresh tokens:**
   ```rust
   // Short-lived access token (1 hour) + long-lived refresh token (7 days)
   pub struct SessionTokens {
       access_token: String,      // 1 hour
       refresh_token: String,     // 7 days
       access_expires_at: DateTime<Utc>,
       refresh_expires_at: DateTime<Utc>,
   }
   ```

2. **Add session revocation support:**
   ```rust
   pub struct Model {
       // ... existing fields
       pub revoked_at: Option<ChronoDateTimeUtc>,
       pub revocation_reason: Option<String>,
   }

   pub async fn revoke_session(&self, session_id: &str, reason: &str) {
       // Update session with revocation timestamp
   }

   pub async fn revoke_all_user_sessions(&self, user_id: i32, reason: &str) {
       // Invalidate all sessions for user (e.g., after password change)
   }
   ```

3. **Implement session binding:**
   ```rust
   pub struct Model {
       // ... existing fields
       pub client_ip: Option<String>,
       pub user_agent_hash: Option<String>,
   }

   // Validate session binding on each request
   if session.client_ip != current_ip {
       // Require re-authentication
   }
   ```

4. **Add concurrent session management:**
   ```rust
   const MAX_SESSIONS_PER_USER: usize = 5;

   // When creating new session, check count
   let active_sessions = count_active_sessions(user_id).await;
   if active_sessions >= MAX_SESSIONS_PER_USER {
       // Revoke oldest session or require explicit logout
   }
   ```

5. **Implement "remember me" functionality** properly:
   - Short sessions (1 hour) for normal use
   - Extended sessions (7 days) only if user opts in
   - Require re-authentication for sensitive operations even with valid session

---

### 2.5 Weak Password Policy (HIGH)

**Severity:** HIGH
**CWE:** CWE-521 (Weak Password Requirements)
**OWASP:** A07:2021 - Identification and Authentication Failures

**Location:**
- `/home/michiel/dev/layercake-tool/layercake-core/src/services/auth_service.rs:22-35`

**Description:**

Password validation only enforces minimum length of 8 characters with no other requirements:

```rust
pub fn hash_password(password: &str) -> CoreResult<String> {
    if password.is_empty() {
        return Err(CoreError::validation("Password cannot be empty"));
    }

    if password.len() < 8 {
        return Err(CoreError::validation(
            "Password must be at least 8 characters long",
        ));
    }
    // ⚠️ No complexity requirements
    // ⚠️ No common password checks
    // ⚠️ No breach database checks
}
```

This allows weak passwords like:
- `password`
- `12345678`
- `aaaaaaaa`
- `qwertyui`

**Impact:**

- **Brute Force Success:** Weak passwords easily guessed
- **Dictionary Attacks:** Common passwords not blocked
- **Credential Stuffing:** Leaked passwords likely reused
- **Social Engineering:** Easy-to-guess passwords

**Remediation:**

1. **Implement comprehensive password policy:**
   ```rust
   pub fn validate_password_strength(password: &str) -> CoreResult<()> {
       if password.len() < 12 {
           return Err(CoreError::validation(
               "Password must be at least 12 characters"
           ));
       }

       if password.len() > 128 {
           return Err(CoreError::validation(
               "Password too long (max 128 characters)"
           ));
       }

       // Check for character diversity
       let has_lowercase = password.chars().any(|c| c.is_lowercase());
       let has_uppercase = password.chars().any(|c| c.is_uppercase());
       let has_digit = password.chars().any(|c| c.is_numeric());
       let has_special = password.chars().any(|c| !c.is_alphanumeric());

       let char_type_count = [has_lowercase, has_uppercase, has_digit, has_special]
           .iter()
           .filter(|&&x| x)
           .count();

       if char_type_count < 3 {
           return Err(CoreError::validation(
               "Password must contain at least 3 of: lowercase, uppercase, digit, special character"
           ));
       }

       Ok(())
   }
   ```

2. **Add common password blocklist:**
   ```rust
   const COMMON_PASSWORDS: &[&str] = &[
       "password", "12345678", "qwerty", "admin",
       // ... top 10,000 common passwords
   ];

   if COMMON_PASSWORDS.contains(&password.to_lowercase().as_str()) {
       return Err(CoreError::validation("Password is too common"));
   }
   ```

3. **Integrate with breach database** (e.g., Have I Been Pwned API):
   ```rust
   async fn check_password_breach(password: &str) -> CoreResult<()> {
       // SHA-1 hash the password
       // Check k-anonymity API
       // Reject if found in breaches
   }
   ```

4. **Implement password entropy scoring:**
   ```rust
   fn calculate_password_entropy(password: &str) -> f64 {
       // Calculate entropy bits
       // Require minimum 50 bits
   }
   ```

5. **Add password history** to prevent reuse:
   ```rust
   // Store hash of last 5 passwords
   // Prevent immediate reuse
   ```

---

## 3. Medium-Priority Findings

### 3.1 Email Validation is Insufficient (MEDIUM)

**Severity:** MEDIUM
**CWE:** CWE-20 (Improper Input Validation)
**OWASP:** A03:2021 - Injection

**Location:**
- `/home/michiel/dev/layercake-tool/layercake-core/src/services/auth_service.rs:54-99`

**Description:**

Email validation uses basic string parsing instead of proper RFC 5322 validation:

```rust
pub fn validate_email(email: &str) -> CoreResult<()> {
    // ⚠️ Overly simplistic validation
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return Err(CoreError::validation(
            "Invalid email format: must contain exactly one @",
        ));
    }
    // ... basic checks only
}
```

This accepts invalid emails like:
- `test@@example.com` (rejected, but...)
- `test@.example.com` (allowed but invalid)
- `test@example..com` (allowed but invalid)
- `test@-example.com` (allowed but invalid)

**Impact:**

- **Data Quality:** Invalid emails stored in database
- **Email Delivery Failure:** Unable to send notifications
- **Account Enumeration:** Predictable email patterns
- **Business Logic Bypass:** Email-based workflows fail

**Remediation:**

1. **Use proper email validation library:**
   ```rust
   use email_address::EmailAddress;

   pub fn validate_email(email: &str) -> CoreResult<()> {
       EmailAddress::parse(email)
           .map_err(|_| CoreError::validation("Invalid email format"))?;
       Ok(())
   }
   ```

2. **Implement email verification:**
   - Send verification email after registration
   - Mark email as verified in database
   - Require verification for sensitive operations

3. **Normalise email addresses:**
   ```rust
   fn normalise_email(email: &str) -> String {
       email.trim().to_lowercase()
   }
   ```

---

### 3.2 No Protection Against Timing Attacks (MEDIUM)

**Severity:** MEDIUM
**CWE:** CWE-208 (Observable Timing Discrepancy)
**OWASP:** A02:2021 - Cryptographic Failures

**Location:**
- `/home/michiel/dev/layercake-tool/layercake-server/src/graphql/mutations/auth.rs:91-136`

**Description:**

Login endpoint has observable timing differences based on whether user exists:

```rust
async fn login(&self, ctx: &Context<'_>, input: LoginInput) -> Result<LoginResponse> {
    let user = users::Entity::find()
        .filter(users::Column::Email.eq(&input.email))
        .one(&context.db)
        .await?
        .ok_or_else(|| StructuredError::unauthorized("Invalid email or password"))?;
    // ⚠️ If user not found, returns immediately (fast)

    let is_valid = AuthService::verify_password(&input.password, &user.password_hash)?;
    // ⚠️ If user found, bcrypt verification takes ~100ms (slow)
}
```

**Impact:**

- **User Enumeration:** Attackers can determine if email exists
- **Reduced Attack Complexity:** Focus on valid accounts only
- **Information Disclosure:** Account existence revealed

**Remediation:**

1. **Always perform password hash even for non-existent users:**
   ```rust
   async fn login(&self, ctx: &Context<'_>, input: LoginInput) -> Result<LoginResponse> {
       let user = users::Entity::find()
           .filter(users::Column::Email.eq(&input.email))
           .one(&context.db)
           .await?;

       // Always hash a dummy password to maintain constant time
       let (hash_to_verify, user_id) = match user {
           Some(ref u) => (u.password_hash.as_str(), Some(u.id)),
           None => (DUMMY_PASSWORD_HASH, None),  // Constant-time dummy hash
       };

       let is_valid = AuthService::verify_password(&input.password, hash_to_verify)?;

       if is_valid && user_id.is_some() {
           // Proceed with login
       } else {
           // Same error for both cases
           Err(StructuredError::unauthorized("Invalid email or password"))
       }
   }
   ```

2. **Use constant-time string comparison** for security-sensitive comparisons

---

### 3.3 Insufficient Input Validation on File Uploads (MEDIUM)

**Severity:** MEDIUM
**CWE:** CWE-434 (Unrestricted Upload of File with Dangerous Type)
**OWASP:** A03:2021 - Injection

**Location:**
- `/home/michiel/dev/layercake-tool/layercake-server/src/graphql/mutations/data_set.rs:23-52,75-111`

**Description:**

File upload mutations accept base64-encoded file content with minimal validation:

```rust
async fn create_data_set_from_file(
    &self,
    ctx: &Context<'_>,
    input: CreateDataSetInput,
) -> Result<DataSet> {
    let file_bytes = base64::engine::general_purpose::STANDARD
        .decode(&input.file_content)?;  // ⚠️ No size limit check

    // ⚠️ Filename trusted from client
    // ⚠️ No MIME type validation
    // ⚠️ No content scanning
}
```

**Impact:**

- **Denial of Service:** Extremely large files exhaust memory
- **Storage Exhaustion:** Disk space consumed
- **Malware Upload:** Executable files could be uploaded
- **Path Traversal:** Malicious filenames like `../../evil.txt`

**Remediation:**

1. **Implement file size limits:**
   ```rust
   const MAX_FILE_SIZE: usize = 50 * 1024 * 1024;  // 50 MB

   if file_bytes.len() > MAX_FILE_SIZE {
       return Err(StructuredError::validation(
           "file_size",
           format!("File too large. Maximum size is {} MB", MAX_FILE_SIZE / 1024 / 1024)
       ));
   }
   ```

2. **Validate file type by content (magic bytes), not extension:**
   ```rust
   use infer::Infer;

   let info = Infer::new();
   let file_type = info.get(&file_bytes)
       .ok_or_else(|| StructuredError::validation("file_type", "Unknown file type"))?;

   const ALLOWED_TYPES: &[&str] = &["text/csv", "text/plain", "application/json"];
   if !ALLOWED_TYPES.contains(&file_type.mime_type()) {
       return Err(StructuredError::validation("file_type", "File type not allowed"));
   }
   ```

3. **Sanitise filenames:**
   ```rust
   fn sanitise_filename(filename: &str) -> String {
       filename
           .chars()
           .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
           .collect::<String>()
           .trim_start_matches('.')
           .to_string()
   }
   ```

4. **Implement virus scanning** for uploaded files (if applicable)

5. **Store files outside web root** with random names

---

### 3.4 GraphQL Playground Enabled in Production (MEDIUM)

**Severity:** MEDIUM
**CWE:** CWE-200 (Exposure of Sensitive Information)
**OWASP:** A05:2021 - Security Misconfiguration

**Location:**
- `/home/michiel/dev/layercake-tool/layercake-server/src/server/app.rs:366-377`

**Description:**

GraphQL Playground is unconditionally enabled on all `/graphql` and `/projections/graphql` endpoints:

```rust
async fn graphql_playground() -> impl axum::response::IntoResponse {
    axum::response::Html(async_graphql::http::playground_source(
        async_graphql::http::GraphQLPlaygroundConfig::new("/graphql"),
    ))
}
```

**Impact:**

- **Information Disclosure:** Interactive schema exploration
- **Attack Surface:** Facilitates manual testing of attacks
- **Development Tools in Production:** Unprofessional appearance

**Remediation:**

1. **Disable playground in production:**
   ```rust
   let app = app.route(
       "/graphql",
       post(graphql_handler).options(|| async { axum::http::StatusCode::OK }),
   );

   // Only add playground in development
   #[cfg(debug_assertions)]
   let app = app.route("/graphql", get(graphql_playground));
   ```

2. **Or require authentication:**
   ```rust
   async fn authenticated_playground(
       ctx: Context,
   ) -> Result<impl IntoResponse> {
       require_admin_role(&ctx).await?;
       Ok(Html(playground_source(...)))
   }
   ```

---

## 4. Low-Priority Findings

### 4.1 Permissive CORS Configuration (LOW)

**Severity:** LOW
**CWE:** CWE-942 (Overly Permissive Cross-domain Whitelist)
**OWASP:** A05:2021 - Security Misconfiguration

**Location:**
- `/home/michiel/dev/layercake-tool/layercake-server/src/server/app.rs:200-223`

**Description:**

When CORS origin is not specified, the server allows **any origin** (`Any`):

```rust
None => CorsLayer::new()
    .allow_origin(Any)  // ⚠️ Allows requests from ANY domain
    .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
    .allow_headers(Any)
    .allow_credentials(false),
```

**Impact:**

- **Data Exposure:** Any website can read API responses
- **Reduced Security Posture:** Browser security features bypassed
- **Cross-Origin Information Leakage:** Unintended data access

**Remediation:**

1. **Always require explicit CORS origin:**
   ```rust
   let cors_origin = cors_origin
       .ok_or_else(|| anyhow!("CORS origin must be specified in production"))?;
   ```

2. **Support multiple origins with validation:**
   ```rust
   const ALLOWED_ORIGINS: &[&str] = &[
       "https://app.layercake.example.com",
       "https://admin.layercake.example.com",
   ];

   if !ALLOWED_ORIGINS.contains(&origin) {
       return Err(anyhow!("Origin not allowed"));
   }
   ```

---

### 4.2 Overly Verbose Error Messages (LOW)

**Severity:** LOW
**CWE:** CWE-209 (Generation of Error Message Containing Sensitive Information)
**OWASP:** A04:2021 - Insecure Design

**Location:**
- `/home/michiel/dev/layercake-tool/layercake-server/src/graphql/errors.rs:75-79`
- Various mutation handlers

**Description:**

Error messages include operation context that could aid attackers:

```rust
pub fn database(operation: &str, cause: impl std::fmt::Display) -> Error {
    Error::new(format!("Database error during {}: {}", operation, cause))
        .extend_with(|_, e| {
            e.set("code", "DATABASE_ERROR");
            e.set("operation", operation);  // ⚠️ Leaks internal operation names
        })
}
```

Example error: `"Database error during users::Entity::find (email): connection timeout"`

**Impact:**

- **Information Disclosure:** Internal database structure revealed
- **Attack Intelligence:** Error patterns aid reconnaissance
- **Technology Stack Disclosure:** Database type and ORM exposed

**Remediation:**

1. **Use generic error messages in production:**
   ```rust
   pub fn database(operation: &str, cause: impl std::fmt::Display) -> Error {
       #[cfg(debug_assertions)]
       let message = format!("Database error during {}: {}", operation, cause);

       #[cfg(not(debug_assertions))]
       let message = "Database operation failed".to_string();

       Error::new(message).extend_with(|_, e| {
           e.set("code", "DATABASE_ERROR");
           // Only include operation in development
           #[cfg(debug_assertions)]
           e.set("operation", operation);
       })
   }
   ```

2. **Log detailed errors server-side:**
   ```rust
   tracing::error!(
       operation = operation,
       error = %cause,
       "Database error"
   );
   ```

3. **Return error IDs** for correlation:
   ```rust
   let error_id = Uuid::new_v4();
   tracing::error!(error_id = %error_id, "Database error: {}", cause);
   Error::new("Database operation failed")
       .extend_with(|_, e| {
           e.set("error_id", error_id.to_string());
       })
   ```

---

### 4.3 No Security Headers (LOW)

**Severity:** LOW
**CWE:** CWE-693 (Protection Mechanism Failure)
**OWASP:** A05:2021 - Security Misconfiguration

**Location:**
- `/home/michiel/dev/layercake-tool/layercake-server/src/server/app.rs` (missing middleware)

**Description:**

The server does not set security-related HTTP headers:
- No `X-Content-Type-Options: nosniff`
- No `X-Frame-Options: DENY`
- No `Strict-Transport-Security`
- No `Content-Security-Policy`
- No `Referrer-Policy`

**Impact:**

- **Clickjacking:** Application can be framed
- **MIME Sniffing Attacks:** Browser interprets files incorrectly
- **Man-in-the-Middle:** No HSTS enforcement

**Remediation:**

1. **Add security headers middleware:**
   ```rust
   use tower_http::set_header::SetResponseHeaderLayer;
   use axum::http::header;

   let security_headers = ServiceBuilder::new()
       .layer(SetResponseHeaderLayer::if_not_present(
           header::X_CONTENT_TYPE_OPTIONS,
           HeaderValue::from_static("nosniff"),
       ))
       .layer(SetResponseHeaderLayer::if_not_present(
           header::X_FRAME_OPTIONS,
           HeaderValue::from_static("DENY"),
       ))
       .layer(SetResponseHeaderLayer::if_not_present(
           header::STRICT_TRANSPORT_SECURITY,
           HeaderValue::from_static("max-age=31536000; includeSubDomains"),
       ))
       .layer(SetResponseHeaderLayer::if_not_present(
           header::CONTENT_SECURITY_POLICY,
           HeaderValue::from_static("default-src 'self'"),
       ));

   app.layer(security_headers);
   ```

---

## 5. Positive Security Practices Observed

While this review identified several critical issues, the codebase demonstrates some good security practices:

### 5.1 Use of bcrypt for Password Hashing ✓

**Location:** `layercake-core/src/services/auth_service.rs:33`

The application correctly uses bcrypt with the `DEFAULT_COST` (12 rounds) for password hashing, which is appropriate for 2026. Bcrypt is a well-tested, slow hash function resistant to GPU-based attacks.

```rust
hash(password, DEFAULT_COST)  // ✓ Good: bcrypt with appropriate cost
```

**Recommendation:** Consider increasing cost factor to 13-14 for new installations as hardware improves.

---

### 5.2 Parameterised Queries via SeaORM ✓

**Location:** Throughout codebase

The application consistently uses SeaORM for database access, which provides automatic protection against SQL injection through parameterised queries:

```rust
let user = users::Entity::find()
    .filter(users::Column::Email.eq(&input.email))  // ✓ Parameterised
    .one(&context.db)
    .await?;
```

No raw SQL concatenation was found in the security-critical authentication code.

---

### 5.3 Strong Typing and Rust Safety ✓

The use of Rust's type system provides inherent memory safety protections:
- No buffer overflows
- No use-after-free vulnerabilities
- Guaranteed thread safety
- Compile-time prevention of many vulnerability classes

---

### 5.4 Structured Error Handling ✓

**Location:** `layercake-server/src/graphql/errors.rs`

The codebase implements structured error types with consistent error codes, facilitating proper error handling and reducing the risk of information leakage through error messages.

---

### 5.5 Session Expiration Implemented ✓

**Location:** `layercake-core/src/services/authorization.rs:52-54`

Sessions have expiration timestamps and are validated on each request:

```rust
if session.expires_at <= Utc::now() {
    return Err(CoreError::unauthorized("Session expired"));
}
```

While the implementation needs improvements (see High-Priority findings), the basic mechanism is in place.

---

### 5.6 Role-Based Access Control Framework ✓

**Location:** `layercake-core/src/services/authorization.rs`

The application implements a hierarchical RBAC system with roles (Owner, Editor, Viewer) and proper permission checking logic. The framework is sound, though the bypass mechanism undermines it.

---

## 6. Recommendations

### 6.1 Immediate Actions (Within 1 Week)

1. **[CRITICAL]** Remove or properly secure `LAYERCAKE_LOCAL_AUTH_BYPASS` environment variable
2. **[CRITICAL]** Replace predictable session ID generation with cryptographically secure randomness
3. **[CRITICAL]** Implement CSRF protection for all GraphQL mutations
4. **[HIGH]** Disable GraphQL introspection in production environments
5. **[HIGH]** Add query complexity and depth limits to GraphQL schema
6. **[HIGH]** Implement rate limiting on authentication endpoints

### 6.2 Short-Term Actions (Within 1 Month)

1. **[HIGH]** Improve session management with refresh tokens and revocation support
2. **[HIGH]** Enforce stronger password policies with common password blocklist
3. **[MEDIUM]** Add comprehensive input validation for file uploads
4. **[MEDIUM]** Implement timing-attack protections in authentication
5. **[MEDIUM]** Disable GraphQL Playground in production or require authentication
6. **[LOW]** Add security HTTP headers middleware
7. **[LOW]** Restrict CORS to specific allowed origins
8. **[LOW]** Implement generic error messages for production

### 6.3 Long-Term Actions (Within 3 Months)

1. Implement comprehensive audit logging for all security events
2. Add two-factor authentication (2FA/MFA) support
3. Implement email verification for new account registrations
4. Add IP-based geolocation and anomaly detection for logins
5. Implement Content Security Policy headers for XSS protection
6. Add dependency vulnerability scanning to CI/CD pipeline
7. Conduct penetration testing after critical fixes are deployed
8. Implement security monitoring and alerting
9. Create incident response procedures
10. Regular security training for development team

### 6.4 Security Development Lifecycle Improvements

1. **Code Review Process:**
   - Mandatory security review for authentication/authorisation changes
   - Use of security-focused linters and static analysis tools
   - Peer review requirements for sensitive code paths

2. **Testing Requirements:**
   - Security-specific test cases for authentication flows
   - Fuzzing tests for input validation
   - Integration tests for authorisation boundaries

3. **Dependency Management:**
   - Automated dependency vulnerability scanning (e.g., `cargo audit`)
   - Regular updates of dependencies with security patches
   - Removal of unused dependencies

4. **Deployment Hardening:**
   - Environment variable validation on startup
   - Fail-safe defaults (deny by default)
   - Configuration audit logging
   - Deployment checklist for security settings

---

## 7. OWASP Top 10 2021 Coverage

| OWASP Category | Status | Findings |
|----------------|--------|----------|
| **A01:2021 - Broken Access Control** | ❌ **AFFECTED** | Auth bypass, CSRF, introspection, session issues |
| **A02:2021 - Cryptographic Failures** | ❌ **AFFECTED** | Weak session IDs, timing attacks, password policy |
| **A03:2021 - Injection** | ✅ **PROTECTED** | SeaORM parameterisation, but weak file validation |
| **A04:2021 - Insecure Design** | ❌ **AFFECTED** | No query complexity limits, auth bypass design flaw |
| **A05:2021 - Security Misconfiguration** | ❌ **AFFECTED** | Playground enabled, permissive CORS, no security headers |
| **A06:2021 - Vulnerable Components** | ⚠️ **UNKNOWN** | Dependencies not audited during review |
| **A07:2021 - Identification and Authentication Failures** | ❌ **SEVERELY AFFECTED** | Multiple critical auth issues |
| **A08:2021 - Software and Data Integrity Failures** | ✅ **MINIMAL RISK** | No CI/CD security issues identified |
| **A09:2021 - Security Logging and Monitoring Failures** | ⚠️ **PARTIAL** | Logging exists but no security monitoring |
| **A10:2021 - Server-Side Request Forgery (SSRF)** | ⚠️ **NOT REVIEWED** | No obvious SSRF vectors identified |

---

## 8. Conclusion

The Layercake application demonstrates a foundation of security-conscious development with the use of strong cryptographic primitives (bcrypt), safe database access patterns (SeaORM), and structured error handling. However, several **critical security vulnerabilities** require immediate attention before this application can be safely deployed in a production environment.

### Critical Risks Summary

The three critical findings—authentication bypass, CSRF vulnerabilities, and weak session ID generation—pose severe risks that could lead to:
- Complete system compromise
- Unauthorised data access and modification
- Account takeover attacks
- Privilege escalation

### Production Readiness Assessment

**Current Status:** ⚠️ **NOT PRODUCTION-READY**

The application should **NOT** be deployed to production until:
1. All CRITICAL findings are remediated
2. All HIGH-priority findings are addressed
3. Security testing validates the fixes
4. Security monitoring is implemented

### Estimated Remediation Effort

- **Critical fixes:** 3-5 developer days
- **High-priority fixes:** 5-7 developer days
- **Medium-priority fixes:** 3-5 developer days
- **Testing and validation:** 3-5 developer days

**Total:** Approximately 2-3 weeks of focused security work

### Next Steps

1. **Immediate:** Disable production deployments until critical fixes are complete
2. **Week 1:** Address all CRITICAL and HIGH findings
3. **Week 2:** Implement MEDIUM-priority fixes and security testing
4. **Week 3:** Validate fixes, implement monitoring, prepare for secure deployment
5. **Ongoing:** Implement long-term recommendations and security development practices

---

## Appendix A: Testing Recommendations

### A.1 Security Test Cases to Implement

1. **Authentication Tests:**
   - Brute force resistance testing
   - Session fixation testing
   - Session hijacking testing
   - Weak password rejection testing
   - Multi-factor authentication bypass testing

2. **Authorisation Tests:**
   - Privilege escalation testing
   - Horizontal access control testing (user A accessing user B's data)
   - Vertical access control testing (viewer accessing admin functions)
   - IDOR (Insecure Direct Object Reference) testing

3. **GraphQL Security Tests:**
   - Query complexity DoS testing
   - Introspection availability testing
   - Batch query limit testing
   - Mutation CSRF testing

4. **Input Validation Tests:**
   - File upload size limit testing
   - Malicious filename testing
   - MIME type validation testing
   - Base64 decoding error handling

### A.2 Penetration Testing Focus Areas

1. Authentication and session management
2. Authorization and access control boundaries
3. GraphQL API security (introspection, complexity, batching)
4. File upload functionality
5. CSRF protection validation
6. Rate limiting effectiveness

---

## Appendix B: References

### Security Standards and Guidelines

- **OWASP Top 10 2021:** https://owasp.org/Top10/
- **OWASP API Security Top 10:** https://owasp.org/www-project-api-security/
- **OWASP GraphQL Security Cheat Sheet:** https://cheatsheetseries.owasp.org/cheatsheets/GraphQL_Cheat_Sheet.html
- **OWASP Authentication Cheat Sheet:** https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html
- **CWE (Common Weakness Enumeration):** https://cwe.mitre.org/

### Rust Security Resources

- **Rust Security Guidelines:** https://anssi-fr.github.io/rust-guide/
- **RustSec Advisory Database:** https://rustsec.org/
- **cargo-audit:** https://github.com/RustSec/rustsec/tree/main/cargo-audit

### GraphQL Security

- **GraphQL Security Best Practices:** https://graphql.org/learn/best-practices/
- **async-graphql Security:** https://async-graphql.github.io/async-graphql/en/security.html

---

**End of Security Review Report**

*This report should be treated as confidential and shared only with authorised personnel. The findings represent security vulnerabilities that could be exploited if publicly disclosed before remediation.*
