//! Ensures `<workspace>/web/dist/index.html` exists when the `serve-dashboard`
//! feature is enabled, so `rust_embed`'s macro doesn't fail on a fresh
//! checkout where `pnpm build` hasn't run yet. The placeholder is only
//! created if the file is missing — a real build is never overwritten.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    if std::env::var_os("CARGO_FEATURE_SERVE_DASHBOARD").is_none() {
        return;
    }

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is set by cargo");
    let dist_dir = std::path::PathBuf::from(&manifest_dir)
        .join("..")
        .join("..")
        .join("web")
        .join("dist");
    let index = dist_dir.join("index.html");

    println!("cargo:rerun-if-changed={}", index.display());

    if index.exists() {
        return;
    }

    if let Err(e) = std::fs::create_dir_all(&dist_dir) {
        println!(
            "cargo:warning=could not create {}: {e}",
            dist_dir.display()
        );
        return;
    }

    let placeholder = "<!doctype html><html lang=\"en\"><head><meta charset=\"UTF-8\"><title>know-now dashboard</title></head><body><h1>Dashboard not built</h1><p>Run <code>pnpm build</code> in <code>web/</code> and rebuild the binary.</p></body></html>\n";

    if let Err(e) = std::fs::write(&index, placeholder) {
        println!("cargo:warning=could not write {}: {e}", index.display());
    }
}
