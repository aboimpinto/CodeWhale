//! Skills command group dispatch.
//!
//! Commands: skills, skill, review, restore

use crate::commands::{CommandResult, restore, review, skills};
use crate::tui::app::App;

/// Dispatch a skills-group command.
///
/// Returns `None` if the command is not recognised as a skills command.
pub fn dispatch(command: &str, arg: Option<&str>, app: &mut App) -> Option<CommandResult> {
    match command {
        "skills" | "jinengliebiao" => Some(skills::list_skills(app, arg)),
        "skill" | "jineng" => Some(skills::run_skill(app, arg)),
        "review" | "shencha" => Some(review::review(app, arg)),
        "restore" => Some(restore::restore(app, arg)),
        _ => None,
    }
}
