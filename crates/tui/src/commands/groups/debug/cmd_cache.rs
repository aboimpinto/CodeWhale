//! `/cache` command — inspect and manage the turn cache.

use crate::commands::traits::{CommandInfo, RegisterCommand};
use crate::localization::MessageId;
use crate::tui::app::App;

use super::CommandResult;

pub(in crate::commands) const COMMAND_INFO: CommandInfo = CommandInfo {
    name: "cache",
    aliases: &[],
    usage: "/cache [count|inspect|stats|zones|warmup]",
    description_id: MessageId::CmdCacheDescription,
};

pub(in crate::commands) struct CacheCmd;

impl RegisterCommand for CacheCmd {
    fn info() -> &'static CommandInfo {
        &COMMAND_INFO
    }

    fn execute(app: &mut App, arg: Option<&str>) -> CommandResult {
        super::debug::cache(app, arg)
    }
}
