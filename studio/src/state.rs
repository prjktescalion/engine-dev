//! Central studio state. Held in a single GPUI entity; views subscribe to it.

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use engine::ecs::{EntityId, Sprite as EngineSprite, Velocity as EngineVelocity};
use engine::Engine;
use gpui::{Context, EventEmitter};

use crate::model::{
    Asset, Component, EditorSettings, Entity, SceneFile, SpriteComponent, Tool, TransformComponent,
};
use crate::services::{fs as fs_svc, scene as scene_svc, settings as settings_svc};

/// Fires whenever a view should re-render.
pub struct StateChanged;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayState {
    Stopped,
    Playing,
    Paused,
}

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

    pub engine: Engine,
    pub play_state: PlayState,
    /// Maps studio UUID → engine EntityId so transforms can be written back
    /// to the scene each tick.
    pub entity_id_map: HashMap<String, EntityId>,
    /// Saved on play-start so Stop can revert to authored positions.
    pub authored_transforms: HashMap<String, TransformComponent>,
    /// Smoothed ticks-per-second, shown in the menubar status pill.
    pub tps: f32,
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
            engine: Engine::new(),
            play_state: PlayState::Stopped,
            entity_id_map: HashMap::new(),
            authored_transforms: HashMap::new(),
            tps: 0.0,
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

    pub fn update_entity(
        &mut self,
        id: &str,
        f: impl FnOnce(&mut Entity),
        cx: &mut Context<Self>,
    ) {
        if let Some(e) = self.scene.entities.iter_mut().find(|e| e.id == id) {
            f(e);
            cx.emit(StateChanged);
            cx.notify();
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
        self.stop_internal();
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
        self.stop_internal();
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

    // ----- Engine bridge ---------------------------------------------------

    /// Copy authored scene into the engine world. Populates id_map.
    pub fn compile_into_engine(&mut self) {
        self.engine.reset();
        self.entity_id_map.clear();
        for ent in &self.scene.entities {
            let eid = self.engine.spawn(ent.name.clone());
            self.entity_id_map.insert(ent.id.clone(), eid);
            if let Some(world_ent) = self.engine.world.get_mut(eid) {
                if let Some(t) = ent.transform() {
                    world_ent.transform.x = t.x;
                    world_ent.transform.y = t.y;
                    world_ent.transform.scale_x = t.scale_x;
                    world_ent.transform.scale_y = t.scale_y;
                    world_ent.transform.rotation = t.rotation;
                }
                if let Some(v) = ent.velocity() {
                    world_ent.velocity = Some(EngineVelocity {
                        vx: v.vx,
                        vy: v.vy,
                        vrot: v.vrot,
                    });
                }
                if let Some(s) = ent.sprite() {
                    world_ent.sprite = Some(EngineSprite {
                        asset_path: s.asset_path.clone(),
                    });
                }
            }
        }
    }

    pub fn start_play(&mut self, cx: &mut Context<Self>) {
        self.authored_transforms = self
            .scene
            .entities
            .iter()
            .filter_map(|e| e.transform().map(|t| (e.id.clone(), t.clone())))
            .collect();
        self.compile_into_engine();
        self.play_state = PlayState::Playing;
        self.log(
            format!("Play — {} entities in world.", self.engine.entity_count()),
            cx,
        );
    }

    pub fn pause_play(&mut self, cx: &mut Context<Self>) {
        if self.play_state == PlayState::Playing {
            self.play_state = PlayState::Paused;
            self.log("Paused.", cx);
        } else if self.play_state == PlayState::Paused {
            self.play_state = PlayState::Playing;
            self.log("Resumed.", cx);
        }
    }

    pub fn stop_play(&mut self, cx: &mut Context<Self>) {
        if self.play_state == PlayState::Stopped {
            return;
        }
        self.stop_internal();
        self.log("Stopped — reverted to authored transforms.", cx);
    }

    fn stop_internal(&mut self) {
        // Restore authored transforms onto the scene.
        for ent in &mut self.scene.entities {
            if let Some(orig) = self.authored_transforms.get(&ent.id) {
                if let Some(t) = ent.transform_mut() {
                    *t = orig.clone();
                }
            }
        }
        self.play_state = PlayState::Stopped;
        self.engine.reset();
        self.entity_id_map.clear();
        self.authored_transforms.clear();
        self.tps = 0.0;
    }

    /// Run one simulation step and copy positions back into the scene so the
    /// canvas re-renders. Called by the play-loop task.
    pub fn step_engine(&mut self, dt: f32, cx: &mut Context<Self>) {
        if self.play_state != PlayState::Playing {
            return;
        }
        self.engine.tick(dt);
        // EMA over ticks-per-second.
        let inst = if dt > 0.0 { 1.0 / dt } else { 0.0 };
        self.tps = self.tps * 0.9 + inst * 0.1;

        // Write engine transforms back to the scene so the canvas updates.
        for ent in &mut self.scene.entities {
            let Some(&eid) = self.entity_id_map.get(&ent.id) else {
                continue;
            };
            let Some(world_ent) = self.engine.world.get(eid) else {
                continue;
            };
            if let Some(t) = ent.transform_mut() {
                t.x = world_ent.transform.x;
                t.y = world_ent.transform.y;
                t.rotation = world_ent.transform.rotation;
            }
        }
        cx.emit(StateChanged);
        cx.notify();
    }
}
