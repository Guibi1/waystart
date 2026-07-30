use gpui::{App, AppContext, Entity, Global, IntoElement, RenderOnce, Window};

mod home;
mod search;

pub use home::HomePage;
pub use search::SearchPage;

#[derive(Clone, IntoElement)]
pub enum Page {
    Home(Entity<HomePage>),
    Search(Entity<SearchPage>),
}

impl Global for Page {}

impl Page {
    pub fn on_search(&mut self, search_term: &str, cx: &mut App) {
        if search_term.is_empty() && matches!(self, Page::Search(_)) {
            // Switch to home page when search is cleared from search page
            *self = Page::Home(cx.new(HomePage::new));
        } else if !search_term.is_empty() && matches!(self, Page::Home(_)) {
            // Switch to search page when searching from home page
            *self = Page::Search(cx.new(|cx| SearchPage::new(search_term, cx)));
        } else {
            match self {
                Page::Home(_) => {}
                Page::Search(page) => page.update(cx, |page, cx| page.on_search(search_term, cx)),
            }
        }
    }

    pub fn select_prev(&self, cx: &mut App) {
        match self {
            Page::Home(page) => page.update(cx, |page, cx| page.select_prev(cx)),
            Page::Search(page) => page.update(cx, |page, cx| page.select_prev(cx)),
        }
    }

    pub fn select_next(&self, cx: &mut App) {
        match self {
            Page::Home(page) => page.update(cx, |page, cx| page.select_next(cx)),
            Page::Search(page) => page.update(cx, |page, cx| page.select_next(cx)),
        }
    }

    pub fn execute_entry(&self, window: &mut Window, cx: &mut App) {
        match self {
            Page::Home(page) => page.update(cx, |page, cx| page.execute_entry(&(), window, cx)),
            Page::Search(page) => page.update(cx, |page, cx| page.execute_entry(&(), window, cx)),
        }
    }

    pub fn toggle_favorite(&self, cx: &mut App) {
        match self {
            Page::Home(page) => page.update(cx, |page, cx| page.toggle_favorite(cx)),
            Page::Search(page) => page.update(cx, |page, cx| page.toggle_favorite(cx)),
        }
    }
}

impl RenderOnce for Page {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        match self {
            Page::Home(page) => page.clone().into_any_element(),
            Page::Search(page) => page.clone().into_any_element(),
        }
    }
}
