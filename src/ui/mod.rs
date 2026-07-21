use std::borrow::Cow;

use gpui::{App, AssetSource};
use rust_embed::RustEmbed;

mod actions;
pub mod elements;
mod pages;
mod waystart;

pub use waystart::Waystart;

pub fn init(cx: &mut App) {
    waystart::init(cx);
    elements::init(cx);
}

#[derive(RustEmbed)]
#[folder = "assets"]
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        if path.is_empty() {
            return Ok(None);
        }

        Ok(Self::get(path).map(|f| f.data))
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<gpui::SharedString>> {
        Ok(Self::iter()
            .filter_map(|p| p.starts_with(path).then(|| p.into()))
            .collect())
    }
}
