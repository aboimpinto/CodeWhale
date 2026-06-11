//! Config command group dispatch.
//!
//! Commands: config, settings, status, statusline, mode, theme, verbose,
//! trust, logout, slop, lsp, sidebar

use crate::commands::{CommandResult, config, status};
use crate::tui::app::App;

/// Dispatch a config-group command.
///
/// Returns `None` if the command is not recognised as a config command.
pub fn dispatch(command: &str, arg: Option<&str>, app: &mut App) -> Option<CommandResult> {
    match command {
        "config" => Some(config::config_command(app, arg)),
        "sidebar" => Some(config::sidebar(app, arg)),
        "settings" => Some(config::show_settings(app)),
        "status" => Some(status::status(app)),
        "statusline" => Some(config::status_line(app)),
        "mode" => Some(config::mode(app, arg)),
        "jihua" => Some(config::mode(app, Some("plan"))),
        "zidong" => Some(config::mode(app, Some("yolo"))),
        "theme" => Some(config::theme(app, arg)),
        "verbose" => Some(config::verbose(app, arg)),
        "trust" | "xinren" => Some(config::trust(app, arg)),
        "logout" => Some(config::logout(app)),
        "slop" | "canzha" => Some(config::slop(app, arg)),
        "lsp" => Some(config::lsp_command(app, arg)),
        _ => None,
    }
}
