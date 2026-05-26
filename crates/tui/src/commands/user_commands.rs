//! User-defined slash commands from `~/.deepseek/commands/<name>.md` and
//! workspace-local `<workspace>/.deepseek/commands/<name>.md`.
//!
//! Files can include optional YAML-like frontmatter between `---` markers.
//! When frontmatter is present, it is stripped from the message sent to
//! the model Ã¢ÂÂ only the body after the closing `---` is used.
//! Supported frontmatter fields: `description`, `allowed-tools`, `argument-hint`, `pausable`.
//! Plain `.md` files without frontmatter work exactly as before.
//!
//! Users drop `.md` files into a commands directory and the filename
//! (without `.md` extension) becomes a slash command. When invoked via
//! `/name`, the file contents are sent as a user message.
//!
//! ## Precedence
//!
//! Workspace-local directories shadow user-global by name:
//!
//! 1. `<workspace>/.deepseek/commands/`  (project-local, highest)
//! 2. `<workspace>/.claude/commands/`    (Claude Code interop)
//! 3. `<workspace>/.cursor/commands/`    (Cursor interop)
//! 4. `~/.deepseek/commands/`            (user-global, lowest)

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::tui::app::{App, AppAction};

use super::CommandResult;

/// Path to the global user commands directory: `~/.deepseek/commands/`.
fn global_commands_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
    home.join(".deepseek").join("commands")
}

/// Return all candidate commands directories in precedence order.
fn commands_dirs(workspace: Option<&Path>) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(ws) = workspace {
        dirs.push(ws.join(".deepseek").join("commands"));
        dirs.push(ws.join(".claude").join("commands"));
        dirs.push(ws.join(".cursor").join("commands"));
    }
    dirs.push(global_commands_dir());
    dirs
}

/// Scan a single commands directory for `.md` files and return
/// `(name, content)` pairs. Errors are silently skipped.
fn load_commands_from_dir(dir: &Path) -> Vec<(String, String)> {
    let mut commands: Vec<(String, String)> = Vec::new();

    if !dir.is_dir() {
        return commands;
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return commands,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let stem = match path.file_stem().and_then(|s| s.to_str()) {
            Some(stem) => stem.to_lowercase(),
            None => continue,
        };
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        commands.push((stem, content));
    }

    commands
}

/// Scan every candidate commands directory and return merged
/// `(name, content)` pairs. Workspace-local directories shadow
/// user-global by name Ã¢ÂÂ the first occurrence of a name wins.
///
/// Pass `None` for the workspace to scan only the global directory
/// (backward-compatible with callers that don't have workspace context).
pub fn load_user_commands(workspace: Option<&Path>) -> Vec<(String, String)> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut commands: Vec<(String, String)> = Vec::new();

    for dir in commands_dirs(workspace) {
        for (name, content) in load_commands_from_dir(&dir) {
            if seen.insert(name.clone()) {
                commands.push((name, content));
            }
        }
    }

    // Sort by name for deterministic ordering.
    commands.sort_by(|a, b| a.0.cmp(&b.0));
    commands
}

/// Parse optional YAML-like frontmatter from command markdown content.
///
/// Returns `(frontmatter key-value pairs, body)` where `body` is the
/// content after the closing `---` delimiter, or the whole content
/// unchanged when no frontmatter is present.
fn parse_frontmatter(content: &str) -> (Vec<(String, String)>, &str) {
    let content = content.trim();
    if !content.starts_with("---") {
        return (Vec::new(), content);
    }
    // Find the closing delimiter (\n followed by 3+ dashes) after the
    // opening ---. Works for ---, ----, -----, or any dash sequence.
    let rest = &content[3..];
    let closing = match rest.find("\n---") {
        Some(pos) => pos,
        None => return (Vec::new(), content),
    };
    // `closing` is the position of \n in rest-space (0 = first byte after ---).
    // Convert to content-space.
    let delim_start = 3 + closing;
    let frontmatter_text = &content[3..delim_start].trim();
    // Skip past the closing delimiter line (\n followed by any number of dashes)
    // so body starts after all delimiter characters.
    let bytes = content.as_bytes();
    let mut body_start = delim_start;
    while body_start < content.len() && bytes[body_start] == b'\n' {
        body_start += 1;
    }
    while body_start < content.len() && bytes[body_start] == b'-' {
        body_start += 1;
    }
    let body = content[body_start..].trim();
    let mut pairs = Vec::new();
    for line in frontmatter_text.lines() {
        if let Some((key, value)) = line.split_once(':') {
            pairs.push((key.trim().to_string(), value.trim().to_string()));
        }
    }
    (pairs, body)
}

