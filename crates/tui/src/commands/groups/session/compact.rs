//! `/compact` command — trigger context compaction.

use crate::commands::traits::{CommandInfo, RegisterCommand};
use crate::localization::MessageId;
use crate::tui::app::{App, AppAction};

use super::CommandResult;

pub(in crate::commands) const COMMAND_INFO: CommandInfo = CommandInfo {
    name: "compact",
    aliases: &["yasuo"],
    usage: "/compact",
    description_id: MessageId::CmdCompactDescription,
};

/// Handler wrapper suitable for FunctionCommand registration.
fn run(app: &mut App, _arg: Option<&str>) -> CommandResult {
    compact(app)
}

pub(in crate::commands) struct CompactCmd;

impl RegisterCommand for CompactCmd {
    fn info() -> &'static CommandInfo {
        &COMMAND_INFO
    }

    fn execute(app: &mut App, arg: Option<&str>) -> CommandResult {
        run(app, arg)
    }
}

/// Trigger context compaction
pub fn compact(_app: &mut App) -> CommandResult {
    // Trigger immediate compaction via engine
    CommandResult::with_message_and_action(
        "Context compaction triggered...".to_string(),
        AppAction::CompactContext,
    )
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
    fn test_compact_toggles_state() {
        let tmpdir = TempDir::new().unwrap();
        let mut app = create_test_app_with_tmpdir(&tmpdir);

        let result = compact(&mut app);
        assert!(result.message.is_some());
        let msg = result.message.unwrap();
        assert!(msg.contains("compaction") || msg.contains("Compact"));
        assert!(matches!(result.action, Some(AppAction::CompactContext)));
    }
}
