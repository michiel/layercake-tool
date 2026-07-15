//! `layercake schema dump` — emit the GraphQL API surface.
//!
//! Delegates to `layercake-server`, which builds the schema standalone (no
//! database or request context needed to generate the type system), so agents
//! can learn the full API without booting a server.

use anyhow::Result;

/// Print the GraphQL SDL. With `json`, print the introspection result instead.
pub async fn dump(json: bool) -> Result<()> {
    if json {
        println!("{}", layercake_server::graphql::introspection_json().await?);
    } else {
        print!("{}", layercake_server::graphql::sdl());
    }
    Ok(())
}
