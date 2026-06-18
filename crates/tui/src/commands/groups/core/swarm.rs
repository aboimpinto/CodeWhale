//! `/swarm` command — WhaleFlow-backed multi-agent swarm.

use crate::commands::traits::{CommandInfo, RegisterCommand};
use crate::localization::MessageId;
use crate::tui::app::{App, AppAction};

use super::CommandResult;

pub(in crate::commands) const COMMAND_INFO: CommandInfo = CommandInfo {
    name: "swarm",
    aliases: &["fanout", "qun"],
    usage: "/swarm [N] <task>",
    description_id: MessageId::CmdSwarmDescription,
};

pub(in crate::commands) struct SwarmCmd;

impl RegisterCommand for SwarmCmd {
    fn info() -> &'static CommandInfo {
        &COMMAND_INFO
    }

    fn execute(app: &mut App, arg: Option<&str>) -> CommandResult {
        swarm(app, arg)
    }
}

/// Run a WhaleFlow-backed multi-agent swarm: high-fanout headless sub-agents
/// over one task. This is an overlay on the current mode (Agent/Plan/YOLO), not
/// a fourth mode — it instructs the model to decompose and fan out, collecting
/// compact result summaries rather than child transcripts (#3178).
pub fn swarm(_app: &mut App, arg: Option<&str>) -> CommandResult {
    let (max_depth, task) = match super::util::parse_depth_prefixed_arg(arg, 1) {
        Ok(parsed) => parsed,
        Err(message) => return CommandResult::error(message),
    };
    let task = match task {
        Some(task) if !task.trim().is_empty() => task.trim().to_string(),
        _ => {
            return CommandResult::error(
                "Usage: /swarm [N] <task>\n\n\
                 Runs a multi-agent swarm: decomposes the task and fans out \
                 headless sub-agents (recursive depth N, 0-3, default 1), then \
                 synthesizes their results.",
            );
        }
    };
    let message = format!(
        "Run a multi-agent swarm for this task: {task:?}. Decompose it into independent, parallelizable subtasks and open one headless sub-agent per subtask with `agent_open` (pass `max_depth: {max_depth}` for nested delegation, and an `agent_type`/role that fits each subtask — explore for research, review for verification, implementer for edits). Run them concurrently; poll each worker with nonblocking `agent_eval`, synthesize results as they arrive, and pass `block:true` only for a deliberate final wait. Keep the fanout proportional to the task, and verify any claimed side effects before reporting success."
    );
    CommandResult::with_message_and_action(
        format!("Dispatching a swarm at depth {max_depth}..."),
        AppAction::SendMessage(message),
    )
}
