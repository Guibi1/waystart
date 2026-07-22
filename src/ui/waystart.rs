use gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    KeyBinding, ParentElement, Render, Styled, Window, div,
};

use crate::config::Config;
use crate::quick_access::Quicks;
use crate::ui::actions::{Close, ExecuteEntry, SelectNext, SelectPrev, ToggleFavorite};
use crate::ui::elements::{Icon, Separator, Shortcut, TextInput};
use crate::ui::pages::{HomePage, Page};

const CONTEXT: &str = "Waystart";

pub(super) fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("up", SelectPrev, Some(CONTEXT)),
        KeyBinding::new("down", SelectNext, Some(CONTEXT)),
        KeyBinding::new("shift-tab", SelectPrev, Some(CONTEXT)),
        KeyBinding::new("tab", SelectNext, Some(CONTEXT)),
        KeyBinding::new("enter", ExecuteEntry, Some(CONTEXT)),
        KeyBinding::new("secondary-d", ToggleFavorite, Some(CONTEXT)),
        KeyBinding::new("escape", Close, Some(CONTEXT)),
    ]);
}

pub struct Waystart {
    page: Page,
    focus_handle: FocusHandle,
    search_bar: Entity<TextInput>,
}

impl Waystart {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let search_bar = cx.new(|_| {
            TextInput::new(focus_handle.clone()).placeholder("Search for apps and commands...")
        });

        cx.observe(&search_bar, |this, _, cx| this.filter_results(cx))
            .detach();

        Self {
            page: Page::Home(cx.new(HomePage::new)),
            focus_handle,
            search_bar,
        }
    }

    pub fn reset_search(&mut self, cx: &mut Context<Self>) {
        self.search_bar
            .update(cx, |search_bar, _| search_bar.reset());
        self.filter_results(cx);
    }

    fn filter_results(&mut self, cx: &mut Context<Self>) {
        let search_term = self.search_bar.read(cx).content().trim().to_string();
        self.page.on_search(&search_term, cx);
    }

    fn on_close(_: &Close, window: &mut Window, _cx: &mut App) {
        window.remove_window();
    }

    fn select_prev<A>(&mut self, _: &A, _window: &mut Window, cx: &mut Context<Self>) {
        self.page.select_prev(cx);
    }

    fn select_next<A>(&mut self, _: &A, _window: &mut Window, cx: &mut Context<Self>) {
        self.page.select_next(cx);
    }

    fn execute_entry<A>(&mut self, _: &A, window: &mut Window, cx: &mut Context<Self>) {
        self.page.execute_entry(window, cx);
    }

    fn toggle_favorite<A>(&mut self, _: &A, _window: &mut Window, cx: &mut Context<Self>) {
        self.page.toggle_favorite(cx);
    }
}

impl Render for Waystart {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let config = cx.global::<Config>();
        let quicks = cx.global::<Quicks>();

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(config.theme.background)
            .text_color(config.theme.foreground)
            .font_family(config.theme.font_family.clone())
            .border_color(config.theme.accent)
            .border_1()
            .rounded_lg()
            .overflow_hidden()
            .track_focus(&self.focus_handle(cx))
            .key_context(CONTEXT)
            .on_action::<Close>(Self::on_close)
            .on_action::<SelectPrev>(cx.listener(Self::select_prev))
            .on_action::<SelectNext>(cx.listener(Self::select_next))
            .on_action::<ExecuteEntry>(cx.listener(Self::execute_entry))
            .on_action::<ToggleFavorite>(cx.listener(Self::toggle_favorite))
            .child(
                div()
                    .h_16()
                    .flex()
                    .pl_6()
                    .items_center()
                    .child(Icon::Search.build(config.theme.foreground))
                    .child(self.search_bar.clone()),
            )
            .child(Separator::new())
            .child(self.page.clone())
            .child(Separator::new())
            .child(
                div()
                    .h_12()
                    .flex()
                    .px_4()
                    .items_center()
                    .gap_2()
                    .child(div().flex().gap_2().children(quicks.iter_any_element()))
                    .child(
                        div()
                            .ml_auto()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child("Open")
                            .child(Shortcut::new("↵")),
                    ),
            )
    }
}

impl Focusable for Waystart {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
