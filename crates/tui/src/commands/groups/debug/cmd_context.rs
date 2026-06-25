//! `/context` command — show context report.

use crate::commands::traits::{CommandInfo, RegisterCommand};
use crate::localization::MessageId;
use crate::tui::app::App;

use super::CommandResult;

pub(in crate::commands) const COMMAND_INFO: CommandInfo = CommandInfo {
    name: "context",
    aliases: &["ctx"],
    usage: "/context [report|json|summary]",
    description_id: MessageId::CmdContextDescription,
};

pub(in crate::commands) struct ContextCmd;

impl RegisterCommand for ContextCmd {
    fn info() -> &'static CommandInfo {
        &COMMAND_INFO
    }

    fn execute(app: &mut App, arg: Option<&str>) -> CommandResult {
        super::debug::context(app, arg)
    }
}
