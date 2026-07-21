use std::rc::Rc;

use gpui::{
    Context, InteractiveElement, IntoElement, ParentElement, Render, ScrollStrategy,
    StatefulInteractiveElement, Styled, UniformListScrollHandle, Window, div, uniform_list,
};

use crate::config::Config;
use crate::finder::favorites::Favorites;
use crate::finder::{Entry, Finders};
use crate::ui::elements::EntryButton;

pub struct SearchPage {
    selected: usize,
    entries: Vec<Rc<dyn Entry>>,
    list_scroll_handle: UniformListScrollHandle,
}

impl SearchPage {
    pub fn new(search_term: &str, cx: &mut Context<Self>) -> Self {
        SearchPage {
            selected: 0,
            entries: cx.global::<Finders>().filtered_entries(search_term),
            list_scroll_handle: UniformListScrollHandle::new(),
        }
    }

    pub(super) fn on_search(&mut self, search_term: &str, cx: &mut Context<Self>) {
        self.entries = cx.global::<Finders>().filtered_entries(search_term);
        self.selected = 0;
        self.list_scroll_handle
            .scroll_to_item(0, ScrollStrategy::Top);
    }

    pub(super) fn select_prev(&mut self, cx: &mut Context<Self>) {
        if self.selected == 0 {
            self.selected = self.entries.len().saturating_sub(1);
        } else {
            self.selected -= 1;
        };
        self.list_scroll_handle
            .scroll_to_item(self.selected, ScrollStrategy::Top);
        cx.notify();
    }

    pub(super) fn select_next(&mut self, cx: &mut Context<Self>) {
        if self.selected + 1 == self.entries.len() {
            self.selected = 0;
        } else {
            self.selected += 1;
        };
        self.list_scroll_handle
            .scroll_to_item(self.selected, ScrollStrategy::Top);
        cx.notify();
    }

    pub(super) fn execute_entry<A>(&mut self, _: &A, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(entry) = self.entries.get(self.selected) {
            entry.execute(window, cx);
        };
    }

    pub(super) fn toggle_favorite(&self, cx: &mut Context<Self>) {
        if let Some(entry) = self.entries.get(self.selected)
            && entry.can_favorite()
        {
            cx.global_mut::<Favorites>().insert(entry.id().clone());
        }
    }
}

impl Render for SearchPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let config = cx.global::<Config>();

        div()
            .flex_grow_1()
            .flex()
            .flex_col()
            .gap_1()
            .px_2()
            .child(
                div()
                    .px_5()
                    .py_1()
                    .text_color(config.theme.muted_foreground)
                    .child("Results"),
            )
            .child(
                uniform_list(
                    "entry_list",
                    self.entries.len(),
                    cx.processor(move |this, range: std::ops::Range<usize>, _, cx| {
                        this.entries
                            .iter()
                            .enumerate()
                            .skip(range.start)
                            .take(range.end - range.start)
                            .map(|(i, entry)| {
                                div()
                                    .id(entry.id().clone())
                                    .child(EntryButton::new(entry.clone(), this.selected == i))
                                    .on_click(cx.listener(Self::execute_entry))
                                    .on_mouse_move(cx.listener(move |this, _, _, cx| {
                                        if this.selected != i {
                                            this.selected = i;
                                            cx.notify();
                                        }
                                    }))
                            })
                            .collect()
                    }),
                )
                .track_scroll(&self.list_scroll_handle)
                .flex_grow_1()
                .pb_2(),
            )
    }
}