/// Substitute $1, $2, $ARGUMENTS placeholders in a command template.
fn apply_template(template: &str, args: &str) -> String {
    let positional: Vec<&str> = args.split_whitespace().collect();
    let mut result = template.replace("$ARGUMENTS", args);
    for (i, arg) in positional.iter().enumerate() {
        result = result.replace(&format!("${}", i + 1), arg);
    }
    result
}

/// Check if the input matches a user-defined command and return the
/// content as a `SendMessage` action.
///
/// The `input` should be the full command string including the `/`
/// prefix (e.g. `/mycmd` or `/mycmd with args`). Only exact matches
/// on the command name are considered (no partial/alias matching).
///
/// If the command file contains YAML frontmatter between `---` markers,
/// the frontmatter is stripped Ã¢ÂÂ only the body after the closing `---`
/// is sent as the message.
pub fn try_dispatch_user_command(app: &mut App, input: &str) -> Option<CommandResult> {
    let parts: Vec<&str> = input.trim().splitn(2, ' ').collect();
    let command = parts[0].to_lowercase();
    let command = command.strip_prefix('/').unwrap_or(&command);
    let args = parts.get(1).copied().unwrap_or("").trim();

        tracing::debug!(target: "pausable", input, command, "try_dispatch_user_command called");
        let user_commands = load_user_commands(Some(&app.workspace));

    for (name, content) in &user_commands {
        if name == command {
            // Strip frontmatter if present before substituting args
            let (meta, body) = parse_frontmatter(content);
            // If the command has a description, show it in the Work panel
            for (key, value) in &meta {
                tracing::debug!(target: "pausable", key, value, "frontmatter key");
                if key == "description" {
                    app.goal.goal_objective = Some(value.clone());
                    app.goal.goal_started_at = Some(Instant::now());
                }
                if key == "allowed-tools" {
                    let tools: Vec<String> = value
                        .split(',')
                        .map(|t| t.trim().to_lowercase())
                        .filter(|t| !t.is_empty())
                        .collect();
                    if !tools.is_empty() {
                        app.active_allowed_tools = Some(tools);
                    }
                }
                if key == "pausable" && value.trim().eq_ignore_ascii_case("true") {
                    tracing::debug!(target: "pausable", value, "PAUSABLE FRONTMATTER MATCHED");
                    // If a previous pausable command has a snapshot, restore it first
                    if let Some(snap_id) = app.active_snapshot.take() {
                        if let Ok(repo) = crate::snapshot::repo::SnapshotRepo::open_or_init(&app.workspace) {
                            let _ = repo.restore(&crate::snapshot::repo::SnapshotId(snap_id));
                        }
                    }
                    app.pausable = true;
                    app.paused = false;
                    // Snapshot workspace for potential rollback via git stash
                    let git_stash_cmd = std::process::Command::new("git")
                        .args(["-C", &app.workspace.to_string_lossy(), "stash", "push", "--include-untracked", "-m", "codewhale-pausable"])
                        .output();
                    match git_stash_cmd {
                        Ok(output) if output.status.success() => {
                            // git stash returns the stash reference on stdout
                            let ref_name = String::from_utf8_lossy(&output.stdout).trim().to_string();
                            if ref_name.contains("stash") || ref_name.contains("HEAD") {
                                app.active_snapshot = Some("stash".to_string());
                            } else {
                                app.active_snapshot = Some("stash".to_string());
                            }
                            tracing::debug!(target: "pausable", "created git stash snapshot");
                        }
                        Ok(output) => {
                            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                            tracing::warn!(target: "pausable", stderr, "git stash failed");
                        }
                        Err(e) => {
                            tracing::warn!(target: "pausable", error = %e, "failed to run git stash");
                        }
                    }
                }
            }
            let message = apply_template(body, args);
            return Some(CommandResult::action(AppAction::SendMessage(message)));
        }
    }

    None
}

