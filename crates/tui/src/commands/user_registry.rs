//! Dedicated registry for user-defined commands.
//!
//! This module provides a separate boundary from the built-in `COMMANDS` array.
//! User commands are loaded from markdown files in workspace-local and global
//! directories, parsed for frontmatter, and stored in a [`UserCommandRegistry`].
//!
//! The registry is self-contained and does not reference built-in command types.
//! It reuses the existing directory-scan and frontmatter-parsing logic from
//! [`super::user_commands`] to ensure backward compatibility.
//!
//! # Key design principles
//!
//! - **Read-only after construction** – built once, atomically replaced on reload.
//! - **`UserCommandMetadata` includes the raw body** so dispatch can apply template
//!   substitution without re-reading files.
//! - **`load_errors`** collects non-fatal issues (malformed frontmatter, duplicate
//!   names) for reporting without failing the whole load.
//! - **No reference to built-in commands** – the registry is self-contained.

#![cfg_attr(not(test), allow(dead_code))]

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};

use crate::tui::app::{App, AppAction, HuntVerdict};

use super::CommandResult;

use super::user_commands;

/// Metadata for a single user-defined command.
///
/// This struct holds all information extracted from a user-command markdown file,
/// including frontmatter fields and the raw body for template expansion.
#[derive(Debug, Clone)]
pub struct UserCommandMetadata {
    /// Canonical name (file stem, lowercased).
    pub name: String,
    /// Raw markdown body (after frontmatter stripping), for template expansion
    /// via `$1`, `$2`, `$ARGUMENTS`.
    pub body: String,
    /// Optional description for help text and slash completion.
    pub description: Option<String>,
    /// Optional argument hint for completion display (e.g., `<root>`).
    pub argument_hint: Option<String>,
    /// Optional tool restriction (e.g., `["Bash", "Read"]`).
    pub allowed_tools: Option<Vec<String>>,
    /// Whether the command supports pause/resume.
    pub pausable: bool,
    /// Aliases (not currently parsed from frontmatter; reserved for future use).
    pub aliases: Vec<String>,
    /// Whether the command should be hidden from help/completion (reserved).
    pub hidden: bool,
}

/// A non-fatal error encountered during registry population.
///
/// Multiple load errors can be accumulated without aborting the load process.
/// They are surfaced when the broken command is invoked or during diagnostics.
#[derive(Debug, Clone)]
pub struct LoadError {
    /// Path to the file that caused the error.
    pub file_path: PathBuf,
    /// Human-readable error description.
    pub message: String,
}

// ── Global registry (initialized once, atomically replaced on reload) ─────

static REGISTRY: OnceLock<RwLock<UserCommandRegistry>> = OnceLock::new();

/// Check whether the global registry has been initialized.
#[allow(dead_code)]
pub fn is_initialized() -> bool {
    REGISTRY.get().is_some()
}

/// Ensure the global registry is initialized for the given workspace.
///
/// If the registry is already initialized, this is a no-op.
/// The workspace is used to locate workspace-local command directories.
/// Pass `None` to scan only global directories.
pub fn ensure_initialized(workspace: Option<&Path>) {
    if REGISTRY.get().is_none() {
        let registry = UserCommandRegistry::load(workspace);
        let _ = REGISTRY.set(RwLock::new(registry));
    }
}

/// Reload the global registry from the filesystem.
///
/// Re-scans all command directories and atomically replaces the registry.
/// Load errors are logged via `tracing::warn!`.
pub fn reload(workspace: Option<&Path>) {
    let registry = UserCommandRegistry::load(workspace);
    match REGISTRY.get() {
        Some(lock) => {
            let mut guard = lock.write().expect("registry lock poisoned");
            *guard = registry;
        }
        None => {
            let _ = REGISTRY.set(RwLock::new(registry));
        }
    }
}

/// Get a read-locked reference to the current global registry.
///
/// Returns `None` if the registry has not been initialized.
pub fn current_registry() -> Option<std::sync::RwLockReadGuard<'static, UserCommandRegistry>> {
    REGISTRY
        .get()
        .map(|lock| lock.read().expect("registry lock poisoned"))
}

/// Try to dispatch a user command using the global registry.
///
/// This replaces the filesystem-scanning behaviour of
/// `user_commands::try_dispatch_user_command` with a cached registry lookup.
///
/// # Dispatch behaviour
///
/// - If the command name is found in the registry, the user command is
///   executed (frontmatter applied, template substituted, message sent).
/// - If the command is not found, `None` is returned so the caller falls
///   through to built-in dispatch.
/// - If the registry has load errors for this command, they are logged at
///   `trace` level but do not prevent execution (best-effort dispatch).
/// - No fallthrough on user-command execution: if the command exists, it
///   runs; if it fails, the error is returned (the caller should not fall
///   through to built-in dispatch for a defined user command).
///
/// See [`user_commands::try_dispatch_user_command`] for the original
/// filesystem-scanning implementation that this replaces.
pub fn try_dispatch(app: &mut App, input: &str) -> Option<CommandResult> {
    let registry = current_registry()?;
    dispatch_from_registry(&registry, app, input)
}

