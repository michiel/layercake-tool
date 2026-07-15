//! `layercake api` — agent-facing helpers for a running server.
//!
//! `info` prints the endpoints/headers an agent needs; `call` POSTs a GraphQL
//! operation to a running instance. This is the HTTP path (talks to a live
//! `layercake serve`), distinct from `layercake query`, which hits the DB file
//! directly with no server.

use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use serde_json::json;

/// Build the base URL from an explicit `--url` or host/port.
fn base_url(url: Option<&str>, host: &str, port: u16) -> String {
    match url {
        Some(u) => u.trim_end_matches('/').to_string(),
        None => format!("http://{}:{}", host, port),
    }
}

#[derive(Serialize)]
struct ApiInfo {
    base_url: String,
    graphql: String,
    graphql_ws: String,
    health: String,
    collaboration_ws: String,
    session_header: &'static str,
    database: String,
}

/// Print the endpoints and headers for a running server.
pub fn info(url: Option<&str>, host: &str, port: u16, database: &str, json_out: bool) -> Result<()> {
    let base = base_url(url, host, port);
    let ws_base = base.replacen("http", "ws", 1);
    let info = ApiInfo {
        graphql: format!("{}/graphql", base),
        graphql_ws: format!("{}/graphql/ws", ws_base),
        health: format!("{}/health", base),
        collaboration_ws: format!("{}/ws/collaboration?project_id=<N>", ws_base),
        session_header: "x-layercake-session",
        database: database.to_string(),
        base_url: base,
    };

    if json_out {
        println!("{}", serde_json::to_string_pretty(&info)?);
    } else {
        println!("Base URL:           {}", info.base_url);
        println!("GraphQL (HTTP):     {}", info.graphql);
        println!("GraphQL (WS):       {}", info.graphql_ws);
        println!("Health:             {}", info.health);
        println!("Collaboration (WS): {}", info.collaboration_ws);
        println!("Session header:     {}: <id>", info.session_header);
        println!("Database file:      {}", info.database);
    }
    Ok(())
}

/// Resolve a `--variables` argument that is either inline JSON or `@path`.
fn resolve_variables(variables: Option<&str>) -> Result<serde_json::Value> {
    match variables {
        None => Ok(json!({})),
        Some(v) => {
            let raw = if let Some(path) = v.strip_prefix('@') {
                std::fs::read_to_string(path)
                    .with_context(|| format!("reading variables file {}", path))?
            } else {
                v.to_string()
            };
            serde_json::from_str(&raw).context("parsing --variables JSON")
        }
    }
}

/// POST a GraphQL operation to a running server and print the JSON response.
pub async fn call(
    query: &str,
    variables: Option<&str>,
    url: Option<&str>,
    host: &str,
    port: u16,
    session: Option<&str>,
) -> Result<()> {
    let endpoint = format!("{}/graphql", base_url(url, host, port));
    let vars = resolve_variables(variables)?;

    let body = json!({ "query": query, "variables": vars });

    let client = reqwest::Client::new();
    let mut req = client
        .post(&endpoint)
        .header("content-type", "application/json");
    if let Some(s) = session {
        req = req.header("x-layercake-session", s);
    }

    let response = req
        .json(&body)
        .send()
        .await
        .with_context(|| format!("POST {} (is a server running there?)", endpoint))?;

    let status = response.status();
    let text = response.text().await?;

    // Pretty-print if the body is JSON; otherwise print raw + surface non-2xx.
    match serde_json::from_str::<serde_json::Value>(&text) {
        Ok(value) => println!("{}", serde_json::to_string_pretty(&value)?),
        Err(_) => println!("{}", text),
    }

    if !status.is_success() {
        return Err(anyhow!("server returned HTTP {}", status));
    }
    Ok(())
}
