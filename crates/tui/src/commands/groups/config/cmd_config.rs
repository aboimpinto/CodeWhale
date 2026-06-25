//! `/config` command — open the configuration view.

use crate::commands::traits::{CommandInfo, RegisterCommand};
use crate::localization::MessageId;
use crate::tui::app::App;

use super::CommandResult;

pub(in crate::commands) const COMMAND_INFO: CommandInfo = CommandInfo {
    name: "config",
    // /experiments is a discoverable entry to the same view: the Experimental
    // section exposes the WhaleFlow, goal, and sub-agent opt-ins (#3182).
    aliases: &["experiments", "experimental"],
    usage: "/config",
    description_id: MessageId::CmdConfigDescription,
};

pub(in crate::commands) struct ConfigCmd;

impl RegisterCommand for ConfigCmd {
    fn info() -> &'static CommandInfo {
        &COMMAND_INFO
    }

    fn execute(app: &mut App, arg: Option<&str>) -> CommandResult {
        super::config::config_command(app, arg)
    }
}