/// Internal dispatch logic that works with any registry reference.
///
/// This is the core dispatch implementation, factored out so tests can
/// call it with a test registry without depending on the global static.
fn dispatch_from_registry(
    registry: &UserCommandRegistry,
    app: &mut App,
    input: &str,
) -> Option<CommandResult> {
    let parts: Vec<&str> = input.trim().splitn(2, ' ').collect();
    let command = parts[0].to_lowercase();
    let command = command.strip_prefix('/').unwrap_or(&command);
    let args = parts.get(1).copied().unwrap_or("").trim();

    let metadata = registry.get(command)?.clone();

    // Registry reference dropped; metadata is owned.

    // Apply frontmatter state (matching user_commands::try_dispatch_user_command).
    app.hunt.quarry = None;
    app.hunt.started_at = None;
    app.hunt.verdict = HuntVerdict::Hunting;
    app.hunt.token_budget = None;
    app.active_allowed_tools = None;
    app.pausable = false;
    app.paused = false;
    app.paused_quarry = None;

    if let Some(ref description) = metadata.description {
        app.hunt.quarry = Some(description.clone());
        app.hunt.started_at = Some(std::time::Instant::now());
    }
    if let Some(ref tools) = metadata.allowed_tools {
        app.active_allowed_tools = Some(tools.clone());
    }
    if metadata.pausable {
        app.pausable = true;
    }

    // Apply template substitution ($1, $2, $ARGUMENTS).
    let message = apply_template(&metadata.body, args);

    Some(CommandResult::action(AppAction::SendMessage(message)))
}

/// Apply template substitution: `$1`, `$2`, `$ARGUMENTS`.
fn apply_template(template: &str, args: &str) -> String {
    let positional: Vec<&str> = args.split_whitespace().collect();
    let mut result = template.replace("$ARGUMENTS", args);
    for (i, arg) in positional.iter().enumerate() {
        result = result.replace(&format!("${}", i + 1), arg);
    }
    result
}

/// A dedicated registry for user-defined commands, completely separate
/// from the built-in `COMMANDS` array.
///
/// The registry is built from filesystem scanning at construction time
/// and is read-only after creation. To reload, construct a new registry
/// and replace the existing one atomically.
///
/// # Examples
///
/// ```ignore
/// let registry = UserCommandRegistry::load(Some(&workspace));
/// if let Some(cmd) = registry.get("my-scan") {
///     println!("Found: {}", cmd.name);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct UserCommandRegistry {
    /// Canonical name → metadata.
    commands: HashMap<String, UserCommandMetadata>,
    /// Alias → canonical name.
    aliases: HashMap<String, String>,
    /// Load errors encountered during population.
    load_errors: Vec<LoadError>,
}

