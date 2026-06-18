//! `/new` command — start a fresh session.

use crate::commands::traits::{CommandInfo, RegisterCommand};
use crate::localization::MessageId;
use crate::tui::app::{App, AppAction};

use super::CommandResult;

pub(in crate::commands) const COMMAND_INFO: CommandInfo = CommandInfo {
    name: "new",
    aliases: &[],
    usage: "/new [--force]",
    description_id: MessageId::CmdNewDescription,
};

pub(in crate::commands) struct NewCmd;

impl RegisterCommand for NewCmd {
    fn info() -> &'static CommandInfo {
        &COMMAND_INFO
    }

    fn execute(app: &mut App, arg: Option<&str>) -> CommandResult {
        new_session(app, arg)
    }
}

/// Start a fresh saved session from the current TUI state.
pub fn new_session(app: &mut App, arg: Option<&str>) -> CommandResult {
    let force = match arg.map(str::trim).filter(|s| !s.is_empty()) {
        None => false,
        Some("--force" | "force") => true,
        Some(other) => {
            return CommandResult::error(format!(
                "Usage: /new [--force]\n\nUnknown argument: {other}"
            ));
        }
    };

    if !force {
        let blockers = new_session_blockers(app);
        if !blockers.is_empty() {
            return CommandResult::error(format!(
                "Cannot start a new session while {}. Run `/new --force` to discard pending work and start a fresh session.",
                blockers.join(", ")
            ));
        }
    }

    let new_id = uuid::Uuid::new_v4().to_string();
    app.reset_conversation_state();
    app.clear_input();
    app.session_artifacts.clear();
    app.session_context_references.clear();
    app.tool_evidence.clear();
    app.current_session_id = Some(new_id.clone());
    app.session_title = Some("New Session".to_string());
    app.scroll_to_bottom();

    CommandResult::with_message_and_action(
        format!(
            "Started new session {} (New Session). Previous sessions remain available via /resume.",
            crate::session_manager::truncate_id(&new_id)
        ),
        AppAction::SyncSession {
            session_id: Some(new_id),
            messages: Vec::new(),
            system_prompt: None,
            model: app.model.clone(),
            workspace: app.workspace.clone(),
        },
    )
}

fn new_session_blockers(app: &App) -> Vec<&'static str> {
    let mut blockers = Vec::new();
    if !app.input.trim().is_empty() {
        blockers.push("the composer has unsent text");
    }
    if !app.queued_messages.is_empty() || app.queued_draft.is_some() {
        blockers.push("queued messages are pending");
    }
    if app.is_loading || app.runtime_turn_status.as_deref() == Some("in_progress") {
        blockers.push("a turn is in progress");
    }
    if app.is_compacting {
        blockers.push("context compaction is running");
    }
    if app.task_panel.iter().any(|task| task.status == "running") {
        blockers.push("background tasks are running");
    }
    blockers
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
    fn new_session_from_resumed_state_creates_distinct_empty_session() {
        let tmpdir = TempDir::new().unwrap();
        let mut app = create_test_app_with_tmpdir(&tmpdir);
        app.current_session_id = Some("old-session".to_string());
        app.session_title = Some("Old Session".to_string());
        app.api_messages.push(crate::models::Message {
            role: "user".to_string(),
            content: vec![crate::models::ContentBlock::Text {
                text: "continue this thread".to_string(),
                cache_control: None,
            }],
        });
        app.add_message(crate::tui::history::HistoryCell::System {
            content: "old transcript".to_string(),
        });
        app.system_prompt = Some(crate::models::SystemPrompt::Text("old prompt".to_string()));
        app.session.total_tokens = 123;
        app.session.session_cost = 1.25;

        let result = new_session(&mut app, None);

        assert!(!result.is_error, "{:?}", result.message);
        let new_id = app.current_session_id.clone().expect("new session id");
        assert_ne!(new_id, "old-session");
        assert_eq!(app.session_title.as_deref(), Some("New Session"));
        assert!(app.api_messages.is_empty());
        assert!(app.history.is_empty());
        assert!(app.system_prompt.is_none());
        assert_eq!(app.session.total_tokens, 0);
        assert_eq!(app.session.session_cost, 0.0);
        assert!(
            result
                .message
                .as_deref()
                .unwrap_or_default()
                .contains("/resume")
        );
        match result.action {
            Some(AppAction::SyncSession {
                session_id,
                messages,
                system_prompt,
                ..
            }) => {
                assert_eq!(session_id.as_deref(), Some(new_id.as_str()));
                assert!(messages.is_empty());
                assert!(system_prompt.is_none());
            }
            other => panic!("expected SyncSession action, got {other:?}"),
        }
    }

    #[test]
    fn new_session_blocks_unsent_input_without_force() {
        let tmpdir = TempDir::new().unwrap();
        let mut app = create_test_app_with_tmpdir(&tmpdir);
        app.current_session_id = Some("old-session".to_string());
        app.input = "draft text".to_string();

        let result = new_session(&mut app, None);

        assert!(result.is_error);
        assert_eq!(app.current_session_id.as_deref(), Some("old-session"));
        assert_eq!(app.input, "draft text");
        assert!(result.action.is_none());
        assert!(
            result
                .message
                .as_deref()
                .unwrap_or_default()
                .contains("/new --force")
        );
    }

    #[test]
    fn new_session_force_discards_unsent_input() {
        let tmpdir = TempDir::new().unwrap();
        let mut app = create_test_app_with_tmpdir(&tmpdir);
        app.current_session_id = Some("old-session".to_string());
        app.input = "draft text".to_string();

        let result = new_session(&mut app, Some("--force"));

        assert!(!result.is_error, "{:?}", result.message);
        assert_ne!(app.current_session_id.as_deref(), Some("old-session"));
        assert!(app.input.is_empty());
        assert!(matches!(result.action, Some(AppAction::SyncSession { .. })));
    }

    #[test]
    fn new_session_blocks_in_flight_turn_without_force() {
        let tmpdir = TempDir::new().unwrap();
        let mut app = create_test_app_with_tmpdir(&tmpdir);
        app.current_session_id = Some("old-session".to_string());
        app.is_loading = true;

        let result = new_session(&mut app, None);

        assert!(result.is_error);
        assert_eq!(app.current_session_id.as_deref(), Some("old-session"));
        assert!(result.action.is_none());
    }
}
