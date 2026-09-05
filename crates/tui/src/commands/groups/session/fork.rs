//! `/fork` command — interactive picker (#576) + direct fork.

use codewhale_command_contract::facets::CommandSessionLifecycleContext;
use codewhale_command_contract::handler::{CommandContexts, CommandHandler};
use codewhale_command_contract::metadata::{
    CommandInfo as ContractInfo, RegisterCommand as ContractRegisterCommand,
};

use crate::commands::traits::{CommandInfo, RegisterCommand};
use crate::localization::MessageId;
use crate::tui::app::App;
use crate::tui::session_picker::SessionPickerView;

use super::CommandResult;

pub(in crate::commands) const COMMAND_INFO: CommandInfo = CommandInfo {
    name: "fork",
    aliases: &["f"],
    usage: "/fork [session_id|picker]",
    description_id: MessageId::CmdForkDescription,
};

pub(in crate::commands) struct ForkCmd;

impl RegisterCommand for ForkCmd {
    fn info() -> &'static CommandInfo {
        &COMMAND_INFO
    }

    fn execute(app: &mut App, arg: Option<&str>) -> CommandResult {
        let trimmed = arg.map(str::trim).filter(|s| !s.is_empty());
        if let Some(a) = trimmed {
            if matches!(
                a.to_ascii_lowercase().as_str(),
                "picker" | "list" | "--picker" | "pick"
            ) {
                app.view_stack
                    .push(SessionPickerView::new(&app.workspace, app.ui_locale));
                return CommandResult::message(
                    "Fork picker: select a session and then run `/fork <id>` to fork it.",
                );
            }
            return super::session::fork_from_session(app, a);
        }
        super::session::fork(app)
    }
}

// ---------------------------------------------------------------------------
// FEAT-023 Phase 4 (D3/D5/D6): portable contextual registration and handler.
// The handler owns parsing, branch order, exact messages, guidance appends,
// and action composition; all concrete host work stays behind the lifecycle
// facet. Missing lifecycle authority fails safely with the exact capability
// error (never a panic).
// ---------------------------------------------------------------------------

pub(in crate::commands) const CONTRACT_INFO: ContractInfo = ContractInfo {
    name: "fork",
    aliases: &["f"],
    usage: "/fork [session_id|picker]",
    description_key: "cmd_fork_description",
};

impl ContractRegisterCommand<CommandResult> for ForkCmd {
    fn info() -> &'static ContractInfo {
        &CONTRACT_INFO
    }
    fn handler() -> CommandHandler<CommandResult> {
        CommandHandler::Contextual {
            capabilities:
                codewhale_command_contract::handler::CommandCapabilities::SESSION_LIFECYCLE,
            handler: fork_contextual,
        }
    }
}

pub(in crate::commands) fn fork_contextual(
    contexts: CommandContexts<'_>,
    arg: Option<&str>,
) -> CommandResult {
    let mut parts = contexts.into_parts();
    let Some(lifecycle) = parts.lifecycle.as_deref_mut() else {
        return CommandResult::error(
            "Command capability unavailable: session_lifecycle".to_string(),
        );
    };
    fork_portable(lifecycle, arg)
}

pub(in crate::commands) fn fork_portable(
    lifecycle: &mut dyn CommandSessionLifecycleContext,
    arg: Option<&str>,
) -> CommandResult {
    let trimmed = arg.map(str::trim).filter(|s| !s.is_empty());
    if let Some(a) = trimmed {
        if matches!(
            a.to_ascii_lowercase().as_str(),
            "picker" | "list" | "--picker" | "pick"
        ) {
            lifecycle.open_picker(None);
            return CommandResult::message(
                "Fork picker: select a session and then run `/fork <id>` to fork it.".to_string(),
            );
        }
        if lifecycle.transition_blocked() {
            return CommandResult::error(
                "Cannot fork a session while runtime work is active. Wait for the current turn, maintenance, and background tasks to finish, or cancel that specific work first."
                    .to_string(),
            );
        }
        return match lifecycle.fork_from(a) {
            Ok(receipt) => CommandResult::with_message_and_action(
                format!(
                    "Forked session {} -> {} (spawn_depth {})",
                    receipt.parent_label,
                    receipt.fork_label,
                    receipt.spawn_depth.unwrap_or_default()
                ),
                super::sync_session_action(receipt.sync),
            ),
            Err(error) => CommandResult::error(error),
        };
    }
    if lifecycle.transition_blocked() {
        return CommandResult::error(
            "Cannot fork a session while runtime work is active. Wait for the current turn, maintenance, and background tasks to finish, or cancel that specific work first."
                .to_string(),
        );
    }
    match lifecycle.fork_active() {
        Ok(receipt) => CommandResult::with_message_and_action(
            format!(
                "Forked session {} -> {}",
                receipt.parent_label, receipt.fork_label
            ),
            super::sync_session_action(receipt.sync),
        ),
        Err(error) => CommandResult::error(error),
    }
}
