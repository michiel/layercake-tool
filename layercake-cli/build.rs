//! Ensure the crate recompiles when embedded assets change.
//!
//! `include_dir!` bakes `../docs-tool` into the binary at compile time, but
//! Cargo does not otherwise know that adding/removing a doc file should trigger
//! a rebuild. Emitting `rerun-if-changed` for the directory makes new docs show
//! up without needing to touch a source file.
fn main() {
    println!("cargo:rerun-if-changed=../docs-tool");
}
