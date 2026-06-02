//! Root view — assembles the menu bar, four panels, and settings modal.

use std::time::Duration;

use gpui::{
    div, prelude::*, px, rgb, App, Context, Entity, EventEmitter, IntoElement, ParentElement,
    Render, Styled, Task, Window,
};

use super::menubar::MenuBar;
use super::panels::{
    asset_browser::AssetBrowser, canvas::SceneCanvas, console::Console, hierarchy::Hierarchy,
    inspector::Inspector, settings_modal::SettingsModal,
};
use super::theme;
use crate::services::scene as scene_svc;
use crate::state::{PlayState, StudioState};

/// Holds modal/transient UI flags + dispatches dialog actions.
pub struct StudioActions {
    pub state: Entity<StudioState>,
    pub show_settings: bool,
    /// Handle to the running play loop. Dropped on stop so the task exits.
    pub play_task: Option<Task<()>>,
}

pub struct ActionsChanged;
impl EventEmitter<ActionsChanged> for StudioActions {}

impl StudioActions {
    pub fn new(state: Entity<StudioState>) -> Self {
        Self {
            state,
            show_settings: false,
            play_task: None,
        }
    }

    pub fn play(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        // Already running? Resume if paused.
        let current = self.state.read(cx).play_state;
        match current {
            PlayState::Playing => return,
            PlayState::Paused => {
                self.state.update(cx, |s, cx| s.pause_play(cx));
                return;
            }
            PlayState::Stopped => {}
        }

        self.state.update(cx, |s, cx| s.start_play(cx));

        // Spawn the play loop. ~60Hz tick. Loop exits when play_state leaves
        // Playing|Paused (i.e. on Stop), or when the entity goes away.
        let state = self.state.clone();
        let task = cx.spawn(async move |_this, cx| {
            let executor = cx.background_executor().clone();
            let dt = 1.0_f32 / 60.0;
            let dur = Duration::from_secs_f32(dt);
            loop {
                executor.timer(dur).await;
                let keep_going = state.update(cx, |s, cx| match s.play_state {
                    PlayState::Playing => {
                        s.step_engine(dt, cx);
                        true
                    }
                    PlayState::Paused => true,
                    PlayState::Stopped => false,
                });
                if !keep_going {
                    break;
                }
            }
        });
        self.play_task = Some(task);
        cx.emit(ActionsChanged);
        cx.notify();
    }

    pub fn pause(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| s.pause_play(cx));
        cx.emit(ActionsChanged);
        cx.notify();
    }

    pub fn stop(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.state.update(cx, |s, cx| s.stop_play(cx));
        // Drop the task handle; the loop notices Stopped and exits.
        self.play_task = None;
        cx.emit(ActionsChanged);
        cx.notify();
    }

    /// Drop a few entities with velocities into the scene so Play has
    /// something visible to do.
    pub fn spawn_demo(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        use crate::model::{Component, Entity as SceneEntity, TransformComponent, VelocityComponent};
        self.state.update(cx, |s, cx| {
            let seeds = [
                ("Rocket A", 200.0, 200.0, 120.0, 60.0),
                ("Rocket B", 500.0, 350.0, -90.0, 80.0),
                ("Rocket C", 320.0, 450.0, 70.0, -100.0),
            ];
            for (name, x, y, vx, vy) in seeds {
                let mut e = SceneEntity::new(name);
                e.components.push(Component::Transform(TransformComponent {
                    x,
                    y,
                    ..Default::default()
                }));
                e.components
                    .push(Component::Velocity(VelocityComponent { vx, vy, vrot: 0.5 }));
                s.add_entity(e, cx);
            }
            s.log("Spawned 3 demo entities. Press Play.", cx);
        });
    }

    pub fn toggle_settings(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.show_settings = !self.show_settings;
        cx.emit(ActionsChanged);
        cx.notify();
    }

    pub fn new_scene(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let state = self.state.clone();
        state.update(cx, |s, cx| s.new_scene(cx));
    }

    pub fn open_project(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let Some(path) = rfd::FileDialog::new().pick_folder() else {
            return;
        };
        self.state.update(cx, |s, cx| s.open_project(path, cx));
    }

    pub fn save_scene(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let existing = self.state.read(cx).scene_path.clone();
        let path = match existing {
            Some(p) => p,
            None => match rfd::FileDialog::new()
                .add_filter("NeuDel scene", &["ndscene"])
                .save_file()
            {
                Some(p) => p,
                None => return,
            },
        };
        let state = self.state.clone();
        state.update(cx, |s, cx| {
            if let Err(e) = s.save_scene(path, cx) {
                s.log(format!("save failed: {e}"), cx);
            }
        });
    }

    pub fn load_scene(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("NeuDel scene", &["ndscene"])
            .pick_file()
        else {
            return;
        };
        let state = self.state.clone();
        state.update(cx, |s, cx| {
            if let Err(e) = s.load_scene(path, cx) {
                s.log(format!("load failed: {e}"), cx);
            }
        });
        // Suppress unused-import warning for the trait import while keeping
        // the symbol available for future raw-file operations.
        let _ = scene_svc::load;
    }
}

