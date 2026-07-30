use gpui::{App, Resource, SharedString, Window};

use crate::finder::Entry;

pub struct MathEntry {
    pub result: evalexpr::Value,
    pub text: SharedString,
}

impl Entry for MathEntry {
    fn id(&self) -> SharedString {
        self.text.clone()
    }

    fn score(&self) -> u32 {
        u32::MAX
    }

    fn text(&self) -> SharedString {
        self.text.clone()
    }

    fn description(&self) -> Option<SharedString> {
        None
    }

    fn icon(&self) -> Option<Resource> {
        None
    }

    fn can_favorite(&self) -> bool {
        false
    }

    fn execute(&self, window: &mut Window, cx: &mut App) {
        cx.write_to_clipboard(self.result.to_string().into());
        window.remove_window();
    }
}
