#![cfg(feature = "mcp")]

use axum_mcp::prelude::{ClientContext, SecurityContext};

const CAP_USER_ID_PREFIX: &str = "user_id:";
const CAP_USER_TYPE_PREFIX: &str = "user_type:";
const CAP_PROJECT_PREFIX: &str = "scoped_project:";
const CAP_AGENT_FLAG: &str = "mcp_agent";

pub fn build_user_security_context(
    client: ClientContext,
    user_id: i32,
    user_type: &str,
    scoped_project_id: Option<i32>,
) -> SecurityContext {
    let mut capabilities = vec![
        "authenticated".to_string(),
        format!("{CAP_USER_ID_PREFIX}{user_id}"),
        format!("{CAP_USER_TYPE_PREFIX}{user_type}"),
    ];

    if user_type.eq_ignore_ascii_case("mcp_agent") {
        capabilities.push(CAP_AGENT_FLAG.to_string());
    } else {
        capabilities.push("human".to_string());
    }

    if let Some(project_id) = scoped_project_id {
        capabilities.push(format!("{CAP_PROJECT_PREFIX}{project_id}"));
    }

    SecurityContext::authenticated(client, capabilities)
}

pub fn scoped_project_from_context(context: &SecurityContext) -> Option<i32> {
    context
        .capabilities
        .iter()
        .find_map(|cap| cap.strip_prefix(CAP_PROJECT_PREFIX))
        .and_then(|value| value.parse::<i32>().ok())
}