pub struct Studio {
    state: Entity<StudioState>,
    actions: Entity<StudioActions>,
    menubar: Entity<MenuBar>,
    hierarchy: Entity<Hierarchy>,
    canvas: Entity<SceneCanvas>,
    inspector: Entity<Inspector>,
    assets: Entity<AssetBrowser>,
    console: Entity<Console>,
    settings_modal: Entity<SettingsModal>,
}

impl Studio {
    pub fn new(state: Entity<StudioState>, cx: &mut Context<Self>) -> Self {
        let actions = cx.new(|_cx| StudioActions::new(state.clone()));
        cx.observe(&state, |_, _, cx| cx.notify()).detach();
        cx.observe(&actions, |_, _, cx| cx.notify()).detach();

        Self {
            menubar: cx.new(|_cx| MenuBar::new(state.clone(), actions.clone())),
            hierarchy: cx.new(|cx| Hierarchy::new(state.clone(), cx)),
            canvas: cx.new(|cx| SceneCanvas::new(state.clone(), cx)),
            inspector: cx.new(|cx| Inspector::new(state.clone(), cx)),
            assets: cx.new(|cx| AssetBrowser::new(state.clone(), cx)),
            console: cx.new(|cx| Console::new(state.clone(), cx)),
            settings_modal: cx.new(|cx| SettingsModal::new(state.clone(), actions.clone(), cx)),
            state,
            actions,
        }
    }
}

impl Render for Studio {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(theme::BG))
            .text_color(rgb(theme::TEXT))
            .text_size(px(13.))
            .child(self.menubar.clone())
            .child(
                // Workbench: hierarchy | canvas | inspector
                div()
                    .flex()
                    .flex_row()
                    .flex_grow(1.0)
                    .min_h(px(0.))
                    .child(self.hierarchy.clone())
                    .child(self.canvas.clone())
                    .child(self.inspector.clone()),
            )
            .child(
                // Bottom dock: assets | console
                div()
                    .flex()
                    .flex_row()
                    .h(px(180.))
                    .border_t_1()
                    .border_color(rgb(theme::BORDER))
                    .child(
                        div()
                            .w(px(420.))
                            .h_full()
                            .border_r_1()
                            .border_color(rgb(theme::BORDER))
                            .bg(rgb(theme::PANEL))
                            .child(self.assets.clone()),
                    )
                    .child(
                        div()
                            .flex_grow(1.0)
                            .h_full()
                            .bg(rgb(theme::PANEL))
                            .child(self.console.clone()),
                    ),
            )
            .child(self.settings_modal.clone())
    }
}

pub fn run(cx: &mut App) {
    use gpui::{size, Bounds, WindowBounds, WindowOptions};
    let bounds = Bounds::centered(None, size(px(1440.0), px(900.0)), cx);
    let opts = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        ..Default::default()
    };
    cx.open_window(opts, |_window, cx| {
        let state = cx.new(|_cx| StudioState::default());
        cx.new(|cx| Studio::new(state, cx))
    })
    .unwrap();
    cx.activate(true);
}
