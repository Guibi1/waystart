use std::path::PathBuf;
use std::sync::LazyLock;

use gpui::{Global, Rgba, SharedString, rgb};
use serde::Deserialize;

#[derive(Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub terminal: Option<String>,

    pub theme: ThemeConfig,
}

#[derive(Deserialize)]
#[serde(default)]
pub struct ThemeConfig {
    pub font_family: SharedString,

    pub background: Rgba,
    pub foreground: Rgba,
    pub border: Rgba,
    pub muted: Rgba,
    pub muted_foreground: Rgba,
    pub accent: Rgba,
    pub accent_foreground: Rgba,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            font_family: SharedString::from("Cascadia Code PL"),

            background: rgb(0x1e1e2e),
            foreground: rgb(0xcdd6f4),
            border: rgb(0x45475a),
            muted: rgb(0x313244),
            muted_foreground: rgb(0x6c7086),
            accent: rgb(0xcba6f7),
            accent_foreground: rgb(0x1e1e2e),
        }
    }
}

impl Global for Config {}

impl Config {
    pub fn load() -> Self {
        match std::fs::read_to_string(&*CONFIG_SAVE_PATH) {
            Ok(file) => toml::from_str(&file).expect("Failed to parse the config file"),
            Err(_) => Self::default(),
        }
    }
}

static CONFIG_SAVE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    dirs::data_dir()
        .expect("Failed to get config directory")
        .join("waystart.toml")
});
