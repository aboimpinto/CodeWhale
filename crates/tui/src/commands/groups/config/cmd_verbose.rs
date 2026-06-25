//! `/verbose` command — toggle live transcript detail.

use crate::commands::traits::{CommandInfo, RegisterCommand};
use crate::localization::MessageId;
use crate::tui::app::App;

use super::CommandResult;

pub(in crate::commands) const COMMAND_INFO: CommandInfo = CommandInfo {
    name: "verbose",
    aliases: &[],
    usage: "/verbose [on|off]",
    description_id: MessageId::CmdVerboseDescription,
};

pub(in crate::commands) struct VerboseCmd;

impl RegisterCommand for VerboseCmd {
    fn info() -> &'static CommandInfo {
        &COMMAND_INFO
    }

    fn execute(app: &mut App, arg: Option<&str>) -> CommandResult {
        super::config::verbose(app, arg)
    }
}
