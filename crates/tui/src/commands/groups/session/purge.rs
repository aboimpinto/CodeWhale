//! `/purge` command — trigger agent-driven context purging.

use crate::tui::app::{App, AppAction};

use super::CommandResult;

/// Trigger agent-driven context purging.
pub fn purge(_app: &mut App) -> CommandResult {
    CommandResult::with_message_and_action(
        "Agent context purge triggered...".to_string(),
        AppAction::PurgeContext,
    )
}
