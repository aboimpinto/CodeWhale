//! `/mode` command — switch between agent, plan, and yolo modes.

use crate::commands::traits::{CommandInfo, RegisterCommand};
use crate::localization::MessageId;
use crate::tui::app::App;

use super::CommandResult;

pub(in crate::commands) const COMMAND_INFO: CommandInfo = CommandInfo {
    name: "mode",
    aliases: &["jihua", "zidong"],
    usage: "/mode [agent|plan|yolo|1|2|3]",
    description_id: MessageId::CmdModeDescription,
};

pub(in crate::commands) struct ModeCmd;

impl RegisterCommand for ModeCmd {
    fn info() -> &'static CommandInfo {
        &COMMAND_INFO
    }

    fn execute(app: &mut App, arg: Option<&str>) -> CommandResult {
        super::config::mode(app, arg)
    }
}
