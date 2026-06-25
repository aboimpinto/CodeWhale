//! Debug command area: token/cost introspection, cache tooling, undo/retry,
//! and the change log.

mod balance;
mod change;
// This group dir intentionally has a `debug.rs` child module with the same
// name. The module_inception allow is a permanent structure rationale, not
// migration scaffolding; see docs/architecture/command-dispatch.md.
#[allow(clippy::module_inception)]
mod debug;
mod cmd_cache;
mod cmd_context;
mod cmd_cost;
mod cmd_diff;
mod cmd_edit;
mod cmd_retry;
mod cmd_system;
mod cmd_tokens;
mod cmd_undo;

use crate::commands::CommandResult;
use crate::commands::traits::{Command, CommandGroup, FunctionCommand, RegisterCommand};
use crate::tui::app::App;

use self::balance::BalanceCmd;
use self::change::ChangeCmd;
use self::cmd_cache::CacheCmd;
use self::cmd_context::ContextCmd;
use self::cmd_cost::CostCmd;
use self::cmd_diff::DiffCmd;
use self::cmd_edit::EditCmd;
use self::cmd_retry::RetryCmd;
use self::cmd_system::SystemCmd;
use self::cmd_tokens::TokensCmd;
use self::cmd_undo::UndoCmd;

pub struct DebugCommands;

impl CommandGroup for DebugCommands {
    fn commands(&self) -> Vec<Box<dyn Command>> {
        vec![
            Box::new(FunctionCommand::new(TokensCmd::info(), TokensCmd::execute)),
            Box::new(FunctionCommand::new(CostCmd::info(), CostCmd::execute)),
            Box::new(FunctionCommand::new(BalanceCmd::info(), BalanceCmd::execute)),
            Box::new(FunctionCommand::new(CacheCmd::info(), CacheCmd::execute)),
            Box::new(FunctionCommand::new(ChangeCmd::info(), ChangeCmd::execute)),
            Box::new(FunctionCommand::new(SystemCmd::info(), SystemCmd::execute)),
            Box::new(FunctionCommand::new(ContextCmd::info(), ContextCmd::execute)),
            Box::new(FunctionCommand::new(EditCmd::info(), EditCmd::execute)),
            Box::new(FunctionCommand::new(DiffCmd::info(), DiffCmd::execute)),
            Box::new(FunctionCommand::new(UndoCmd::info(), UndoCmd::execute)),
            Box::new(FunctionCommand::new(RetryCmd::info(), RetryCmd::execute)),
        ]
    }
}

#[allow(dead_code)]
pub(in crate::commands) fn dispatch(
    app: &mut App,
    command: &str,
    arg: Option<&str>,
) -> Option<CommandResult> {
    let result = match command {
        "tokens" => debug::tokens(app),
        "cost" => debug::cost(app),
        "balance" => balance::balance(app),
        "cache" => debug::cache(app, arg),
        "change" => change::change(app, arg),
        "system" | "xitong" => debug::system_prompt(app),
        "context" | "ctx" => debug::context(app, arg),
        "edit" => debug::edit(app),
        "diff" => debug::diff(app),
        "undo" => {
            // Try surgical patch-undo first; fall back to conversation undo
            // if no snapshots are available or if the snapshot undo couldn't
            // find anything useful.
            let result = debug::patch_undo(app);
            if result.message.as_deref().is_none_or(|m| {
                m.starts_with("No snapshots found")
                    || m.starts_with("No older tool or pre-turn")
                    || m.starts_with("Snapshot repo")
            }) {
                debug::undo_conversation(app)
            } else {
                result
            }
        }
        "retry" | "chongshi" => debug::retry(app),
        _ => return None,
    };
    Some(result)
}
