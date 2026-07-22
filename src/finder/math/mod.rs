use std::rc::Rc;

use crate::finder::{Entry, Finder, math::entry::MathEntry};

mod entry;

pub struct MathFinder {}

impl Finder for MathFinder {
    fn new() -> Self {
        Self {}
    }

    fn default_entries(&self) -> Option<Vec<Rc<dyn Entry>>> {
        None
    }

    fn filtered_entries(
        &self,
        _matcher: &mut nucleo_matcher::Matcher,
        search_term: &str,
    ) -> Option<Vec<Rc<dyn Entry>>> {
        // `=<expr>` forces the calculator and surfaces errors
        if let Some(search_term) = search_term.strip_prefix('=') {
            return Some(vec![match calc::eval(search_term) {
                Ok(result) => Rc::new(MathEntry::result(result)),
                Err(err) => Rc::new(MathEntry::error(err)),
            }]);
        }

        // only attempt inputs that look like math, so plain text searches
        // (which may contain unit words) don't produce calculator entries
        if !search_term.chars().any(|c| c.is_ascii_digit()) {
            return None;
        }

        let result = calc::eval(search_term).ok()?;
        Some(vec![Rc::new(MathEntry::result(result))])
    }
}
