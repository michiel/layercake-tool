//! `layercake doc` — print embedded agent-facing documentation.
//!
//! The `docs-tool/` tree (workflow/ and command/ subdirs) is compiled into the
//! binary via `include_dir!`, so docs ship with the binary, work offline, and
//! always match its version. Each `<type>/<name>.md` maps to
//! `layercake doc <type> <name>`.

use anyhow::{anyhow, Result};
use include_dir::{include_dir, Dir};

/// The docs-tool tree, embedded at compile time (path relative to this crate).
static DOCS: Dir<'static> = include_dir!("../docs-tool");

/// Print the named doc from the given subdirectory (`workflow` or `command`).
pub fn print_doc(kind: &str, name: &str) -> Result<()> {
    let path = format!("{}/{}.md", kind, name);
    match DOCS.get_file(&path) {
        Some(file) => {
            let text = file
                .contents_utf8()
                .ok_or_else(|| anyhow!("doc {} is not valid UTF-8", path))?;
            print!("{}", text);
            Ok(())
        }
        None => Err(anyhow!(
            "no {} doc named '{}'.\n\n{}",
            kind,
            name,
            list_string()
        )),
    }
}

/// Print a listing of every available workflow and command doc.
pub fn print_list() {
    print!("{}", list_string());
}

fn list_string() -> String {
    let mut out = String::from("Available documentation:\n");
    for kind in ["workflow", "command", "guide"] {
        out.push_str(&format!("\n{}s:\n", kind));
        let mut names = names_in(kind);
        names.sort();
        if names.is_empty() {
            out.push_str("  (none)\n");
        } else {
            for name in names {
                out.push_str(&format!("  layercake doc {} {}\n", kind, name));
            }
        }
    }
    out
}

fn names_in(kind: &str) -> Vec<String> {
    match DOCS.get_dir(kind) {
        Some(dir) => dir
            .files()
            .iter()
            .filter_map(|f| f.path().file_stem())
            .map(|s| s.to_string_lossy().into_owned())
            .collect(),
        None => Vec::new(),
    }
}
