use std::rc::Rc;

use gpui::{
    App, ImageSource, InteractiveElement, IntoElement, ObjectFit, ParentElement, RenderOnce,
    Styled, StyledImage, TextOverflow, Window, div, img, prelude::FluentBuilder,
};

use crate::config::Config;
use crate::finder::Entry;

#[derive(IntoElement)]
pub struct EntryButton {
    entry: Rc<dyn Entry>,
    selected: bool,
    favorite: bool,
}

impl EntryButton {
    pub fn new(entry: Rc<dyn Entry>, selected: bool) -> EntryButton {
        EntryButton {
            entry,
            selected,
            favorite: false,
        }
    }

    pub fn favorite(mut self, favorite: bool) -> Self {
        self.favorite = favorite;
        self
    }
}

impl RenderOnce for EntryButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let config = cx.global::<Config>();

        div()
            .flex()
            .items_center()
            .gap_4()
            .rounded_lg()
            .text_overflow(TextOverflow::Truncate("...".into()))
            .when_else(
                self.favorite,
                |this| {
                    this.w_32()
                        .h_24()
                        .flex_col()
                        .justify_center()
                        .text_center()
                        .text_overflow(TextOverflow::Truncate("...".into()))
                        .hover(|this| this.bg(config.theme.muted))
                },
                |this| this.w_full().h_12().px_4(),
            )
            .when_some(self.entry.icon(), |this, icon| {
                this.child(
                    img(ImageSource::Resource(icon))
                        .flex_shrink_0()
                        .size_7()
                        .object_fit(ObjectFit::Contain),
                )
            })
            .child(div().flex_grow_1().child(self.entry.text()))
            .when(self.selected, |this| {
                this.bg(config.theme.muted).when_some(
                    self.entry.description(),
                    |this, description| {
                        this.child(
                            div()
                                .flex_shrink_1()
                                .text_sm()
                                .text_color(config.theme.muted_foreground)
                                .text_overflow(TextOverflow::Truncate("...".into()))
                                .child(description),
                        )
                    },
                )
            })
    }
}
