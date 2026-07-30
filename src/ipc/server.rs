use std::cell::RefCell;
use std::rc::Rc;

use smol::io::{AsyncBufReadExt, BufReader};
use smol::net::unix::{UnixListener, UnixStream};
use smol::stream::StreamExt;

use gpui::{AsyncApp, Entity, WindowHandle};

use crate::ipc::*;
use crate::open_window;
use crate::ui::Waystart;

#[derive(Clone)]
pub struct SocketServer {
    app: AsyncApp,
    waystart: Entity<Waystart>,
    window: Rc<RefCell<Option<WindowHandle<Waystart>>>>,
}

impl SocketServer {
    pub fn new(app: AsyncApp, waystart: Entity<Waystart>) -> Self {
        Self {
            app,
            waystart,
            window: Rc::new(RefCell::new(None)),
        }
    }

    pub fn listen(&self) {
        if std::fs::exists(SOCKET_PATH).ok().unwrap_or(false) {
            std::fs::remove_file(SOCKET_PATH).expect("Failed to remove existing IPC socket");
        }

        let listener = UnixListener::bind(SOCKET_PATH).expect("Failed to bind IPC socket");

        let this = self.clone();
        self.app
            .spawn(async move |cx| {
                loop {
                    match listener.accept().await {
                        Ok((stream, _)) => {
                            let window = this.window.clone();
                            let waystart = this.waystart.clone();
                            cx.spawn(async move |cx| {
                                Self::handle_ipc_stream(stream, window, waystart, cx).await
                            })
                            .detach();
                        }
                        Err(e) => {
                            eprintln!("Failed to accept IPC connection: {}", e);
                        }
                    }
                }
            })
            .detach();
    }

    async fn handle_ipc_stream(
        stream: UnixStream,
        window: Rc<RefCell<Option<WindowHandle<Waystart>>>>,
        waystart: Entity<Waystart>,
        cx: &mut AsyncApp,
    ) {
        let reader = BufReader::new(stream);
        let mut lines = reader.lines();

        while let Some(Ok(message)) = lines.next().await {
            match message.as_bytes() {
                MESSAGE_OPEN => cx.update(|cx| {
                    let mut window = window.borrow_mut();
                    if window.map(|w| w.is_active(cx).is_none()).unwrap_or(true) {
                        *window = Some(open_window(cx, waystart.clone()));
                    }
                }),
                MESSAGE_CLOSE => cx.update(|cx| {
                    let mut window = window.borrow_mut();
                    if let Some(window) = window.take()
                        && window.is_active(cx).is_some()
                    {
                        window
                            .update(cx, |_, window, _| window.remove_window())
                            .unwrap();
                    }
                }),
                MESSAGE_TOGGLE => cx.update(|cx| {
                    let mut window = window.borrow_mut();
                    if let Some(window) = window.take()
                        && window.is_active(cx).is_some()
                    {
                        window
                            .update(cx, |_, window, _| window.remove_window())
                            .unwrap();
                    } else {
                        *window = Some(open_window(cx, waystart.clone()));
                    }
                }),
                MESSAGE_QUIT => cx.update(|cx| cx.quit()),
                _ => {
                    eprintln!("Received unknown IPC message: {}", message);
                    return;
                }
            }
        }
    }
}
