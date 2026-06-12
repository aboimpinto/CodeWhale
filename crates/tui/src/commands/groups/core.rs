//! Core command group dispatch.
//!
//! Commands: anchor, help, clear, exit, model, models, provider, queue,
//! stash, hooks, subagents, agent, links, feedback, hf, home, workspace,
//! attach, task, jobs, mcp, network, rlm, profile

use crate::commands::{
    CommandResult, agent, anchor, attachment, core, feedback, hf, hooks, jobs, mcp, network,
    provider, queue, rlm, stash, task,
};
use crate::tui::app::App;

/// Dispatch a core-group command.
///
/// Returns `None` if the command is not recognised as a core command.
pub fn dispatch(command: &str, arg: Option<&str>, app: &mut App) -> Option<CommandResult> {
    match command {
        "anchor" | "maodian" => Some(anchor::anchor(app, arg)),
        "help" | "?" | "bangzhu" | "帮助" => Some(core::help(app, arg)),
        "clear" | "qingping" => Some(core::clear(app)),
        "exit" | "quit" | "q" | "tuichu" => Some(core::exit()),
        "model" | "moxing" => Some(core::model(app, arg)),
        "models" | "moxingliebiao" => Some(core::models(app)),
        "provider" => Some(provider::provider(app, arg)),
        "queue" | "queued" => Some(queue::queue(app, arg)),
        "stash" | "park" => Some(stash::stash(app, arg)),
        "hooks" | "hook" | "gouzi" => Some(hooks::hooks(app, arg)),
        "subagents" | "agents" | "zhinengti" => Some(core::subagents(app)),
        "agent" | "daili" => Some(agent(app, arg)),
        "links" | "dashboard" | "api" | "lianjie" => Some(core::deepseek_links(app)),
        "feedback" => Some(feedback::feedback(app, arg)),
        "hf" | "huggingface" => Some(hf::hf(app, arg)),
        "home" | "stats" | "overview" | "zhuye" | "shouye" => Some(core::home_dashboard(app)),
        "workspace" | "cwd" => Some(core::workspace_switch(app, arg)),
        "attach" | "image" | "media" | "fujian" => Some(attachment::attach(app, arg)),
        "task" | "tasks" => Some(task::task(app, arg)),
        "jobs" | "job" | "zuoye" => Some(jobs::jobs(app, arg)),
        "mcp" => Some(mcp::mcp(app, arg)),
        "network" => Some(network::network(app, arg)),
        "profile" | "dangan" => Some(core::profile_switch(app, arg)),
        "rlm" | "recursive" | "digui" => Some(rlm(app, arg)),
        _ => None,
    }
}
