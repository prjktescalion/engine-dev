//! Top menu bar with File/Project actions.

use gpui::{
    div, prelude::*, px, rgb, ClickEvent, Context, Entity, IntoElement, ParentElement, Render,
    SharedString, Styled, Window,
};

use super::theme;
use crate::state::StudioState;
use crate::ui::root::StudioActions;

pub struct MenuBar {
    pub state: Entity<StudioState>,
    pub actions: Entity<StudioActions>,
}

impl MenuBar {
    pub fn new(state: Entity<StudioState>, actions: Entity<StudioActions>) -> Self {
        Self { state, actions }
    }

    fn button(
        &self,
        label: &'static str,
        cx: &mut Context<Self>,
        on_click: impl Fn(&mut StudioActions, &mut Window, &mut Context<StudioActions>) + 'static,
    ) -> impl IntoElement {
        let actions = self.actions.clone();
        div()
            .px(px(10.))
            .py(px(4.))
            .rounded(px(4.))
            .text_color(rgb(theme::TEXT))
            .hover(|s| s.bg(rgb(theme::PANEL_ALT)))
            .cursor_pointer()
            .child(SharedString::from(label))
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(move |_this, _ev: &gpui::MouseDownEvent, window, cx| {
                    let on_click = &on_click;
                    actions.update(cx, |a, cx| on_click(a, window, cx));
                }),
            )
    }
}

impl Render for MenuBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let scene_name = state.scene.name.clone();
        let dirty = state.scene_path.is_none() && !state.scene.entities.is_empty();
        let title = if dirty {
            format!("{} *", scene_name)
        } else {
            scene_name
        };

        div()
            .flex()
            .flex_row()
            .items_center()
            .h(px(36.))
            .px(px(12.))
            .gap(px(2.))
            .bg(rgb(theme::PANEL))
            .border_b_1()
            .border_color(rgb(theme::BORDER))
            .child(
                div()
                    .px(px(8.))
                    .text_color(rgb(theme::ACCENT))
                    .child(SharedString::from("NeuDel-II")),
            )
            .child(
                div()
                    .w(px(1.))
                    .h(px(20.))
                    .mx(px(6.))
                    .bg(rgb(theme::BORDER)),
            )
            .child(self.button("New Scene", cx, |a, window, cx| a.new_scene(window, cx)))
            .child(self.button("Open Project...", cx, |a, window, cx| {
                a.open_project(window, cx)
            }))
            .child(self.button("Load Scene...", cx, |a, window, cx| {
                a.load_scene(window, cx)
            }))
            .child(self.button("Save Scene", cx, |a, window, cx| {
                a.save_scene(window, cx)
            }))
            .child(
                div()
                    .w(px(1.))
                    .h(px(20.))
                    .mx(px(6.))
                    .bg(rgb(theme::BORDER)),
            )
            .child(self.button("Settings", cx, |a, window, cx| {
                a.toggle_settings(window, cx)
            }))
            .child(div().flex_grow())
            .child(
                div()
                    .text_color(rgb(theme::TEXT_DIM))
                    .child(SharedString::from(title)),
            )
    }
}
