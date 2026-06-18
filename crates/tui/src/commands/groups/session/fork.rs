//! `/fork` command — fork the active conversation into a new saved sibling session.

use crate::session_manager::{
    SessionManager, create_saved_session_with_id_and_mode, create_saved_session_with_mode,
};
use crate::tui::app::{App, AppAction};

use super::CommandResult;

/// Fork the active conversation into a new saved sibling session and switch to it.
pub fn fork(app: &mut App) -> CommandResult {
    if app.api_messages.is_empty() {
        return CommandResult::error("Nothing to fork. Send or load a message first.");
    }

    let manager = match SessionManager::default_location() {
        Ok(manager) => manager,
        Err(err) => {
            return CommandResult::error(format!("could not open sessions directory: {err}"));
        }
    };

    let parent_id = app
        .current_session_id
        .clone()
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let mut parent = create_saved_session_with_id_and_mode(
        parent_id,
        &app.api_messages,
        &app.model,
        &app.workspace,
        u64::from(app.session.total_tokens),
        app.system_prompt.as_ref(),
        Some(app.mode.label()),
    );
    app.sync_cost_to_metadata(&mut parent.metadata);
    parent.artifacts = app.session_artifacts.clone();

    if let Err(err) = manager.save_session(&parent) {
        return CommandResult::error(format!("Failed to save parent session: {err}"));
    }

    let mut forked = create_saved_session_with_mode(
        &app.api_messages,
        &app.model,
        &app.workspace,
        u64::from(app.session.total_tokens),
        app.system_prompt.as_ref(),
        Some(app.mode.label()),
    );
    forked.metadata.copy_cost_from(&parent.metadata);
    forked.metadata.mark_forked_from(&parent.metadata);

    if let Err(err) = manager.save_session(&forked) {
        return CommandResult::error(format!("Failed to save forked session: {err}"));
    }

    app.current_session_id = Some(forked.metadata.id.clone());
    let fork_id = forked.metadata.id.clone();
    let parent_label = crate::session_manager::truncate_id(&parent.metadata.id).to_string();
    let fork_label = crate::session_manager::truncate_id(&fork_id).to_string();

    CommandResult::with_message_and_action(
        format!("Forked session {parent_label} -> {fork_label}"),
        AppAction::SyncSession {
            session_id: Some(fork_id),
            messages: app.api_messages.clone(),
            system_prompt: app.system_prompt.clone(),
            model: app.model.clone(),
            workspace: app.workspace.clone(),
        },
    )
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
    fn fork_saves_parent_and_switches_to_child_session() {
        let tmpdir = TempDir::new().unwrap();
        let _lock = crate::test_support::lock_test_env();
        let home = tmpdir.path().join("home");
        std::fs::create_dir_all(&home).unwrap();
        let home_guard = EnvVarGuard::set("HOME", &home);
        let previous_home = home_guard.previous();
        let mut app = create_test_app_with_tmpdir(&tmpdir);
        app.current_session_id = Some("parent-session".to_string());
        app.api_messages.push(crate::models::Message {
            role: "user".to_string(),
            content: vec![crate::models::ContentBlock::Text {
                text: "try another path".to_string(),
                cache_control: None,
            }],
        });

        let result = fork(&mut app);

        assert!(!result.is_error, "{:?}", result.message);
        let new_id = app.current_session_id.clone().expect("fork session id");
        assert_ne!(new_id, "parent-session");
        assert!(result.message.as_deref().unwrap_or("").contains("Forked"));
        assert!(matches!(result.action, Some(AppAction::SyncSession { .. })));

        let manager = SessionManager::default_location().unwrap();
        let parent = manager
            .load_session("parent-session")
            .expect("parent saved");
        let child = manager.load_session(&new_id).expect("child saved");
        assert_eq!(parent.messages.len(), 1);
        assert_eq!(
            child.metadata.parent_session_id.as_deref(),
            Some("parent-session")
        );
        assert_eq!(child.metadata.forked_from_message_count, Some(1));
        drop(home_guard);
        assert_eq!(std::env::var_os("HOME"), previous_home);
    }
}
