//! `/purge` command.

use crate::commands::traits::{CommandInfo, RegisterCommand};
use crate::localization::MessageId;
use crate::tui::app::App;

use super::CommandResult;

pub(in crate::commands) const COMMAND_INFO: CommandInfo = CommandInfo {
    name: "purge",
    aliases: &["qingchu"],
    usage: "/purge",
    description_id: MessageId::CmdPurgeDescription,
};

pub(in crate::commands) struct PurgeCmd;

impl RegisterCommand for PurgeCmd {
    fn info() -> &'static CommandInfo {
        &COMMAND_INFO
    }

    fn execute(app: &mut App, _arg: Option<&str>) -> CommandResult {
        super::session::purge(app)
    }
}

// ---------------------------------------------------------------------------
// FEAT-023 Phase 4 (D3/D6): portable pure registration. `/purge` emits the
// existing receipt + action with no host context bundle; any argument is
// ignored exactly like the baseline.
// ---------------------------------------------------------------------------

use codewhale_command_contract::handler::CommandHandler;
use codewhale_command_contract::metadata::{
    CommandInfo as ContractInfo, RegisterCommand as ContractRegisterCommand,
};

use crate::tui::app::AppAction;

pub(in crate::commands) const CONTRACT_INFO: ContractInfo = ContractInfo {
    name: "purge",
    aliases: &["qingchu"],
    usage: "/purge",
    description_key: "cmd_purge_description",
};

impl ContractRegisterCommand<CommandResult> for PurgeCmd {
    fn info() -> &'static ContractInfo {
        &CONTRACT_INFO
    }

    fn handler() -> CommandHandler<CommandResult> {
        CommandHandler::Pure(purge_pure)
    }
}

/// Pure `/purge` — byte-identical to the baseline `session::purge`.
pub(in crate::commands) fn purge_pure(_arg: Option<&str>) -> CommandResult {
    CommandResult::with_message_and_action(
        "Agent context purge triggered...".to_string(),
        AppAction::PurgeContext,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pure_purge_is_byte_identical_to_baseline() {
        for arg in [None, Some("anything")] {
            let portable = purge_pure(arg);
            let mut app = crate::test_support::test_app_with_options(
                crate::test_support::test_tui_options(std::path::PathBuf::from(".")),
            );
            let baseline = super::super::session::purge(&mut app);
            assert_eq!(portable.message, baseline.message);
            assert_eq!(portable.is_error, baseline.is_error);
            assert_eq!(portable.action, baseline.action);
        }
    }
}
