//! Filesystem services — directory enumeration, image base64 encoding,
//! and script-stub generation. Native replacements for the old Tauri commands.

use anyhow::{Context, Result};
use base64::Engine as _;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::model::{Asset, AssetKind};

const MAX_DEPTH: usize = 8;

/// Recursively scan `root` and return the asset tree. Filters out dotfiles,
/// `node_modules`, and `target` (matching the legacy editor's behavior).
pub fn list_dir(root: &Path) -> Result<Vec<Asset>> {
    list_dir_recursive(root, 0)
}

fn list_dir_recursive(path: &Path, depth: usize) -> Result<Vec<Asset>> {
    if depth >= MAX_DEPTH {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    let entries =
        fs::read_dir(path).with_context(|| format!("read_dir {}", path.display()))?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') || name == "node_modules" || name == "target" {
            continue;
        }
        let entry_path = entry.path();
        let is_dir = entry_path.is_dir();
        let kind = classify(&entry_path, is_dir);
        let children = if is_dir {
            list_dir_recursive(&entry_path, depth + 1).unwrap_or_default()
        } else {
            Vec::new()
        };
        out.push(Asset {
            id: Uuid::new_v4().to_string(),
            name,
            path: entry_path,
            kind,
            is_dir,
            children,
        });
    }
    out.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });
    Ok(out)
}

fn classify(path: &Path, is_dir: bool) -> AssetKind {
    if is_dir {
        return AssetKind::Dir;
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg" | "webp" => AssetKind::Image,
        "rs" | "java" | "py" | "js" | "ts" | "lua" | "rhai" => AssetKind::Script,
        "wav" | "mp3" | "ogg" | "flac" | "aac" => AssetKind::Audio,
        _ => AssetKind::Other,
    }
}

/// Read an image and return a base64 data URL with the right MIME type.
pub fn read_image_as_data_url(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let mime = match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("webp") => "image/webp",
        Some("bmp") => "image/bmp",
        _ => "application/octet-stream",
    };
    let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(format!("data:{};base64,{}", mime, encoded))
}

/// Write a boilerplate script file for the chosen language.
pub fn create_script(path: &Path, lang: &str) -> Result<()> {
    let body = match lang {
        "rust" => RUST_TEMPLATE,
        "java" => JAVA_TEMPLATE,
        "python" => PYTHON_TEMPLATE,
        _ => return Err(anyhow::anyhow!("unknown lang: {lang}")),
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, body)?;
    Ok(())
}

const RUST_TEMPLATE: &str = "// NeuDel-II Rust script\n\npub fn init() {}\n\npub fn update(dt: f32) {\n    let _ = dt;\n}\n";

const JAVA_TEMPLATE: &str = "// NeuDel-II Java script\n\npublic class Script {\n    public static void init() {}\n    public static void update(float dt) {}\n}\n";

const PYTHON_TEMPLATE: &str = "# NeuDel-II Python script\n\ndef init():\n    pass\n\ndef update(dt):\n    pass\n";

/// Convenience: resolve a possibly-relative path against the project root.
pub fn resolve(root: Option<&PathBuf>, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else if let Some(r) = root {
        r.join(path)
    } else {
        path.to_path_buf()
    }
}
