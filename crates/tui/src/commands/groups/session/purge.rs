//! `/purge` command — trigger agent-driven context purging.

use crate::commands::traits::{CommandInfo, RegisterCommand};
use crate::localization::MessageId;
use crate::tui::app::{App, AppAction};

use super::CommandResult;

pub(in crate::commands) const COMMAND_INFO: CommandInfo = CommandInfo {
    name: "purge",
    aliases: &["qingchu"],
    usage: "/purge",
    description_id: MessageId::CmdPurgeDescription,
};

/// Handler wrapper suitable for FunctionCommand registration.
fn run(app: &mut App, _arg: Option<&str>) -> CommandResult {
    purge(app)
}

pub(in crate::commands) struct PurgeCmd;

impl RegisterCommand for PurgeCmd {
    fn info() -> &'static CommandInfo {
        &COMMAND_INFO
    }

    fn execute(app: &mut App, arg: Option<&str>) -> CommandResult {
        run(app, arg)
    }
}

/// Trigger agent-driven context purging.
pub fn purge(_app: &mut App) -> CommandResult {
    CommandResult::with_message_and_action(
        "Agent context purge triggered...".to_string(),
        AppAction::PurgeContext,
    )
}
