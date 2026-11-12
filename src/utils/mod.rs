use std::fs;
use std::path::{Path, PathBuf};
use anyhow::bail;

pub fn is_image(p: &Path) -> bool {
    p.extension()
        .and_then(|e| e.to_str())
        .map(|e| matches!(e.to_ascii_lowercase().as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp"))
        .unwrap_or(false)
}

pub fn collect_inputs(inputs: &[PathBuf]) -> anyhow::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for p in inputs {
        if p.is_dir() {
            let mut v: Vec<_> = fs::read_dir(p)?
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.is_file() && is_image(p))
                .collect();
            v.sort();
            files.extend(v);
        } else if p.is_file() {
            files.push(p.clone());
        } else {
            bail!("no existe: {}", p.display());
        }
    }
    Ok(files)
}

pub fn slug(s: &str) -> String {
    let s: String = s
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '_' })
        .collect();
    let s = s.trim_matches('_').to_string();
    if s.is_empty() { "pack".into() } else { s }
}