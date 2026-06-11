//! Group-owned built-in command dispatch modules.
//!
//! Each module in this directory owns the dispatch logic for its command
//! group. The central `commands::execute` delegates to these group
//! dispatch functions instead of holding a monolithic match statement.
//!
//! See EPIC-001 / FEAT-001 for the full refactoring plan.

pub mod config;
pub mod core;
pub mod debug;
pub mod memory;
pub mod project;
pub mod session;
pub mod skills;
pub mod utility;

use crate::commands::CommandResult;
use crate::tui::app::App;

/// Common dispatch signature for all command group dispatch functions.
///
/// Returns `None` when the command is not recognised by this group
/// (allowing the caller to fall through to the next group or to
/// unknown-command handling).
#[allow(dead_code)]
pub type GroupDispatch =
    fn(command: &str, arg: Option<&str>, app: &mut App) -> Option<CommandResult>;

#[cfg(test)]
mod tests {
    use crate::commands::{CommandResult, groups};
    use crate::config::Config;
    use crate::tui::app::{App, TuiOptions};
    use std::path::PathBuf;

    fn test_app() -> App {
        let options = TuiOptions {
            model: "test-model".to_string(),
            workspace: PathBuf::from("."),
            config_path: None,
            config_profile: None,
            allow_shell: false,
            use_alt_screen: true,
            use_mouse_capture: false,
            use_bracketed_paste: true,
            max_subagents: 1,
            skills_dir: PathBuf::from("."),
            memory_path: PathBuf::from("memory.md"),
            notes_path: PathBuf::from("notes.txt"),
            mcp_config_path: PathBuf::from("mcp.json"),
            use_memory: false,
            start_in_agent_mode: false,
            skip_onboarding: true,
            yolo: false,
            resume_session_id: None,
            initial_input: None,
        };
        App::new(options, &Config::default())
    }

    /// Helper to run a group dispatch test table.
    /// Each entry is (group_name, command_to_test, expected_to_handle).
    fn run_table(tests: &[(&str, &str, bool)]) {
        for &(group, cmd, expected) in tests {
            let mut app = test_app();
            let result = match group {
                "core" => groups::core::dispatch(cmd, None, &mut app),
                "session" => groups::session::dispatch(cmd, None, &mut app),
                "config" => groups::config::dispatch(cmd, None, &mut app),
                "debug" => groups::debug::dispatch(cmd, None, &mut app),
                "project" => groups::project::dispatch(cmd, None, &mut app),
                "memory" => groups::memory::dispatch(cmd, None, &mut app),
                "skills" => groups::skills::dispatch(cmd, None, &mut app),
                "utility" => groups::utility::dispatch(cmd, None, &mut app),
                _ => panic!("unknown group: {group}"),
            };
            if expected {
                assert!(
                    result.is_some(),
                    "{group} dispatch should handle /{cmd}, got None"
                );
            } else {
                assert!(
                    result.is_none(),
                    "{group} dispatch should NOT handle /{cmd}, got Some"
                );
            }
        }
    }

    // ── Core group ─────────────────────────────────────────────────

    #[test]
    fn core_group_handles_its_commands() {
        run_table(&[
            ("core", "anchor", true),
            ("core", "help", true),
            ("core", "clear", true),
            ("core", "exit", true),
            ("core", "model", true),
            ("core", "models", true),
            ("core", "provider", true),
            ("core", "queue", true),
            ("core", "stash", true),
            ("core", "hooks", true),
            ("core", "subagents", true),
            ("core", "agent", true),
            ("core", "links", true),
            ("core", "feedback", true),
            ("core", "home", true),
            ("core", "workspace", true),
            ("core", "attach", true),
            ("core", "task", true),
            ("core", "jobs", true),
            ("core", "mcp", true),
            ("core", "network", true),
            ("core", "profile", true),
            ("core", "rlm", true),
            // Aliases
            ("core", "maodian", true),
            ("core", "?", true),
            ("core", "bangzhu", true),
            ("core", "qingping", true),
            ("core", "quit", true),
            ("core", "q", true),
            ("core", "moxing", true),
            ("core", "queued", true),
            ("core", "park", true),
            ("core", "gouzi", true),
            ("core", "agents", true),
            ("core", "daili", true),
            ("core", "dashboard", true),
            ("core", "api", true),
            ("core", "stats", true),
            ("core", "cwd", true),
            ("core", "dangan", true),
            ("core", "recursive", true),
            ("core", "digui", true),
            // Commands owned by other groups
            ("core", "rename", false),
            ("core", "config", false),
            ("core", "skills", false),
            ("core", "balance", false),
        ]);
    }

