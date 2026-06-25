//! `/undo` command — undo the last tool call or conversation turn.

use crate::commands::traits::{CommandInfo, RegisterCommand};
use crate::localization::MessageId;
use crate::tui::app::App;

use super::CommandResult;

pub(in crate::commands) const COMMAND_INFO: CommandInfo = CommandInfo {
    name: "undo",
    aliases: &[],
    usage: "/undo",
    description_id: MessageId::CmdUndoDescription,
};

pub(in crate::commands) struct UndoCmd;

impl RegisterCommand for UndoCmd {
    fn info() -> &'static CommandInfo {
        &COMMAND_INFO
    }

    fn execute(app: &mut App, _arg: Option<&str>) -> CommandResult {
        // Try surgical patch-undo first; fall back to conversation undo
        // if no snapshots are available or if the snapshot undo couldn't
        // find anything useful.
        let result = super::debug::patch_undo(app);
        if result.message.as_deref().is_none_or(|m| {
            m.starts_with("No snapshots found")
                || m.starts_with("No older tool or pre-turn")
                || m.starts_with("Snapshot repo")
        }) {
            super::debug::undo_conversation(app)
        } else {
            result
        }
    }
}
