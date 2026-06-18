//! `/profile` command — switch to a named configuration profile.

use crate::commands::traits::{CommandInfo, RegisterCommand};
use crate::localization::MessageId;
use super::CommandResult;
use crate::tui::app::{App, AppAction};

pub(in crate::commands) const COMMAND_INFO: CommandInfo = CommandInfo {
    name: "profile",
    aliases: &["dangan"],
    usage: "/profile <name>",
    description_id: MessageId::CmdHelpDescription,
};

pub(in crate::commands) struct ProfileCmd;

impl RegisterCommand for ProfileCmd {
    fn info() -> &'static CommandInfo {
        &COMMAND_INFO
    }

    fn execute(app: &mut App, arg: Option<&str>) -> CommandResult {
        profile_switch(app, arg)
    }
}

/// Switch to a configured profile.
pub fn profile_switch(_app: &mut App, arg: Option<&str>) -> CommandResult {
    let profile_name = match arg {
        Some(name) if !name.trim().is_empty() => name.trim().to_string(),
        _ => {
            return CommandResult::error(
                "Usage: /profile <name>\n\nSwitch to a named config profile. Profiles are defined in ~/.codewhale/config.toml under [profiles] sections.",
            );
        }
    };
    CommandResult::with_message_and_action(
        format!("Switching to profile '{profile_name}'..."),
        AppAction::SwitchProfile {
            profile: profile_name,
        },
    )
}
