//! `/logout` command — clear the active provider API key.

use crate::commands::traits::{CommandInfo, RegisterCommand};
use crate::localization::MessageId;
use crate::tui::app::App;

use super::CommandResult;

pub(in crate::commands) const COMMAND_INFO: CommandInfo = CommandInfo {
    name: "logout",
    aliases: &[],
    usage: "/logout",
    description_id: MessageId::CmdLogoutDescription,
};

pub(in crate::commands) struct LogoutCmd;

impl RegisterCommand for LogoutCmd {
    fn info() -> &'static CommandInfo {
        &COMMAND_INFO
    }

    fn execute(app: &mut App, _arg: Option<&str>) -> CommandResult {
        super::config::logout(app)
    }
}
