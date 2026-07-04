use std::path::PathBuf;

/// Build script for maki.
///
/// Ensures muse-web's `dist/` directory exists before `rust-embed` tries to
/// include it at compile time. If `dist/` is missing (e.g. fresh checkout),
/// attempts to build the frontend automatically.
///
/// Set `MAKI_SKIP_WEB_BUILD=1` to bypass the automatic build (useful in dev
/// when running `vite dev` separately and iterating on Rust code only).
fn main() {
    println!("cargo:rerun-if-changed=../muse-web/dist/index.html");

    if std::env::var("MAKI_SKIP_WEB_BUILD").is_ok() {
        return;
    }

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let web_dir = manifest_dir.join("..").join("muse-web");
    let dist_dir = web_dir.join("dist");

    // If dist/index.html already exists, assume the frontend was built
    // (e.g. by a CI pipeline or a prior build). Don't rebuild unnecessarily.
    if dist_dir.join("index.html").exists() {
        return;
    }

    // Try to build the frontend automatically.
    // Prefer pnpm, fall back to npm.
    let mgr = if web_dir.join("pnpm-lock.yaml").exists() {
        "pnpm"
    } else {
        "npm"
    };

    eprintln!("maki build: muse-web/dist not found, building frontend with {mgr}...");

    let status = std::process::Command::new(mgr)
        .arg("install")
        .current_dir(&web_dir)
        .status();

    if let Ok(s) = status {
        if !s.success() {
            panic!(
                "maki build: {} install failed in muse-web. \
                 Run `cd muse-web && {} install` manually, \
                 or set MAKI_SKIP_WEB_BUILD=1 to skip.",
                mgr, mgr
            );
        }
    }

    let status = std::process::Command::new(mgr)
        .arg("run")
        .arg("build")
        .current_dir(&web_dir)
        .status();

    match status {
        Ok(s) if s.success() => {
            eprintln!("maki build: muse-web built successfully");
        }
        _ => {
            panic!(
                "maki build: Failed to build muse-web. \
                 Run `cd muse-web && {} run build` manually, \
                 or set MAKI_SKIP_WEB_BUILD=1 to skip.",
                mgr
            );
        }
    }
}
