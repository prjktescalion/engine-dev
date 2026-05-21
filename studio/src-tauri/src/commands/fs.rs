use base64::Engine;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Asset {
    pub id: String,
    pub name: String,
    pub path: String,
    pub kind: String,
    pub is_dir: bool,
    pub children: Vec<Asset>,
}

fn ext_kind(ext: &str) -> &'static str {
    match ext {
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg" | "webp" => "image",
        "rs" | "java" | "py" | "js" | "ts" | "lua" | "rhai" => "script",
        "wav" | "mp3" | "ogg" | "flac" | "aac" => "audio",
        _ => "other",
    }
}

fn path_id(s: &str) -> String {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    format!("{:016x}", std::hash::Hasher::finish(&h))
}

fn scan_dir(dir: &Path, depth: u32) -> Vec<Asset> {
    if depth > 8 {
        return vec![];
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return vec![];
    };
    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|e| {
        let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
        (!is_dir, e.file_name())
    });

    entries
        .into_iter()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || name == "node_modules" || name == "target" {
                return None;
            }
            let entry_path = entry.path();
            let path_str = entry_path.to_string_lossy().to_string();
            let is_dir = entry_path.is_dir();

            let kind = if is_dir {
                "dir".to_string()
            } else {
                let ext = entry_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                ext_kind(&ext).to_string()
            };

            let children = if is_dir {
                scan_dir(&entry_path, depth + 1)
            } else {
                vec![]
            };

            Some(Asset {
                id: path_id(&path_str),
                name,
                path: path_str,
                kind,
                is_dir,
                children,
            })
        })
        .collect()
}

#[tauri::command]
pub fn list_dir(path: String) -> Result<Vec<Asset>, String> {
    let dir = Path::new(&path);
    if !dir.is_dir() {
        return Err(format!("Not a directory: {}", path));
    }
    Ok(scan_dir(dir, 0))
}

#[tauri::command]
pub fn create_script(path: String, lang: String) -> Result<(), String> {
    let content = match lang.as_str() {
        "rust" => "// NeuDel-II Rust script\n\npub fn init() {}\n\npub fn update(_delta: f32) {}\n",
        "java" => "// NeuDel-II Java script\n\npublic class Script {\n    public void init() {}\n    public void update(float delta) {}\n}\n",
        "python" => "# NeuDel-II Python script\n\ndef init(): pass\n\ndef update(delta: float): pass\n",
        _ => return Err(format!("Unknown language: {}", lang)),
    };
    if let Some(parent) = Path::new(&path).parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn read_image(path: String) -> Result<String, String> {
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    let ext = Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase();
    let mime = match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        _ => "image/png",
    };
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(format!("data:{};base64,{}", mime, b64))
}
