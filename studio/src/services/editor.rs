//! Launch an external editor for a script file.

use anyhow::{anyhow, Result};
use std::path::Path;
use std::process::Command;

use crate::model::{EditorChoice, EditorSettings};

pub fn open_in_editor(settings: &EditorSettings, file: &Path) -> Result<()> {
    let bin = match settings.editor {
        EditorChoice::Vscode => "code".to_string(),
        EditorChoice::Jetbrains => "idea".to_string(),
        EditorChoice::Custom => {
            if settings.custom_path.trim().is_empty() {
                return Err(anyhow!("custom editor path is empty"));
            }
            settings.custom_path.clone()
        }
    };
    Command::new(&bin)
        .arg(file)
        .spawn()
        .map_err(|e| anyhow!("failed to launch `{bin}`: {e}"))?;
    Ok(())
}
