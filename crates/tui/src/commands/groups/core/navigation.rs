#[path = "hotbar.rs"]
pub mod hotbar;
#[path = "transcript.rs"]
pub mod transcript;
#[path = "turn.rs"]
pub mod turn;

use codewhale_command_contract::handler::CommandHandler;
use codewhale_command_contract::metadata::{CommandInfo, RegisterCommand};
use codewhale_command_contract::outcome::CoreNavigationCommandResult;

pub fn portable_handlers() -> [(
    &'static CommandInfo,
    CommandHandler<CoreNavigationCommandResult>,
); 3] {
    [
        (hotbar::HotbarCmd::info(), hotbar::HotbarCmd::handler()),
        (
            transcript::TranscriptCmd::info(),
            transcript::TranscriptCmd::handler(),
        ),
        (turn::TurnCmd::info(), turn::TurnCmd::handler()),
    ]
}
