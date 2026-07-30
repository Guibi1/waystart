use gpui::{App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div, px};

use crate::config::Config;

#[derive(IntoElement)]
pub struct Shortcut {
    text: SharedString,
}

impl Shortcut {
    pub fn new(text: impl Into<SharedString>) -> Shortcut {
        Shortcut { text: text.into() }
    }
}

impl RenderOnce for Shortcut {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let config = cx.global::<Config>();

        div()
            .text_sm()
            .px_1p5()
            .py_1()
            .line_height(px(12.))
            .rounded_sm()
            .bg(config.theme.accent)
            .text_color(config.theme.accent_foreground)
            .child(self.text)
    }
}
