use std::env;
use std::path::{Path, PathBuf};

fn main() {
    if let Some(value) = resolve_api_url() {
        println!("cargo:rustc-env=API_URL={}", value);
    }
}

fn resolve_api_url() -> Option<String> {
    candidate_paths()
        .into_iter()
        .find_map(|path| read_api_url_from(&path))
        .or_else(|| env::var("API_URL").ok())
}

fn read_api_url_from(path: &Path) -> Option<String> {
    let iter = dotenvy::from_path_iter(path).ok()?;
    for item in iter {
        let (key, value) = item.ok()?;
        if key == "API_URL" {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn candidate_paths() -> Vec<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut paths = vec![manifest_dir.join(".env")];

    if let Some(parent) = manifest_dir.parent() {
        paths.push(parent.join(".env"));
        if let Some(root) = parent.parent() {
            paths.push(root.join(".env"));
        }
    }

    paths
}
