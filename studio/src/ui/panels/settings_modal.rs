//! Modal overlay for editor settings.

use gpui::{
    div, prelude::*, px, rgb, Context, Entity, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, Render, SharedString, Styled, Window,
};

use crate::model::{EditorChoice, Theme};
use crate::services::settings as settings_svc;
use crate::state::StudioState;
use crate::ui::root::StudioActions;
use crate::ui::theme;

pub struct SettingsModal {
    pub state: Entity<StudioState>,
    pub actions: Entity<StudioActions>,
}

impl SettingsModal {
    pub fn new(
        state: Entity<StudioState>,
        actions: Entity<StudioActions>,
        cx: &mut Context<Self>,
    ) -> Self {
        cx.observe(&state, |_, _, cx| cx.notify()).detach();
        cx.observe(&actions, |_, _, cx| cx.notify()).detach();
        Self { state, actions }
    }

    fn radio(
        &self,
        label: &'static str,
        active: bool,
        cx: &mut Context<Self>,
        mut on_click: impl FnMut(&mut StudioState, &mut Context<StudioState>) + 'static,
    ) -> impl IntoElement {
        let state = self.state.clone();
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(8.))
            .py(px(4.))
            .px(px(10.))
            .rounded(px(4.))
            .bg(rgb(if active { theme::ACCENT_DIM } else { theme::PANEL_ALT }))
            .text_color(rgb(if active { theme::ACCENT } else { theme::TEXT }))
            .hover(|s| s.bg(rgb(theme::BORDER)))
            .cursor_pointer()
            .child(SharedString::from(if active { "●" } else { "○" }))
            .child(SharedString::from(label))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |_this, _ev: &MouseDownEvent, _window, cx| {
                    state.update(cx, |s, cx| {
                        on_click(s, cx);
                        let _ = settings_svc::save(&s.settings);
                    });
                }),
            )
    }
}

impl Render for SettingsModal {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let visible = self.actions.read(cx).show_settings;
        if !visible {
            return div();
        }
        let settings = self.state.read(cx).settings.clone();
        let actions = self.actions.clone();

        div()
            .absolute()
            .top_0()
            .left_0()
            .w_full()
            .h_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(gpui::rgba(0x00000080))
            .child(
                div()
                    .w(px(420.))
                    .bg(rgb(theme::PANEL))
                    .border_1()
                    .border_color(rgb(theme::BORDER))
                    .rounded(px(8.))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .h(px(36.))
                            .px(px(14.))
                            .border_b_1()
                            .border_color(rgb(theme::BORDER))
                            .child(
                                div()
                                    .flex_grow()
                                    .text_color(rgb(theme::TEXT))
                                    .child(SharedString::from("Editor Settings")),
                            )
                            .child(
                                div()
                                    .px(px(8.))
                                    .text_color(rgb(theme::TEXT_DIM))
                                    .hover(|s| s.bg(rgb(theme::PANEL_ALT)))
                                    .cursor_pointer()
                                    .child(SharedString::from("×"))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |_this, _ev: &MouseDownEvent, window, cx| {
                                            actions.update(cx, |a, cx| {
                                                a.toggle_settings(window, cx)
                                            });
                                        }),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(8.))
                            .p(px(14.))
                            .child(
                                div()
                                    .text_color(rgb(theme::TEXT_DIM))
                                    .child(SharedString::from("External editor")),
                            )
                            .child(self.radio(
                                "VS Code (code)",
                                settings.editor == EditorChoice::Vscode,
                                cx,
                                |s, _cx| s.settings.editor = EditorChoice::Vscode,
                            ))
                            .child(self.radio(
                                "JetBrains IDEA (idea)",
                                settings.editor == EditorChoice::Jetbrains,
                                cx,
                                |s, _cx| s.settings.editor = EditorChoice::Jetbrains,
                            ))
                            .child(self.radio(
                                "Custom binary path",
                                settings.editor == EditorChoice::Custom,
                                cx,
                                |s, _cx| s.settings.editor = EditorChoice::Custom,
                            ))
                            .child(
                                div()
                                    .mt(px(6.))
                                    .text_color(rgb(theme::TEXT_DIM))
                                    .child(SharedString::from("Theme")),
                            )
                            .child(self.radio(
                                "Dark",
                                settings.theme == Theme::Dark,
                                cx,
                                |s, _cx| s.settings.theme = Theme::Dark,
                            ))
                            .child(self.radio(
                                "Light (unimplemented)",
                                settings.theme == Theme::Light,
                                cx,
                                |s, _cx| s.settings.theme = Theme::Light,
                            )),
                    ),
            )
    }
}
