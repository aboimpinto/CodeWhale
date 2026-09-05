use codewhale_command_contract::facets::CommandSessionLifecycleContext;
use codewhale_command_contract::handler::{CommandContexts, CommandHandler};
use codewhale_command_contract::metadata::{
    CommandInfo as ContractInfo, RegisterCommand as ContractRegisterCommand,
};

use super::CommandResult;
use crate::commands::traits::{CommandInfo, RegisterCommand};
use crate::localization::MessageId;
use crate::tui::app::App;
pub(in crate::commands) const COMMAND_INFO: CommandInfo = CommandInfo {
    name: "branch",
    aliases: &[],
    usage: "/branch <entry_id>",
    description_id: MessageId::CmdBranchDescription,
};
pub(in crate::commands) struct BranchCmd;
impl RegisterCommand for BranchCmd {
    fn info() -> &'static CommandInfo {
        &COMMAND_INFO
    }
    fn execute(app: &mut App, arg: Option<&str>) -> CommandResult {
        branch(app, arg)
    }
}
fn branch(app: &mut App, arg: Option<&str>) -> CommandResult {
    if app.session_transition_blocked() {
        return CommandResult::error(
            "Cannot branch while runtime work is active. Wait for the turn to finish, or cancel it first.",
        );
    }
    let Some(entry_id) = arg.map(str::trim).filter(|s| !s.is_empty()) else {
        if let Some(session_id) = app.current_session_id.as_deref()
            && let Ok(manager) = crate::session_manager::SessionManager::default_location()
            && let Ok(mut session) = manager.load_session(session_id)
        {
            session.ensure_journal();
            if let Some(journal) = session.journal.as_ref()
                && let Some(leaf) = journal.leaf_id.as_deref()
            {
                return CommandResult::message(format!(
                    "Current leaf: {leaf}\nUse `/branch <entry_id>` to move the leaf (history is never rewritten).\nUse `/tree` to list entry ids."
                ));
            }
        }
        return CommandResult::message(
            "Usage: /branch <entry_id>\nMoves the active leaf to an existing entry. Future appends become children of that entry.\nHistory is never rewritten — branching only moves the leaf.\n\nUse `/tree` to see entry ids.",
        );
    };
    let session_id = match app.current_session_id.clone() {
        Some(id) => id,
        None => {
            return CommandResult::error(
                "No active session to branch. Resume or create a session first.",
            );
        }
    };
    let manager = match crate::session_manager::SessionManager::default_location() {
        Ok(m) => m,
        Err(e) => return CommandResult::error(format!("could not open sessions directory: {e}")),
    };
    let mut session = match manager.load_session(&session_id) {
        Ok(s) => s,
        Err(e) => return CommandResult::error(format!("could not load session {session_id}: {e}")),
    };
    session.ensure_journal();
    let journal_len_before = session
        .journal
        .as_ref()
        .map(|j| j.entries.len())
        .unwrap_or(0);
    match session.journal_branch_to(entry_id) {
        Ok(()) => {
            if let Err(e) = manager.save_session(&session) {
                return CommandResult::error(format!("branch saved but persist failed: {e}"));
            }
            app.api_messages = session.messages.clone();
            let leaf = session
                .leaf_id
                .clone()
                .unwrap_or_else(|| "(none)".to_string());
            let msg = format!(
                "Branched to entry {entry_id} (leaf now {leaf}); journal entries {journal_len_before} (history preserved, leaf moved only)"
            );
            CommandResult::message(msg)
        }
        Err(e) => CommandResult::error(format!(
            "branch failed: {e}. Use `/tree` to see valid entry ids."
        )),
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
    name: "branch",
    aliases: &[],
    usage: "/branch <entry_id>",
    description_key: "cmd_branch_description",
};

impl ContractRegisterCommand<CommandResult> for BranchCmd {
    fn info() -> &'static ContractInfo {
        &CONTRACT_INFO
    }
    fn handler() -> CommandHandler<CommandResult> {
        CommandHandler::Contextual {
            capabilities:
                codewhale_command_contract::handler::CommandCapabilities::SESSION_LIFECYCLE,
            handler: branch_contextual,
        }
    }
}

pub(in crate::commands) fn branch_contextual(
    contexts: CommandContexts<'_>,
    arg: Option<&str>,
) -> CommandResult {
    let mut parts = contexts.into_parts();
    let Some(lifecycle) = parts.lifecycle.as_deref_mut() else {
        return CommandResult::error(
            "Command capability unavailable: session_lifecycle".to_string(),
        );
    };
    branch_portable(lifecycle, arg)
}

pub(in crate::commands) fn branch_portable(
    lifecycle: &mut dyn CommandSessionLifecycleContext,
    arg: Option<&str>,
) -> CommandResult {
    if lifecycle.transition_blocked() {
        return CommandResult::error(
            "Cannot branch while runtime work is active. Wait for the turn to finish, or cancel it first."
                .to_string(),
        );
    }
    let Some(entry_id) = arg.map(str::trim).filter(|s| !s.is_empty()) else {
        if let Some(leaf) = lifecycle.branch_current_leaf_hint() {
            return CommandResult::message(format!(
                "Current leaf: {leaf}\nUse `/branch <entry_id>` to move the leaf (history is never rewritten).\nUse `/tree` to list entry ids."
            ));
        }
        return CommandResult::message(
            "Usage: /branch <entry_id>\nMoves the active leaf to an existing entry. Future appends become children of that entry.\nHistory is never rewritten — branching only moves the leaf.\n\nUse `/tree` to see entry ids."
                .to_string(),
        );
    };
    match lifecycle.branch_to(entry_id) {
        Ok(outcome) => CommandResult::message(format!(
            "Branched to entry {entry_id} (leaf now {}); journal entries {} (history preserved, leaf moved only)",
            outcome.leaf_display, outcome.journal_entries_before
        )),
        Err(error) => CommandResult::error(error),
    }
}
