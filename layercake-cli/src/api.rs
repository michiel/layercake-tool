//! `layercake api` — agent-facing helpers for a running server.
//!
//! `info` prints the endpoints/headers an agent needs; `call` POSTs a GraphQL
//! operation to a running instance. This is the HTTP path (talks to a live
//! `layercake serve`), distinct from `layercake query`, which hits the DB file
//! directly with no server.

use anyhow::{anyhow, Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use serde_json::json;
use std::time::Duration;
use tokio_tungstenite::tungstenite::Message;

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
    /// Whether the server actually responded on `/health`.
    reachable: bool,
    /// The version reported by the running server, if reachable.
    server_version: Option<String>,
    /// If the requested port wasn't reachable, other localhost ports that are.
    detected_ports: Vec<u16>,
}

/// Probe `/health` at a base URL; return the reported version on success.
async fn probe_health(base: &str) -> Option<String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(600))
        .build()
        .ok()?;
    let resp = client.get(format!("{}/health", base)).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let body: serde_json::Value = resp.json().await.ok()?;
    // Treat any healthy JSON as reachable; surface version if present.
    Some(
        body.get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
    )
}

/// Print the endpoints and headers for a running server, verifying against a
/// live `/health` probe rather than trusting the requested port blindly.
pub async fn info(
    url: Option<&str>,
    host: &str,
    port: u16,
    database: &str,
    json_out: bool,
) -> Result<()> {
    let base = base_url(url, host, port);
    let ws_base = base.replacen("http", "ws", 1);

    let server_version = probe_health(&base).await;
    let reachable = server_version.is_some();

    // If the requested endpoint isn't answering, scan a few common local ports
    // so the user isn't stuck on the "wrong port" trap.
    let mut detected_ports = Vec::new();
    if !reachable && url.is_none() {
        for candidate in [3000u16, 3001, 8080, 8000] {
            if candidate == port {
                continue;
            }
            let candidate_base = format!("http://{}:{}", host, candidate);
            if probe_health(&candidate_base).await.is_some() {
                detected_ports.push(candidate);
            }
        }
    }

    let info = ApiInfo {
        graphql: format!("{}/graphql", base),
        graphql_ws: format!("{}/graphql/ws", ws_base),
        health: format!("{}/health", base),
        collaboration_ws: format!("{}/ws/collaboration?project_id=<N>", ws_base),
        session_header: "x-layercake-session",
        database: database.to_string(),
        base_url: base,
        reachable,
        server_version,
        detected_ports,
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
        match &info.server_version {
            Some(v) => println!("Server:             reachable (version {})", v),
            None => {
                println!("Server:             NOT reachable at this address");
                if !info.detected_ports.is_empty() {
                    let hint: Vec<String> =
                        info.detected_ports.iter().map(|p| p.to_string()).collect();
                    println!(
                        "                    a layercake server IS answering on port(s): {} — try --port {}",
                        hint.join(", "),
                        info.detected_ports[0]
                    );
                }
            }
        }
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

/// Connect to the collaboration WebSocket and hold a presence session so the
/// agent appears as a collaborator in the multi-user UI until interrupted.
pub async fn join(
    project: i32,
    name: String,
    id: Option<String>,
    color: Option<String>,
    is_agent: bool,
    url: Option<&str>,
    host: &str,
    port: u16,
) -> Result<()> {
    let base = base_url(url, host, port);
    let ws_base = base.replacen("http", "ws", 1);
    let ws_url = format!("{}/ws/collaboration?project_id={}", ws_base, project);

    // A stable per-session id if the caller didn't supply one. Kept simple —
    // uniqueness only needs to hold within a running server.
    let user_id = id.unwrap_or_else(|| format!("agent-{}-{}", project, std::process::id()));
    let avatar_color = color.unwrap_or_else(|| "#7c3aed".to_string());

    eprintln!("Connecting to {} …", ws_url);
    let (mut ws, _resp) = tokio_tungstenite::connect_async(&ws_url)
        .await
        .with_context(|| format!("connecting to {} (is a server running there?)", ws_url))?;

    // Announce presence.
    let join = json!({
        "type": "join_session",
        "data": {
            "userId": user_id,
            "userName": name,
            "avatarColor": avatar_color,
            "isAgent": is_agent,
        }
    });
    ws.send(Message::Text(join.to_string().into())).await?;
    eprintln!(
        "Joined project {} as \"{}\" ({}). Press Ctrl-C to leave.",
        project, name, user_id
    );

    // Heartbeat + drain incoming messages until Ctrl-C or the socket closes.
    let mut ticker = tokio::time::interval(Duration::from_secs(10));
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                if ws.send(Message::Text(json!({"type":"ping"}).to_string().into())).await.is_err() {
                    eprintln!("Connection closed by server.");
                    break;
                }
            }
            msg = ws.next() => {
                match msg {
                    Some(Ok(Message::Text(t))) => {
                        // Surface presence/errors on stdout so agents can react.
                        println!("{}", t);
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        eprintln!("Connection closed.");
                        break;
                    }
                    Some(Err(e)) => {
                        return Err(anyhow!("websocket error: {}", e));
                    }
                    _ => {}
                }
            }
            _ = tokio::signal::ctrl_c() => {
                eprintln!("\nLeaving …");
                let leave = json!({"type":"leave_session","data":{"documentId":null}});
                let _ = ws.send(Message::Text(leave.to_string().into())).await;
                let _ = ws.close(None).await;
                break;
            }
        }
    }
    Ok(())
}
