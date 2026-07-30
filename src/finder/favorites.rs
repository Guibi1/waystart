use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::LazyLock;

use gpui::{Global, SharedString};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Favorites {
    favorites: HashSet<SharedString>,
}

impl Favorites {
    pub fn load() -> Self {
        match std::fs::read_to_string(&*FAVORITES_SAVE_PATH) {
            Ok(file) => toml::from_str(&file).expect("Failed to parse favorites"),
            Err(_) => Self {
                favorites: HashSet::new(),
            },
        }
    }

    pub async fn save(&self) {
        let content = toml::to_string(self).expect("Failed to serialize favorites");
        if let Err(err) = smol::fs::write(&*FAVORITES_SAVE_PATH, content).await {
            eprintln!(
                "Failed to save favorites at {}: {}",
                FAVORITES_SAVE_PATH.to_string_lossy(),
                err
            );
        }
    }
}

impl Deref for Favorites {
    type Target = HashSet<SharedString>;

    fn deref(&self) -> &Self::Target {
        &self.favorites
    }
}

impl DerefMut for Favorites {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.favorites
    }
}

impl Global for Favorites {}

static FAVORITES_SAVE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    dirs::data_dir()
        .expect("Failed to get data directory")
        .join("waystart.toml")
});
