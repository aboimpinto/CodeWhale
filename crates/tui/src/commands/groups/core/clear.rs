//! `/clear` command — reset the active conversation.

use crate::tui::app::{App, AppAction};

use super::CommandResult;

/// Clear conversation history
pub fn clear(app: &mut App) -> CommandResult {
    let todos_cleared = reset_conversation_state(app);
    app.current_session_id = None;
    let locale = app.ui_locale;
    let message = if todos_cleared {
        crate::localization::tr(locale, crate::localization::MessageId::ClearConversation)
            .to_string()
    } else {
        crate::localization::tr(locale, crate::localization::MessageId::ClearConversationBusy)
            .to_string()
    };
    CommandResult::with_message_and_action(
        message,
        AppAction::SyncSession {
            session_id: None,
            messages: Vec::new(),
            system_prompt: None,
            model: app.model.clone(),
            workspace: app.workspace.clone(),
        },
    )
}

/// Reset the active conversation without choosing the next session id.
pub(crate) fn reset_conversation_state(app: &mut App) -> bool {
    app.clear_history();
    app.mark_history_updated();
    app.api_messages.clear();
    app.system_prompt = None;
    app.viewport.transcript_selection.clear();
    app.queued_messages.clear();
    app.queued_draft = None;
    app.session.total_tokens = 0;
    app.session.total_conversation_tokens = 0;
    app.session.reset_token_breakdown();
    app.session.session_cost = 0.0;
    app.session.session_cost_cny = 0.0;
    app.session.subagent_cost = 0.0;
    app.session.subagent_cost_cny = 0.0;
    app.session.subagent_cost_event_seqs.clear();
    app.session.displayed_cost_high_water = 0.0;
    app.session.displayed_cost_high_water_cny = 0.0;
    let todos_cleared = app.clear_todos();
    app.tool_log.clear();
    app.tool_cells.clear();
    app.tool_details_by_cell.clear();
    app.exploring_entries.clear();
    app.ignored_tool_calls.clear();
    app.pending_tool_uses.clear();
    app.last_exec_wait_command = None;
    app.session.last_prompt_tokens = None;
    app.session.last_completion_tokens = None;
    app.session.last_prompt_cache_hit_tokens = None;
    app.session.last_prompt_cache_miss_tokens = None;
    app.session.last_reasoning_replay_tokens = None;
    app.session.turn_cache_history.clear();
    app.session.last_cache_inspection = None;
    app.session.last_warmup_key = None;
    app.session.last_tool_catalog = None;
    app.session.last_base_url = None;
    todos_cleared
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::PromptInspection;
    use crate::config::Config;
    use crate::models::Message;
    use crate::tui::app::{App, TuiOptions, TurnCacheRecord};
    use crate::tui::history::HistoryCell;
    use std::path::PathBuf;
    use std::time::Instant;
    fn create_test_app() -> App {
        let options = TuiOptions {
            model: "deepseek-v4-pro".to_string(),
            workspace: PathBuf::from("/tmp/test-workspace"),
            config_path: None,
            config_profile: None,
            allow_shell: false,
            use_alt_screen: true,
            use_mouse_capture: false,
            use_bracketed_paste: true,
            max_subagents: 1,
            skills_dir: PathBuf::from("/tmp/test-skills"),
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
        app.ui_locale = crate::localization::Locale::En;
        app.api_provider = crate::config::ApiProvider::Deepseek;
        app.model = "deepseek-v4-pro".to_string();
        app.auto_model = false;
        app.model_ids_passthrough = false;
        app
    }

    #[test]
    fn test_clear_resets_all_state() {
        let mut app = create_test_app();
        // Set up some state
        app.history.push(HistoryCell::User {
            content: "test".to_string(),
        });
        app.api_messages.push(Message {
            role: "user".to_string(),
            content: vec![],
        });
        app.session.total_conversation_tokens = 100;
        app.tool_log.push("test".to_string());
        app.current_session_id = Some("existing-session".to_string());
        app.session_artifacts
            .push(crate::artifacts::ArtifactRecord {
                id: "art_call_big".to_string(),
                kind: crate::artifacts::ArtifactKind::ToolOutput,
                session_id: "existing-session".to_string(),
                tool_call_id: "call-big".to_string(),
                tool_name: "exec_shell".to_string(),
                created_at: chrono::Utc::now(),
                byte_size: 128,
                preview: "tool output".to_string(),
                storage_path: PathBuf::from("/tmp/tool_outputs/call-big.txt"),
            });

        let result = clear(&mut app);
        assert!(result.message.is_some());
        assert!(app.history.is_empty());
        assert!(app.api_messages.is_empty());
        assert_eq!(app.session.total_conversation_tokens, 0);
        assert!(app.tool_log.is_empty());
        assert!(app.tool_cells.is_empty());
        assert!(app.tool_details_by_cell.is_empty());
        assert!(app.session_artifacts.is_empty());
        assert!(app.current_session_id.is_none());
        assert!(matches!(result.action, Some(AppAction::SyncSession { .. })));
    }

    #[test]
    fn clear_resets_session_telemetry() {
        let mut app = create_test_app();
        app.session.total_tokens = 234;
        app.session.total_conversation_tokens = 123;
        app.session.session_cost = 0.42;
        app.session.session_cost_cny = 3.05;
        app.session.subagent_cost = 0.11;
        app.session.subagent_cost_cny = 0.80;
        app.session.subagent_cost_event_seqs.insert(7);
        app.session.displayed_cost_high_water = 0.53;
        app.session.displayed_cost_high_water_cny = 3.85;
        app.session.last_prompt_cache_hit_tokens = Some(70);
        app.session.last_prompt_cache_miss_tokens = Some(30);
        app.session.last_reasoning_replay_tokens = Some(12);
        app.session.last_warmup_key = None;
        app.session.last_tool_catalog = Some(Vec::new());
        app.session.last_base_url = Some("https://api.deepseek.com".to_string());
        app.session.last_cache_inspection = Some(PromptInspection {
            base_static_prefix_hash: "base".to_string(),
            full_request_prefix_hash: "full".to_string(),
            tool_catalog_hash: String::new(),
            layers: Vec::new(),
        });
        app.push_turn_cache_record(TurnCacheRecord {
            input_tokens: 100,
            output_tokens: 25,
            cache_hit_tokens: Some(70),
            cache_miss_tokens: Some(30),
            reasoning_replay_tokens: Some(12),
            recorded_at: Instant::now(),
        });

        clear(&mut app);

        assert_eq!(app.session.total_tokens, 0);
        assert_eq!(app.session.total_conversation_tokens, 0);
        assert_eq!(app.session.session_cost, 0.0);
        assert_eq!(app.session.session_cost_cny, 0.0);
        assert_eq!(app.session.subagent_cost, 0.0);
        assert_eq!(app.session.subagent_cost_cny, 0.0);
        assert!(app.session.subagent_cost_event_seqs.is_empty());
        assert_eq!(app.session.displayed_cost_high_water, 0.0);
        assert_eq!(app.session.displayed_cost_high_water_cny, 0.0);
        assert_eq!(app.session.last_prompt_cache_hit_tokens, None);
        assert_eq!(app.session.last_prompt_cache_miss_tokens, None);
        assert_eq!(app.session.last_reasoning_replay_tokens, None);
        assert!(app.session.turn_cache_history.is_empty());
        assert_eq!(app.session.last_cache_inspection, None);
        assert_eq!(app.session.last_warmup_key, None);
        assert_eq!(app.session.last_tool_catalog, None);
        assert_eq!(app.session.last_base_url, None);
    }
}
