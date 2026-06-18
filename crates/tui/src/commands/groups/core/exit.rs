//! `/exit` command — quit the application.

use crate::commands::traits::{CommandInfo, RegisterCommand};
use crate::localization::MessageId;
use crate::tui::app::{App, AppAction};

use super::CommandResult;

pub(in crate::commands) const COMMAND_INFO: CommandInfo = CommandInfo {
    name: "exit",
    aliases: &["quit", "q", "tuichu"],
    usage: "/exit",
    description_id: MessageId::CmdExitDescription,
};

/// Handler wrapper for FunctionCommand registration.
fn run(_app: &mut App, _arg: Option<&str>) -> CommandResult {
    exit()
}

pub(in crate::commands) struct ExitCmd;

impl RegisterCommand for ExitCmd {
    fn info() -> &'static CommandInfo {
        &COMMAND_INFO
    }

    fn execute(app: &mut App, arg: Option<&str>) -> CommandResult {
        run(app, arg)
    }
}

/// Exit the application
pub fn exit() -> CommandResult {
    CommandResult::action(AppAction::Quit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_returns_quit_action() {
        let result = exit();
        assert!(result.message.is_none());
        assert!(matches!(result.action, Some(AppAction::Quit)));
    }
}
