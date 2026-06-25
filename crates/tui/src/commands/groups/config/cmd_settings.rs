//! `/settings` command — show all settings.

use crate::commands::traits::{CommandInfo, RegisterCommand};
use crate::localization::MessageId;
use crate::tui::app::App;

use super::CommandResult;

pub(in crate::commands) const COMMAND_INFO: CommandInfo = CommandInfo {
    name: "settings",
    aliases: &[],
    usage: "/settings",
    description_id: MessageId::CmdSettingsDescription,
};

pub(in crate::commands) struct SettingsCmd;

impl RegisterCommand for SettingsCmd {
    fn info() -> &'static CommandInfo {
        &COMMAND_INFO
    }

    fn execute(app: &mut App, _arg: Option<&str>) -> CommandResult {
        super::config::show_settings(app)
    }
}
