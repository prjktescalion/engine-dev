//! Editor settings — persisted to the user's config directory.

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

use crate::model::EditorSettings;

fn settings_path() -> Result<PathBuf> {
    let base = dirs::config_dir().context("no config dir on this platform")?;
    Ok(base.join("neudel").join("settings.json"))
}

pub fn load() -> EditorSettings {
    try_load().unwrap_or_default()
}

fn try_load() -> Result<EditorSettings> {
    let path = settings_path()?;
    if !path.exists() {
        return Ok(EditorSettings::default());
    }
    let raw = fs::read_to_string(&path)?;
    let s = serde_json::from_str(&raw)?;
    Ok(s)
}

pub fn save(settings: &EditorSettings) -> Result<()> {
    let path = settings_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(settings)?;
    fs::write(&path, json)?;
    Ok(())
}
