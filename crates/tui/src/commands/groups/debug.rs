//! Debug command group dispatch.
//!
//! Commands: translate, tokens, cost, cache, system, context, edit, diff,
//! undo, retry, balance, change

use crate::commands::{CommandResult, balance, change, core, debug};
use crate::tui::app::App;

/// Dispatch a debug-group command.
///
/// Returns `None` if the command is not recognised as a debug command.
pub fn dispatch(command: &str, arg: Option<&str>, app: &mut App) -> Option<CommandResult> {
    match command {
        "translate" | "translation" | "transale" => Some(core::translate(app)),
        "tokens" => Some(debug::tokens(app)),
        "cost" => Some(debug::cost(app)),
        "balance" => Some(balance::balance(app)),
        "cache" => Some(debug::cache(app, arg)),
        "change" => Some(change::change(app, arg)),
        "system" | "xitong" => Some(debug::system_prompt(app)),
        "context" | "ctx" => Some(debug::context(app)),
        "edit" => Some(debug::edit(app)),
        "diff" => Some(debug::diff(app)),
        "undo" => {
            // Try surgical patch-undo first; fall back to conversation undo
            // if no snapshots are available or if the snapshot undo couldn't
            // find anything useful.
            let result = debug::patch_undo(app);
            if result.message.as_deref().is_none_or(|m| {
                m.starts_with("No snapshots found")
                    || m.starts_with("No tool or pre-turn")
                    || m.starts_with("Snapshot repo")
            }) {
                Some(debug::undo_conversation(app))
            } else {
                Some(result)
            }
        }
        "retry" | "chongshi" => Some(debug::retry(app)),
        _ => None,
    }
}
