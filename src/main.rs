use gpui::{
    App, AppContext, Bounds, Entity, Focusable, QuitMode, TitlebarOptions, WindowBounds,
    WindowDecorations, WindowHandle, WindowKind, WindowOptions, point, px, size,
};
use gpui_platform::application;

use crate::config::Config;
use crate::finder::Finders;
use crate::finder::desktop::frequency::DESKTOP_FREQUENCIES;
use crate::finder::favorites::Favorites;
use crate::finder::wifi::WifiManager;
use crate::ipc::client::{SocketClient, SocketMessage};
use crate::ipc::server::SocketServer;
use crate::quick_access::Quicks;
use crate::ui::Waystart;

mod cli;
mod config;
mod finder;
mod ipc;
mod quick_access;
mod ui;

fn main() {
    let message = match cli::Waystart::from_env_or_exit().subcommand {
        cli::WaystartCmd::Open(_) => SocketMessage::Open,
        cli::WaystartCmd::Close(_) => SocketMessage::Close,
        cli::WaystartCmd::Toggle(_) => SocketMessage::Toggle,
        cli::WaystartCmd::Standalone(options) => {
            if !options.force
                && let Ok(client) = SocketClient::try_connect()
            {
                client.send_message_socket(SocketMessage::Open);
                return;
            } else {
                create_app(false);
                return;
            }
        }
        cli::WaystartCmd::Daemon(options) => {
            if options.exit {
                SocketMessage::Quit
            } else {
                create_app(true);
                return;
            }
        }
    };

    let client = SocketClient::connect();
    client.send_message_socket(message);
}

fn create_app(daemon: bool) {
    application()
        .with_assets(ui::Assets)
        .with_quit_mode(if daemon {
            QuitMode::Explicit
        } else {
            QuitMode::LastWindowClosed
        })
        .run(move |cx| {
            ui::init(cx);
            cx.set_global(Config::load());
            cx.set_global(Favorites::load());
            cx.set_global(Finders::new());
            cx.set_global(Quicks::new());

            cx.spawn(async move |cx| {
                if let Ok(wifi) = WifiManager::new(cx).await {
                    cx.update(move |cx| cx.set_global(wifi));
                }
            })
            .detach();

            cx.on_app_quit(|_| DESKTOP_FREQUENCIES.save()).detach();
            cx.on_app_quit(|cx| {
                let favorites = cx.remove_global::<Favorites>();
                async move { favorites.save().await }
            })
            .detach();

            let waystart = cx.new(Waystart::new);
            cx.on_window_closed({
                let waystart = waystart.clone();
                move |cx, _| waystart.update(cx, |waystart, cx| waystart.reset_search(cx))
            })
            .detach();

            if daemon {
                let server = SocketServer::new(cx.to_async(), waystart);
                server.listen();
            } else {
                open_window(cx, waystart);
            }
        });
}

pub fn open_window(cx: &mut App, waystart: Entity<Waystart>) -> WindowHandle<Waystart> {
    let bounds = Bounds::centered(None, size(px(800.), px(500.)), cx);
    cx.open_window(
        WindowOptions {
            #[cfg(target_os = "linux")]
            kind: WindowKind::LayerShell(gpui::layer_shell::LayerShellOptions {
                namespace: "waystart".to_string(),
                anchor: gpui::layer_shell::Anchor::all(),
                margin: None,
                keyboard_interactivity: gpui::layer_shell::KeyboardInteractivity::Exclusive,
                ..Default::default()
            }),
            #[cfg(not(target_os = "linux"))]
            kind: WindowKind::PopUp,
            is_resizable: false,
            is_minimizable: false,
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            window_decorations: Some(WindowDecorations::Client),
            titlebar: Some(TitlebarOptions {
                title: Some("Waystart".into()),
                appears_transparent: true,
                traffic_light_position: Some(point(px(-100.0), px(0.0))),
            }),
            ..Default::default()
        },
        |window, cx| {
            window.focus(&waystart.focus_handle(cx), cx);
            waystart
        },
    )
    .unwrap()
}
