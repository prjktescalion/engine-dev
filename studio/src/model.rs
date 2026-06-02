//! Studio data model — entities, components, assets, settings.
//!
//! The on-disk scene format (`.ndscene`) is JSON-compatible with the legacy
//! TypeScript studio, so files written by the old editor still load here.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Component {
    Transform(TransformComponent),
    Sprite(SpriteComponent),
    Script(ScriptComponent),
    Velocity(VelocityComponent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformComponent {
    pub x: f32,
    pub y: f32,
    #[serde(rename = "scaleX")]
    pub scale_x: f32,
    #[serde(rename = "scaleY")]
    pub scale_y: f32,
    pub rotation: f32,
}

impl Default for TransformComponent {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteComponent {
    #[serde(rename = "assetPath")]
    pub asset_path: String,
    #[serde(rename = "dataUrl")]
    pub data_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptComponent {
    #[serde(rename = "filePath")]
    pub file_path: String,
    pub lang: ScriptLang,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VelocityComponent {
    pub vx: f32,
    pub vy: f32,
    #[serde(default)]
    pub vrot: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScriptLang {
    Rust,
    Java,
    Python,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub components: Vec<Component>,
}

impl Entity {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            components: Vec::new(),
        }
    }

    pub fn transform(&self) -> Option<&TransformComponent> {
        self.components.iter().find_map(|c| match c {
            Component::Transform(t) => Some(t),
            _ => None,
        })
    }

    pub fn transform_mut(&mut self) -> Option<&mut TransformComponent> {
        self.components.iter_mut().find_map(|c| match c {
            Component::Transform(t) => Some(t),
            _ => None,
        })
    }

    pub fn sprite(&self) -> Option<&SpriteComponent> {
        self.components.iter().find_map(|c| match c {
            Component::Sprite(s) => Some(s),
            _ => None,
        })
    }

    pub fn velocity(&self) -> Option<&VelocityComponent> {
        self.components.iter().find_map(|c| match c {
            Component::Velocity(v) => Some(v),
            _ => None,
        })
    }

    pub fn velocity_mut(&mut self) -> Option<&mut VelocityComponent> {
        self.components.iter_mut().find_map(|c| match c {
            Component::Velocity(v) => Some(v),
            _ => None,
        })
    }

    pub fn ensure_velocity(&mut self) -> &mut VelocityComponent {
        if self.velocity().is_none() {
            self.components
                .push(Component::Velocity(VelocityComponent::default()));
        }
        self.velocity_mut().expect("just inserted")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneFile {
    pub version: u32,
    pub name: String,
    pub entities: Vec<Entity>,
}

impl SceneFile {
    pub const CURRENT_VERSION: u32 = 1;

    pub fn new(name: impl Into<String>) -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            name: name.into(),
            entities: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssetKind {
    Image,
    Script,
    Audio,
    Other,
    Dir,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub kind: AssetKind,
    pub is_dir: bool,
    pub children: Vec<Asset>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EditorChoice {
    Vscode,
    Jetbrains,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Dark,
    Light,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorSettings {
    pub editor: EditorChoice,
    pub custom_path: String,
    pub theme: Theme,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            editor: EditorChoice::Vscode,
            custom_path: String::new(),
            theme: Theme::Dark,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Select,
    Move,
    Scale,
    Rotate,
}
