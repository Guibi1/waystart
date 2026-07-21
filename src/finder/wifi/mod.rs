use std::rc::Rc;

use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};

use crate::finder::wifi::entry::WifiEntry;
use crate::finder::{Entry, Finder};

mod entry;
mod nm;

pub use nm::WifiManager;

pub struct WifiFinder {
    entries: Vec<Rc<WifiEntry>>,
}

impl Finder for WifiFinder {
    fn new() -> Self {
        Self { entries: vec![] }
    }

    fn default_entries(&self) -> Option<Vec<Rc<dyn Entry>>> {
        Some(
            self.entries
                .iter()
                .map(|entry| entry.clone() as Rc<dyn Entry>)
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
                        .score(
                            nucleo_matcher::Utf32String::from(entry.network.ssid.clone()).slice(..),
                            matcher,
                        )
                        .map(|_score| entry.clone() as Rc<dyn Entry>)
                })
                .collect(),
        )
    }
}
