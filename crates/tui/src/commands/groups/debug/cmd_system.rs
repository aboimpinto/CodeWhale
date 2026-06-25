//! `/system` command — show the system prompt.

use crate::commands::traits::{CommandInfo, RegisterCommand};
use crate::localization::MessageId;
use crate::tui::app::App;

use super::CommandResult;

pub(in crate::commands) const COMMAND_INFO: CommandInfo = CommandInfo {
    name: "system",
    aliases: &["xitong"],
    usage: "/system",
    description_id: MessageId::CmdSystemDescription,
};

pub(in crate::commands) struct SystemCmd;

impl RegisterCommand for SystemCmd {
    fn info() -> &'static CommandInfo {
        &COMMAND_INFO
    }

    fn execute(app: &mut App, _arg: Option<&str>) -> CommandResult {
        super::debug::system_prompt(app)
    }
}
