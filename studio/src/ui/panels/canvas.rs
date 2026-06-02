//! Scene canvas — 32px grid background and absolute-positioned entity sprites.
//!
//! GPUI doesn't ship a ready-made 2D-game canvas, so the renderer is built out
//! of `canvas()` (for the grid) and absolutely-positioned `div`/`img` children
//! (for each entity). Good enough for an editor preview; the real renderer is
//! `engine::renderer` (wgpu) once that's wired up.

use std::path::PathBuf;

use gpui::{
    canvas, div, fill, img, point, prelude::*, px, rgb, size, Bounds, Context, Entity,
    IntoElement, MouseButton, MouseDownEvent, ParentElement, Pixels, Render, SharedString, Styled,
    Window,
};

use crate::state::StudioState;
use crate::ui::theme;

const GRID_STEP: f32 = 32.0;

pub struct SceneCanvas {
    pub state: Entity<StudioState>,
}

impl SceneCanvas {
    pub fn new(state: Entity<StudioState>, cx: &mut Context<Self>) -> Self {
        cx.observe(&state, |_, _, cx| cx.notify()).detach();
        Self { state }
    }
}

impl Render for SceneCanvas {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state_ref = self.state.read(cx);
        let entities = state_ref.scene.entities.clone();
        let selected = state_ref.selected_entity.clone();
        let pending_drop = state_ref.pending_drop_asset.clone();
        let state = self.state.clone();

        let mut surface = div()
            .id("scene-canvas")
            .relative()
            .flex_grow(1.0)
            .h_full()
            .bg(rgb(theme::BG))
            .overflow_hidden()
            // Grid via canvas element.
            .child(
                canvas(
                    |_bounds, _window, _cx| (),
                    move |bounds: Bounds<Pixels>, _state, window, _cx| {
                        let color = theme::rgba(theme::GRID);
                        let mut x = bounds.origin.x;
                        while x < bounds.origin.x + bounds.size.width {
                            let line = Bounds {
                                origin: point(x, bounds.origin.y),
                                size: size(px(1.0), bounds.size.height),
                            };
                            window.paint_quad(fill(line, color));
                            x += px(GRID_STEP);
                        }
                        let mut y = bounds.origin.y;
                        while y < bounds.origin.y + bounds.size.height {
                            let line = Bounds {
                                origin: point(bounds.origin.x, y),
                                size: size(bounds.size.width, px(1.0)),
                            };
                            window.paint_quad(fill(line, color));
                            y += px(GRID_STEP);
                        }
                    },
                )
                .size_full()
                .absolute()
                .top_0()
                .left_0(),
            )
            // Click empty canvas — either place pending asset or deselect.
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |_this, ev: &MouseDownEvent, _window, cx| {
                    let x = f32::from(ev.position.x);
                    let y = f32::from(ev.position.y);
                    state.update(cx, |s, cx| {
                        if let Some(asset) = s.pending_drop_asset.clone() {
                            s.place_asset(asset, x, y, cx);
                        } else {
                            s.select_entity(None, cx);
                        }
                    });
                }),
            );

        // Pending-drop banner.
        if let Some(p) = pending_drop {
            let name = p
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("asset")
                .to_string();
            surface = surface.child(
                div()
                    .absolute()
                    .top(px(8.))
                    .left(px(8.))
                    .px(px(10.))
                    .py(px(4.))
                    .rounded(px(4.))
                    .bg(rgb(theme::ACCENT_DIM))
                    .text_color(rgb(theme::ACCENT))
                    .child(SharedString::from(format!(
                        "Click anywhere to place: {name}"
                    ))),
            );
        }

        // Entities as absolute children.
        for ent in &entities {
            let Some(t) = ent.transform() else { continue };
            let is_sel = selected.as_deref() == Some(ent.id.as_str());
            let w = 64.0 * t.scale_x;
            let h = 64.0 * t.scale_y;
            let id_for_select = ent.id.clone();
            let state_for_select = self.state.clone();

            let mut node = div()
                .absolute()
                .left(px(t.x - w / 2.0))
                .top(px(t.y - h / 2.0))
                .w(px(w))
                .h(px(h))
                .border_2()
                .border_color(rgb(if is_sel { theme::ACCENT } else { theme::BORDER }))
                .bg(rgb(theme::PANEL_ALT))
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgb(theme::TEXT_DIM))
                .cursor_pointer()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |_this, ev: &MouseDownEvent, _window, cx| {
                        let id = id_for_select.clone();
                        state_for_select.update(cx, |s, cx| s.select_entity(Some(id), cx));
                        cx.stop_propagation();
                        let _ = ev;
                    }),
                );

            let has_velocity = ent.velocity().is_some();
            if let Some(sprite) = ent.sprite() {
                let path = PathBuf::from(&sprite.asset_path);
                if path.exists() {
                    node = node.child(img(path).w(px(w)).h(px(h)));
                } else {
                    node = node.child(SharedString::from(ent.name.clone()));
                }
            } else {
                node = node.child(SharedString::from(ent.name.clone()));
            }
            if has_velocity {
                // Tiny ring in the corner that says "this thing moves"
                node = node.child(
                    div()
                        .absolute()
                        .top(px(-4.))
                        .right(px(-4.))
                        .w(px(8.))
                        .h(px(8.))
                        .rounded(px(4.))
                        .bg(rgb(theme::ACCENT)),
                );
            }
            surface = surface.child(node);
        }

        surface
    }
}
