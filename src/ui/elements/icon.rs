use gpui::{App, Hsla, IntoElement, RenderOnce, SharedString, Styled, Window, svg};

#[derive(Clone, Copy)]
pub enum Icon {
    Lock,
    Power,
    Restart,
    Search,
    Sleep,
    Wifi,
    WifiOff,
}

impl Icon {
    pub fn build(&self, color: impl Into<Hsla>) -> IconElement {
        let path = match self {
            Icon::Lock => "lock.svg",
            Icon::Power => "power.svg",
            Icon::Restart => "restart.svg",
            Icon::Search => "search.svg",
            Icon::Sleep => "sleep.svg",
            Icon::Wifi => "wifi.svg",
            Icon::WifiOff => "wifi-off.svg",
        };

        IconElement::new(path.into(), color)
    }
}

#[derive(IntoElement)]
pub struct IconElement {
    path: SharedString,
    color: Hsla,
}

impl IconElement {
    pub fn new(path: SharedString, color: impl Into<Hsla>) -> IconElement {
        IconElement {
            path,
            color: color.into(),
        }
    }
}

impl RenderOnce for IconElement {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        svg()
            .flex_none()
            .size_4()
            .text_color(self.color)
            .path(self.path)
    }
}
