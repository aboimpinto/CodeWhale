//! `/rename` command — portable handler over the session-control facet.
//!
//! The handler owns argument trimming and the usage boundary; sanitization,
//! the 100-character limit (applied to the sanitized title as in the
//! baseline), first-snapshot recovery, persistence, and publication stay in
//! the atomic `rename_session` delegate so ordering cannot drift.

use super::CommandResult;
use codewhale_command_contract::facets::CommandSessionControlContext;
use codewhale_command_contract::handler::{CommandContexts, CommandHandler};
use codewhale_command_contract::metadata::{
    CommandInfo as ContractInfo, RegisterCommand as ContractRegisterCommand,
};

pub(in crate::commands) struct RenameCmd;

pub(in crate::commands) const CONTRACT_INFO: ContractInfo = ContractInfo {
    name: "rename",
    aliases: &["gaiming", "chongmingming"],
    usage: "/rename <new title>",
    description_key: "cmd_rename_description",
};

impl ContractRegisterCommand<CommandResult> for RenameCmd {
    fn info() -> &'static ContractInfo {
        &CONTRACT_INFO
    }
    fn handler() -> CommandHandler<CommandResult> {
        CommandHandler::Contextual {
            capabilities: codewhale_command_contract::handler::CommandCapabilities::SESSION_CONTROL,
            handler: rename_contextual,
        }
    }
}

pub(in crate::commands) fn rename_contextual(
    contexts: CommandContexts<'_>,
    arg: Option<&str>,
) -> CommandResult {
    let mut parts = contexts.into_parts();
    let Some(control) = parts.control.as_deref_mut() else {
        return CommandResult::error("Command capability unavailable: session_control".to_string());
    };
    rename_portable(control, arg)
}

pub(in crate::commands) fn rename_portable(
    control: &mut dyn CommandSessionControlContext,
    arg: Option<&str>,
) -> CommandResult {
    let Some(raw) = arg.map(str::trim).filter(|s| !s.is_empty()) else {
        return CommandResult::error("Usage: /rename <new title>");
    };
    match control.rename_session(raw) {
        Ok(receipt) => CommandResult::message(format!("Session renamed to \"{}\"", receipt.title)),
        Err(error) => CommandResult::error(error),
    }
}

#[cfg(test)]
mod tests {
    /// Message with the canonical "Error: " prefix removed so exact strings
    /// compare against the baseline text.
    fn message(result: &super::CommandResult) -> &str {
        result
            .message
            .as_deref()
            .map(|m| m.strip_prefix("Error: ").unwrap_or(m))
            .unwrap_or("")
    }

    use super::*;
    use codewhale_command_contract::facets::SessionTitleReceipt;

    fn control_fake() -> super::super::control_test_support::FakeControl {
        super::super::control_test_support::FakeControl::default()
    }

    #[test]
    fn rename_usage_boundaries_and_success_messages_are_exact() {
        let mut no_arg = control_fake();
        let result = rename_portable(&mut no_arg, None);
        assert!(result.is_error);
        assert_eq!(message(&result), "Usage: /rename <new title>");
        assert!(no_arg.calls.is_empty(), "no delegate call for blank input");

        let mut blank = control_fake();
        let result = rename_portable(&mut blank, Some("   "));
        assert!(result.is_error);
        assert_eq!(message(&result), "Usage: /rename <new title>");

        let mut ok = control_fake();
        ok.rename = Some(Ok(SessionTitleReceipt {
            title: "New Name".to_string(),
        }));
        let result = rename_portable(&mut ok, Some("New Name"));
        assert!(!result.is_error);
        assert_eq!(message(&result), "Session renamed to \"New Name\"");
        assert_eq!(ok.calls, vec!["rename_session(New Name)".to_string()]);
        assert!(result.action.is_none(), "/rename emits no action");
    }

    #[test]
    fn rename_host_errors_pass_through_exactly() {
        let mut fake = control_fake();
        fake.rename = Some(Err("Could not save session: boom".to_string()));
        let result = rename_portable(&mut fake, Some("Whatever"));
        assert!(result.is_error);
        assert_eq!(message(&result), "Could not save session: boom");
    }

    #[test]
    fn rename_missing_control_authority_fails_safely() {
        let contexts = codewhale_command_contract::handler::CommandContexts::empty();
        let result = rename_contextual(contexts, None);
        assert!(result.is_error);
        assert_eq!(
            message(&result),
            "Command capability unavailable: session_control"
        );
    }
}
