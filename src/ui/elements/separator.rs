use gpui::{App, IntoElement, RenderOnce, Styled, Window, div, px};

use crate::config::Config;

#[derive(IntoElement)]
pub struct Separator {}

impl Separator {
    pub fn new() -> Self {
        Separator {}
    }
}

impl RenderOnce for Separator {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let config = cx.global::<Config>();

        div().h(px(1.)).mx_6().bg(config.theme.muted)
    }
}
