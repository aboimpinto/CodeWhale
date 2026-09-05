//! `/compact` command.

use crate::commands::traits::{CommandInfo, RegisterCommand};
use crate::localization::MessageId;
use crate::tui::app::App;

use super::CommandResult;

pub(in crate::commands) const COMMAND_INFO: CommandInfo = CommandInfo {
    name: "compact",
    aliases: &["yasuo"],
    usage: "/compact [focus]",
    description_id: MessageId::CmdCompactDescription,
};

pub(in crate::commands) struct CompactCmd;

impl RegisterCommand for CompactCmd {
    fn info() -> &'static CommandInfo {
        &COMMAND_INFO
    }

    fn execute(app: &mut App, arg: Option<&str>) -> CommandResult {
        super::session::compact(app, arg)
    }
}

// ---------------------------------------------------------------------------
// FEAT-023 Phase 4 (D3/D6): portable pure registration. `/compact` parses and
// normalizes its focus argument and emits the existing receipt + action with
// no host context bundle (the baseline `App` parameter is unused).
// ---------------------------------------------------------------------------

use codewhale_command_contract::handler::CommandHandler;
use codewhale_command_contract::metadata::{
    CommandInfo as ContractInfo, RegisterCommand as ContractRegisterCommand,
};

use crate::tui::app::AppAction;

pub(in crate::commands) const CONTRACT_INFO: ContractInfo = ContractInfo {
    name: "compact",
    aliases: &["yasuo"],
    usage: "/compact [focus]",
    description_key: "cmd_compact_description",
};

impl ContractRegisterCommand<CommandResult> for CompactCmd {
    fn info() -> &'static ContractInfo {
        &CONTRACT_INFO
    }

    fn handler() -> CommandHandler<CommandResult> {
        CommandHandler::Pure(compact_pure)
    }
}

/// Pure `/compact` — byte-identical to the baseline `session::compact`.
pub(in crate::commands) fn compact_pure(arg: Option<&str>) -> CommandResult {
    let focus = arg
        .map(str::trim)
        .filter(|focus| !focus.is_empty())
        .map(str::to_string);
    let receipt = match focus.as_deref() {
        Some(focus) => format!("Context compaction triggered (focus: {focus})..."),
        None => "Context compaction triggered...".to_string(),
    };
    CommandResult::with_message_and_action(receipt, AppAction::CompactContext { focus })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pure_compact_is_byte_identical_to_baseline() {
        let cases: [Option<&str>; 4] = [
            None,
            Some("   "),
            Some("the auth refactor"),
            Some("  padded  "),
        ];
        for arg in cases {
            let portable = compact_pure(arg);
            let mut app = crate::test_support::test_app_with_options(
                crate::test_support::test_tui_options(std::path::PathBuf::from(".")),
            );
            let baseline = super::super::session::compact(&mut app, arg);
            assert_eq!(
                portable.message, baseline.message,
                "message parity for {arg:?}"
            );
            assert_eq!(portable.is_error, baseline.is_error);
            assert_eq!(
                portable.action, baseline.action,
                "action parity for {arg:?}"
            );
        }
    }
}
