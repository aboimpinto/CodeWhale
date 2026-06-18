//! `/rlm` command — Recursive Language Model (RLM) turn.

use crate::tui::app::{App, AppAction};

use super::CommandResult;

/// Execute a Recursive Language Model (RLM) turn — Algorithm 1 from
/// Zhang et al. (arXiv:2512.24601).
///
/// The user's prompt text is passed as the argument. It will be stored
/// in the REPL as the `PROMPT` variable. The root LLM will only see
/// metadata about the REPL state, never the prompt text directly.
pub fn rlm(app: &mut App, arg: Option<&str>) -> CommandResult {
    let (max_depth, target) = match super::util::parse_depth_prefixed_arg(arg, 1) {
        Ok(parsed) => parsed,
        Err(message) => return CommandResult::error(message),
    };
    let target = match target {
        Some(p) if !p.trim().is_empty() => p.trim().to_string(),
        _ => {
            return CommandResult::error(
                "Usage: /rlm [N] <file_or_text>\n\n\
                 Opens a persistent RLM context with sub_rlm depth N (0-3, default 1).",
            );
        }
    };

    let source_arg = if super::util::resolves_to_existing_file(app, &target) {
        format!(r#"file_path: "{target}""#)
    } else {
        format!("content: {target:?}")
    };
    let message = format!(
        "Open and use a persistent RLM session for this request. Call `rlm_open` with name `slash_rlm` and {source_arg}. Then call `rlm_configure` with `sub_rlm_max_depth: {max_depth}`. Use `rlm_eval` to inspect the context through `peek`, `search`, and `chunk`, and call `finalize(...)` from the REPL when ready. If a `var_handle` is returned, use `handle_read` for bounded slices or projections before answering."
    );

    CommandResult::with_message_and_action(
        format!("Opening persistent RLM context at depth {max_depth}..."),
        AppAction::SendMessage(message),
    )
}
