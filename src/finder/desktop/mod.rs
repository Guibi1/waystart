use std::rc::Rc;

use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};

use crate::finder::desktop::entry::DesktopEntry;
use crate::finder::desktop::frequency::Frequencies;
use crate::finder::desktop::terminal::create_terminal_command;
use crate::finder::{Entry, Finder};

mod entry;
pub mod frequency;
mod terminal;

pub struct DesktopFinder {
    entries: Vec<Rc<DesktopEntry>>,
    frequencies: Frequencies,
}

impl Finder for DesktopFinder {
    fn new() -> Self {
        Self {
            entries: DesktopEntry::load(),
            frequencies: Frequencies::load(),
        }
    }

    fn default_entries(&self) -> Option<Vec<Rc<dyn Entry>>> {
        Some(
            self.entries
                .iter()
                .map(|entry| {
                    entry.set_score(self.frequencies.score(&entry.id()));
                    entry.clone() as Rc<dyn Entry>
                })
                .collect(),
        )
    }

    fn filtered_entries(
        &self,
        matcher: &mut nucleo_matcher::Matcher,
        search_term: &str,
    ) -> Option<Vec<Rc<dyn Entry>>> {
        let search_pattern = Pattern::new(
            search_term,
            CaseMatching::Ignore,
            Normalization::Smart,
            AtomKind::Fuzzy,
        );

        Some(
            self.entries
                .iter()
                .filter_map(|entry| {
                    search_pattern
                        .score(entry.haystack.slice(..), matcher)
                        .map(|score| {
                            entry.set_score(score);
                            entry.clone() as Rc<dyn Entry>
                        })
                })
                .collect(),
        )
    }
}
