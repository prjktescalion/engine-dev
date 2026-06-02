//! Right panel — properties of the selected entity.
//!
//! No inline numeric input yet; values are nudged via +/- buttons.

use gpui::{
    div, prelude::*, px, rgb, Context, Entity, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, Render, SharedString, Styled, Window,
};

use crate::model::Component;
use crate::services::editor as editor_svc;
use crate::state::StudioState;
use crate::ui::theme;

pub struct Inspector {
    pub state: Entity<StudioState>,
}

impl Inspector {
    pub fn new(state: Entity<StudioState>, cx: &mut Context<Self>) -> Self {
        cx.observe(&state, |_, _, cx| cx.notify()).detach();
        Self { state }
    }

    fn nudge_button(
        &self,
        label: &'static str,
        cx: &mut Context<Self>,
        on_click: impl Fn(&mut StudioState, &mut Context<StudioState>) + 'static,
    ) -> impl IntoElement {
        let state = self.state.clone();
        div()
            .px(px(6.))
            .py(px(2.))
            .rounded(px(3.))
            .bg(rgb(theme::PANEL_ALT))
            .text_color(rgb(theme::TEXT))
            .hover(|s| s.bg(rgb(theme::BORDER)))
            .cursor_pointer()
            .child(SharedString::from(label))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |_this, _ev: &MouseDownEvent, _window, cx| {
                    state.update(cx, |s, cx| on_click(s, cx));
                }),
            )
    }

    fn row(
        &self,
        label: &'static str,
        value: String,
        cx: &mut Context<Self>,
        minus: impl Fn(&mut StudioState, &mut Context<StudioState>) + 'static,
        plus: impl Fn(&mut StudioState, &mut Context<StudioState>) + 'static,
    ) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.))
            .py(px(2.))
            .child(
                div()
                    .w(px(60.))
                    .text_color(rgb(theme::TEXT_DIM))
                    .child(SharedString::from(label)),
            )
            .child(self.nudge_button("−", cx, minus))
            .child(
                div()
                    .min_w(px(60.))
                    .px(px(6.))
                    .text_color(rgb(theme::TEXT))
                    .child(SharedString::from(value)),
            )
            .child(self.nudge_button("+", cx, plus))
    }
}

impl Render for Inspector {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state_ref = self.state.read(cx);
        let selected = state_ref.selected().cloned();
        let settings = state_ref.settings.clone();

        let mut panel = div()
            .flex()
            .flex_col()
            .w(px(280.))
            .h_full()
            .bg(rgb(theme::PANEL))
            .border_l_1()
            .border_color(rgb(theme::BORDER))
            .child(
                div()
                    .h(px(28.))
                    .px(px(12.))
                    .py(px(6.))
                    .border_b_1()
                    .border_color(rgb(theme::BORDER))
                    .text_color(rgb(theme::TEXT_DIM))
                    .child(SharedString::from("INSPECTOR")),
            );

        let Some(ent) = selected else {
            return panel.child(
                div()
                    .px(px(12.))
                    .py(px(20.))
                    .text_color(rgb(theme::TEXT_DIM))
                    .child(SharedString::from("(no entity selected)")),
            );
        };

        let id = ent.id.clone();
        panel = panel.child(
            div()
                .px(px(12.))
                .py(px(10.))
                .text_color(rgb(theme::TEXT))
                .child(SharedString::from(format!("Name: {}", ent.name))),
        );

        if let Some(t) = ent.transform().cloned() {
            let id_x_m = id.clone();
            let id_x_p = id.clone();
            let id_y_m = id.clone();
            let id_y_p = id.clone();
            let id_sx_m = id.clone();
            let id_sx_p = id.clone();
            let id_sy_m = id.clone();
            let id_sy_p = id.clone();
            let id_r_m = id.clone();
            let id_r_p = id.clone();
            panel = panel.child(
                div()
                    .px(px(12.))
                    .py(px(6.))
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .text_color(rgb(theme::ACCENT))
                            .child(SharedString::from("Transform")),
                    )
                    .child(self.row("x", format!("{:.1}", t.x), cx,
                        move |s, cx| { s.update_transform(&id_x_m, |t| t.x -= 8.0, cx) },
                        move |s, cx| { s.update_transform(&id_x_p, |t| t.x += 8.0, cx) }))
                    .child(self.row("y", format!("{:.1}", t.y), cx,
                        move |s, cx| { s.update_transform(&id_y_m, |t| t.y -= 8.0, cx) },
                        move |s, cx| { s.update_transform(&id_y_p, |t| t.y += 8.0, cx) }))
                    .child(self.row("scaleX", format!("{:.2}", t.scale_x), cx,
                        move |s, cx| { s.update_transform(&id_sx_m, |t| t.scale_x = (t.scale_x - 0.1).max(0.1), cx) },
                        move |s, cx| { s.update_transform(&id_sx_p, |t| t.scale_x += 0.1, cx) }))
                    .child(self.row("scaleY", format!("{:.2}", t.scale_y), cx,
                        move |s, cx| { s.update_transform(&id_sy_m, |t| t.scale_y = (t.scale_y - 0.1).max(0.1), cx) },
                        move |s, cx| { s.update_transform(&id_sy_p, |t| t.scale_y += 0.1, cx) }))
                    .child(self.row("rotation", format!("{:.2}", t.rotation), cx,
                        move |s, cx| { s.update_transform(&id_r_m, |t| t.rotation -= 0.1, cx) },
                        move |s, cx| { s.update_transform(&id_r_p, |t| t.rotation += 0.1, cx) })),
            );
        }

        if let Some(sprite) = ent.sprite() {
            panel = panel.child(
                div()
                    .px(px(12.))
                    .py(px(6.))
                    .child(
                        div()
                            .text_color(rgb(theme::ACCENT))
                            .child(SharedString::from("Sprite")),
                    )
                    .child(
                        div()
                            .text_color(rgb(theme::TEXT_DIM))
                            .text_size(px(11.))
                            .child(SharedString::from(sprite.asset_path.clone())),
                    ),
            );
        }

        let scripts: Vec<_> = ent
            .components
            .iter()
            .filter_map(|c| match c {
                Component::Script(s) => Some(s.clone()),
                _ => None,
            })
            .collect();
        if !scripts.is_empty() {
            let mut block = div()
                .px(px(12.))
                .py(px(6.))
                .flex()
                .flex_col()
                .child(
                    div()
                        .text_color(rgb(theme::ACCENT))
                        .child(SharedString::from("Scripts")),
                );
            for sc in scripts {
                let path = sc.file_path.clone();
                let settings_clone = settings.clone();
                block = block.child(
                    div()
                        .flex()
                        .flex_row()
                        .gap(px(8.))
                        .py(px(2.))
                        .child(
                            div()
                                .flex_grow(1.0)
                                .text_color(rgb(theme::TEXT_DIM))
                                .text_size(px(11.))
                                .child(SharedString::from(path.clone())),
                        )
                        .child(
                            div()
                                .px(px(8.))
                                .text_color(rgb(theme::ACCENT))
                                .hover(|s| s.bg(rgb(theme::PANEL_ALT)))
                                .cursor_pointer()
                                .child(SharedString::from("Open"))
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |_this, _ev: &MouseDownEvent, _window, cx| {
                                        let p = std::path::PathBuf::from(&path);
                                        if let Err(e) = editor_svc::open_in_editor(
                                            &settings_clone,
                                            &p,
                                        ) {
                                            eprintln!("open_in_editor: {e}");
                                        }
                                        let _ = cx;
                                    }),
                                ),
                        ),
                );
            }
            panel = panel.child(block);
        }

        panel
    }
}
