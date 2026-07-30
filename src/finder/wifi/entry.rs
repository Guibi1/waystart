use gpui::{App, Resource, SharedString, Window};

use crate::finder::{Entry, wifi::WifiManager};

pub struct WifiEntry {
    pub network: nmrs::Network,
}

impl Entry for WifiEntry {
    fn id(&self) -> SharedString {
        self.network.ssid.clone().into()
    }

    fn score(&self) -> u32 {
        self.network.strength.unwrap_or_default() as u32
    }

    fn text(&self) -> SharedString {
        self.network.ssid.clone().into()
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
        let nm = cx.global::<WifiManager>();
        if !self.network.secured {
            nm.connect(&self.network, nmrs::WifiSecurity::Open);
        } else if self.network.is_psk {
            nm.connect(
                &self.network,
                nmrs::WifiSecurity::WpaPsk {
                    psk: "".to_string(),
                },
            );
        }
        window.remove_window();
    }
}