    // ── Session group ──────────────────────────────────────────────

    #[test]
    fn session_group_handles_its_commands() {
        run_table(&[
            ("session", "rename", true),
            ("session", "save", true),
            ("session", "fork", true),
            ("session", "new", true),
            ("session", "sessions", true),
            ("session", "relay", true),
            ("session", "load", true),
            ("session", "compact", true),
            ("session", "purge", true),
            ("session", "export", true),
            // Aliases
            ("session", "gaiming", true),
            ("session", "chongmingming", true),
            ("session", "branch", true),
            ("session", "resume", true),
            ("session", "batonpass", true),
            ("session", "jiazai", true),
            ("session", "yasuo", true),
            ("session", "qingchu", true),
            ("session", "daochu", true),
            // Commands owned by other groups
            ("session", "clear", false),
            ("session", "config", false),
            ("session", "skills", false),
        ]);
    }

    // ── Config group ───────────────────────────────────────────────

    #[test]
    fn config_group_handles_its_commands() {
        run_table(&[
            ("config", "config", true),
            ("config", "settings", true),
            ("config", "status", true),
            ("config", "statusline", true),
            ("config", "mode", true),
            ("config", "theme", true),
            ("config", "verbose", true),
            ("config", "trust", true),
            ("config", "logout", true),
            ("config", "slop", true),
            ("config", "lsp", true),
            // Aliases
            ("config", "jihua", true),
            ("config", "zidong", true),
            ("config", "xinren", true),
            ("config", "canzha", true),
            // Commands owned by other groups
            ("config", "help", false),
            ("config", "exit", false),
            ("config", "memory", false),
        ]);
    }

    // ── Debug group ────────────────────────────────────────────────

    #[test]
    fn debug_group_handles_its_commands() {
        run_table(&[
            ("debug", "translate", true),
            ("debug", "tokens", true),
            ("debug", "cost", true),
            ("debug", "balance", true),
            ("debug", "cache", true),
            ("debug", "change", true),
            ("debug", "system", true),
            ("debug", "context", true),
            ("debug", "edit", true),
            ("debug", "diff", true),
            ("debug", "undo", true),
            ("debug", "retry", true),
            // Aliases
            ("debug", "translation", true),
            ("debug", "transale", true),
            ("debug", "xitong", true),
            ("debug", "ctx", true),
            ("debug", "chongshi", true),
            // Commands owned by other groups
            ("debug", "help", false),
            ("debug", "config", false),
            ("debug", "memory", false),
        ]);
    }

    // ── Project group ──────────────────────────────────────────────

    #[test]
    fn project_group_handles_its_commands() {
        run_table(&[
            ("project", "init", true),
            ("project", "share", true),
            ("project", "goal", true),
            ("project", "hunt", true),
            // Aliases
            ("project", "mubiao", true),
            ("project", "狩猎", true),
            // Commands owned by other groups
            ("project", "clear", false),
            ("project", "config", false),
            ("project", "skills", false),
        ]);
    }

    // ── Memory group ───────────────────────────────────────────────

    #[test]
    fn memory_group_handles_its_commands() {
        run_table(&[
            ("memory", "memory", true),
            ("memory", "note", true),
            // Commands owned by other groups
            ("memory", "clear", false),
            ("memory", "exit", false),
            ("memory", "help", false),
        ]);
    }

    // ── Skills group ───────────────────────────────────────────────

    #[test]
    fn skills_group_handles_its_commands() {
        run_table(&[
            ("skills", "skills", true),
            ("skills", "skill", true),
            ("skills", "review", true),
            ("skills", "restore", true),
            // Aliases
            ("skills", "jinengliebiao", true),
            ("skills", "jineng", true),
            ("skills", "shencha", true),
            // Commands owned by other groups
            ("skills", "clear", false),
            ("skills", "config", false),
            ("skills", "help", false),
        ]);
    }

