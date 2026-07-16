//! `layercake doctor` — scan a project for structural health problems.

use anyhow::{anyhow, Result};
use layercake_core::database::connection::establish_connection;
use layercake_core::doctor::{run_diagnostics, Severity};
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};
use std::time::Duration;

/// Whether a table exists in the connected SQLite database.
async fn has_table(db: &DatabaseConnection, name: &str) -> Result<bool> {
    let stmt = Statement::from_sql_and_values(
        db.get_database_backend(),
        "SELECT name FROM sqlite_master WHERE type='table' AND name=?",
        [name.into()],
    );
    Ok(db.query_one(stmt).await?.is_some())
}

/// Resolve the database path: explicit `--database` wins; otherwise ask a
/// running server's `/health` (so `doctor --project N` works cwd-independently
/// when the server is up).
async fn resolve_database(
    database: Option<&str>,
    url: Option<&str>,
    host: &str,
    port: u16,
) -> Result<String> {
    if let Some(db) = database {
        return Ok(db.to_string());
    }
    let base = match url {
        Some(u) => u.trim_end_matches('/').to_string(),
        None => format!("http://{}:{}", host, port),
    };
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(800))
        .build()?;
    let resp = client
        .get(format!("{}/health", base))
        .send()
        .await
        .map_err(|e| {
            anyhow!(
                "no --database given and could not reach a server at {} to resolve it ({}). \
                 Pass --database <path>, or --host/--port/--url of a running server.",
                base,
                e
            )
        })?;
    let body: serde_json::Value = resp.json().await?;
    body.get("database")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| {
            anyhow!(
                "server at {} did not report a database path (older server?). Pass --database.",
                base
            )
        })
}

pub async fn run(
    project: i32,
    database: Option<&str>,
    url: Option<&str>,
    host: &str,
    port: u16,
    strict: bool,
    json: bool,
) -> Result<()> {
    let database = resolve_database(database, url, host, port).await?;

    // Never create a database by resolving to a wrong/relative path — the file
    // must already exist, and open it read-only so diagnostics can't mutate it.
    if database != ":memory:" && !std::path::Path::new(&database).exists() {
        return Err(anyhow!(
            "database file does not exist: {}\n(if a server is running, pass --port/--url so \
             doctor resolves its absolute path, or pass an absolute --database)",
            database
        ));
    }
    let db_url = if database == ":memory:" {
        "sqlite::memory:".to_string()
    } else {
        format!("sqlite://{}?mode=ro", database)
    };
    let db = establish_connection(&db_url).await?;

    // Sentinel: a real layercake DB has a `plans` table. Fail clearly rather
    // than propagating a raw "no such table" from a check.
    if !has_table(&db, "plans").await? {
        return Err(anyhow!(
            "this doesn't look like a layercake database (no 'plans' table): {}",
            database
        ));
    }

    let report = run_diagnostics(&db, project).await?;

    let warning_count = report
        .findings
        .iter()
        .filter(|f| f.severity == Severity::Warning)
        .count();

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return finish(report.error_count(), warning_count, strict);
    }

    if report.is_healthy() {
        println!("✓ project {}: no problems found", project);
        return Ok(());
    }

    println!("Project {} — {} finding(s):\n", project, report.findings.len());
    for f in &report.findings {
        let tag = match f.severity {
            Severity::Error => "ERROR",
            Severity::Warning => "WARN ",
            Severity::Info => "INFO ",
        };
        println!("  [{}] {}: {}", tag, f.check, f.message);
    }
    finish(report.error_count(), warning_count, strict)
}

fn finish(error_count: usize, warning_count: usize, strict: bool) -> Result<()> {
    let fail = error_count > 0 || (strict && warning_count > 0);
    if fail {
        // Non-zero exit so scripts/CI can gate on it.
        std::process::exit(1);
    }
    Ok(())
}