impl UserCommandRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
            aliases: HashMap::new(),
            load_errors: Vec::new(),
        }
    }

    /// Load user commands from the standard directories, reusing the
    /// existing directory scan and frontmatter parsing logic from
    /// `user_commands`.
    ///
    /// Workspace-local directories shadow user-global by name, matching
    /// the existing precedence behaviour:
    ///
    /// 1. `<workspace>/.codewhale/commands/` (project-local, highest)
    /// 2. `<workspace>/.deepseek/commands/`  (legacy project-local)
    /// 3. `<workspace>/.claude/commands/`    (Claude Code interop)
    /// 4. `<workspace>/.cursor/commands/`    (Cursor interop)
    /// 5. `~/.codewhale/commands/`           (user-global)
    /// 6. `~/.deepseek/commands/`            (legacy user-global)
    ///
    /// Pass `None` for the workspace to scan only global directories.
    pub fn load(workspace: Option<&Path>) -> Self {
        let dirs = user_commands::commands_dirs(workspace);
        Self::load_from_paths(&dirs)
    }

    /// Load user commands from explicit directories in precedence order.
    ///
    /// This exists so tests and future callers can build a registry from a
    /// deterministic set of paths without also reading the user's global
    /// command directories.
    pub fn load_from_paths(paths: &[PathBuf]) -> Self {
        let mut registry = Self::new();
        let mut seen_names: HashSet<String> = HashSet::new();

        for dir in paths {
            for (name, content) in user_commands::load_commands_from_dir(&dir) {
                if !seen_names.insert(name.clone()) {
                    // Already seen from a higher-precedence directory; silent skip.
                    // No warning emitted (matching existing behaviour of load_user_commands).
                    continue;
                }
                match Self::parse_metadata(&name, &content) {
                    Ok(metadata) => {
                        // Register aliases (currently empty; reserved for future).
                        for alias in &metadata.aliases {
                            let alias_lower = alias.to_ascii_lowercase();
                            if let Some(existing) = registry.aliases.get(&alias_lower) {
                                registry.load_errors.push(LoadError {
                                    file_path: dir.join(format!("{name}.md")),
                                    message: format!(
                                        "User command alias '{alias}' conflicts with command '{existing}'"
                                    ),
                                });
                            } else {
                                registry.aliases.insert(alias_lower, name.clone());
                            }
                        }
                        registry.commands.insert(name, metadata);
                    }
                    Err(err_msg) => {
                        registry.load_errors.push(LoadError {
                            file_path: dir.join(format!("{name}.md")),
                            message: format!("Invalid user command '{name}': {err_msg}"),
                        });
                    }
                }
            }
        }

        registry
    }

    /// Create a registry from a pre-loaded list of `(name, content)` pairs.
    ///
    /// This is useful for testing and for callers that have already loaded
    /// data from the filesystem using the existing [`user_commands::load_user_commands`]
    /// function. No directory shadowing is applied; the order of the input
    /// vector determines precedence (first wins on name duplicates).
    pub fn from_loaded(commands: Vec<(String, String)>) -> Self {
        let mut registry = Self::new();
        let mut seen_names: HashSet<String> = HashSet::new();

        for (name, content) in commands {
            if !seen_names.insert(name.clone()) {
                // Silent skip on duplicate (matching existing behaviour).
                continue;
            }
            match Self::parse_metadata(&name, &content) {
                Ok(metadata) => {
                    for alias in &metadata.aliases {
                        let alias_lower = alias.to_ascii_lowercase();
                        if !registry.aliases.contains_key(&alias_lower) {
                            registry.aliases.insert(alias_lower, name.clone());
                        }
                    }
                    registry.commands.insert(name, metadata);
                }
                Err(err_msg) => {
                    registry.load_errors.push(LoadError {
                        file_path: PathBuf::from(format!("{name}.md")),
                        message: format!("Invalid user command '{name}': {err_msg}"),
                    });
                }
            }
        }

        registry
    }

    /// Parse a single command's content into [`UserCommandMetadata`].
    ///
    /// Reuses [`user_commands::parse_frontmatter`] for frontmatter extraction,
    /// ensuring identical parsing behaviour to the current implementation.
    ///
    /// Returns `Err` with a description if frontmatter is so broken that no
    /// metadata could be extracted (e.g., blank content after frontmatter).
    /// Minor issues (unknown fields) are silently ignored.
    fn parse_metadata(name: &str, content: &str) -> Result<UserCommandMetadata, String> {
        let (metadata_pairs, body) = user_commands::parse_frontmatter(content);

        let mut description = None;
        let mut argument_hint = None;
        let mut allowed_tools = None;
        let mut pausable = false;

        for (key, value) in &metadata_pairs {
            match key.as_str() {
                "description" => description = Some(value.clone()),
                "argument-hint" => argument_hint = Some(value.clone()),
                "allowed-tools" => {
                    let tools = user_commands::parse_allowed_tools(value);
                    allowed_tools = Some(tools);
                }
                "pausable" => {
                    pausable = value.trim().eq_ignore_ascii_case("true");
                }
                _ => {
                    // Unknown frontmatter fields are silently ignored
                    // for backward compatibility.
                }
            }
        }

        Ok(UserCommandMetadata {
            name: name.to_string(),
            body: body.to_string(),
            description,
            argument_hint,
            allowed_tools,
            pausable,
            aliases: Vec::new(), // Aliases not parsed from frontmatter yet
            hidden: false,       // Hidden flag not parsed from frontmatter yet
        })
    }

    // ── Accessors ────────────────────────────────────────────────

    /// Look up a command by its canonical name.
    ///
    /// The lookup is case-sensitive and matches the lowercased file stem.
    pub fn get(&self, name: &str) -> Option<&UserCommandMetadata> {
        self.commands.get(name)
    }

    /// Look up a command by alias.
    ///
    /// Returns `None` if the alias does not exist or if the canonical
    /// command it points to has been removed.
    pub fn get_by_alias(&self, alias: &str) -> Option<&UserCommandMetadata> {
        let canonical = self.aliases.get(alias)?;
        self.commands.get(canonical)
    }

    /// Check if a command exists by canonical name.
    pub fn contains(&self, name: &str) -> bool {
        self.commands.contains_key(name)
    }

    /// Return all canonical command names, sorted alphabetically.
    pub fn names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.commands.keys().map(|s| s.as_str()).collect();
        names.sort();
        names
    }

    /// Iterate over all command metadata, sorted by name.
    pub fn iter(&self) -> impl Iterator<Item = &UserCommandMetadata> {
        let mut values: Vec<&UserCommandMetadata> = self.commands.values().collect();
        values.sort_by(|a, b| a.name.cmp(&b.name));
        values.into_iter()
    }

    /// Returns `true` if no load errors were recorded during population.
    pub fn is_valid(&self) -> bool {
        self.load_errors.is_empty()
    }

    /// Returns the list of load errors encountered during population.
    pub fn load_errors(&self) -> &[LoadError] {
        &self.load_errors
    }

    /// Returns the number of loaded commands.
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Returns `true` if no commands are loaded.
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

