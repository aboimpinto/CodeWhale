//! `/diff` command — show diff of changes in the current session.

use crate::commands::traits::{CommandInfo, RegisterCommand};
use crate::localization::MessageId;
use crate::tui::app::App;

use super::CommandResult;

pub(in crate::commands) const COMMAND_INFO: CommandInfo = CommandInfo {
    name: "diff",
    aliases: &[],
    usage: "/diff",
    description_id: MessageId::CmdDiffDescription,
};

pub(in crate::commands) struct DiffCmd;

impl RegisterCommand for DiffCmd {
    fn info() -> &'static CommandInfo {
        &COMMAND_INFO
    }

    fn execute(app: &mut App, _arg: Option<&str>) -> CommandResult {
        super::debug::diff(app)
    }
}
