//! `/save` command — save the current session to a file.

use std::path::PathBuf;

use crate::session_manager::create_saved_session_with_mode;
use crate::tui::app::App;

use super::CommandResult;

/// Save session to file.
///
/// When an explicit path is given, the session is exported there
/// (user-visible explicit export).  Without a path, v0.8.44 saves
/// into the managed session directory (`~/.codewhale/sessions`
/// or legacy `~/.deepseek/sessions`) so repo-local `session_*.json`
/// artifacts are no longer created by default.
pub fn save(app: &mut App, path: Option<&str>) -> CommandResult {
    let save_path = if let Some(p) = path {
        PathBuf::from(p)
    } else {
        let dir = crate::session_manager::default_sessions_dir()
            .unwrap_or_else(|_| app.workspace.clone());
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        dir.join(format!("session_{timestamp}.json"))
    };

    let messages = app.api_messages.clone();
    let mut session = create_saved_session_with_mode(
        &messages,
        &app.model,
        &app.workspace,
        u64::from(app.session.total_tokens),
        app.system_prompt.as_ref(),
        Some(app.mode.label()),
    );
    app.sync_cost_to_metadata(&mut session.metadata);
    session.artifacts = app.session_artifacts.clone();

    let sessions_dir = save_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map_or_else(|| app.workspace.clone(), std::path::Path::to_path_buf);

    match std::fs::create_dir_all(&sessions_dir) {
        Ok(()) => {
            let json = match serde_json::to_string_pretty(&session) {
                Ok(j) => j,
                Err(e) => return CommandResult::error(format!("Failed to serialize session: {e}")),
            };
            match std::fs::write(&save_path, json) {
                Ok(()) => {
                    app.current_session_id = Some(session.metadata.id.clone());
                    CommandResult::message(format!(
                        "Session saved to {} (ID: {})",
                        save_path.display(),
                        crate::session_manager::truncate_id(&session.metadata.id)
                    ))
                }
                Err(e) => CommandResult::error(format!("Failed to save session: {e}")),
            }
        }
        Err(e) => CommandResult::error(format!("Failed to create directory: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::test_support::EnvVarGuard;
    use crate::tui::app::{App, TuiOptions};
    use tempfile::TempDir;

    fn create_test_app_with_tmpdir(tmpdir: &TempDir) -> App {
        let options = TuiOptions {
            model: "deepseek-v4-pro".to_string(),
            workspace: tmpdir.path().to_path_buf(),
            config_path: None,
            config_profile: None,
            allow_shell: false,
            use_alt_screen: true,
            use_mouse_capture: false,
            use_bracketed_paste: true,
            max_subagents: 1,
            skills_dir: tmpdir.path().join("skills"),
            memory_path: tmpdir.path().join("memory.md"),
            notes_path: tmpdir.path().join("notes.txt"),
            mcp_config_path: tmpdir.path().join("mcp.json"),
            use_memory: false,
            start_in_agent_mode: false,
            skip_onboarding: true,
            yolo: false,
            resume_session_id: None,
            initial_input: None,
        };
        App::new(options, &Config::default())
    }

    #[test]
    fn test_save_creates_file_and_sets_session_id() {
        let tmpdir = TempDir::new().unwrap();
        let mut app = create_test_app_with_tmpdir(&tmpdir);
        let save_path = tmpdir.path().join("test_session.json");

        let result = save(&mut app, Some(save_path.to_str().unwrap()));
        assert!(result.message.is_some());
        let msg = result.message.unwrap();
        assert!(msg.contains("Session saved to"));
        assert!(msg.contains("ID:"));
        assert!(app.current_session_id.is_some());
        assert!(save_path.exists());
    }

    #[test]
    fn save_preserves_artifact_registry() {
        let tmpdir = TempDir::new().unwrap();
        let mut app = create_test_app_with_tmpdir(&tmpdir);
        let save_path = tmpdir.path().join("artifact_session.json");
        app.session_artifacts
            .push(crate::artifacts::ArtifactRecord {
                id: "art_call_big".to_string(),
                kind: crate::artifacts::ArtifactKind::ToolOutput,
                session_id: "artifact-session".to_string(),
                tool_call_id: "call-big".to_string(),
                tool_name: "exec_shell".to_string(),
                created_at: chrono::Utc::now(),
                byte_size: 512_000,
                preview: "cargo test output".to_string(),
                storage_path: tmpdir.path().join("call-big.txt"),
            });

        let result = save(&mut app, Some(save_path.to_str().unwrap()));

        assert!(!result.is_error);
        let saved: crate::session_manager::SavedSession =
            serde_json::from_str(&std::fs::read_to_string(save_path).unwrap()).unwrap();
        assert_eq!(saved.artifacts, app.session_artifacts);
    }

    #[test]
    fn test_save_with_default_path_uses_managed_sessions_dir() {
        let tmpdir = TempDir::new().unwrap();
        let _lock = crate::test_support::lock_test_env();
        // Set CODEWHALE_HOME so the managed sessions directory lands inside the
        // temp dir rather than the real user home. Pre-create the directory so
        // resolve_state_dir picks it up instead of falling back to legacy.
        let home = tmpdir.path().join("home");
        let sessions_dir = home.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();
        let codewhale_home = EnvVarGuard::set("CODEWHALE_HOME", &home);
        let previous_codewhale_home = codewhale_home.previous();
        let mut app = create_test_app_with_tmpdir(&tmpdir);
        let result = save(&mut app, None);
        assert!(result.message.is_some());
        let msg = result.message.unwrap();
        // Give it a moment to ensure file is written
        std::thread::sleep(std::time::Duration::from_millis(10));
        let entries: Vec<_> = if sessions_dir.exists() {
            std::fs::read_dir(&sessions_dir)
                .unwrap()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_name().to_string_lossy().starts_with("session_"))
                .collect()
        } else {
            Vec::new()
        };
        drop(codewhale_home);
        // Session should be saved to the managed dir, not the workspace root.
        assert!(
            !entries.is_empty(),
            "expected session file in {sessions_dir:?}, got none; msg: {msg}"
        );
        assert_eq!(std::env::var_os("CODEWHALE_HOME"), previous_codewhale_home);
    }
}
