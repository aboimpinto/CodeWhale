//! `/retry` command — retry the last conversation turn.

use crate::commands::traits::{CommandInfo, RegisterCommand};
use crate::localization::MessageId;
use crate::tui::app::App;

use super::CommandResult;

pub(in crate::commands) const COMMAND_INFO: CommandInfo = CommandInfo {
    name: "retry",
    aliases: &["chongshi"],
    usage: "/retry",
    description_id: MessageId::CmdRetryDescription,
};

pub(in crate::commands) struct RetryCmd;

impl RegisterCommand for RetryCmd {
    fn info() -> &'static CommandInfo {
        &COMMAND_INFO
    }

    fn execute(app: &mut App, _arg: Option<&str>) -> CommandResult {
        super::debug::retry(app)
    }
}
