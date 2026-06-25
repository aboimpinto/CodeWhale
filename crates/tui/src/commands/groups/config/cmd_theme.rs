//! `/theme` command — set the UI theme.

use crate::commands::traits::{CommandInfo, RegisterCommand};
use crate::localization::MessageId;
use crate::tui::app::App;

use super::CommandResult;

pub(in crate::commands) const COMMAND_INFO: CommandInfo = CommandInfo {
    name: "theme",
    aliases: &[],
    usage: "/theme [name]",
    description_id: MessageId::CmdThemeDescription,
};

pub(in crate::commands) struct ThemeCmd;

impl RegisterCommand for ThemeCmd {
    fn info() -> &'static CommandInfo {
        &COMMAND_INFO
    }

    fn execute(app: &mut App, arg: Option<&str>) -> CommandResult {
        super::config::theme(app, arg)
    }
}
