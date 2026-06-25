//! `/tokens` command — show token usage for the current session.

use crate::commands::traits::{CommandInfo, RegisterCommand};
use crate::localization::MessageId;
use crate::tui::app::App;

use super::CommandResult;

pub(in crate::commands) const COMMAND_INFO: CommandInfo = CommandInfo {
    name: "tokens",
    aliases: &[],
    usage: "/tokens",
    description_id: MessageId::CmdTokensDescription,
};

pub(in crate::commands) struct TokensCmd;

impl RegisterCommand for TokensCmd {
    fn info() -> &'static CommandInfo {
        &COMMAND_INFO
    }

    fn execute(app: &mut App, _arg: Option<&str>) -> CommandResult {
        super::debug::tokens(app)
    }
}
