use gpui::{
    Anchor, App, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled, Window, div,
};

use crate::config::Config;
use crate::finder::wifi::WifiManager;
use crate::quick_access::QuickAccess;
use crate::ui::elements::{Dropdown, DropdownContent, Icon};

#[derive(Clone, IntoElement)]
pub struct WifiQuickAccess {}

impl WifiQuickAccess {
    pub fn new() -> Self {
        WifiQuickAccess {}
    }
}

impl QuickAccess for WifiQuickAccess {
    fn any_element(&self) -> gpui::AnyElement {
        self.clone().into_any_element()
    }
}

impl RenderOnce for WifiQuickAccess {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let config = cx.global::<Config>();
        let Some(wifi) = cx.try_global::<WifiManager>().cloned() else {
            return div()
                .size_8()
                .flex()
                .items_center()
                .justify_center()
                .rounded_lg()
                .child(Icon::WifiOff.build(config.theme.muted_foreground))
                .into_any_element();
        };

        Dropdown::new("quick-wifi")
            .anchor(Anchor::TopRight)
            .trigger(
                div()
                    .size_8()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_lg()
                    .hover(|style| style.bg(config.theme.muted))
                    .child(Icon::Wifi.build(config.theme.foreground)),
            )
            .content(move |cx| {
                let wifi = wifi.clone();
                let enabled = wifi.enabled();
                wifi.list()
                    .into_iter()
                    .fold(DropdownContent::new(cx).w_40(), |drop, network| {
                        drop.item(
                            network.ssid.clone(),
                            network.ssid.clone(),
                            Some(Icon::Wifi),
                            move |_, cx| {
                                cx.global::<WifiManager>()
                                    .connect(&network, nmrs::WifiSecurity::Open)
                            },
                        )
                    })
                    .separate()
                    .item(
                        "quick-wifi-toggle",
                        if enabled {
                            "Disable Wi-Fi"
                        } else {
                            "Enable Wi-Fi"
                        },
                        Some(Icon::Restart),
                        move |window, _| match wifi.enable(!enabled) {
                            Ok(_) => window.remove_window(),
                            Err(e) => {
                                eprintln!("Failed to toggle wifi: {}", e);
                            }
                        },
                    )
            })
            .into_any_element()
    }
}
