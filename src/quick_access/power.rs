use std::process::Command;

use gpui::{
    Anchor, App, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled, Window, div,
};

use crate::config::Config;
use crate::quick_access::QuickAccess;
use crate::ui::elements::{Dropdown, DropdownContent, Icon};

#[derive(Clone, IntoElement)]
pub struct PowerQuickAccess {}

impl PowerQuickAccess {
    pub fn new() -> Self {
        PowerQuickAccess {}
    }
}

impl QuickAccess for PowerQuickAccess {
    fn any_element(&self) -> gpui::AnyElement {
        self.clone().into_any_element()
    }
}

impl RenderOnce for PowerQuickAccess {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let config = cx.global::<Config>();

        Dropdown::new("quick-power")
            .anchor(Anchor::TopRight)
            .trigger(
                div()
                    .size_8()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_lg()
                    .hover(|style| style.bg(config.theme.muted))
                    .child(Icon::Power.build(config.theme.foreground)),
            )
            .content(|cx| {
                DropdownContent::new(cx)
                    .w_40()
                    .item("quick-power-lock", "Lock", Some(Icon::Lock), |window, _| {
                        match Command::new("loginctl").arg("lock-session").spawn() {
                            Ok(_) => window.remove_window(),
                            Err(e) => {
                                eprintln!("Failed to lock session: {}", e);
                            }
                        }
                    })
                    .item(
                        "quick-power-sleep",
                        "Sleep",
                        Some(Icon::Sleep),
                        |window, _| match Command::new("systemctl").arg("suspend").spawn() {
                            Ok(_) => window.remove_window(),
                            Err(e) => {
                                eprintln!("Failed to sleep: {}", e);
                            }
                        },
                    )
                    .item(
                        "quick-power-shut-down",
                        "Shut down",
                        Some(Icon::Power),
                        |window, _| match Command::new("systemctl").arg("poweroff").spawn() {
                            Ok(_) => window.remove_window(),
                            Err(e) => {
                                eprintln!("Failed to shut down: {}", e);
                            }
                        },
                    )
                    .item(
                        "quick-power-restart",
                        "Restart",
                        Some(Icon::Restart),
                        |window, _| match Command::new("systemctl").arg("reboot").spawn() {
                            Ok(_) => window.remove_window(),
                            Err(e) => {
                                eprintln!("Failed to restart: {}", e);
                            }
                        },
                    )
            })
    }
}
