use std::cell::RefCell;
use std::cmp::Reverse;
use std::rc::Rc;

use gpui::{App, Global, Resource, SharedString, Window};

use crate::finder::desktop::DesktopFinder;
use crate::finder::math::MathFinder;
use crate::finder::wifi::WifiFinder;

pub mod desktop;
pub mod favorites;
pub mod math;
pub mod wifi;

pub struct Finders {
    finders: Vec<Box<dyn Finder>>,
    matcher: RefCell<nucleo_matcher::Matcher>,
}

impl Global for Finders {}
impl Finders {
    pub fn new() -> Self {
        Self {
            finders: vec![
                Box::new(DesktopFinder::new()),
                Box::new(MathFinder::new()),
                Box::new(WifiFinder::new()),
            ],
            matcher: RefCell::new(nucleo_matcher::Matcher::default()),
        }
    }

    pub fn default_entries(&self) -> Vec<Rc<dyn Entry>> {
        let mut entries = self
            .finders
            .iter()
            .filter_map(|finder| finder.default_entries())
            .flatten()
            .collect::<Vec<_>>();
        entries.sort_by_key(|entry| Reverse(entry.score()));
        entries
    }

    pub fn filtered_entries(&self, search_term: &str) -> Vec<Rc<dyn Entry>> {
        let mut matcher = self.matcher.borrow_mut();
        let mut entries = self
            .finders
            .iter()
            .filter_map(|finder| finder.filtered_entries(&mut matcher, search_term))
            .flatten()
            .collect::<Vec<_>>();
        entries.sort_by_key(|entry| Reverse(entry.score()));
        entries
    }
}

pub trait Finder {
    fn new() -> Self
    where
        Self: Sized;

    /// Returns the entries when no search is performed.
    fn default_entries(&self) -> Option<Vec<Rc<dyn Entry>>>;

    /// Returns the entries that match the given pattern.
    fn filtered_entries(
        &self,
        matcher: &mut nucleo_matcher::Matcher,
        search_term: &str,
    ) -> Option<Vec<Rc<dyn Entry>>>;
}

pub trait Entry {
    /// Get a unique identifier for this entry.
    fn id(&self) -> SharedString;

    /// Get a unique identifier for this entry.
    fn score(&self) -> u32;

    /// Get the main text of this entry.
    fn text(&self) -> SharedString;

    /// Get the subtle description of this entry.
    fn description(&self) -> Option<SharedString>;

    /// Get the icon of this entry.
    fn icon(&self) -> Option<Resource>;

    /// If this entry can be favorited.
    fn can_favorite(&self) -> bool;

    /// Execute this entry per user's request.
    fn execute(&self, window: &mut Window, cx: &mut App);
}
