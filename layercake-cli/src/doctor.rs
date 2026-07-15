//! `layercake doctor` — scan a project for structural health problems.

use anyhow::Result;
use layercake_core::database::connection::{establish_connection, get_database_url};
use layercake_core::doctor::{run_diagnostics, Severity};

pub async fn run(project: i32, database: Option<&str>, json: bool) -> Result<()> {
    let db = establish_connection(&get_database_url(database)).await?;
    let report = run_diagnostics(&db, project).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return finish(report.error_count());
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
    finish(report.error_count())
}

fn finish(error_count: usize) -> Result<()> {
    if error_count > 0 {
        // Non-zero exit so scripts/CI can gate on it.
        std::process::exit(1);
    }
    Ok(())
}
