//! Config command area: settings, modes, themes, trust, and status surfaces.

// This group dir intentionally has a `config.rs` child module with the same
// name. The module_inception allow is a permanent structure rationale, not
// migration scaffolding; see docs/architecture/command-dispatch.md.
#[allow(clippy::module_inception)]
pub mod config;
mod cmd_config;
mod cmd_logout;
mod cmd_mode;
mod cmd_settings;
mod cmd_statusline;
mod cmd_theme;
mod cmd_trust;
mod cmd_verbose;
mod status;

use crate::commands::CommandResult;
use crate::commands::traits::{Command, CommandGroup, CommandInfo, FunctionCommand, RegisterCommand};
use crate::localization::MessageId;
use crate::tui::app::App;

use self::cmd_config::ConfigCmd;
use self::cmd_logout::LogoutCmd;
use self::cmd_mode::ModeCmd;
use self::cmd_settings::SettingsCmd;
use self::cmd_statusline::StatuslineCmd;
use self::cmd_theme::ThemeCmd;
use self::cmd_trust::TrustCmd;
use self::cmd_verbose::VerboseCmd;

pub struct ConfigCommands;

impl CommandGroup for ConfigCommands {
    fn commands(&self) -> Vec<Box<dyn Command>> {
        vec![
            Box::new(FunctionCommand::new(ConfigCmd::info(), ConfigCmd::execute)),
            Box::new(FunctionCommand::new(SettingsCmd::info(), SettingsCmd::execute)),
            Box::new(FunctionCommand::new(status::StatusCmd::info(), status::StatusCmd::execute)),
            Box::new(FunctionCommand::new(StatuslineCmd::info(), StatuslineCmd::execute)),
            Box::new(FunctionCommand::new(ModeCmd::info(), ModeCmd::execute)),
            Box::new(FunctionCommand::new(ThemeCmd::info(), ThemeCmd::execute)),
            Box::new(FunctionCommand::new(VerboseCmd::info(), VerboseCmd::execute)),
            Box::new(FunctionCommand::new(TrustCmd::info(), TrustCmd::execute)),
            Box::new(FunctionCommand::new(LogoutCmd::info(), LogoutCmd::execute)),
            Box::new(FunctionCommand::new(&SIDEBAR_INFO, run_sidebar)),
            Box::new(FunctionCommand::new(&DEBT_INFO, run_debt)),
        ]
    }
}

static SIDEBAR_INFO: CommandInfo = CommandInfo {
    name: "sidebar",
    aliases: &[],
    usage: "/sidebar [on|off|auto|work|tasks|agents|context] [--save]",
    description_id: MessageId::CmdSidebarDescription,
};
static DEBT_INFO: CommandInfo = CommandInfo {
    name: "debt",
    aliases: &["cleanup"],
    usage: "/debt [query|export]",
    description_id: MessageId::CmdSlopDescription,
};

fn run_registered(app: &mut App, name: &str, arg: Option<&str>) -> CommandResult {
    dispatch(app, name, arg).expect("registered config command should dispatch")
}

fn run_sidebar(app: &mut App, arg: Option<&str>) -> CommandResult {
    run_registered(app, "sidebar", arg)
}
fn run_debt(app: &mut App, arg: Option<&str>) -> CommandResult {
    run_registered(app, "debt", arg)
}

pub(in crate::commands) fn dispatch(
    app: &mut App,
    command: &str,
    arg: Option<&str>,
) -> Option<CommandResult> {
    let result = match command {
        "config" | "experiments" | "experimental" => config::config_command(app, arg),
        "sidebar" => config::sidebar(app, arg),
        "settings" => config::show_settings(app),
        "status" => status::status(app),
        "statusline" => config::status_line(app),
        "mode" => config::mode(app, arg),
        "jihua" => config::mode(app, Some("plan")),
        "zidong" => config::mode(app, Some("yolo")),
        "theme" => config::theme(app, arg),
        "verbose" => config::verbose(app, arg),
        "trust" | "xinren" => config::trust(app, arg),
        "logout" => config::logout(app),
        "debt" | "cleanup" | "slop" | "canzha" => config::slop(app, arg),
        _ => return None,
    };
    Some(result)
}
