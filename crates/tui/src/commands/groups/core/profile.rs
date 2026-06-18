//! `/profile` command — switch to a named configuration profile.

use super::CommandResult;
use crate::tui::app::{App, AppAction};

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
