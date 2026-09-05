//! `/save` command.

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
    name: "save",
    aliases: &[],
    usage: "/save [path]",
    description_id: MessageId::CmdSaveDescription,
};

pub(in crate::commands) struct SaveCmd;

impl RegisterCommand for SaveCmd {
    fn info() -> &'static CommandInfo {
        &COMMAND_INFO
    }

    fn execute(app: &mut App, arg: Option<&str>) -> CommandResult {
        super::session::save(app, arg)
    }
}

pub(in crate::commands) const CONTRACT_INFO: ContractInfo = ContractInfo {
    name: "save",
    aliases: &[],
    usage: "/save [path]",
    description_key: "cmd_save_description",
};

impl ContractRegisterCommand<CommandResult> for SaveCmd {
    fn info() -> &'static ContractInfo {
        &CONTRACT_INFO
    }
    fn handler() -> CommandHandler<CommandResult> {
        CommandHandler::Contextual {
            capabilities:
                codewhale_command_contract::handler::CommandCapabilities::SESSION_LIFECYCLE,
            handler: save_contextual,
        }
    }
}

pub(in crate::commands) fn save_contextual(
    contexts: CommandContexts<'_>,
    arg: Option<&str>,
) -> CommandResult {
    let mut parts = contexts.into_parts();
    let Some(lifecycle) = parts.lifecycle.as_deref_mut() else {
        return CommandResult::error(
            "Command capability unavailable: session_lifecycle".to_string(),
        );
    };
    save_portable(lifecycle, arg)
}

pub(in crate::commands) fn save_portable(
    lifecycle: &mut dyn CommandSessionLifecycleContext,
    arg: Option<&str>,
) -> CommandResult {
    let explicit = arg
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(str::to_string);
    match lifecycle.save_session(explicit) {
        Ok(receipt) => CommandResult::message(format!(
            "Session saved to {} (ID: {})",
            receipt.display_path, receipt.truncated_id
        )),
        Err(error) => CommandResult::error(error),
    }
}
