use std::{process::Command, sync::LazyLock};

use crate::config::Config;

pub(super) fn create_terminal_command(config: &Config, exec: &[String]) -> Command {
    let terminal = config.terminal.as_deref().unwrap_or_else(|| *TERMINAL);
    let mut command = Command::new(terminal);
    command.arg("-e").args(exec);
    command
}

static TERMINAL: LazyLock<&str> = LazyLock::new(|| {
    let paths = std::env::split_paths(&std::env::var_os("PATH").unwrap()).collect::<Vec<_>>();

    [
        "ghostty",
        "kitty",
        "alacritty",
        "foot",
        "gnome-terminal",
        "konsole",
    ]
    .into_iter()
    .find(|term| paths.iter().any(|dir| dir.join(term).is_file()))
    .expect(
        "Failed to find a terminal emulator in your PATH. Please use the config to specify one.",
    )
});
