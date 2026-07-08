use std::collections::BTreeSet;

use crate::errors::CoreError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Actor {
    pub user_id: Option<i32>,
    roles: BTreeSet<String>,
    scopes: BTreeSet<String>,
    is_system: bool,
}

impl Actor {
    pub fn user(user_id: i32) -> Self {
        Self {
            user_id: Some(user_id),
            roles: BTreeSet::new(),
            scopes: BTreeSet::new(),
            is_system: false,
        }
    }

    pub fn system() -> Self {
        Self {
            user_id: None,
            roles: BTreeSet::new(),
            scopes: BTreeSet::new(),
            is_system: true,
        }
    }

    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.roles.insert(role.into());
        self
    }

    pub fn with_scope(mut self, scope: impl Into<String>) -> Self {
        self.scopes.insert(scope.into());
        self
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(role)
    }

    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.contains(scope)
    }

    pub fn is_system(&self) -> bool {
        self.is_system
    }
}

pub struct SystemActor;

impl SystemActor {
    pub fn internal() -> Actor {
        Actor::system()
    }
}

pub trait Authorizer {
    fn authorize(&self, actor: &Actor, action: &str) -> Result<(), CoreError>;
}

pub struct AllowAllAuthorizer;

impl Authorizer for AllowAllAuthorizer {
    fn authorize(&self, _actor: &Actor, _action: &str) -> Result<(), CoreError> {
        Ok(())
    }
}

/// Environment variable that, when set to a truthy value, disables all
/// authorization checks. Intended for local single-user development only.
pub const LOCAL_AUTH_BYPASS_ENV: &str = "LAYERCAKE_LOCAL_AUTH_BYPASS";

/// Returns whether the local authorization bypass is enabled.
///
/// This is the single canonical implementation used by both `layercake-core`
/// and `layercake-server` so their behaviour can never diverge. The default
/// when the variable is unset is `false` (bypass OFF): authorization fails
/// closed. Bypass must be an explicit opt-in (`=1|true|yes|on`).
pub fn local_auth_bypass_enabled() -> bool {
    bypass_value_is_truthy(std::env::var(LOCAL_AUTH_BYPASS_ENV).ok().as_deref())
}

/// Pure truthiness check for the bypass env var, extracted so it can be unit
/// tested without mutating process-global environment. `None` means unset.
fn bypass_value_is_truthy(value: Option<&str>) -> bool {
    match value {
        Some(value) => {
            let normalized = value.trim().to_ascii_lowercase();
            matches!(normalized.as_str(), "1" | "true" | "yes" | "on")
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::bypass_value_is_truthy;

    #[test]
    fn bypass_defaults_off_when_unset() {
        assert!(!bypass_value_is_truthy(None));
    }

    #[test]
    fn bypass_on_for_truthy_values() {
        for v in ["1", "true", "TRUE", " yes ", "on", "On"] {
            assert!(bypass_value_is_truthy(Some(v)), "expected true for {v:?}");
        }
    }

    #[test]
    fn bypass_off_for_falsey_values() {
        for v in ["0", "false", "no", "off", "", "anything"] {
            assert!(!bypass_value_is_truthy(Some(v)), "expected false for {v:?}");
        }
    }
}
