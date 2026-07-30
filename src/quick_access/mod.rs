use gpui::{AnyElement, Global};

use crate::quick_access::{power::PowerQuickAccess, wifi::WifiQuickAccess};

pub mod power;
pub mod wifi;

pub struct Quicks {
    quicks: Vec<Box<dyn QuickAccess>>,
}

impl Global for Quicks {}
impl Quicks {
    pub fn new() -> Self {
        Self {
            quicks: vec![
                Box::new(PowerQuickAccess::new()),
                Box::new(WifiQuickAccess::new()),
            ],
        }
    }

    pub fn iter_any_element(&self) -> impl Iterator<Item = AnyElement> {
        self.quicks.iter().map(|q| q.any_element())
    }
}

pub trait QuickAccess {
    fn any_element(&self) -> AnyElement;
}
