//! Scene serialization. `.ndscene` files are JSON, version-tagged.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::model::SceneFile;

pub fn save(path: &Path, scene: &SceneFile) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(scene)?;
    fs::write(path, json).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub fn load(path: &Path) -> Result<SceneFile> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let scene: SceneFile = serde_json::from_str(&raw).context("parse .ndscene")?;
    Ok(scene)
}
