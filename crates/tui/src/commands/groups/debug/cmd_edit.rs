//! `/edit` command — edit the current conversation state.

use crate::commands::traits::{CommandInfo, RegisterCommand};
use crate::localization::MessageId;
use crate::tui::app::App;

use super::CommandResult;

pub(in crate::commands) const COMMAND_INFO: CommandInfo = CommandInfo {
    name: "edit",
    aliases: &[],
    usage: "/edit",
    description_id: MessageId::CmdEditDescription,
};

pub(in crate::commands) struct EditCmd;

impl RegisterCommand for EditCmd {
    fn info() -> &'static CommandInfo {
        &COMMAND_INFO
    }

    fn execute(app: &mut App, _arg: Option<&str>) -> CommandResult {
        super::debug::edit(app)
    }
}
