//! `/statusline` command.

use crate::commands::traits::{CommandInfo, RegisterCommand};
use crate::localization::MessageId;
use crate::tui::app::App;

use super::CommandResult;

pub(in crate::commands) const COMMAND_INFO: CommandInfo = CommandInfo {
    name: "statusline",
    aliases: &[],
    usage: "/statusline",
    description_id: MessageId::CmdStatuslineDescription,
};

pub(in crate::commands) struct StatuslineCmd;

impl RegisterCommand for StatuslineCmd {
    fn info() -> &'static CommandInfo {
        &COMMAND_INFO
    }

    fn execute(app: &mut App, _arg: Option<&str>) -> CommandResult {
        super::config::status_line(app)
    }
}
