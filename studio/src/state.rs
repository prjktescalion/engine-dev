//! Central studio state. Held in a single GPUI entity; views subscribe to it.

use std::path::PathBuf;

use anyhow::Result;
use gpui::{Context, EventEmitter};

use crate::model::{
    Asset, Component, EditorSettings, Entity, SceneFile, SpriteComponent, Tool, TransformComponent,
};
use crate::services::{fs as fs_svc, scene as scene_svc, settings as settings_svc};

/// Fires whenever a view should re-render.
pub struct StateChanged;

pub struct StudioState {
    pub scene: SceneFile,
    pub scene_path: Option<PathBuf>,
    pub selected_entity: Option<String>,

    pub project_root: Option<PathBuf>,
    pub assets: Vec<Asset>,
    pub assets_loading: bool,
    pub assets_error: Option<String>,

    pub settings: EditorSettings,
    pub tool: Tool,
    pub console: Vec<String>,

    /// When set, the next click on the canvas creates an entity for this asset
    /// at the click position. A primitive stand-in for HTML5 drag-and-drop.
    pub pending_drop_asset: Option<PathBuf>,
}

impl Default for StudioState {
    fn default() -> Self {
        Self {
            scene: SceneFile::new("untitled"),
            scene_path: None,
            selected_entity: None,
            project_root: None,
            assets: Vec::new(),
            assets_loading: false,
            assets_error: None,
            settings: settings_svc::load(),
            tool: Tool::Select,
            console: vec!["NeuDel-II studio ready.".into()],
            pending_drop_asset: None,
        }
    }
}

impl EventEmitter<StateChanged> for StudioState {}

impl StudioState {
    pub fn log(&mut self, msg: impl Into<String>, cx: &mut Context<Self>) {
        self.console.push(msg.into());
        if self.console.len() > 500 {
            let drop = self.console.len() - 500;
            self.console.drain(..drop);
        }
        cx.emit(StateChanged);
        cx.notify();
    }

    pub fn select_entity(&mut self, id: Option<String>, cx: &mut Context<Self>) {
        self.selected_entity = id;
        cx.emit(StateChanged);
        cx.notify();
    }

    pub fn selected(&self) -> Option<&Entity> {
        let id = self.selected_entity.as_ref()?;
        self.scene.entities.iter().find(|e| &e.id == id)
    }

    pub fn selected_mut(&mut self) -> Option<&mut Entity> {
        let id = self.selected_entity.clone()?;
        self.scene.entities.iter_mut().find(|e| e.id == id)
    }

    pub fn add_entity(&mut self, entity: Entity, cx: &mut Context<Self>) {
        let id = entity.id.clone();
        self.scene.entities.push(entity);
        self.selected_entity = Some(id);
        cx.emit(StateChanged);
        cx.notify();
    }

    pub fn remove_entity(&mut self, id: &str, cx: &mut Context<Self>) {
        self.scene.entities.retain(|e| e.id != id);
        if self.selected_entity.as_deref() == Some(id) {
            self.selected_entity = None;
        }
        cx.emit(StateChanged);
        cx.notify();
    }

    pub fn rename_entity(&mut self, id: &str, name: String, cx: &mut Context<Self>) {
        if let Some(e) = self.scene.entities.iter_mut().find(|e| e.id == id) {
            e.name = name;
            cx.emit(StateChanged);
            cx.notify();
        }
    }

    pub fn update_transform(
        &mut self,
        id: &str,
        f: impl FnOnce(&mut TransformComponent),
        cx: &mut Context<Self>,
    ) {
        if let Some(e) = self.scene.entities.iter_mut().find(|e| e.id == id) {
            if let Some(t) = e.transform_mut() {
                f(t);
                cx.emit(StateChanged);
                cx.notify();
            }
        }
    }

    pub fn open_project(&mut self, root: PathBuf, cx: &mut Context<Self>) {
        self.project_root = Some(root.clone());
        self.assets_loading = true;
        self.assets_error = None;
        match fs_svc::list_dir(&root) {
            Ok(assets) => {
                self.assets = assets;
                self.log(format!("Opened project {}", root.display()), cx);
            }
            Err(e) => {
                self.assets_error = Some(e.to_string());
                self.log(format!("Project scan failed: {e}"), cx);
            }
        }
        self.assets_loading = false;
        cx.emit(StateChanged);
        cx.notify();
    }

    pub fn refresh_assets(&mut self, cx: &mut Context<Self>) {
        let Some(root) = self.project_root.clone() else {
            return;
        };
        match fs_svc::list_dir(&root) {
            Ok(a) => self.assets = a,
            Err(e) => self.assets_error = Some(e.to_string()),
        }
        cx.emit(StateChanged);
        cx.notify();
    }

    pub fn new_scene(&mut self, cx: &mut Context<Self>) {
        self.scene = SceneFile::new("untitled");
        self.scene_path = None;
        self.selected_entity = None;
        self.log("New scene.", cx);
    }

    pub fn save_scene(&mut self, path: PathBuf, cx: &mut Context<Self>) -> Result<()> {
        scene_svc::save(&path, &self.scene)?;
        self.scene_path = Some(path.clone());
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            self.scene.name = stem.to_string();
        }
        self.log(format!("Saved scene to {}", path.display()), cx);
        Ok(())
    }

    pub fn load_scene(&mut self, path: PathBuf, cx: &mut Context<Self>) -> Result<()> {
        let scene = scene_svc::load(&path)?;
        self.scene = scene;
        self.scene_path = Some(path.clone());
        self.selected_entity = None;
        self.log(format!("Loaded scene {}", path.display()), cx);
        Ok(())
    }

    pub fn place_asset(&mut self, asset_path: PathBuf, x: f32, y: f32, cx: &mut Context<Self>) {
        let name = asset_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("entity")
            .to_string();
        let data_url = fs_svc::read_image_as_data_url(&asset_path).unwrap_or_default();
        let mut entity = Entity::new(name);
        entity.components.push(Component::Transform(TransformComponent {
            x,
            y,
            ..TransformComponent::default()
        }));
        entity.components.push(Component::Sprite(SpriteComponent {
            asset_path: asset_path.to_string_lossy().into_owned(),
            data_url,
        }));
        self.add_entity(entity, cx);
        self.pending_drop_asset = None;
    }
}
