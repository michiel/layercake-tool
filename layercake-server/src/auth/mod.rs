use layercake_core::auth::{Actor, Authorizer};
use layercake_core::errors::CoreError;

pub struct DefaultAuthorizer;

impl Authorizer for DefaultAuthorizer {
    fn authorize(&self, actor: &Actor, action: &str) -> Result<(), CoreError> {
        if actor.is_system() {
            return Ok(());
        }

        if actor.user_id.is_none() {
            return Err(CoreError::unauthorized("User is not authenticated"));
        }

        if actor.has_role("admin") {
            return Ok(());
        }

        if actor.has_scope(action) {
            return Ok(());
        }

        if action.starts_with("read:") {
            if actor.has_role("viewer") || actor.has_role("editor") || actor.has_role("owner") {
                return Ok(());
            }
        }

        if action.starts_with("write:") {
            if actor.has_role("editor") || actor.has_role("owner") {
                return Ok(());
            }
        }

        if action.starts_with("admin:") && actor.has_role("owner") {
            return Ok(());
        }

        Err(CoreError::forbidden(format!(
            "Actor is not authorized for {}",
            action
        )))
    }
}
