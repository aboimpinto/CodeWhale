//! Project command group dispatch.
//!
//! Commands: init, goal/hunt, share

use crate::commands::{CommandResult, goal, init, share};
use crate::tui::app::App;

/// Dispatch a project-group command.
///
/// Returns `None` if the command is not recognised as a project command.
pub fn dispatch(command: &str, arg: Option<&str>, app: &mut App) -> Option<CommandResult> {
    match command {
        "init" => Some(init::init(app)),
        "share" => Some(share::share(app, arg)),
        "goal" | "hunt" | "mubiao" | "狩猎" => Some(goal::hunt(app, arg)),
        _ => None,
    }
}
