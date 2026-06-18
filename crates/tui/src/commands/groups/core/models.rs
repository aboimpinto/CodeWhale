//! `/models` command — fetch and list available models.

use crate::commands::traits::{CommandInfo, RegisterCommand};
use crate::localization::MessageId;
use crate::tui::app::{App, AppAction};

use super::CommandResult;

pub(in crate::commands) const COMMAND_INFO: CommandInfo = CommandInfo {
    name: "models",
    aliases: &["moxingliebiao"],
    usage: "/models",
    description_id: MessageId::CmdModelsDescription,
};

/// Handler wrapper for FunctionCommand registration.
fn run(_app: &mut App, _arg: Option<&str>) -> CommandResult {
    models()
}

pub(in crate::commands) struct ModelsCmd;

impl RegisterCommand for ModelsCmd {
    fn info() -> &'static CommandInfo {
        &COMMAND_INFO
    }

    fn execute(app: &mut App, arg: Option<&str>) -> CommandResult {
        run(app, arg)
    }
}

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
