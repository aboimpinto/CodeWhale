//! `/models` command — fetch and list available models.

use crate::tui::app::AppAction;

use super::CommandResult;

/// Fetch and list available models from the configured API endpoint.
pub fn models() -> CommandResult {
    CommandResult::action(AppAction::FetchModels)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_models_triggers_fetch_action() {
        let result = models();
        assert!(result.message.is_none());
        assert!(matches!(result.action, Some(AppAction::FetchModels)));
    }
}
