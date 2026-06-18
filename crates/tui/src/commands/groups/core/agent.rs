//! `/agent` command — open a persistent sub-agent session.

use crate::tui::app::{App, AppAction};

use super::CommandResult;

/// Open a persistent sub-agent session from a slash command.
pub fn agent(_app: &mut App, arg: Option<&str>) -> CommandResult {
    let (max_depth, task) = match super::util::parse_depth_prefixed_arg(arg, 1) {
        Ok(parsed) => parsed,
        Err(message) => return CommandResult::error(message),
    };
    let task = match task {
        Some(task) if !task.trim().is_empty() => task.trim().to_string(),
        _ => {
            return CommandResult::error(
                "Usage: /agent [N] <task>\n\n\
                 Opens a persistent sub-agent session with recursive agent depth N (0-3, default 1).",
            );
        }
    };
    let message = format!(
        "Open a persistent sub-agent session for this task. Call `agent_open` with name `slash_agent`, `prompt: {task:?}`, and `max_depth: {max_depth}`. Use nonblocking `agent_eval` to poll the current projection or send follow-up input while you keep working; pass `block:true` only when you deliberately want to wait for a terminal result. Use `handle_read` on the returned transcript_handle if you need more detail. Verify any claimed side effects before reporting success."
    );
    CommandResult::with_message_and_action(
        format!("Opening persistent sub-agent at depth {max_depth}..."),
        AppAction::SendMessage(message),
    )
}
