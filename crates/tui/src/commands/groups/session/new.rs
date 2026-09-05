//! `/new` command.

use codewhale_command_contract::facets::CommandSessionLifecycleContext;
use codewhale_command_contract::handler::{CommandContexts, CommandHandler};
use codewhale_command_contract::metadata::{
    CommandInfo as ContractInfo, RegisterCommand as ContractRegisterCommand,
};

use crate::commands::traits::{CommandInfo, RegisterCommand};
use crate::localization::MessageId;
use crate::tui::app::App;

use super::CommandResult;

pub(in crate::commands) const COMMAND_INFO: CommandInfo = CommandInfo {
    name: "new",
    aliases: &[],
    usage: "/new [--force]",
    description_id: MessageId::CmdNewDescription,
};

pub(in crate::commands) struct NewCmd;

impl RegisterCommand for NewCmd {
    fn info() -> &'static CommandInfo {
        &COMMAND_INFO
    }

    fn execute(app: &mut App, arg: Option<&str>) -> CommandResult {
        super::session::new_session(app, arg)
    }
}

pub(in crate::commands) const CONTRACT_INFO: ContractInfo = ContractInfo {
    name: "new",
    aliases: &[],
    usage: "/new [--force]",
    description_key: "cmd_new_description",
};

impl ContractRegisterCommand<CommandResult> for NewCmd {
    fn info() -> &'static ContractInfo {
        &CONTRACT_INFO
    }
    fn handler() -> CommandHandler<CommandResult> {
        CommandHandler::Contextual {
            capabilities:
                codewhale_command_contract::handler::CommandCapabilities::SESSION_LIFECYCLE,
            handler: new_contextual,
        }
    }
}

pub(in crate::commands) fn new_contextual(
    contexts: CommandContexts<'_>,
    arg: Option<&str>,
) -> CommandResult {
    let mut parts = contexts.into_parts();
    let Some(lifecycle) = parts.lifecycle.as_deref_mut() else {
        return CommandResult::error(
            "Command capability unavailable: session_lifecycle".to_string(),
        );
    };
    new_portable(lifecycle, arg)
}

pub(in crate::commands) fn new_portable(
    lifecycle: &mut dyn CommandSessionLifecycleContext,
    arg: Option<&str>,
) -> CommandResult {
    let force = match arg.map(str::trim).filter(|s| !s.is_empty()) {
        None => false,
        Some("--force" | "force") => true,
        Some(other) => {
            return CommandResult::error(format!(
                "Usage: /new [--force]\n\nUnknown argument: {other}"
            ));
        }
    };
    if lifecycle.transition_blocked() {
        return CommandResult::error(
            "Cannot start a new session while runtime work is active. Wait for the current turn, maintenance, and background tasks to finish, or cancel that specific work. `/new --force` only discards draft or queued input."
                .to_string(),
        );
    }
    match lifecycle.fresh_session(force) {
        Ok(receipt) => CommandResult::with_message_and_action(
            format!(
                "Started new session {} (New Session). Previous sessions remain available via /resume.",
                receipt.truncated_id
            ),
            super::sync_session_action(receipt.sync),
        ),
        Err(error) => CommandResult::error(error),
    }
}
