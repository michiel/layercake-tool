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