    // ── Utility group ──────────────────────────────────────────────

    #[test]
    fn utility_group_accepts_no_commands() {
        // The utility group is a placeholder with no owned commands.
        // balance and change are owned by the debug group.
        let mut app = test_app();
        assert!(groups::utility::dispatch("balance", None, &mut app).is_none());
        assert!(groups::utility::dispatch("change", None, &mut app).is_none());
        assert!(groups::utility::dispatch("clear", None, &mut app).is_none());
        assert!(groups::utility::dispatch("help", None, &mut app).is_none());
    }

    // ── Cross-group boundary tests ─────────────────────────────────

    #[test]
    fn no_command_is_handled_by_two_groups() {
        // Verify that no command name is claimed by more than one group.
        let all_commands: &[&str] = &[
            // Core
            "anchor",
            "maodian",
            "help",
            "?",
            "bangzhu",
            "帮助",
            "clear",
            "qingping",
            "exit",
            "quit",
            "q",
            "tuichu",
            "model",
            "moxing",
            "models",
            "moxingliebiao",
            "provider",
            "queue",
            "queued",
            "stash",
            "park",
            "hooks",
            "hook",
            "gouzi",
            "subagents",
            "agents",
            "zhinengti",
            "agent",
            "daili",
            "links",
            "dashboard",
            "api",
            "lianjie",
            "feedback",
            "home",
            "stats",
            "overview",
            "zhuye",
            "shouye",
            "workspace",
            "cwd",
            "attach",
            "image",
            "media",
            "fujian",
            "task",
            "tasks",
            "jobs",
            "job",
            "zuoye",
            "mcp",
            "network",
            "profile",
            "dangan",
            "rlm",
            "recursive",
            "digui",
            // Session
            "rename",
            "gaiming",
            "chongmingming",
            "save",
            "fork",
            "branch",
            "new",
            "sessions",
            "resume",
            "relay",
            "batonpass",
            "接力",
            "load",
            "jiazai",
            "compact",
            "yasuo",
            "purge",
            "qingchu",
            "export",
            "daochu",
            // Config
            "config",
            "settings",
            "status",
            "statusline",
            "mode",
            "jihua",
            "zidong",
            "theme",
            "verbose",
            "trust",
            "xinren",
            "logout",
            "slop",
            "canzha",
            "lsp",
            // Debug
            "translate",
            "translation",
            "transale",
            "tokens",
            "cost",
            "balance",
            "cache",
            "change",
            "system",
            "xitong",
            "context",
            "ctx",
            "edit",
            "diff",
            "undo",
            "retry",
            "chongshi",
            // Project
            "init",
            "share",
            "goal",
            "hunt",
            "mubiao",
            "狩猎",
            // Memory
            "memory",
            "note",
            // Skills
            "skills",
            "jinengliebiao",
            "skill",
            "jineng",
            "review",
            "shencha",
            "restore",
        ];

        let groups: &[(
            &str,
            fn(&str, Option<&str>, &mut App) -> Option<CommandResult>,
        )] = &[
            ("core", |c, a, app| groups::core::dispatch(c, a, app)),
            ("session", |c, a, app| groups::session::dispatch(c, a, app)),
            ("config", |c, a, app| groups::config::dispatch(c, a, app)),
            ("debug", |c, a, app| groups::debug::dispatch(c, a, app)),
            ("project", |c, a, app| groups::project::dispatch(c, a, app)),
            ("memory", |c, a, app| groups::memory::dispatch(c, a, app)),
            ("skills", |c, a, app| groups::skills::dispatch(c, a, app)),
            ("utility", |c, a, app| groups::utility::dispatch(c, a, app)),
        ];

        for &cmd in all_commands {
            let mut matches: Vec<&str> = Vec::new();
            // We still use run_table-style but check uniqueness
            for &(name, dispatch_fn) in groups {
                let mut app = test_app();
                if dispatch_fn(cmd, None, &mut app).is_some() {
                    matches.push(name);
                }
            }
            assert!(!matches.is_empty(), "/{cmd} is not claimed by any group!");
            assert!(
                matches.len() <= 1,
                "/{cmd} is claimed by multiple groups: {:?}",
                matches
            );
        }
    }
}
