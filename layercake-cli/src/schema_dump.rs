//! `layercake schema dump` — emit the GraphQL API surface.
//!
//! Delegates to `layercake-server`, which builds the schema standalone (no
//! database or request context needed to generate the type system), so agents
//! can learn the full API without booting a server.

use anyhow::{anyhow, Result};

/// Print the GraphQL SDL. Filters narrow the output; `json` prints the
/// introspection result instead (ignores filters).
pub async fn dump(json: bool, only_mutations: bool, only_inputs: bool) -> Result<()> {
    if json {
        println!("{}", layercake_server::graphql::introspection_json().await?);
        return Ok(());
    }

    let sdl = layercake_server::graphql::sdl();
    if !only_mutations && !only_inputs {
        print!("{}", sdl);
        return Ok(());
    }

    for block in sdl_blocks(&sdl) {
        let keep = (only_inputs && block.trim_start().starts_with("input "))
            || (only_mutations && is_mutation_block(&block));
        if keep {
            println!("{}\n", block.trim_end());
        }
    }
    Ok(())
}

/// Print just the named type's SDL block (type / input / enum / interface /
/// union / scalar).
pub fn print_type(name: &str) -> Result<()> {
    let sdl = layercake_server::graphql::sdl();
    for block in sdl_blocks(&sdl) {
        if block_defines(&block, name) {
            print!("{}", block);
            return Ok(());
        }
    }
    Err(anyhow!(
        "type '{}' not found in the schema (try `layercake schema dump | grep -i {}`)",
        name,
        name
    ))
}

/// Split an SDL string into top-level definition blocks. A block starts at a
/// line beginning a definition keyword and runs to its closing `}` (or the line
/// itself for one-line defs like `scalar`).
fn sdl_blocks(sdl: &str) -> Vec<String> {
    const KEYWORDS: [&str; 6] = ["type ", "input ", "enum ", "interface ", "union ", "scalar "];
    let mut blocks = Vec::new();
    let mut current = String::new();
    let mut depth = 0i32;
    let mut in_block = false;

    for line in sdl.lines() {
        if !in_block && KEYWORDS.iter().any(|k| line.starts_with(k)) {
            in_block = true;
            current = String::new();
        }
        if in_block {
            current.push_str(line);
            current.push('\n');
            depth += line.matches('{').count() as i32;
            depth -= line.matches('}').count() as i32;

            let block_has_brace = current.contains('{');
            let complete = if block_has_brace {
                depth <= 0 // multi-line brace block closed
            } else {
                true // one-line def (e.g. `scalar Foo`)
            };
            if complete {
                blocks.push(std::mem::take(&mut current));
                in_block = false;
                depth = 0;
            }
        }
    }
    if !current.trim().is_empty() {
        blocks.push(current);
    }
    blocks
}

fn block_defines(block: &str, name: &str) -> bool {
    let first = block.lines().next().unwrap_or("");
    // "type Foo {" / "input Foo {" / "enum Foo {" / "scalar Foo"
    first
        .split_whitespace()
        .nth(1)
        .map(|n| n.trim_end_matches('{').trim() == name)
        .unwrap_or(false)
}

fn is_mutation_block(block: &str) -> bool {
    let first = block.lines().next().unwrap_or("");
    first.starts_with("type Mutation")
}
