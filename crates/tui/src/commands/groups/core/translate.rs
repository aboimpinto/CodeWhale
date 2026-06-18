//! `/translate` command — toggle output translation.

use crate::commands::traits::{CommandInfo, RegisterCommand};
use crate::localization::{MessageId, tr};

use super::CommandResult;
use crate::tui::app::App;

pub(in crate::commands) const COMMAND_INFO: CommandInfo = CommandInfo {
    name: "translate",
    aliases: &["translation", "transale"],
    usage: "/translate",
    description_id: MessageId::CmdTranslateDescription,
};

/// Handler wrapper for FunctionCommand registration.
fn run(app: &mut App, _arg: Option<&str>) -> CommandResult {
    translate(app)
}

pub(in crate::commands) struct TranslateCmd;

impl RegisterCommand for TranslateCmd {
    fn info() -> &'static CommandInfo {
        &COMMAND_INFO
    }

    fn execute(app: &mut App, arg: Option<&str>) -> CommandResult {
        run(app, arg)
    }
}

/// Toggle output translation to the current system language on/off.
///
/// When enabled, the model is instructed to respond in the current locale and an
/// interception layer translates any remaining English output before it
/// reaches the user.
pub fn translate(app: &mut App) -> CommandResult {
    app.translation_enabled = !app.translation_enabled;
    let locale = app.ui_locale;
    if app.translation_enabled {
        CommandResult::message(tr(locale, MessageId::CmdTranslateOn))
    } else {
        CommandResult::message(tr(locale, MessageId::CmdTranslateOff))
    }
}
