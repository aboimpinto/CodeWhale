//! Shared lifecycle primitives for managed command execution.
//!
//! This is intentionally scoped to the current custom slash-command work while
//! leaving stable extension points for future action types such as skills,
//! agents, MCP calls, and plugins. It does not execute MCP/agent behavior.

use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleSource {
    CustomSlashCommand,
    Skill,
    Agent,
    Mcp,
    Plugin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleStatus {
    Idle,
    Running,
    Paused,
    Cancelled,
    Completed,
    Failed,
}

impl LifecycleStatus {
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Cancelled | Self::Completed | Self::Failed)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleTransition {
    Start,
    Pause,
    Resume,
    Cancel,
    Complete,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RollbackStrategy {
    None,
    GitStash { marker: String },
    ExternalCompensation,
}

impl Default for RollbackStrategy {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecyclePolicy {
    pub source: LifecycleSource,
    pub pausable: bool,
    pub allowed_tools: Option<Vec<String>>,
    pub rollback: RollbackStrategy,
}

impl LifecyclePolicy {
    pub fn custom_slash_command(
        allowed_tools: Option<Vec<String>>,
        pausable: bool,
        rollback: RollbackStrategy,
    ) -> Self {
        Self {
            source: LifecycleSource::CustomSlashCommand,
            pausable,
            allowed_tools,
            rollback,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolReceiptStatus {
    Started,
    Succeeded,
    Failed,
    Blocked,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolReceipt {
    pub id: String,
    pub name: String,
    pub input_summary: String,
    pub output_summary: Option<String>,
    pub status: ToolReceiptStatus,
    pub duration_ms: Option<u128>,
}

impl ToolReceipt {
    pub fn started(
        id: impl Into<String>,
        name: impl Into<String>,
        input_summary: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            input_summary: input_summary.into(),
            output_summary: None,
            status: ToolReceiptStatus::Started,
            duration_ms: None,
        }
    }

    pub fn settle(
        &mut self,
        status: ToolReceiptStatus,
        output_summary: impl Into<String>,
        duration: Option<Duration>,
    ) {
        self.status = status;
        self.output_summary = Some(output_summary.into());
        self.duration_ms = duration.map(|d| d.as_millis());
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandLifecycle {
    pub work_id: String,
    pub title: String,
    pub status: LifecycleStatus,
    pub policy: LifecyclePolicy,
    pub receipts: Vec<ToolReceipt>,
}

impl CommandLifecycle {
    pub fn start_custom_slash_command(
        command_name: impl Into<String>,
        title: impl Into<String>,
        allowed_tools: Option<Vec<String>>,
        pausable: bool,
        rollback: RollbackStrategy,
    ) -> Self {
        Self {
            work_id: format!("slash:{}", command_name.into()),
            title: title.into(),
            status: LifecycleStatus::Running,
            policy: LifecyclePolicy::custom_slash_command(allowed_tools, pausable, rollback),
            receipts: Vec::new(),
        }
    }

    pub fn apply(&mut self, transition: LifecycleTransition) {
        self.status = next_status(self.status, transition);
    }

    pub fn record_tool_started(
        &mut self,
        id: impl Into<String>,
        name: impl Into<String>,
        input_summary: impl Into<String>,
    ) {
        let id = id.into();
        if let Some(existing) = self.receipts.iter_mut().find(|receipt| receipt.id == id) {
            *existing = ToolReceipt::started(id, name, input_summary);
            return;
        }
        self.receipts
            .push(ToolReceipt::started(id, name, input_summary));
    }

    pub fn record_tool_complete(
        &mut self,
        id: &str,
        name: impl Into<String>,
        input_summary: impl Into<String>,
        status: ToolReceiptStatus,
        output_summary: impl Into<String>,
        duration: Option<Duration>,
    ) {
        if let Some(existing) = self.receipts.iter_mut().find(|receipt| receipt.id == id) {
            existing.settle(status, output_summary, duration);
            return;
        }
        let mut receipt = ToolReceipt::started(id.to_string(), name, input_summary);
        receipt.settle(status, output_summary, duration);
        self.receipts.push(receipt);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreToolUseDecision {
    Allow,
    Deny { message: String },
}

impl PreToolUseDecision {
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allow)
    }
}

/// Internal PreToolUse policy check for command-scoped tool permissions.
///
/// This deliberately accepts a generic allowed-tools slice so future action
/// types can reuse it without depending on custom slash-command frontmatter.
pub fn check_allowed_tools(
    tool_name: &str,
    allowed_tools: Option<&[String]>,
) -> PreToolUseDecision {
    let Some(allowed_tools) = allowed_tools else {
        return PreToolUseDecision::Allow;
    };
    if allowed_tools
        .iter()
        .any(|allowed| allowed.eq_ignore_ascii_case(tool_name))
    {
        return PreToolUseDecision::Allow;
    }
    PreToolUseDecision::Deny {
        message: format!(
            "Tool '{tool_name}' is not in the allowed-tools list for the current command"
        ),
    }
}

pub fn next_status(current: LifecycleStatus, transition: LifecycleTransition) -> LifecycleStatus {
    use LifecycleStatus as S;
    use LifecycleTransition as T;

    match (current, transition) {
        (_, T::Start) => S::Running,
        (S::Running, T::Pause) => S::Paused,
        (S::Paused, T::Resume) => S::Running,
        (S::Running | S::Paused, T::Cancel) => S::Cancelled,
        (S::Running, T::Complete) => S::Completed,
        (S::Running | S::Paused, T::Fail) => S::Failed,
        (terminal, _) if terminal.is_terminal() => terminal,
        (state, _) => state,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_state_machine_covers_expected_custom_command_path() {
        let mut lifecycle = CommandLifecycle::start_custom_slash_command(
            "git-scan",
            "Scan nested git repositories",
            None,
            true,
            RollbackStrategy::GitStash {
                marker: "codewhale-pausable".to_string(),
            },
        );

        assert_eq!(lifecycle.status, LifecycleStatus::Running);
        lifecycle.apply(LifecycleTransition::Pause);
        assert_eq!(lifecycle.status, LifecycleStatus::Paused);
        lifecycle.apply(LifecycleTransition::Resume);
        assert_eq!(lifecycle.status, LifecycleStatus::Running);
        lifecycle.apply(LifecycleTransition::Complete);
        assert_eq!(lifecycle.status, LifecycleStatus::Completed);
    }

    #[test]
    fn cancelled_state_is_terminal_when_late_completion_arrives() {
        let mut lifecycle = CommandLifecycle::start_custom_slash_command(
            "git-scan",
            "Scan nested git repositories",
            None,
            true,
            RollbackStrategy::None,
        );

        lifecycle.apply(LifecycleTransition::Pause);
        lifecycle.apply(LifecycleTransition::Cancel);
        lifecycle.apply(LifecycleTransition::Complete);

        assert_eq!(lifecycle.status, LifecycleStatus::Cancelled);
    }

    #[test]
    fn completed_state_is_terminal_when_late_cancel_arrives() {
        let mut lifecycle = CommandLifecycle::start_custom_slash_command(
            "git-scan",
            "Scan nested git repositories",
            None,
            true,
            RollbackStrategy::None,
        );

        lifecycle.apply(LifecycleTransition::Complete);
        lifecycle.apply(LifecycleTransition::Cancel);

        assert_eq!(lifecycle.status, LifecycleStatus::Completed);
    }

    #[test]
    fn allowed_tools_check_allows_missing_policy() {
        assert!(check_allowed_tools("exec_shell", None).is_allowed());
    }

    #[test]
    fn allowed_tools_check_is_case_insensitive() {
        let allowed = vec!["Exec_Shell".to_string(), "read_file".to_string()];
        assert!(check_allowed_tools("exec_shell", Some(&allowed)).is_allowed());
    }

    #[test]
    fn allowed_tools_check_denies_unlisted_tool() {
        let allowed = vec!["read_file".to_string()];
        let decision = check_allowed_tools("exec_shell", Some(&allowed));

        assert_eq!(
            decision,
            PreToolUseDecision::Deny {
                message:
                    "Tool 'exec_shell' is not in the allowed-tools list for the current command"
                        .to_string()
            }
        );
    }

    #[test]
    fn tool_receipts_start_and_settle_in_place() {
        let mut lifecycle = CommandLifecycle::start_custom_slash_command(
            "git-scan",
            "Scan nested git repositories",
            Some(vec!["exec_shell".to_string()]),
            true,
            RollbackStrategy::None,
        );

        lifecycle.record_tool_started("tool-1", "exec_shell", "{\"command\":\"git status\"}");
        lifecycle.record_tool_complete(
            "tool-1",
            "exec_shell",
            "{}",
            ToolReceiptStatus::Succeeded,
            "clean",
            Some(Duration::from_millis(42)),
        );

        assert_eq!(lifecycle.receipts.len(), 1);
        let receipt = &lifecycle.receipts[0];
        assert_eq!(receipt.id, "tool-1");
        assert_eq!(receipt.name, "exec_shell");
        assert_eq!(receipt.status, ToolReceiptStatus::Succeeded);
        assert_eq!(receipt.output_summary.as_deref(), Some("clean"));
        assert_eq!(receipt.duration_ms, Some(42));
    }

    #[test]
    fn lifecycle_policy_has_future_action_type_slots_without_enabling_them() {
        let source_types = [
            LifecycleSource::CustomSlashCommand,
            LifecycleSource::Skill,
            LifecycleSource::Agent,
            LifecycleSource::Mcp,
            LifecycleSource::Plugin,
        ];

        assert_eq!(source_types.len(), 5);
    }
}
