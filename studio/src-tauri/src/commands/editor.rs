use tauri::AppHandle;

use crate::commands::settings::get_settings;

fn editor_binary(editor: &str, custom_path: &str) -> String {
    match editor {
        "vscode" => "code".to_string(),
        "jetbrains" => "idea".to_string(),
        "custom" if !custom_path.is_empty() => custom_path.to_string(),
        _ => "code".to_string(),
    }
}

#[tauri::command]
pub fn open_in_editor(
    app: AppHandle,
    file_path: String,
    editor_override: Option<String>,
) -> Result<(), String> {
    let settings = get_settings(app)?;

    let binary = editor_override
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| editor_binary(&settings.editor, &settings.custom_path));

    std::process::Command::new(&binary)
        .arg(&file_path)
        .spawn()
        .map_err(|e| format!("Failed to launch '{}': {}", binary, e))?;

    Ok(())
}
