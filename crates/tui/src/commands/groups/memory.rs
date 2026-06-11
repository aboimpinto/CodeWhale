//! Memory command group dispatch.
//!
//! Commands: memory, note

use crate::commands::{CommandResult, memory, note};
use crate::tui::app::App;

/// Dispatch a memory-group command.
///
/// Returns `None` if the command is not recognised as a memory command.
pub fn dispatch(command: &str, arg: Option<&str>, app: &mut App) -> Option<CommandResult> {
    match command {
        "note" => Some(note::note(app, arg)),
        "memory" => Some(memory::memory(app, arg)),
        _ => None,
    }
}
