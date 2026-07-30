pub mod client;
pub mod server;

const SOCKET_PATH: &str = "/tmp/waystart.sock";

const MESSAGE_OPEN: &[u8] = b"open";
const MESSAGE_CLOSE: &[u8] = b"close";
const MESSAGE_TOGGLE: &[u8] = b"toggle";
const MESSAGE_QUIT: &[u8] = b"quit";
