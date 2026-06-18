//! `/workspace` command — view or switch the current workspace.

use std::path::PathBuf;

use crate::commands::traits::{CommandInfo, RegisterCommand};
use crate::localization::MessageId;
use crate::tui::app::{App, AppAction};

use super::CommandResult;

pub(in crate::commands) const COMMAND_INFO: CommandInfo = CommandInfo {
    name: "workspace",
    aliases: &["cwd"],
    usage: "/workspace [path]",
    description_id: MessageId::CmdWorkspaceDescription,
};

pub(in crate::commands) struct WorkspaceCmd;

impl RegisterCommand for WorkspaceCmd {
    fn info() -> &'static CommandInfo {
        &COMMAND_INFO
    }

    fn execute(app: &mut App, arg: Option<&str>) -> CommandResult {
        workspace_switch(app, arg)
    }
}

/// Switch or view the current workspace directory.
pub fn workspace_switch(app: &mut App, arg: Option<&str>) -> CommandResult {
    let Some(raw_path) = arg.map(str::trim).filter(|path| !path.is_empty()) else {
        return CommandResult::message(format!("Current workspace: {}", app.workspace.display()));
    };

    let expanded = match expand_workspace_path(raw_path) {
        Ok(path) => path,
        Err(message) => return CommandResult::error(message),
    };
    let candidate = if expanded.is_absolute() {
        expanded
    } else {
        app.workspace.join(expanded)
    };

    if !candidate.exists() {
        return CommandResult::error(format!("Workspace does not exist: {}", candidate.display()));
    }
    if !candidate.is_dir() {
        return CommandResult::error(format!(
            "Workspace is not a directory: {}",
            candidate.display()
        ));
    }

    let workspace = candidate.canonicalize().unwrap_or(candidate);
    CommandResult::with_message_and_action(
        format!("Switching workspace to {}...", workspace.display()),
        AppAction::SwitchWorkspace { workspace },
    )
}

fn expand_workspace_path(path: &str) -> Result<PathBuf, String> {
    if path == "~" {
        return dirs::home_dir().ok_or_else(|| "Could not resolve home directory".to_string());
    }
    if let Some(rest) = path.strip_prefix("~/") {
        let home =
            dirs::home_dir().ok_or_else(|| "Could not resolve home directory".to_string())?;
        return Ok(home.join(rest));
    }
    Ok(PathBuf::from(path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::tui::app::{App, TuiOptions};
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn create_test_app() -> App {
        let options = TuiOptions {
            model: "deepseek-v4-pro".to_string(),
            workspace: PathBuf::from("/tmp/test-workspace"),
            config_path: None,
            config_profile: None,
            allow_shell: false,
            use_alt_screen: true,
            use_mouse_capture: false,
            use_bracketed_paste: true,
            max_subagents: 1,
            skills_dir: PathBuf::from("/tmp/test-skills"),
            memory_path: PathBuf::from("memory.md"),
            notes_path: PathBuf::from("notes.txt"),
            mcp_config_path: PathBuf::from("mcp.json"),
            use_memory: false,
            start_in_agent_mode: false,
            skip_onboarding: true,
            yolo: false,
            resume_session_id: None,
            initial_input: None,
        };
        let mut app = App::new(options, &Config::default());
        app.ui_locale = crate::localization::Locale::En;
        app
    }

    #[test]
    fn workspace_without_arg_shows_current_workspace() {
        let mut app = create_test_app();
        let result = workspace_switch(&mut app, None);
        let msg = result.message.expect("workspace should be shown");
        assert!(msg.contains("Current workspace:"));
        assert!(msg.contains("/tmp/test-workspace"));
        assert!(result.action.is_none());
    }

    #[test]
    fn workspace_existing_absolute_dir_returns_switch_action() {
        let mut app = create_test_app();
        let dir = tempdir().expect("temp dir");
        let result = workspace_switch(&mut app, Some(dir.path().to_str().unwrap()));
        assert!(matches!(
            result.action,
            Some(AppAction::SwitchWorkspace { workspace }) if workspace == dir.path().canonicalize().unwrap()
        ));
    }

    #[test]
    fn workspace_relative_dir_resolves_from_current_workspace() {
        let root = tempdir().expect("temp dir");
        let child = root.path().join("child");
        std::fs::create_dir(&child).expect("child dir");
        let mut app = create_test_app();
        app.workspace = root.path().to_path_buf();

        let result = workspace_switch(&mut app, Some("child"));
        assert!(matches!(
            result.action,
            Some(AppAction::SwitchWorkspace { workspace }) if workspace == child.canonicalize().unwrap()
        ));
    }

    #[test]
    fn workspace_rejects_missing_path() {
        let mut app = create_test_app();
        let result = workspace_switch(&mut app, Some("definitely-missing"));
        assert!(result.is_error);
        assert!(result.message.unwrap().contains("does not exist"));
    }

    #[test]
    fn workspace_rejects_file_path() {
        let root = tempdir().expect("temp dir");
        let file = root.path().join("file.txt");
        std::fs::write(&file, "not a directory").expect("test file");
        let mut app = create_test_app();

        let result = workspace_switch(&mut app, Some(file.to_str().unwrap()));
        assert!(result.is_error);
        assert!(result.message.unwrap().contains("not a directory"));
    }
}
