//! Utility command group dispatch.
//!
//! Commands: (none currently — balance and change are dispatched by the
//! debug group, which runs earlier in the dispatch chain).
//!
//! This file is kept as a placeholder for future utility commands that
//! don't fit naturally into other groups (e.g. workspace maintenance,
//! file operations, or cross-cutting utilities).

use crate::commands::CommandResult;
use crate::tui::app::App;

/// Dispatch a utility-group command.
///
/// Returns `None` if the command is not recognised as a utility command.
pub fn dispatch(_command: &str, _arg: Option<&str>, _app: &mut App) -> Option<CommandResult> {
    None
}
