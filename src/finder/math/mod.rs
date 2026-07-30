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
        if let Some(search_term) = search_term.strip_prefix('=') {
            return match evalexpr::eval(search_term) {
                Ok(result) => Some(vec![Rc::new(MathEntry {
                    text: format!("= {}", result).into(),
                    result,
                })]),
                Err(err) => Some(vec![Rc::new(MathEntry {
                    text: err.to_string().into(),
                    result: evalexpr::Value::String(err.to_string()),
                })]),
            };
        }

        let result = evalexpr::eval(search_term).ok()?;
        if let evalexpr::Value::Empty = result {
            None
        } else {
            Some(vec![Rc::new(MathEntry {
                text: format!("= {}", result).into(),
                result,
            })])
        }
    }
}