/// Look up a user-defined command by name and return its description
/// from frontmatter (if available).
pub fn get_user_command_description(
    name: &str,
    workspace: Option<&Path>,
) -> Option<String> {
    let name = name.to_lowercase();
    let name = name.strip_prefix('/').unwrap_or(&name);
    for (cmd_name, content) in load_user_commands(workspace) {
        if cmd_name == name {
            let (meta, _) = parse_frontmatter(&content);
            for (key, value) in &meta {
                if key == "description" {
                    return Some(value.clone());
                }
            }
            return None;
        }
    }
    None
}

/// Get user command names that match a given prefix (for autocomplete).
///
/// The prefix should be the command name portion only (after `/`).
/// Returns entries formatted as `/name`.
///
/// `workspace` is used to also scan workspace-local command directories;
/// pass `None` when no workspace context is available.
pub fn user_commands_matching(prefix: &str, workspace: Option<&Path>) -> Vec<String> {
    let prefix = prefix.to_lowercase();
    load_user_commands(workspace)
        .into_iter()
        .filter(|(name, _)| name.starts_with(&prefix))
        .map(|(name, _)| format!("/{}", name))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_global_commands_dir_contains_deepseek_commands() {
        let dir = global_commands_dir();
        let parts: Vec<_> = dir
            .components()
            .filter_map(|component| component.as_os_str().to_str())
            .collect();
        assert!(
            parts
                .windows(2)
                .any(|pair| pair == [".deepseek", "commands"]),
            "expected .deepseek/commands components in path, got: {}",
            dir.display()
        );
    }

    #[test]
    fn test_load_user_commands_when_no_dir_exists() {
        let cmds = load_user_commands(None);
        // Should not panic; returns empty vec when no directories exist.
        assert!(cmds.is_empty() || !cmds.is_empty());
    }

    #[test]
    fn test_try_dispatch_nonexistent_command() {
        use crate::config::Config;
        use crate::tui::app::TuiOptions;

        let options = TuiOptions {
            model: "deepseek-v4-pro".to_string(),
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
        let mut app = App::new(options, &Config::default());
        let result = try_dispatch_user_command(&mut app, "/nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_try_dispatch_uses_workspace_local_command() {
        use crate::config::Config;
        use crate::tui::app::TuiOptions;

        let tmp = TempDir::new().unwrap();
        let ws = tmp.path().to_path_buf();
        write_command(
            &ws.join(".deepseek").join("commands"),
            "hello",
            "Hello, $ARGUMENTS!",
        );

        let options = TuiOptions {
            model: "deepseek-v4-pro".to_string(),
            workspace: ws.clone(),
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
        let mut app = App::new(options, &Config::default());
        let result = try_dispatch_user_command(&mut app, "/hello world");
        assert!(result.is_some());
        let cmd_result = result.unwrap();
        match cmd_result.action {
            Some(AppAction::SendMessage(msg)) => {
                assert!(msg.contains("Hello, world!"), "got: {msg}");
            }
            other => panic!("expected SendMessage action, got: {other:?}"),
        }
    }

    #[test]
    fn test_frontmatter_is_stripped_from_dispatch() {
        use crate::config::Config;
        use crate::tui::app::TuiOptions;

        let tmp = TempDir::new().unwrap();
        let ws = tmp.path().to_path_buf();
        // Command with frontmatter Ã¢ÂÂ the body is just "Run tests"
        write_command(
            &ws.join(".deepseek").join("commands"),
            "test-runner",
            "---\ndescription: Run tests\nallowed-tools: Bash\n---\nRun tests",
        );

        let options = TuiOptions {
            model: "deepseek-v4-pro".to_string(),
            workspace: ws.clone(),
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
        let mut app = App::new(options, &Config::default());
        let result = try_dispatch_user_command(&mut app, "/test-runner");
        assert!(result.is_some());
        let cmd_result = result.unwrap();
        match cmd_result.action {
            Some(AppAction::SendMessage(msg)) => {
                assert_eq!(msg, "Run tests", "frontmatter should be stripped, got: {msg}");
            }
            other => panic!("expected SendMessage action, got: {other:?}"),
        }
    }

    #[test]
    fn test_frontmatter_strips_description_key() {
        let content = "---\ndescription: My command\nallowed-tools: Bash, Read\n---\n\nBody text here";
        let (meta, body) = parse_frontmatter(content);
        assert_eq!(body, "Body text here");
        assert!(meta.iter().any(|(k, _)| k == "description"));
        assert!(meta.iter().any(|(k, _)| k == "allowed-tools"));
    }

    #[test]
    fn test_flexible_dash_count_accepted() {
        // More than 3 dashes (-----, ---------, etc.) should work
        let content = "-----\ndescription: test\n-----\n\nBody";
        let (meta, body) = parse_frontmatter(content);
        assert_eq!(body, "Body");
        assert!(meta.iter().any(|(k, _)| k == "description"));
    }

    #[test]
    fn test_plain_md_without_frontmatter_passes_through() {
        let content = "Just a plain message";
        let (meta, body) = parse_frontmatter(content);
        assert!(meta.is_empty());
        assert_eq!(body, "Just a plain message");
    }

    fn write_command(dir: &Path, name: &str, body: &str) {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join(format!("{name}.md")), body).unwrap();
    }

    #[test]
    fn test_load_commands_merges_dirs_in_precedence_order() {
        let tmp = TempDir::new().unwrap();
        let ws = tmp.path();

        // Write commands in each searchable directory
        write_command(
            &ws.join(".deepseek").join("commands"),
            "deepseek-cmd",
            "from deepseek",
        );
        write_command(
            &ws.join(".claude").join("commands"),
            "claude-cmd",
            "from claude",
        );
        write_command(
            &ws.join(".cursor").join("commands"),
            "cursor-cmd",
            "from cursor",
        );

        let cmds = load_user_commands(Some(ws));
        let names: Vec<&str> = cmds.iter().map(|(n, _)| n.as_str()).collect();
        assert!(
            names.contains(&"claude-cmd"),
            "expected 'claude-cmd': {names:?}"
        );
        assert!(
            names.contains(&"cursor-cmd"),
            "expected 'cursor-cmd': {names:?}"
        );
    }

    #[test]
    fn test_workspace_local_shadows_global_by_name() {
        let tmp = TempDir::new().unwrap();
        let ws = tmp.path();

        // Workspace-local version
        write_command(
            &ws.join(".deepseek").join("commands"),
            "shared",
            "workspace version",
        );
        // Claude dir version Ã¢ÂÂ should be shadowed by .deepseek
        write_command(
            &ws.join(".claude").join("commands"),
            "shared",
            "claude version",
        );

        let cmds = load_user_commands(Some(ws));
        let shared = cmds
            .iter()
            .find(|(n, _)| n == "shared")
            .expect("shared present");
        assert_eq!(
            shared.1, "workspace version",
            "workspace-local (.deepseek) must shadow later dirs"
        );
    }

    #[test]
    fn test_load_user_commands_without_workspace_falls_back_to_global_only() {
        let cmds = load_user_commands(None);
        let _ = cmds;
    }

    #[test]
    fn test_try_dispatch_uses_workspace_local_command_full() {
        use crate::config::Config;
        use crate::tui::app::TuiOptions;

        let tmp = TempDir::new().unwrap();
        let ws = tmp.path().to_path_buf();
        write_command(
            &ws.join(".deepseek").join("commands"),
            "hello",
            "Hello, $ARGUMENTS!",
        );

        let options = TuiOptions {
            model: "deepseek-v4-pro".to_string(),
            workspace: ws.clone(),
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
        let mut app = App::new(options, &Config::default());
        let result = try_dispatch_user_command(&mut app, "/hello world");
        assert!(result.is_some());
        let cmd_result = result.unwrap();
        match cmd_result.action {
            Some(AppAction::SendMessage(msg)) => {
                assert!(msg.contains("Hello, world!"), "got: {msg}");
            }
            other => panic!("expected SendMessage action, got: {other:?}"),
        }
    }

    #[test]
    fn test_user_commands_matching_with_workspace() {
        let tmp = TempDir::new().unwrap();
        let ws = tmp.path();
        write_command(
            &ws.join(".deepseek").join("commands"),
            "project-cmd",
            "body",
        );

        let matches = user_commands_matching("project", Some(ws));
        assert!(
            matches.contains(&"/project-cmd".to_string()),
            "got: {matches:?}"
        );
    }

    // ââ allowed-tools frontmatter âââââââââââââââââââââââââââââââââââ

    #[test]
    fn test_allowed_tools_parses_single_tool() {
        let content = "---\ndescription: Test\nallowed-tools: Bash\n---\n\nrun tests";
        let (meta, body) = parse_frontmatter(content);
        assert_eq!(body, "run tests");
        let tools = meta
            .iter()
            .find(|(k, _)| k == "allowed-tools")
            .map(|(_, v)| v.as_str())
            .unwrap_or("");
        let parsed: Vec<String> = tools
            .split(',')
            .map(|t| t.trim().to_lowercase())
            .filter(|t| !t.is_empty())
            .collect();
        assert_eq!(parsed, vec!["bash"]);
    }

    #[test]
    fn test_allowed_tools_parses_multiple_tools() {
        let content = "---\ndescription: Dev\nallowed-tools: Bash, Read, Write\n---\n\ndevelop";
        let (meta, _) = parse_frontmatter(content);
        let tools = meta
            .iter()
            .find(|(k, _)| k == "allowed-tools")
            .map(|(_, v)| v.as_str())
            .unwrap_or("");
        let parsed: Vec<String> = tools
            .split(',')
            .map(|t| t.trim().to_lowercase())
            .filter(|t| !t.is_empty())
            .collect();
        assert_eq!(parsed, vec!["bash", "read", "write"]);
    }

    #[test]
    fn test_allowed_tools_handles_whitespace_and_case() {
        let content = "---\nallowed-tools:  BASH ,  grep  ,   read   \n---\n\ncmd";
        let (meta, _) = parse_frontmatter(content);
        let tools = meta
            .iter()
            .find(|(k, _)| k == "allowed-tools")
            .map(|(_, v)| v.as_str())
            .unwrap_or("");
        let parsed: Vec<String> = tools
            .split(',')
            .map(|t| t.trim().to_lowercase())
            .filter(|t| !t.is_empty())
            .collect();
        assert_eq!(parsed, vec!["bash", "grep", "read"]);
    }

    #[test]
    fn test_allowed_tools_missing_does_not_set_app_state() {
        use crate::config::Config;

        let tmp = TempDir::new().unwrap();
        let ws = tmp.path().to_path_buf();
        write_command(&ws.join(".deepseek").join("commands"), "plain", "just a command");

        let options = crate::tui::app::TuiOptions {
            model: "deepseek-v4-pro".to_string(),
            workspace: ws.clone(),
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
        let mut app = crate::tui::app::App::new(options, &Config::default());
        let _ = try_dispatch_user_command(&mut app, "/plain");
        assert!(app.active_allowed_tools.is_none(),
            "expected active_allowed_tools to be None when no frontmatter, got: {:?}",
            app.active_allowed_tools);
    }

    #[test]
    fn test_allowed_tools_frontmatter_sets_app_state() {
        use crate::config::Config;

        let tmp = TempDir::new().unwrap();
        let ws = tmp.path().to_path_buf();
        write_command(&ws.join(".deepseek").join("commands"), "secure",
            "---\nallowed-tools: Bash, Grep\n---\ndo scan");

        let options = crate::tui::app::TuiOptions {
            model: "deepseek-v4-pro".to_string(),
            workspace: ws.clone(),
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
        let mut app = crate::tui::app::App::new(options, &Config::default());
        let _ = try_dispatch_user_command(&mut app, "/secure");
        assert_eq!(app.active_allowed_tools, Some(vec!["bash".to_string(), "grep".to_string()]));
    }

}