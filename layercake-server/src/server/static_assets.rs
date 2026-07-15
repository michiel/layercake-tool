//! Serves the built web UI (`frontend/dist`) embedded directly in the binary.
//!
//! The whole SPA is compiled into the executable via `include_dir!`, so the
//! server ships as a single self-contained file. Requests are matched against
//! embedded files; anything that does not match a real asset falls back to
//! `index.html` so client-side (browser) routing works.

use axum::{
    body::Body,
    extract::Path,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use include_dir::{include_dir, Dir};

/// The built frontend, embedded at compile time.
///
/// Path is relative to this crate's `CARGO_MANIFEST_DIR` (`layercake-server/`),
/// so `../frontend/dist` points at the workspace-level Vite build output. This
/// directory MUST exist when the crate is compiled — run `npm run frontend:build`
/// before `cargo build`.
static WEB_UI: Dir<'static> = include_dir!("../frontend/dist");

/// Serve an embedded asset by path, falling back to `index.html` for SPA routes.
fn serve_path(path: &str) -> Response {
    // Normalise: strip a leading slash so it matches include_dir's relative keys.
    let trimmed = path.trim_start_matches('/');

    // Root request → index.html
    let lookup = if trimmed.is_empty() { "index.html" } else { trimmed };

    if let Some(file) = WEB_UI.get_file(lookup) {
        return file_response(lookup, file.contents());
    }

    // Not a real asset: serve the SPA shell so the client router can handle it.
    match WEB_UI.get_file("index.html") {
        Some(index) => file_response("index.html", index.contents()),
        None => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Web UI not embedded: frontend/dist was empty at build time",
        )
            .into_response(),
    }
}

fn file_response(path: &str, contents: &'static [u8]) -> Response {
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime.as_ref())
        .body(Body::from(contents))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

/// Axum fallback handler: serve the embedded web UI for any unmatched route.
pub async fn spa_fallback(uri: Uri) -> Response {
    serve_path(uri.path())
}

/// Handler for an explicit `/assets/*path` (or similar) nested route, if wired.
#[allow(dead_code)]
pub async fn serve_asset(Path(path): Path<String>) -> Response {
    serve_path(&path)
}
