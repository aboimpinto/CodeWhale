//! `/links` command — show DeepSeek dashboard and API links.

use crate::commands::traits::{CommandInfo, RegisterCommand};
use crate::localization::{MessageId, tr};

use super::CommandResult;
use crate::tui::app::App;

pub(in crate::commands) const COMMAND_INFO: CommandInfo = CommandInfo {
    name: "links",
    aliases: &["dashboard", "api", "lianjie"],
    usage: "/links",
    description_id: MessageId::CmdLinksDescription,
};

/// Handler wrapper for FunctionCommand registration.
fn run(app: &mut App, _arg: Option<&str>) -> CommandResult {
    deepseek_links(app)
}

pub(in crate::commands) struct LinksCmd;

impl RegisterCommand for LinksCmd {
    fn info() -> &'static CommandInfo {
        &COMMAND_INFO
    }

    fn execute(app: &mut App, arg: Option<&str>) -> CommandResult {
        run(app, arg)
    }
}

/// Show DeepSeek dashboard and docs links
pub fn deepseek_links(app: &mut App) -> CommandResult {
    let locale = app.ui_locale;
    CommandResult::message(format!(
        "{}\n\
─────────────────────────────\n\
{} https://platform.deepseek.com\n\
{}      https://platform.deepseek.com/docs\n\n\
{}",
        tr(locale, MessageId::LinksTitle),
        tr(locale, MessageId::LinksDashboard),
        tr(locale, MessageId::LinksDocs),
        tr(locale, MessageId::LinksTip),
    ))
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
        app
    }

    #[test]
    fn test_deepseek_links() {
        let mut app = create_test_app();
        let result = deepseek_links(&mut app);
        assert!(result.message.is_some());
        let msg = result.message.unwrap();
        assert!(msg.contains("DeepSeek Links"));
        assert!(msg.contains("https://platform.deepseek.com"));
        assert!(result.action.is_none());
    }
}
