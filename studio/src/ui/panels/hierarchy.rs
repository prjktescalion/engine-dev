//! Left panel — scrollable list of entities in the current scene.

use gpui::{
    div, prelude::*, px, rgb, Context, Entity, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, Render, SharedString, Styled, Window,
};

use crate::model::{Component, Entity as SceneEntity, TransformComponent};
use crate::state::StudioState;
use crate::ui::theme;

pub struct Hierarchy {
    pub state: Entity<StudioState>,
}

impl Hierarchy {
    pub fn new(state: Entity<StudioState>, cx: &mut Context<Self>) -> Self {
        cx.observe(&state, |_, _, cx| cx.notify()).detach();
        Self { state }
    }

    fn entity_row(
        &self,
        ent: &SceneEntity,
        selected: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let id = ent.id.clone();
        let id_for_select = id.clone();
        let id_for_delete = id.clone();
        let state = self.state.clone();
        let state_for_delete = self.state.clone();
        let name = ent.name.clone();

        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.))
            .px(px(10.))
            .py(px(4.))
            .text_color(rgb(if selected { theme::ACCENT } else { theme::TEXT }))
            .bg(rgb(if selected {
                theme::ACCENT_DIM
            } else {
                theme::PANEL
            }))
            .hover(|s| s.bg(rgb(theme::PANEL_ALT)))
            .cursor_pointer()
            .child(div().flex_grow(1.0).child(SharedString::from(name)))
            .child(
                div()
                    .px(px(6.))
                    .text_color(rgb(theme::DANGER))
                    .hover(|s| s.bg(rgb(theme::BORDER)))
                    .child(SharedString::from("×"))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |_this, _ev: &MouseDownEvent, _window, cx| {
                            let id = id_for_delete.clone();
                            state_for_delete.update(cx, |s, cx| s.remove_entity(&id, cx));
                            cx.stop_propagation();
                        }),
                    ),
            )
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |_this, _ev: &MouseDownEvent, _window, cx| {
                    let id = id_for_select.clone();
                    state.update(cx, |s, cx| s.select_entity(Some(id), cx));
                }),
            )
    }
}

impl Render for Hierarchy {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state_ref = self.state.read(cx);
        let entities = state_ref.scene.entities.clone();
        let selected = state_ref.selected_entity.clone();
        let state = self.state.clone();

        let mut list = div().flex().flex_col().w_full();
        for ent in &entities {
            let is_sel = selected.as_ref() == Some(&ent.id);
            list = list.child(self.entity_row(ent, is_sel, cx));
        }
        if entities.is_empty() {
            list = list.child(
                div()
                    .px(px(12.))
                    .py(px(20.))
                    .text_color(rgb(theme::TEXT_DIM))
                    .child(SharedString::from("(no entities — drop an asset on the canvas)")),
            );
        }

        div()
            .flex()
            .flex_col()
            .w(px(240.))
            .h_full()
            .bg(rgb(theme::PANEL))
            .border_r_1()
            .border_color(rgb(theme::BORDER))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .h(px(28.))
                    .px(px(12.))
                    .border_b_1()
                    .border_color(rgb(theme::BORDER))
                    .text_color(rgb(theme::TEXT_DIM))
                    .child(SharedString::from("HIERARCHY"))
                    .child(div().flex_grow(1.0))
                    .child(
                        div()
                            .px(px(8.))
                            .text_color(rgb(theme::ACCENT))
                            .hover(|s| s.bg(rgb(theme::PANEL_ALT)))
                            .cursor_pointer()
                            .child(SharedString::from("+ Entity"))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |_this, _ev: &MouseDownEvent, _window, cx| {
                                    state.update(cx, |s, cx| {
                                        let idx = s.scene.entities.len() + 1;
                                        let mut ent = SceneEntity::new(format!("Entity {idx}"));
                                        ent.components.push(Component::Transform(
                                            TransformComponent {
                                                x: 200.0,
                                                y: 200.0,
                                                ..Default::default()
                                            },
                                        ));
                                        s.add_entity(ent, cx);
                                    });
                                }),
                            ),
                    ),
            )
            .child(list)
    }
}
