use axum::{
    body::Body,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use mime_guess::from_path;
use rust_embed::RustEmbed;

/// Embedded static assets from muse-web's build output (`muse-web/dist`).
///
/// At compile time `rust-embed` reads the directory and stores every file as
/// a binary blob inside the executable. This lets maki serve the SPA without
/// any external web server or filesystem dependency.
///
/// **Build order:** `muse-web` must be built (`pnpm build`) *before* `cargo
/// build` so that `dist/` is populated. The `build.rs` script at the crate
/// root checks for this and runs the build automatically.
#[derive(RustEmbed)]
#[folder = "../muse-web/dist/"]
struct WebAssets;

/// Serve a single embedded asset by path.
fn serve_asset(path: &str) -> Response {
    match WebAssets::get(path) {
        Some(file) => {
            let mime = from_path(path).first_or_octet_stream();
            let mut response = Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref());
            // Hashed filenames in Vite output are safe to cache aggressively.
            // index.html is never cached so SPA updates are picked up.
            if path != "index.html" {
                response = response.header(
                    header::CACHE_CONTROL,
                    "public, max-age=31536000, immutable",
                );
            } else {
                response = response.header(header::CACHE_CONTROL, "no-cache");
            }
            response
                .body(Body::from(file.data.into_owned()))
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

/// Axum handler for `/assets/*` and other static files at the root level
/// (favicon, manifest, etc.).
pub async fn static_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    if path.is_empty() {
        return serve_asset("index.html");
    }
    serve_asset(path)
}

/// SPA fallback: for any non-API, non-auth route, serve `index.html` so
/// client-side routing works (e.g. `/album/123`, `/artist/45`).
pub async fn spa_fallback() -> Response {
    serve_asset("index.html")
}