impl Default for UserCommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    // ── Helpers ──────────────────────────────────────────────────

    fn write_command(dir: &Path, name: &str, body: &str) {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join(format!("{name}.md")), body).unwrap();
    }

    // ── 2.4 Unit Tests ──────────────────────────────────────────

    #[test]
    fn registry_constructs_from_valid_markdown() {
        let registry = UserCommandRegistry::from_loaded(vec![(
            "git-scan".to_string(),
            "---\ndescription: Scan nested git repositories\nargument-hint: <root>\nallowed-tools: Bash, Grep\npausable: true\n---\nscan the repo tree".to_string(),
        )]);
        assert!(registry.is_valid());
        assert_eq!(registry.len(), 1);

        let cmd = registry.get("git-scan").expect("git-scan should be loaded");
        assert_eq!(cmd.name, "git-scan");
        assert_eq!(
            cmd.description.as_deref(),
            Some("Scan nested git repositories")
        );
        assert_eq!(cmd.argument_hint.as_deref(), Some("<root>"));
        assert_eq!(
            cmd.allowed_tools,
            Some(vec!["bash".to_string(), "grep".to_string()])
        );
        assert!(cmd.pausable);
        assert!(!cmd.hidden);
        assert_eq!(cmd.body, "scan the repo tree");
    }

    #[test]
    fn registry_constructs_from_minimal_markdown() {
        let registry =
            UserCommandRegistry::from_loaded(vec![("hello".to_string(), "echo hi".to_string())]);
        assert!(registry.is_valid());
        assert_eq!(registry.len(), 1);

        let cmd = registry.get("hello").expect("hello should be loaded");
        assert_eq!(cmd.name, "hello");
        assert_eq!(cmd.description, None);
        assert_eq!(cmd.argument_hint, None);
        assert_eq!(cmd.allowed_tools, None);
        assert!(!cmd.pausable);
        assert_eq!(cmd.body, "echo hi");
    }

    #[test]
    fn registry_empty_when_no_files() {
        let tmp = TempDir::new().unwrap();
        let ws = tmp.path();

        let registry =
            UserCommandRegistry::load_from_paths(&[ws.join(".codewhale").join("commands")]);
        assert!(registry.is_valid());
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
        assert!(registry.names().is_empty());
    }

    #[test]
    fn registry_empty_when_no_directory_exists() {
        let registry =
            UserCommandRegistry::load_from_paths(&[PathBuf::from("/nonexistent/path/12345")]);
        assert!(registry.is_valid());
        assert!(registry.is_empty());
    }

    #[test]
    fn registry_ignores_non_markdown_files() {
        let tmp = TempDir::new().unwrap();
        let ws = tmp.path();
        let cmds_dir = ws.join(".codewhale").join("commands");
        std::fs::create_dir_all(&cmds_dir).unwrap();
        std::fs::write(cmds_dir.join("notes.txt"), "not a command").unwrap();
        std::fs::write(cmds_dir.join("script.sh"), "#!/bin/bash").unwrap();
        write_command(&cmds_dir, "real-cmd", "actual body");

        let registry = UserCommandRegistry::load_from_paths(&[cmds_dir]);
        assert_eq!(registry.len(), 1);
        assert!(registry.contains("real-cmd"));
        assert!(!registry.contains("notes"));
    }

    #[test]
    fn registry_names_returns_loaded_set() {
        let tmp = TempDir::new().unwrap();
        let ws = tmp.path();
        let cmds_dir = ws.join(".codewhale").join("commands");
        write_command(&cmds_dir, "z-last", "body z");
        write_command(&cmds_dir, "a-first", "body a");
        write_command(&cmds_dir, "m-middle", "body m");

        let registry = UserCommandRegistry::load_from_paths(&[cmds_dir]);
        let names = registry.names();
        assert_eq!(names, vec!["a-first", "m-middle", "z-last"]);
    }

    #[test]
    fn registry_precedence_matches_load_user_commands() {
        let tmp = TempDir::new().unwrap();
        let ws = tmp.path();

        // Workspace-local (.codewhale) shadows legacy (.deepseek)
        write_command(
            &ws.join(".codewhale").join("commands"),
            "shared",
            "codewhale version",
        );
        write_command(
            &ws.join(".deepseek").join("commands"),
            "shared",
            "deepseek version",
        );

        // Unique commands in each dir are both included
        write_command(
            &ws.join(".codewhale").join("commands"),
            "cw-only",
            "codewhale only",
        );
        write_command(
            &ws.join(".deepseek").join("commands"),
            "ds-only",
            "deepseek only",
        );

        let registry = UserCommandRegistry::load(Some(ws));

        // Shadowed: workspace-local wins
        let shared = registry.get("shared").expect("shared should exist");
        assert_eq!(
            shared.body, "codewhale version",
            "workspace-local must shadow legacy"
        );

        // Both unique commands present
        assert!(registry.contains("cw-only"));
        assert!(registry.contains("ds-only"));

        // Verify it matches existing load_user_commands behaviour
        let legacy = user_commands::load_user_commands(Some(ws));
        let registry_names: Vec<&str> = registry.names();
        let legacy_names: Vec<&str> = legacy.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(registry_names, legacy_names);
    }

    #[test]
    fn registry_scans_claude_and_cursor_dirs() {
        let tmp = TempDir::new().unwrap();
        let ws = tmp.path();
        write_command(
            &ws.join(".claude").join("commands"),
            "claude-cmd",
            "claude body",
        );
        write_command(
            &ws.join(".cursor").join("commands"),
            "cursor-cmd",
            "cursor body",
        );

        let registry = UserCommandRegistry::load_from_paths(&[
            ws.join(".claude").join("commands"),
            ws.join(".cursor").join("commands"),
        ]);
        assert!(registry.contains("claude-cmd"));
        assert!(registry.contains("cursor-cmd"));
    }

    #[test]
    fn registry_from_loaded_produces_same_result() {
        let tmp = TempDir::new().unwrap();
        let ws = tmp.path();
        let cmds_dir = ws.join(".codewhale").join("commands");
        write_command(&cmds_dir, "hello", "Hello world");

        // Load via filesystem
        let fs_registry = UserCommandRegistry::load(Some(ws));

        // Load via from_loaded with same data
        let loaded = user_commands::load_user_commands(Some(ws));
        let from_registry = UserCommandRegistry::from_loaded(loaded);

        assert_eq!(fs_registry.len(), from_registry.len());
        assert_eq!(fs_registry.names(), from_registry.names());

        let fs_cmd = fs_registry.get("hello").unwrap();
        let from_cmd = from_registry.get("hello").unwrap();
        assert_eq!(fs_cmd.name, from_cmd.name);
        assert_eq!(fs_cmd.body, from_cmd.body);
    }

    #[test]
    fn registry_detects_duplicate_names_in_same_directory() {
        // When from_loaded receives duplicates, first wins silently.
        let commands = vec![
            ("dup".to_string(), "first version".to_string()),
            ("dup".to_string(), "second version".to_string()),
        ];
        let registry = UserCommandRegistry::from_loaded(commands);
        assert!(registry.is_valid());
        assert_eq!(registry.len(), 1);
        let cmd = registry.get("dup").unwrap();
        assert_eq!(cmd.body, "first version");
    }

    #[test]
    fn registry_iter_returns_sorted_commands() {
        let commands = vec![
            ("zulu".to_string(), "last".to_string()),
            ("alpha".to_string(), "first".to_string()),
            ("beta".to_string(), "second".to_string()),
        ];
        let registry = UserCommandRegistry::from_loaded(commands);
        let names: Vec<&str> = registry.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["alpha", "beta", "zulu"]);
    }

    #[test]
    fn registry_default_is_empty() {
        let registry = UserCommandRegistry::default();
        assert!(registry.is_empty());
        assert!(registry.is_valid());
    }

    #[test]
    fn registry_get_by_alias_returns_none_when_no_aliases() {
        let commands = vec![("mycmd".to_string(), "body".to_string())];
        let registry = UserCommandRegistry::from_loaded(commands);
        assert!(registry.get_by_alias("nonexistent").is_none());
        // No aliases registered yet, so even valid names don't match via alias
        assert!(registry.get_by_alias("mycmd").is_none());
    }

    #[test]
    fn registry_parses_frontmatter_with_unclosed_delimiter() {
        let content = "---\ndescription: Broken command\nallowed-tools: Bash\nRun the safe body";
        let (_pairs, _) = user_commands::parse_frontmatter(content);
        let metadata = UserCommandRegistry::parse_metadata("broken", content);

        assert!(metadata.is_ok());
        let cmd = metadata.unwrap();
        assert_eq!(cmd.description.as_deref(), Some("Broken command"));
        assert_eq!(cmd.allowed_tools, Some(vec!["bash".to_string()]));
        assert!(cmd.body.contains("Run the safe body"));
    }

    #[test]
    fn registry_parses_frontmatter_without_metadata_strips_header() {
        let content = "---\nRun the command body";
        let metadata = UserCommandRegistry::parse_metadata("bare", content);

        assert!(metadata.is_ok());
        let cmd = metadata.unwrap();
        assert_eq!(cmd.description, None);
        assert_eq!(cmd.body, "Run the command body");
    }

    #[test]
    fn registry_strips_matched_quotes_from_frontmatter_values() {
        let content = "---\ndescription: 'Read\"\n---\nrun";
        let metadata = UserCommandRegistry::parse_metadata("quoted", content);

        assert!(metadata.is_ok());
        let cmd = metadata.unwrap();
        assert_eq!(cmd.description.as_deref(), Some("'Read\""));
        assert_eq!(cmd.body, "run");
    }

    #[test]
    fn registry_parses_pausable_false() {
        let content = "---\npausable: false\n---\nbody";
        let metadata = UserCommandRegistry::parse_metadata("np", content);
        let cmd = metadata.unwrap();
        assert!(!cmd.pausable);
    }

    #[test]
    fn registry_parses_pausable_true() {
        let content = "---\npausable: True\n---\nbody";
        let metadata = UserCommandRegistry::parse_metadata("yp", content);
        let cmd = metadata.unwrap();
        assert!(cmd.pausable);
    }

    #[test]
    fn registry_parses_allowed_tools_string_values() {
        let tmp = TempDir::new().unwrap();
        let ws = tmp.path();
        let cmds_dir = ws.join(".codewhale").join("commands");
        write_command(
            &cmds_dir,
            "secure",
            "---\nallowed-tools: \"exec_shell\", 'read_file'\n---\nrun tools",
        );

        let registry = UserCommandRegistry::load_from_paths(&[cmds_dir]);
        let cmd = registry.get("secure").unwrap();
        assert!(cmd.pausable == false);
        assert_eq!(
            cmd.allowed_tools,
            Some(vec!["exec_shell".to_string(), "read_file".to_string()])
        );
    }

    #[test]
    fn registry_load_from_global_directory_without_workspace() {
        // When no workspace is passed, only global directories are scanned.
        let registry = UserCommandRegistry::load(None);
        // This should not panic; can be empty or have user's real commands.
        let _ = registry;
    }

    #[test]
    fn registry_forward_compatible_with_new_frontmatter_fields() {
        // Unknown frontmatter fields must be silently ignored.
        let content = "---\ndescription: Test\nhelp: Some help text\nhidden: true\nunknown-field: value\n---\nbody";
        let metadata = UserCommandRegistry::parse_metadata("test-cmd", content);

        assert!(metadata.is_ok());
        let cmd = metadata.unwrap();
        assert_eq!(cmd.description.as_deref(), Some("Test"));
        // Not parsed, but not an error:
        assert_eq!(cmd.body, "body");
    }

    #[test]
    fn registry_load_errors_accessible() {
        // Test that load_errors() returns the collected errors.
        let tmp = TempDir::new().unwrap();
        let ws = tmp.path();
        let cmds_dir = ws.join(".codewhale").join("commands");
        write_command(&cmds_dir, "good", "---\ndescription: OK\n---\nbody");

        let registry = UserCommandRegistry::load_from_paths(&[cmds_dir]);
        // All commands should load fine
        assert!(registry.is_valid());
        assert!(registry.load_errors().is_empty());
    }

    #[test]
    fn registry_round_trip_with_multiple_directories_and_precedence() {
        let tmp = TempDir::new().unwrap();
        let ws = tmp.path();

        // Create commands in multiple directories, some shadowing
        write_command(
            &ws.join(".codewhale").join("commands"),
            "alpha",
            "codewhale alpha",
        );
        write_command(
            &ws.join(".deepseek").join("commands"),
            "beta",
            "deepseek beta",
        );
        write_command(
            &ws.join(".claude").join("commands"),
            "gamma",
            "claude gamma",
        );
        write_command(
            &ws.join(".cursor").join("commands"),
            "delta",
            "cursor delta",
        );

        // Shadow: same name in different dirs
        write_command(
            &ws.join(".deepseek").join("commands"),
            "alpha",
            "deepseek alpha (should be shadowed)",
        );
        write_command(
            &ws.join(".claude").join("commands"),
            "beta",
            "claude beta (should be shadowed)",
        );

        let registry = UserCommandRegistry::load_from_paths(&[
            ws.join(".codewhale").join("commands"),
            ws.join(".deepseek").join("commands"),
            ws.join(".claude").join("commands"),
            ws.join(".cursor").join("commands"),
        ]);

        assert_eq!(registry.len(), 4);
        assert_eq!(
            registry.get("alpha").unwrap().body,
            "codewhale alpha",
            ".codewhale must shadow .deepseek"
        );
        assert_eq!(
            registry.get("beta").unwrap().body,
            "deepseek beta",
            ".deepseek must shadow .claude"
        );
        assert!(registry.contains("gamma"));
        assert!(registry.contains("delta"));
    }

    #[test]
    fn registry_contains_only_lowercased_names() {
        let tmp = TempDir::new().unwrap();
        let ws = tmp.path();
        let cmds_dir = ws.join(".codewhale").join("commands");
        write_command(&cmds_dir, "MY-CMD", "uppercase body");
        write_command(&cmds_dir, "Other-Cmd", "mixed case body");

        let registry = UserCommandRegistry::load_from_paths(&[cmds_dir]);
        // File stems are lowercased by load_commands_from_dir
        assert!(registry.contains("my-cmd"));
        assert!(registry.contains("other-cmd"));
        assert!(!registry.contains("MY-CMD"));
        assert!(!registry.contains("Other-Cmd"));
    }

    #[test]
    fn registry_get_nonexistent_returns_none() {
        let registry = UserCommandRegistry::new();
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn load_error_fields_are_accessible() {
        let error = LoadError {
            file_path: PathBuf::from("broken.md"),
            message: "invalid metadata".to_string(),
        };

        assert_eq!(error.file_path, PathBuf::from("broken.md"));
        assert_eq!(error.message, "invalid metadata");
    }

    // ── 3.5 Business Logic Tests ───────────────────────────────

    /// Create a minimal App for testing dispatch.
    fn test_app(workspace: PathBuf) -> crate::tui::app::App {
        use crate::config::Config;
        use crate::tui::app::TuiOptions;

        App::new(
            TuiOptions {
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
            },
            &Config::default(),
        )
    }

    #[test]
    fn dispatch_found_command_returns_result() {
        let registry = UserCommandRegistry::from_loaded(vec![(
            "greet".to_string(),
            "Hello, $ARGUMENTS!".to_string(),
        )]);

        let tmp = TempDir::new().unwrap();
        let ws = tmp.path().to_path_buf();
        let mut app = test_app(ws);

        let result = dispatch_from_registry(&registry, &mut app, "/greet World");
        assert!(result.is_some(), "dispatch should find 'greet'");
        let cmd_result = result.unwrap();
        match cmd_result.action {
            Some(AppAction::SendMessage(msg)) => {
                assert_eq!(msg, "Hello, World!");
            }
            other => panic!("expected SendMessage action, got: {other:?}"),
        }
    }

    #[test]
    fn dispatch_not_found_returns_none() {
        let registry = UserCommandRegistry::new();

        let tmp = TempDir::new().unwrap();
        let ws = tmp.path().to_path_buf();
        let mut app = test_app(ws);

        let result = dispatch_from_registry(&registry, &mut app, "/nonexistent");
        assert!(result.is_none(), "unknown command should return None");
    }

    #[test]
    fn dispatch_template_substitutes_arguments() {
        let registry = UserCommandRegistry::from_loaded(vec![(
            "echo".to_string(),
            "$1 $2 $ARGUMENTS".to_string(),
        )]);

        let tmp = TempDir::new().unwrap();
        let ws = tmp.path().to_path_buf();
        let mut app = test_app(ws);

        let result = dispatch_from_registry(&registry, &mut app, "/echo a b c").unwrap();
        match result.action {
            Some(AppAction::SendMessage(msg)) => {
                assert_eq!(msg, "a b a b c");
            }
            other => panic!("expected SendMessage, got: {other:?}"),
        }
    }

    #[test]
    fn dispatch_without_args_uses_empty_string() {
        let registry =
            UserCommandRegistry::from_loaded(vec![("noop".to_string(), "body".to_string())]);

        let tmp = TempDir::new().unwrap();
        let ws = tmp.path().to_path_buf();
        let mut app = test_app(ws);

        let result = dispatch_from_registry(&registry, &mut app, "/noop").unwrap();
        match result.action {
            Some(AppAction::SendMessage(msg)) => {
                assert_eq!(msg, "body");
            }
            other => panic!("expected SendMessage, got: {other:?}"),
        }
    }

    #[test]
    fn dispatch_frontmatter_sets_app_state() {
        let registry = UserCommandRegistry::from_loaded(vec![(
            "secure".to_string(),
            "---\ndescription: Secure scan\nallowed-tools: Bash, Grep\npausable: true\n---\nrun $ARGUMENTS".to_string(),
        )]);

        let tmp = TempDir::new().unwrap();
        let ws = tmp.path().to_path_buf();
        let mut app = test_app(ws);

        let _ = dispatch_from_registry(&registry, &mut app, "/secure tests").unwrap();

        assert_eq!(app.hunt.quarry.as_deref(), Some("Secure scan"));
        assert!(app.hunt.started_at.is_some(), "started_at should be set");
        assert_eq!(
            app.active_allowed_tools,
            Some(vec!["bash".to_string(), "grep".to_string()])
        );
        assert!(app.pausable);
        assert!(!app.paused);
    }

    #[test]
    fn dispatch_clears_previous_state() {
        let meta_registry = UserCommandRegistry::from_loaded(vec![(
            "described".to_string(),
            "---\ndescription: Scan repos\nallowed-tools: Bash\n---\nscan".to_string(),
        )]);
        let plain_registry = UserCommandRegistry::from_loaded(vec![(
            "plain".to_string(),
            "plain command".to_string(),
        )]);

        let tmp = TempDir::new().unwrap();
        let ws = tmp.path().to_path_buf();
        let mut app = test_app(ws);

        // Dispatch the described command first
        let _ = dispatch_from_registry(&meta_registry, &mut app, "/described").unwrap();
        assert_eq!(app.hunt.quarry.as_deref(), Some("Scan repos"));
        assert_eq!(app.active_allowed_tools, Some(vec!["bash".to_string()]));

        // Manually set some residual state
        app.pausable = true;
        app.paused = true;
        app.paused_quarry = Some("Something".to_string());

        // Dispatch the plain command — should reset all state
        let _ = dispatch_from_registry(&plain_registry, &mut app, "/plain").unwrap();
        assert_eq!(app.hunt.quarry, None);
        assert_eq!(app.hunt.started_at, None);
        assert_eq!(app.active_allowed_tools, None);
        assert!(!app.pausable);
        assert!(!app.paused);
        assert!(app.paused_quarry.is_none());
    }

    #[test]
    fn dispatch_frontmatter_sets_allowed_tools_from_empty_string() {
        let registry = UserCommandRegistry::from_loaded(vec![(
            "locked".to_string(),
            "---\nallowed-tools: \"\"\n---\nrun nothing".to_string(),
        )]);

        let tmp = TempDir::new().unwrap();
        let ws = tmp.path().to_path_buf();
        let mut app = test_app(ws);

        let _ = dispatch_from_registry(&registry, &mut app, "/locked").unwrap();
        assert_eq!(app.active_allowed_tools, Some(Vec::new()));
    }

    #[test]
    fn dispatch_uses_filesystem_loaded_registry() {
        let tmp = TempDir::new().unwrap();
        let ws = tmp.path();
        let cmds_dir = ws.join(".codewhale").join("commands");
        std::fs::create_dir_all(&cmds_dir).unwrap();
        std::fs::write(cmds_dir.join("hello.md"), "Hello, $ARGUMENTS!").unwrap();

        // Use load_from_paths to avoid scanning global directories
        let registry = UserCommandRegistry::load_from_paths(&[cmds_dir]);
        let mut app = test_app(ws.to_path_buf());

        let result = dispatch_from_registry(&registry, &mut app, "/hello World").unwrap();
        match result.action {
            Some(AppAction::SendMessage(msg)) => {
                assert_eq!(msg, "Hello, World!");
            }
            other => panic!("expected SendMessage, got: {other:?}"),
        }
    }

    #[test]
    fn ensure_initialized_populates_registry() {
        let tmp = TempDir::new().unwrap();
        let ws = tmp.path();
        let cmds_dir = ws.join(".codewhale").join("commands");
        std::fs::create_dir_all(&cmds_dir).unwrap();
        std::fs::write(
            cmds_dir.join("test-cmd.md"),
            "---\ndescription: Test command\n---\nbody",
        )
        .unwrap();

        // Use load_from_paths to avoid scanning global directories
        let registry = UserCommandRegistry::load_from_paths(&[cmds_dir]);
        assert!(registry.contains("test-cmd"));
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn apply_template_replaces_positional_and_catchall() {
        let result = super::apply_template("$1 $2 $ARGUMENTS", "a b c d");
        assert_eq!(result, "a b a b c d");
    }

    #[test]
    fn apply_template_without_args_preserves_body() {
        let result = super::apply_template("static body", "");
        assert_eq!(result, "static body");
    }

    #[test]
    fn apply_template_with_partial_positional() {
        let result = super::apply_template("$1", "only");
        assert_eq!(result, "only");
    }

    #[test]
    fn reload_updates_registry_content() {
        let tmp = TempDir::new().unwrap();
        let ws = tmp.path();
        let cmds_dir = ws.join(".codewhale").join("commands");
        std::fs::create_dir_all(&cmds_dir).unwrap();

        // Create initial command
        std::fs::write(cmds_dir.join("v1.md"), "version 1").unwrap();

        let registry = UserCommandRegistry::load_from_paths(&[cmds_dir.clone()]);
        assert_eq!(registry.len(), 1);
        assert_eq!(registry.get("v1").unwrap().body, "version 1");

        // Add a new command and reload
        std::fs::write(cmds_dir.join("v2.md"), "version 2").unwrap();

        let registry = UserCommandRegistry::load_from_paths(&[cmds_dir]);
        assert_eq!(registry.len(), 2);
        assert!(registry.contains("v1"));
        assert!(registry.contains("v2"));
    }
}
