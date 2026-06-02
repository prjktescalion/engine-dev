//! Bottom-left tab — recursive project file tree. Click an image to set it as
//! the pending-drop asset; the next canvas click places it.

use std::path::PathBuf;

use gpui::{
    div, prelude::*, px, rgb, Context, Entity, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, Render, SharedString, Styled, Window,
};

use crate::model::{Asset, AssetKind};
use crate::services::editor as editor_svc;
use crate::state::StudioState;
use crate::ui::theme;

pub struct AssetBrowser {
    pub state: Entity<StudioState>,
}

impl AssetBrowser {
    pub fn new(state: Entity<StudioState>, cx: &mut Context<Self>) -> Self {
        cx.observe(&state, |_, _, cx| cx.notify()).detach();
        Self { state }
    }

    fn icon(kind: AssetKind) -> &'static str {
        match kind {
            AssetKind::Dir => "▸",
            AssetKind::Image => "■",
            AssetKind::Script => "</>",
            AssetKind::Audio => "♪",
            AssetKind::Other => "·",
        }
    }

    fn render_node(&self, asset: &Asset, depth: usize, cx: &mut Context<Self>) -> impl IntoElement {
        let mut col = div().flex().flex_col();
        let kind = asset.kind;
        let path = asset.path.clone();
        let state = self.state.clone();
        let label = asset.name.clone();

        let row = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.))
            .pl(px(8.0 + (depth as f32 * 14.0)))
            .pr(px(8.))
            .py(px(2.))
            .text_color(rgb(theme::TEXT))
            .hover(|s| s.bg(rgb(theme::PANEL_ALT)))
            .cursor_pointer()
            .child(
                div()
                    .w(px(14.))
                    .text_color(rgb(theme::TEXT_DIM))
                    .child(SharedString::from(Self::icon(kind))),
            )
            .child(
                div()
                    .flex_grow(1.0)
                    .text_size(px(12.))
                    .child(SharedString::from(label)),
            )
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |_this, _ev: &MouseDownEvent, _window, cx| {
                    match kind {
                        AssetKind::Image => {
                            let p = path.clone();
                            state.update(cx, |s, cx| {
                                s.pending_drop_asset = Some(p);
                                s.log("Click the canvas to place the asset.", cx);
                            });
                        }
                        AssetKind::Script => {
                            let settings = state.read(cx).settings.clone();
                            if let Err(e) = editor_svc::open_in_editor(&settings, &path) {
                                state.update(cx, |s, cx| s.log(format!("open: {e}"), cx));
                            }
                        }
                        _ => {}
                    }
                }),
            );

        col = col.child(row);

        if asset.is_dir {
            for child in &asset.children {
                col = col.child(self.render_node(child, depth + 1, cx));
            }
        }
        col
    }
}

impl Render for AssetBrowser {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state_ref = self.state.read(cx);
        let assets = state_ref.assets.clone();
        let root = state_ref.project_root.clone();

        let mut body = div().flex().flex_col().w_full().overflow_hidden();
        match (&root, assets.is_empty()) {
            (None, _) => {
                body = body.child(
                    div()
                        .px(px(12.))
                        .py(px(8.))
                        .text_color(rgb(theme::TEXT_DIM))
                        .child(SharedString::from(
                            "Open a project to browse assets (File → Open Project).",
                        )),
                );
            }
            (Some(p), true) => {
                body = body.child(
                    div()
                        .px(px(12.))
                        .py(px(8.))
                        .text_color(rgb(theme::TEXT_DIM))
                        .child(SharedString::from(format!(
                            "No assets in {}",
                            p.display()
                        ))),
                );
            }
            (Some(_), false) => {
                for a in &assets {
                    body = body.child(self.render_node(a, 0, cx));
                }
            }
        }

        div()
            .flex()
            .flex_col()
            .flex_grow(1.0)
            .h_full()
            .child(
                div()
                    .h(px(24.))
                    .px(px(12.))
                    .py(px(4.))
                    .border_b_1()
                    .border_color(rgb(theme::BORDER))
                    .text_color(rgb(theme::TEXT_DIM))
                    .child(SharedString::from("ASSETS")),
            )
            .child(div().id("assets-scroll").overflow_y_scroll().child(body))
    }
}
