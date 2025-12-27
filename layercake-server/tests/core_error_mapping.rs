use async_graphql::{Error, Value};

use layercake_core::errors::CoreError;
use layercake_server::graphql::errors::core_error_to_graphql_error;

fn extension_value<'a>(error: &'a Error, key: &str) -> Option<&'a Value> {
    error.extensions.as_ref().and_then(|ext| ext.get(key))
}

#[test]
fn core_error_maps_to_graphql_codes() {
    let validation = core_error_to_graphql_error(CoreError::validation("invalid"));
    assert_eq!(
        extension_value(&validation, "code"),
        Some(&Value::from("VALIDATION_FAILED"))
    );

    let conflict = core_error_to_graphql_error(CoreError::conflict("conflict"));
    assert_eq!(
        extension_value(&conflict, "code"),
        Some(&Value::from("CONFLICT"))
    );

    let forbidden = core_error_to_graphql_error(CoreError::forbidden("nope"));
    assert_eq!(
        extension_value(&forbidden, "code"),
        Some(&Value::from("FORBIDDEN"))
    );

    let unauthorized = core_error_to_graphql_error(CoreError::unauthorized("auth"));
    assert_eq!(
        extension_value(&unauthorized, "code"),
        Some(&Value::from("UNAUTHORIZED"))
    );

    let unavailable = core_error_to_graphql_error(CoreError::unavailable("down"));
    assert_eq!(
        extension_value(&unavailable, "code"),
        Some(&Value::from("SERVICE_ERROR"))
    );

    let internal = core_error_to_graphql_error(CoreError::internal("boom"));
    assert_eq!(
        extension_value(&internal, "code"),
        Some(&Value::from("INTERNAL_ERROR"))
    );
}

#[test]
fn not_found_error_includes_fields() {
    let error = core_error_to_graphql_error(CoreError::not_found("Project", "12"));
    assert_eq!(
        extension_value(&error, "code"),
        Some(&Value::from("NOT_FOUND"))
    );
    assert_eq!(
        extension_value(&error, "entity"),
        Some(&Value::from("Project"))
    );
    assert_eq!(extension_value(&error, "id"), Some(&Value::from("12")));
}
