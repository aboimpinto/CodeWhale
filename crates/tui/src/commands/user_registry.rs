//! Dedicated registry for user-defined markdown slash commands.
//!
//! This module owns the user-command boundary. Built-in command metadata and
//! dispatch remain in the normal command registry; user commands are loaded
//! from markdown files into this registry and are attempted before built-ins.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};

use crate::tui::app::{App, AppAction, HuntVerdict};

use super::CommandResult;
use super::user_commands;

static USER_COMMAND_REGISTRY: OnceLock<RwLock<UserCommandRegistryState>> = OnceLock::new();

#[derive(Debug, Clone, Default)]
struct UserCommandRegistryState {
    initialized: bool,
    workspace: Option<PathBuf>,
    registry: UserCommandRegistry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserCommandMetadata {
    pub name: String,
    pub body: String,
    pub description: Option<String>,
    pub argument_hint: Option<String>,
    pub allowed_tools: Vec<String>,
    pub pausable: bool,
    pub aliases: Vec<String>,
    pub hidden: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadError {
    pub path: PathBuf,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct UserCommandRegistry {
    commands: HashMap<String, UserCommandMetadata>,
    aliases: HashMap<String, String>,
    load_errors: Vec<LoadError>,
    invalid_commands: HashMap<String, String>,
}

impl UserCommandRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(workspace: Option<&Path>) -> Self {
        // NOTE: user_commands::commands_dirs() and load_commands_from_dir() are the
        // permanent lower-level scanning/parsing layer. This dependency is intentional:
        // user_commands.rs provides shared file I/O, frontmatter parsing, and template
        // support consumed by UserCommandRegistry. See FEAT-003 planning-analysis-report.md
        // (candidate B.1/E.3) for rationale.
        Self::load_from_paths(&user_commands::commands_dirs(workspace))
    }

    pub(crate) fn load_from_paths(paths: &[PathBuf]) -> Self {
        let mut loaded = Vec::new();
        let mut seen = HashSet::new();
        let mut registry = Self::new();

        for dir in paths {
            for (name, content) in user_commands::load_commands_from_dir(dir) {
                let canonical = normalize_name(&name);
                if seen.insert(canonical.clone()) {
                    loaded.push((name, content, dir.join(format!("{canonical}.md"))));
                } else {
                    registry.record_load_error(
                        dir.join(format!("{canonical}.md")),
                        format!(
                            "User command '/{canonical}' is defined more than once; using the first definition"
                        ),
                    );
                }
            }
        }
        loaded.sort_by(|a, b| a.0.cmp(&b.0));
        registry.load_from_entries(loaded);
        registry
    }

    #[cfg(test)]
    pub fn from_loaded(commands: Vec<(String, String)>) -> Self {
        let mut registry = Self::new();
        let loaded = commands
            .into_iter()
            .map(|(name, content)| {
                let path = PathBuf::from(format!("{}.md", normalize_name(&name)));
                (name, content, path)
            })
            .collect();
        registry.load_from_entries(loaded);
        registry
    }

    fn load_from_entries(&mut self, commands: Vec<(String, String, PathBuf)>) {
        for (name, content, path) in commands {
            let (metadata, errors) = parse_metadata(name, &content, &path);
            for error in errors {
                self.record_load_error(error.path.clone(), error.message.clone());
                self.invalid_commands
                    .entry(metadata.name.clone())
                    .or_insert(error.message);
            }

            if self.commands.contains_key(&metadata.name) {
                self.record_load_error(
                    path.clone(),
                    format!(
                        "User command '/{}' is defined more than once; using the first definition",
                        metadata.name
                    ),
                );
                continue;
            }

            for alias in &metadata.aliases {
                let alias = alias.to_ascii_lowercase();
                if let Some(existing) = self.aliases.get(&alias) {
                    self.record_load_error(
                        path.clone(),
                        format!(
                            "User command alias '/{alias}' for '/{}' duplicates user command '/{existing}'; using the first alias definition",
                            metadata.name
                        ),
                    );
                    continue;
                }
                self.aliases.insert(alias, metadata.name.clone());
            }

            self.commands.insert(metadata.name.clone(), metadata);
        }
    }

    fn record_load_error(&mut self, path: PathBuf, message: String) {
        self.load_errors.push(LoadError { path, message });
    }

    pub fn get(&self, name: &str) -> Option<&UserCommandMetadata> {
        let key = normalize_name(name);
        self.commands.get(&key).or_else(|| {
            self.aliases
                .get(&key)
                .and_then(|canonical| self.commands.get(canonical))
        })
    }

    #[cfg(test)]
    pub fn get_by_alias(&self, alias: &str) -> Option<&UserCommandMetadata> {
        let key = normalize_name(alias);
        self.aliases
            .get(&key)
            .and_then(|canonical| self.commands.get(canonical))
    }

    #[cfg(test)]
    pub fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.commands.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn iter(&self) -> impl Iterator<Item = &UserCommandMetadata> {
        self.commands.values()
    }

    #[cfg(test)]
    pub fn is_valid(&self) -> bool {
        self.load_errors.is_empty()
    }

    #[cfg(test)]
    pub fn load_errors(&self) -> &[LoadError] {
        &self.load_errors
    }

    fn dispatch_error(&self, name: &str) -> Option<String> {
        let key = normalize_name(name);
        self.invalid_commands.get(&key).cloned().or_else(|| {
            self.aliases
                .get(&key)
                .and_then(|canonical| self.invalid_commands.get(canonical))
                .cloned()
        })
    }
}

fn parse_metadata(
    name: String,
    content: &str,
    path: &Path,
) -> (UserCommandMetadata, Vec<LoadError>) {
    let canonical = normalize_name(&name);
    let (metadata, body) = user_commands::parse_frontmatter(content);
    let errors = validate_command_content(&canonical, content, path);
    let mut command = UserCommandMetadata {
        name: canonical,
        body: body.to_string(),
        description: None,
        argument_hint: None,
        allowed_tools: Vec::new(),
        pausable: false,
        aliases: Vec::new(),
        hidden: false,
    };

    for (key, value) in metadata {
        match key.as_str() {
            "description" => command.description = Some(value),
            "argument-hint" => command.argument_hint = Some(value),
            "allowed-tools" => command.allowed_tools = user_commands::parse_allowed_tools(&value),
            "pausable" => command.pausable = value.trim().eq_ignore_ascii_case("true"),
            "aliases" | "alias" => {
                command.aliases = value
                    .split(',')
                    .map(normalize_name)
                    .filter(|alias| !alias.is_empty())
                    .collect();
            }
            "hidden" => command.hidden = value.trim().eq_ignore_ascii_case("true"),
            _ => {}
        }
    }

    (command, errors)
}

fn validate_command_content(canonical: &str, content: &str, path: &Path) -> Vec<LoadError> {
    let mut errors = Vec::new();
    if canonical.is_empty() {
        errors.push(LoadError {
            path: path.to_path_buf(),
            message: "User command has an empty command name".to_string(),
        });
    }
    if content.trim().is_empty() {
        errors.push(LoadError {
            path: path.to_path_buf(),
            message: format!("User command '/{canonical}' is empty"),
        });
    }

    let Some(first_line_end) = content.find('\n') else {
        return errors;
    };
    let first = content[..first_line_end].trim_end_matches('\r');
    if !is_frontmatter_delimiter(first.trim()) {
        return errors;
    }

    let mut saw_closing = false;
    for raw_line in content[first_line_end + 1..].split_inclusive('\n') {
        let line = raw_line.trim_end_matches(['\r', '\n']);
        let trimmed = line.trim();
        if is_frontmatter_delimiter(trimmed) {
            saw_closing = true;
            break;
        }
        if trimmed.is_empty() || line.contains(':') {
            continue;
        }
        errors.push(LoadError {
            path: path.to_path_buf(),
            message: format!(
                "User command '/{canonical}' has invalid frontmatter line {trimmed:?}; expected key: value"
            ),
        });
        break;
    }

    if !saw_closing {
        errors.push(LoadError {
            path: path.to_path_buf(),
            message: format!(
                "User command '/{canonical}' has invalid frontmatter; missing closing --- delimiter"
            ),
        });
    }

    errors
}

fn is_frontmatter_delimiter(value: &str) -> bool {
    value.chars().all(|ch| ch == '-') && value.len() >= 3
}

fn normalize_name(name: &str) -> String {
    name.trim().trim_start_matches('/').to_ascii_lowercase()
}

fn normalize_workspace(workspace: Option<&Path>) -> Option<PathBuf> {
    workspace.map(Path::to_path_buf)
}

fn registry_lock() -> &'static RwLock<UserCommandRegistryState> {
    USER_COMMAND_REGISTRY.get_or_init(|| RwLock::new(UserCommandRegistryState::default()))
}

pub fn ensure_initialized(workspace: Option<&Path>) {
    let workspace = normalize_workspace(workspace);
    let lock = registry_lock();
    let should_reload = {
        let guard = lock.read().expect("user command registry lock poisoned");
        !guard.initialized || guard.workspace != workspace
    };

    if should_reload {
        reload(workspace.as_deref());
    }
}

pub fn reload(workspace: Option<&Path>) {
    let workspace = normalize_workspace(workspace);
    let replacement = UserCommandRegistry::load(workspace.as_deref());
    let mut guard = registry_lock()
        .write()
        .expect("user command registry lock poisoned");
    guard.initialized = true;
    guard.workspace = workspace;
    guard.registry = replacement;
}

pub fn current_registry() -> UserCommandRegistry {
    registry_lock()
        .read()
        .expect("user command registry lock poisoned")
        .registry
        .clone()
}

pub fn registry_for_workspace(workspace: Option<&Path>) -> UserCommandRegistry {
    ensure_initialized(workspace);
    current_registry()
}

pub fn try_dispatch(app: &mut App, input: &str) -> Option<CommandResult> {
    let parts: Vec<&str> = input.trim().splitn(2, ' ').collect();
    let command = normalize_name(parts.first().copied().unwrap_or_default());
    let args = parts.get(1).copied().unwrap_or("").trim();

    let registry = registry_for_workspace(Some(&app.workspace));
    if let Some(error) = registry.dispatch_error(&command) {
        return Some(CommandResult::error(error));
    }

    let metadata = registry.get(&command).cloned()?;

    app.hunt.quarry = None;
    app.hunt.started_at = None;
    app.hunt.verdict = HuntVerdict::Hunting;
    app.hunt.token_budget = None;
    app.active_allowed_tools = None;
    app.pausable = false;
    app.paused = false;
    app.paused_quarry = None;

    if let Some(description) = metadata.description.clone() {
        app.hunt.quarry = Some(description);
        app.hunt.started_at = Some(std::time::Instant::now());
    }
    if !metadata.allowed_tools.is_empty() {
        app.active_allowed_tools = Some(metadata.allowed_tools.clone());
    }
    app.pausable = metadata.pausable;

    let message = user_commands::apply_template(&metadata.body, args);
    Some(CommandResult::action(AppAction::SendMessage(message)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn registry_loads_markdown_metadata() {
        let registry = UserCommandRegistry::from_loaded(vec![(
            "review".to_string(),
            "---\ndescription: Review code\nargument-hint: <file>\nallowed-tools: read, grep\npausable: true\n---\nReview $ARGUMENTS".to_string(),
        )]);

        let command = registry.get("review").expect("command loaded");
        assert_eq!(command.description.as_deref(), Some("Review code"));
        assert_eq!(command.argument_hint.as_deref(), Some("<file>"));
        assert_eq!(command.allowed_tools, vec!["read", "grep"]);
        assert!(command.pausable);
        assert_eq!(command.body, "Review $ARGUMENTS");
    }

    #[test]
    fn registry_names_are_sorted() {
        let registry = UserCommandRegistry::from_loaded(vec![
            ("zeta".to_string(), "Z".to_string()),
            ("alpha".to_string(), "A".to_string()),
        ]);
        assert_eq!(registry.names(), vec!["alpha", "zeta"]);
    }

    #[test]
    fn registry_loads_from_paths_with_first_name_wins() {
        let first = TempDir::new().unwrap();
        let second = TempDir::new().unwrap();
        std::fs::write(first.path().join("shadow.md"), "first").unwrap();
        std::fs::write(second.path().join("shadow.md"), "second").unwrap();

        let registry = UserCommandRegistry::load_from_paths(&[
            first.path().to_path_buf(),
            second.path().to_path_buf(),
        ]);

        assert_eq!(registry.get("shadow").unwrap().body, "first");
    }

    #[test]
    fn alias_lookup_uses_metadata_aliases() {
        let registry = UserCommandRegistry::from_loaded(vec![(
            "canonical".to_string(),
            "---\naliases: short, other\n---\nBody".to_string(),
        )]);
        assert_eq!(registry.get_by_alias("short").unwrap().name, "canonical");
        assert_eq!(registry.get("/other").unwrap().body, "Body");
    }

    #[test]
    fn reload_and_current_registry_compile_sentinel() {
        reload(None);
        let registry = current_registry();
        assert!(registry.is_valid());
    }

    fn write_workspace_command(workspace: &Path, name: &str, content: &str) {
        let dir = workspace.join(".codewhale").join("commands");
        std::fs::create_dir_all(&dir).expect("create commands dir");
        std::fs::write(dir.join(format!("{name}.md")), content).expect("write command");
    }

    fn test_app(workspace: PathBuf) -> App {
        let options = crate::tui::app::TuiOptions {
            model: "deepseek-v4-pro".to_string(),
            workspace,
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
        App::new(options, &crate::config::Config::default())
    }

    fn sent_message(result: CommandResult) -> String {
        match result.action {
            Some(AppAction::SendMessage(message)) => message,
            other => panic!("expected SendMessage action, got {other:?}"),
        }
    }

    #[test]
    fn dispatch_prefers_user_command_over_builtin_with_same_name() {
        let tmp = TempDir::new().unwrap();
        write_workspace_command(tmp.path(), "help", "custom help $ARGUMENTS");
        let mut app = test_app(tmp.path().to_path_buf());

        let result = crate::commands::execute("/help links", &mut app);

        assert!(!result.is_error);
        assert_eq!(sent_message(result), "custom help links");
    }

    #[test]
    fn dispatch_prefers_user_alias_over_builtin_alias() {
        let tmp = TempDir::new().unwrap();
        write_workspace_command(
            tmp.path(),
            "attach-review",
            "---\nalias: image\n---\ncustom alias $ARGUMENTS",
        );
        let mut app = test_app(tmp.path().to_path_buf());

        let result = crate::commands::execute("/image screenshot.png", &mut app);

        assert!(!result.is_error, "{:?}", result.message);
        assert_eq!(sent_message(result), "custom alias screenshot.png");
    }

    #[test]
    fn hidden_user_commands_still_dispatch_directly() {
        let tmp = TempDir::new().unwrap();
        write_workspace_command(
            tmp.path(),
            "secret",
            "---\nhidden: true\ndescription: Internal workflow\n---\nsecret $ARGUMENTS",
        );
        let mut app = test_app(tmp.path().to_path_buf());

        let result = crate::commands::execute("/secret now", &mut app);

        assert!(!result.is_error);
        assert_eq!(sent_message(result), "secret now");
        assert_eq!(app.hunt.quarry.as_deref(), Some("Internal workflow"));
    }

    #[test]
    fn duplicate_user_alias_keeps_first_command_and_records_user_command_error() {
        let registry = UserCommandRegistry::from_loaded(vec![
            (
                "first".to_string(),
                "---\nalias: shared\n---\nfirst body".to_string(),
            ),
            (
                "second".to_string(),
                "---\nalias: shared\n---\nsecond body".to_string(),
            ),
        ]);

        let command = registry.get("shared").expect("alias resolves");
        assert_eq!(command.name, "first");
        assert_eq!(command.body, "first body");
        assert!(
            registry.load_errors().iter().any(|error| error
                .message
                .contains("User command alias '/shared'")
                && error.message.contains("/second")),
            "duplicate alias should be recorded as a user-command load error: {:?}",
            registry.load_errors()
        );
    }

    #[test]
    fn duplicate_user_command_name_records_user_command_error() {
        let registry = UserCommandRegistry::from_loaded(vec![
            ("review".to_string(), "first".to_string()),
            ("review".to_string(), "second".to_string()),
        ]);

        assert_eq!(registry.get("review").unwrap().body, "first");
        assert!(
            registry
                .load_errors()
                .iter()
                .any(|error| error.message.contains("User command '/review'")
                    && error.message.contains("defined more than once")),
            "duplicate name should be recorded as a user-command load error: {:?}",
            registry.load_errors()
        );
    }

    #[test]
    fn invalid_frontmatter_dispatch_returns_user_command_error_without_builtin_fallback() {
        let tmp = TempDir::new().unwrap();
        write_workspace_command(
            tmp.path(),
            "help",
            "---\ndescription: Custom help\nnot valid yaml\n---\ncustom help",
        );
        let mut app = test_app(tmp.path().to_path_buf());

        let result = crate::commands::execute("/help", &mut app);

        assert!(result.is_error);
        let message = result.message.expect("error message");
        assert!(message.contains("User command '/help'"), "{message}");
        assert!(message.contains("invalid frontmatter"), "{message}");
    }

    #[test]
    fn empty_user_command_dispatch_returns_user_command_error() {
        let tmp = TempDir::new().unwrap();
        write_workspace_command(tmp.path(), "empty", "\n\t  ");
        let mut app = test_app(tmp.path().to_path_buf());

        let result = crate::commands::execute("/empty", &mut app);

        assert!(result.is_error);
        let message = result.message.expect("error message");
        assert!(message.contains("User command '/empty'"), "{message}");
        assert!(message.contains("empty"), "{message}");
    }
}
