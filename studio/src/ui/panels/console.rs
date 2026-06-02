//! Bottom-right tab — scrollback log.

use gpui::{
    div, prelude::*, px, rgb, Context, Entity, IntoElement, ParentElement, Render, SharedString,
    Styled, Window,
};

use crate::state::StudioState;
use crate::ui::theme;

pub struct Console {
    pub state: Entity<StudioState>,
}

impl Console {
    pub fn new(state: Entity<StudioState>, cx: &mut Context<Self>) -> Self {
        cx.observe(&state, |_, _, cx| cx.notify()).detach();
        Self { state }
    }
}

impl Render for Console {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let lines = self.state.read(cx).console.clone();
        let mut body = div().flex().flex_col().w_full();
        for line in lines {
            body = body.child(
                div()
                    .px(px(12.))
                    .py(px(1.))
                    .text_size(px(11.))
                    .text_color(rgb(theme::TEXT_DIM))
                    .child(SharedString::from(line)),
            );
        }
        div()
            .flex()
            .flex_col()
            .flex_grow()
            .h_full()
            .child(
                div()
                    .h(px(24.))
                    .px(px(12.))
                    .py(px(4.))
                    .border_b_1()
                    .border_color(rgb(theme::BORDER))
                    .text_color(rgb(theme::TEXT_DIM))
                    .child(SharedString::from("CONSOLE")),
            )
            .child(div().id("console-scroll").overflow_y_scroll().child(body))
    }
}
