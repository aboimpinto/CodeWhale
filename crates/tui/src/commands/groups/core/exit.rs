//! `/exit` command — quit the application.

use crate::tui::app::AppAction;

use super::CommandResult;

/// Exit the application
pub fn exit() -> CommandResult {
    CommandResult::action(AppAction::Quit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_returns_quit_action() {
        let result = exit();
        assert!(result.message.is_none());
        assert!(matches!(result.action, Some(AppAction::Quit)));
    }
}
