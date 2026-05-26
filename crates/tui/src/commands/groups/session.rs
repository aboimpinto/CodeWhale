//! Session command group dispatch.
//!
//! Commands: rename, save, fork, new, sessions, relay, load, compact,
//! purge, export

use crate::commands::{CommandResult, relay, rename, session};
use crate::tui::app::App;

/// Dispatch a session-group command.
///
/// Returns `None` if the command is not recognised as a session command.
pub fn dispatch(command: &str, arg: Option<&str>, app: &mut App) -> Option<CommandResult> {
    match command {
        "rename" | "gaiming" | "chongmingming" => Some(rename::rename(app, arg)),
        "save" => Some(session::save(app, arg)),
        "fork" | "branch" => Some(session::fork(app)),
        "new" => Some(session::new_session(app, arg)),
        "sessions" | "resume" => Some(session::sessions(app, arg)),
        "relay" | "batonpass" | "接力" => Some(relay(app, arg)),
        "load" | "jiazai" => Some(session::load(app, arg)),
        "compact" | "yasuo" => Some(session::compact(app)),
        "purge" | "qingchu" => Some(session::purge(app)),
        "export" | "daochu" => Some(session::export(app, arg)),
        _ => None,
    }
}
