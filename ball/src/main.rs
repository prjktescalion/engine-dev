//! Game & Watch *Ball* — standalone runtime binary.
//!
//! `cargo run -p ball` — no studio, no gpui: a winit window, the engine's
//! `render` core (sprite batch + LCD post shader), and the engine ECS driving
//! the game state. The simulation runs on a fixed-timestep accumulator whose
//! interval comes from the game (it shortens as you score — a Game & Watch
//! runs on a discrete clock, so balls snap between stations rather than
//! animate). Rendering happens every vsync; between ticks the LCD is static,
//! which is exactly how the real hardware looks.
//!
//! Controls: ←/→ set the hand pose, Space/Enter restarts after game over,
//! Esc quits.

mod game;
mod layout;

use std::sync::Arc;
use std::time::{Duration, Instant};

use engine::renderer::{Atlas, GpuRenderer};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use game::{Game, Hand};
use layout::{Scene, DESIGN};

struct Active {
    window: Arc<Window>,
    renderer: GpuRenderer,
    scene: Scene,
    atlas: Atlas,
    game: Game,
    last: Instant,
    accumulator: Duration,
}

#[derive(Default)]
struct App {
    active: Option<Active>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.active.is_some() {
            return;
        }
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("NeuDel-II — Game & Watch: BALL")
                        .with_inner_size(LogicalSize::new(
                            DESIGN.0 as f64 * 2.0,
                            DESIGN.1 as f64 * 2.0,
                        ))
                        .with_resizable(false),
                )
                .expect("create window"),
        );

        let scene = layout::build_scene();
        let atlas = Atlas::build(&scene.shapes, 512);
        let size = window.inner_size();
        let renderer = GpuRenderer::new(window.clone(), size.width, size.height, &atlas, DESIGN)
            .expect("GPU init failed");

        self.active = Some(Active {
            window,
            renderer,
            scene,
            atlas,
            game: Game::new(),
            last: Instant::now(),
            accumulator: Duration::ZERO,
        });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(active) = &mut self.active else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => active.renderer.resize(size.width, size.height),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => match code {
                KeyCode::ArrowLeft => active.game.set_hand(Hand::Left),
                KeyCode::ArrowRight => active.game.set_hand(Hand::Right),
                KeyCode::Space | KeyCode::Enter => {
                    if active.game.over {
                        active.game.restart();
                        active.accumulator = Duration::ZERO;
                    }
                }
                KeyCode::Escape => event_loop.exit(),
                _ => {}
            },
            WindowEvent::RedrawRequested => {
                // Fixed-timestep simulation, decoupled from frame rate. The
                // interval is re-read every step because scoring shortens it.
                let now = Instant::now();
                active.accumulator += now - active.last;
                active.last = now;
                let mut ticked = false;
                while active.accumulator >= active.game.tick_interval() {
                    active.accumulator -= active.game.tick_interval();
                    active.game.tick();
                    ticked = true;
                }
                if ticked {
                    active.game.sync_transforms(layout::station_pos);
                    let title = if active.game.over {
                        format!(
                            "NeuDel-II — BALL — GAME OVER, score {} (Space to restart)",
                            active.game.score
                        )
                    } else {
                        format!(
                            "NeuDel-II — BALL — score {}  miss {}/3",
                            active.game.score, active.game.misses
                        )
                    };
                    active.window.set_title(&title);
                }

                let frame = layout::sprites(&active.scene, &active.atlas, &active.game);
                if let Err(e) = active.renderer.render(&frame) {
                    eprintln!("render error: {e}");
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(active) = &self.active {
            active.window.request_redraw();
        }
    }
}

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::default();
    event_loop.run_app(&mut app)?;
    Ok(())
}
