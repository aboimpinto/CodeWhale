//! `/subagents` command — list sub-agent status from the engine.

use crate::localization::{MessageId, tr};
use crate::tui::app::App;
use crate::tui::views::ModalKind;
use crate::tui::views::{SubAgentsView, subagent_view_agents};

use super::CommandResult;

/// List sub-agent status from the engine
pub fn subagents(app: &mut App) -> CommandResult {
    if app.view_stack.top_kind() != Some(ModalKind::SubAgents) {
        let agents = subagent_view_agents(app, &app.subagent_cache);
        app.view_stack.push(SubAgentsView::new(agents));
    }
    app.status_message = Some(tr(app.ui_locale, MessageId::SubagentsFetching).to_string());
    CommandResult::action(crate::tui::app::AppAction::ListSubAgents)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::tui::app::{App, TuiOptions};
    use std::path::PathBuf;

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
        app.api_provider = crate::config::ApiProvider::Deepseek;
        app.model = "deepseek-v4-pro".to_string();
        app
    }

    #[test]
    fn test_subagents_pushes_view_and_sets_status() {
        let mut app = create_test_app();
        let result = subagents(&mut app);
        assert!(result.message.is_none());
        assert!(matches!(
            result.action,
            Some(crate::tui::app::AppAction::ListSubAgents)
        ));
        assert_eq!(app.view_stack.top_kind(), Some(ModalKind::SubAgents));
        assert_eq!(
            app.status_message,
            Some("Fetching sub-agent status...".to_string())
        );
    }
}
