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
use crate::commands::traits::{Command, CommandGroup, FunctionCommand, RegisterCommand};

pub struct SessionCommands;

impl CommandGroup for SessionCommands {
    fn commands(&self) -> Vec<Box<dyn Command>> {
        vec![
            Box::new(FunctionCommand::new(
                rename::RenameCmd::info(),
                rename::RenameCmd::execute,
            )),
            Box::new(FunctionCommand::new(
                save::SaveCmd::info(),
                save::SaveCmd::execute,
            )),
            Box::new(FunctionCommand::new(
                fork::ForkCmd::info(),
                fork::ForkCmd::execute,
            )),
            Box::new(FunctionCommand::new(
                new::NewCmd::info(),
                new::NewCmd::execute,
            )),
            Box::new(FunctionCommand::new(
                sessions::SessionsCmd::info(),
                sessions::SessionsCmd::execute,
            )),
            Box::new(FunctionCommand::new(
                load::LoadCmd::info(),
                load::LoadCmd::execute,
            )),
            Box::new(FunctionCommand::new(
                compact::CompactCmd::info(),
                compact::CompactCmd::execute,
            )),
            Box::new(FunctionCommand::new(
                purge::PurgeCmd::info(),
                purge::PurgeCmd::execute,
            )),
            Box::new(FunctionCommand::new(
                relay::RelayCmd::info(),
                relay::RelayCmd::execute,
            )),
            Box::new(FunctionCommand::new(
                export::ExportCmd::info(),
                export::ExportCmd::execute,
            )),
        ]
    }
}
