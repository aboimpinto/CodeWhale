//! Session command area: saving, forking, resuming, exporting, and the
//! `/relay` session-handoff artifact.

mod compact;
mod export;
mod fork;
mod load;
mod new;
mod purge;
mod relay;
mod rename;
mod save;
mod sessions;
// This group dir contains `mod session;` while a `session.rs` submodule exists with the same
// name — a standard Rust structural pattern. `#[allow(clippy::module_inception)]` is a
// permanent attribute, not migration scaffolding.
// See FEAT-003 planning-analysis-report.md (candidate D.1) for rationale.
#[allow(clippy::module_inception)]
mod session;

use crate::commands::CommandResult;
use crate::commands::traits::{Command, CommandGroup, CommandInfo, FunctionCommand};
use crate::localization::MessageId;
use crate::tui::app::App;

pub struct SessionCommands;

impl CommandGroup for SessionCommands {
    fn commands(&self) -> Vec<Box<dyn Command>> {
        vec![
            Box::new(FunctionCommand::new(&RENAME_INFO, run_rename)),
            Box::new(FunctionCommand::new(&SAVE_INFO, run_save)),
            Box::new(FunctionCommand::new(&FORK_INFO, run_fork)),
            Box::new(FunctionCommand::new(&NEW_INFO, run_new)),
            Box::new(FunctionCommand::new(&SESSIONS_INFO, run_sessions)),
            Box::new(FunctionCommand::new(&LOAD_INFO, run_load)),
            Box::new(FunctionCommand::new(&COMPACT_INFO, run_compact)),
            Box::new(FunctionCommand::new(&PURGE_INFO, run_purge)),
            Box::new(FunctionCommand::new(&RELAY_INFO, run_relay)),
            Box::new(FunctionCommand::new(&EXPORT_INFO, run_export)),
        ]
    }
}

static RENAME_INFO: CommandInfo = CommandInfo {
    name: "rename",
    aliases: &["gaiming", "chongmingming"],
    usage: "/rename <new title>",
    description_id: MessageId::CmdRenameDescription,
};
static SAVE_INFO: CommandInfo = CommandInfo {
    name: "save",
    aliases: &[],
    usage: "/save [path]",
    description_id: MessageId::CmdSaveDescription,
};
static FORK_INFO: CommandInfo = CommandInfo {
    name: "fork",
    aliases: &["branch"],
    usage: "/fork",
    description_id: MessageId::CmdForkDescription,
};
static NEW_INFO: CommandInfo = CommandInfo {
    name: "new",
    aliases: &[],
    usage: "/new [--force]",
    description_id: MessageId::CmdNewDescription,
};
static SESSIONS_INFO: CommandInfo = CommandInfo {
    name: "sessions",
    aliases: &["resume"],
    usage: "/sessions [show|prune <days>]",
    description_id: MessageId::CmdSessionsDescription,
};
static LOAD_INFO: CommandInfo = CommandInfo {
    name: "load",
    aliases: &["jiazai"],
    usage: "/load [path]",
    description_id: MessageId::CmdLoadDescription,
};
static COMPACT_INFO: CommandInfo = CommandInfo {
    name: "compact",
    aliases: &["yasuo"],
    usage: "/compact",
    description_id: MessageId::CmdCompactDescription,
};
static PURGE_INFO: CommandInfo = CommandInfo {
    name: "purge",
    aliases: &["qingchu"],
    usage: "/purge",
    description_id: MessageId::CmdPurgeDescription,
};
static RELAY_INFO: CommandInfo = CommandInfo {
    name: "relay",
    aliases: &["batonpass", "接力"],
    usage: "/relay [focus]",
    description_id: MessageId::CmdRelayDescription,
};
static EXPORT_INFO: CommandInfo = CommandInfo {
    name: "export",
    aliases: &["daochu"],
    usage: "/export [path]",
    description_id: MessageId::CmdExportDescription,
};

fn run_registered(app: &mut App, name: &str, arg: Option<&str>) -> CommandResult {
    dispatch(app, name, arg).expect("registered session command should dispatch")
}

fn run_rename(app: &mut App, arg: Option<&str>) -> CommandResult {
    run_registered(app, "rename", arg)
}
fn run_save(app: &mut App, arg: Option<&str>) -> CommandResult {
    run_registered(app, "save", arg)
}
fn run_fork(app: &mut App, arg: Option<&str>) -> CommandResult {
    run_registered(app, "fork", arg)
}
fn run_new(app: &mut App, arg: Option<&str>) -> CommandResult {
    run_registered(app, "new", arg)
}
fn run_sessions(app: &mut App, arg: Option<&str>) -> CommandResult {
    run_registered(app, "sessions", arg)
}
fn run_load(app: &mut App, arg: Option<&str>) -> CommandResult {
    run_registered(app, "load", arg)
}
fn run_compact(app: &mut App, arg: Option<&str>) -> CommandResult {
    run_registered(app, "compact", arg)
}
fn run_purge(app: &mut App, arg: Option<&str>) -> CommandResult {
    run_registered(app, "purge", arg)
}
fn run_relay(app: &mut App, arg: Option<&str>) -> CommandResult {
    run_registered(app, "relay", arg)
}
fn run_export(app: &mut App, arg: Option<&str>) -> CommandResult {
    run_registered(app, "export", arg)
}

pub(in crate::commands) fn dispatch(
    app: &mut App,
    command: &str,
    arg: Option<&str>,
) -> Option<CommandResult> {
    let result = match command {
        "rename" | "gaiming" | "chongmingming" => rename::rename(app, arg),
        "save" => save::save(app, arg),
        "fork" | "branch" => fork::fork(app),
        "new" => new::new_session(app, arg),
        "sessions" | "resume" => sessions::sessions(app, arg),
        "relay" | "batonpass" | "接力" => relay::relay(app, arg),
        "load" | "jiazai" => load::load(app, arg),
        "compact" | "yasuo" => compact::compact(app),
        "purge" | "qingchu" => purge::purge(app),
        "export" | "daochu" => export::export(app, arg),
        _ => return None,
    };
    Some(result)
}
