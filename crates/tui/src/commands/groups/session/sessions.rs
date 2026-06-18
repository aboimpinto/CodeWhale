//! `/sessions` command — open session picker or manage persisted sessions.

use crate::commands::traits::{CommandInfo, RegisterCommand};
use crate::localization::MessageId;
use crate::tui::app::App;
use crate::tui::session_picker::SessionPickerView;

use super::CommandResult;

pub(in crate::commands) const COMMAND_INFO: CommandInfo = CommandInfo {
    name: "sessions",
    aliases: &["resume"],
    usage: "/sessions [show|prune <days>]",
    description_id: MessageId::CmdSessionsDescription,
};

pub(in crate::commands) struct SessionsCmd;

impl RegisterCommand for SessionsCmd {
    fn info() -> &'static CommandInfo {
        &COMMAND_INFO
    }

    fn execute(app: &mut App, arg: Option<&str>) -> CommandResult {
        sessions(app, arg)
    }
}

/// Open the session picker UI, or run a sub-action like
/// `prune <days>` for housekeeping (#406 phase-1.5).
pub fn sessions(app: &mut App, arg: Option<&str>) -> CommandResult {
    let trimmed = arg.unwrap_or("").trim();
    if trimmed.is_empty() {
        app.view_stack.push(SessionPickerView::new(&app.workspace));
        return CommandResult::ok();
    }

    let mut parts = trimmed.split_whitespace();
    let action = parts.next().unwrap_or("").to_ascii_lowercase();
    match action.as_str() {
        "prune" => prune(app, parts.next()),
        "show" | "list" | "picker" => {
            app.view_stack.push(SessionPickerView::new(&app.workspace));
            CommandResult::ok()
        }
        _ => CommandResult::error(format!(
            "unknown subcommand `{action}`. usage: /sessions [show|prune <days>]"
        )),
    }
}

/// Prune persisted sessions older than `<days>` from
/// `~/.deepseek/sessions/`. Wraps
/// [`crate::session_manager::SessionManager::prune_sessions_older_than`]
/// so users can run a safe cleanup without leaving the TUI. Skips
/// the checkpoint subdirectory (the helper guarantees that already).
fn prune(_app: &mut App, days_arg: Option<&str>) -> CommandResult {
    let days_str = match days_arg {
        Some(s) => s,
        None => {
            return CommandResult::error(
                "usage: /sessions prune <days>   (e.g. `/sessions prune 30` to drop sessions older than 30 days)",
            );
        }
    };
    let days: u64 = match days_str.parse() {
        Ok(n) if n > 0 => n,
        _ => {
            return CommandResult::error(format!(
                "expected a positive integer number of days, got `{days_str}`"
            ));
        }
    };

    let manager = match crate::session_manager::SessionManager::default_location() {
        Ok(m) => m,
        Err(err) => {
            return CommandResult::error(format!("could not open sessions directory: {err}"));
        }
    };

    let max_age = std::time::Duration::from_secs(days.saturating_mul(24 * 60 * 60));
    match manager.prune_sessions_older_than(max_age) {
        Ok(0) => CommandResult::message(format!("no sessions older than {days}d to prune")),
        Ok(n) => CommandResult::message(format!(
            "pruned {n} session{} older than {days}d",
            if n == 1 { "" } else { "s" }
        )),
        Err(err) => CommandResult::error(format!("prune failed: {err}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::tui::app::{App, TuiOptions};
    use tempfile::TempDir;

    fn create_test_app_with_tmpdir(tmpdir: &TempDir) -> App {
        let options = TuiOptions {
            model: "deepseek-v4-pro".to_string(),
            workspace: tmpdir.path().to_path_buf(),
            config_path: None,
            config_profile: None,
            allow_shell: false,
            use_alt_screen: true,
            use_mouse_capture: false,
            use_bracketed_paste: true,
            max_subagents: 1,
            skills_dir: tmpdir.path().join("skills"),
            memory_path: tmpdir.path().join("memory.md"),
            notes_path: tmpdir.path().join("notes.txt"),
            mcp_config_path: tmpdir.path().join("mcp.json"),
            use_memory: false,
            start_in_agent_mode: false,
            skip_onboarding: true,
            yolo: false,
            resume_session_id: None,
            initial_input: None,
        };
        App::new(options, &Config::default())
    }

    #[test]
    fn test_sessions_pushes_picker_view() {
        let tmpdir = TempDir::new().unwrap();
        let mut app = create_test_app_with_tmpdir(&tmpdir);
        let initial_kind = app.view_stack.top_kind();

        let result = sessions(&mut app, None);
        assert_eq!(result.message, None);
        assert!(result.action.is_none());
        // View should have changed (session picker should be on top)
        assert_ne!(app.view_stack.top_kind(), initial_kind);
    }

    #[test]
    fn test_sessions_show_subcommand_pushes_picker_view() {
        let tmpdir = TempDir::new().unwrap();
        let mut app = create_test_app_with_tmpdir(&tmpdir);
        let initial_kind = app.view_stack.top_kind();
        let result = sessions(&mut app, Some("show"));
        assert_eq!(result.message, None);
        assert_ne!(app.view_stack.top_kind(), initial_kind);
    }

    #[test]
    fn test_sessions_prune_requires_days_argument() {
        let tmpdir = TempDir::new().unwrap();
        let mut app = create_test_app_with_tmpdir(&tmpdir);
        let result = sessions(&mut app, Some("prune"));
        assert!(result.is_error);
        assert!(
            result.message.as_deref().unwrap_or("").contains("usage"),
            "expected usage hint: {:?}",
            result.message
        );
    }

    #[test]
    fn test_sessions_prune_rejects_non_positive_days() {
        let tmpdir = TempDir::new().unwrap();
        let mut app = create_test_app_with_tmpdir(&tmpdir);
        for bad in ["0", "-3", "abc", "3.14"] {
            let result = sessions(&mut app, Some(&format!("prune {bad}")));
            assert!(result.is_error, "expected error for `{bad}`");
        }
    }

    #[test]
    fn test_sessions_unknown_subcommand_errors() {
        let tmpdir = TempDir::new().unwrap();
        let mut app = create_test_app_with_tmpdir(&tmpdir);
        let result = sessions(&mut app, Some("teleport"));
        assert!(result.is_error);
        assert!(
            result
                .message
                .as_deref()
                .unwrap_or("")
                .contains("unknown subcommand"),
            "expected unknown-subcommand error: {:?}",
            result.message
        );
    }
}
