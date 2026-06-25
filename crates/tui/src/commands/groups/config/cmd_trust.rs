//! `/trust` command — manage workspace trust settings.

use crate::commands::traits::{CommandInfo, RegisterCommand};
use crate::localization::MessageId;
use crate::tui::app::App;

use super::CommandResult;

pub(in crate::commands) const COMMAND_INFO: CommandInfo = CommandInfo {
    name: "trust",
    aliases: &["xinren"],
    usage: "/trust [on|off|add <path>|remove <path>|list]",
    description_id: MessageId::CmdTrustDescription,
};

pub(in crate::commands) struct TrustCmd;

impl RegisterCommand for TrustCmd {
    fn info() -> &'static CommandInfo {
        &COMMAND_INFO
    }

    fn execute(app: &mut App, arg: Option<&str>) -> CommandResult {
        super::config::trust(app, arg)
    }
}
