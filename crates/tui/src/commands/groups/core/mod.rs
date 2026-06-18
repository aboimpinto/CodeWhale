//! Core command area: model/provider selection, help, navigation, and the
//! persistent RLM / sub-agent entry points.

mod anchor;
// This group dir contains `mod core;` while a `core.rs` submodule exists with the same
// name — a standard Rust structural pattern. `#[allow(clippy::module_inception)]` is a
// permanent attribute, not migration scaffolding.
// See FEAT-003 planning-analysis-report.md (candidate D.1) for rationale.
#[allow(clippy::module_inception)]
mod core;
#[cfg(all(test, feature = "long-running-tests"))]
mod acceptance;
mod agent;
mod clear;
mod exit;
mod feedback;
mod help;
mod hf;
mod home;
mod hooks;
mod links;
mod model;
mod models;
mod profile;
mod provider;
mod queue;
mod rlm;
mod stash;
mod subagents;
mod swarm;
mod translate;
pub mod util;
pub mod voice;
mod workspace;

use crate::commands::CommandResult;
use crate::commands::traits::{Command, CommandGroup, FunctionCommand, RegisterCommand};

pub struct CoreCommands;

impl CommandGroup for CoreCommands {
    fn commands(&self) -> Vec<Box<dyn Command>> {
        vec![
            Box::new(FunctionCommand::new(anchor::AnchorCmd::info(), anchor::AnchorCmd::execute)),
            Box::new(FunctionCommand::new(help::HelpCmd::info(), help::HelpCmd::execute)),
            Box::new(FunctionCommand::new(clear::ClearCmd::info(), clear::ClearCmd::execute)),
            Box::new(FunctionCommand::new(exit::ExitCmd::info(), exit::ExitCmd::execute)),
            Box::new(FunctionCommand::new(model::ModelCmd::info(), model::ModelCmd::execute)),
            Box::new(FunctionCommand::new(models::ModelsCmd::info(), models::ModelsCmd::execute)),
            Box::new(FunctionCommand::new(provider::ProviderCmd::info(), provider::ProviderCmd::execute)),
            Box::new(FunctionCommand::new(queue::QueueCmd::info(), queue::QueueCmd::execute)),
            Box::new(FunctionCommand::new(stash::StashCmd::info(), stash::StashCmd::execute)),
            Box::new(FunctionCommand::new(hooks::HooksCmd::info(), hooks::HooksCmd::execute)),
            Box::new(FunctionCommand::new(subagents::SubagentsCmd::info(), subagents::SubagentsCmd::execute)),
            Box::new(FunctionCommand::new(agent::AgentCmd::info(), agent::AgentCmd::execute)),
            Box::new(FunctionCommand::new(swarm::SwarmCmd::info(), swarm::SwarmCmd::execute)),
            Box::new(FunctionCommand::new(links::LinksCmd::info(), links::LinksCmd::execute)),
            Box::new(FunctionCommand::new(feedback::FeedbackCmd::info(), feedback::FeedbackCmd::execute)),
            Box::new(FunctionCommand::new(hf::HfCmd::info(), hf::HfCmd::execute)),
            Box::new(FunctionCommand::new(home::HomeCmd::info(), home::HomeCmd::execute)),
            Box::new(FunctionCommand::new(workspace::WorkspaceCmd::info(), workspace::WorkspaceCmd::execute)),
            Box::new(FunctionCommand::new(profile::ProfileCmd::info(), profile::ProfileCmd::execute)),
            Box::new(FunctionCommand::new(rlm::RlmCmd::info(), rlm::RlmCmd::execute)),
            Box::new(FunctionCommand::new(translate::TranslateCmd::info(), translate::TranslateCmd::execute)),
            Box::new(FunctionCommand::new(voice::VoiceCmd::info(), voice::VoiceCmd::execute)),
            Box::new(FunctionCommand::new(voice::VoiceSendCmd::info(), voice::VoiceSendCmd::execute)),
            Box::new(FunctionCommand::new(voice::VoiceControlCmd::info(), voice::VoiceControlCmd::execute)),
        ]
    }
}
