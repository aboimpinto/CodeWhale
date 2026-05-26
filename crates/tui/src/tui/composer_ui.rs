use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::tui::app::App;

const COMPOSER_ARROW_SCROLL_LINES: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EscapeAction {
    CloseSlashMenu,
    CancelRequest,
    PauseCommand,
    DiscardQueuedDraft,
    ClearInput,
    Noop,
}

pub(crate) fn next_escape_action(app: &App, slash_menu_open: bool) -> EscapeAction {
    if slash_menu_open {
        EscapeAction::CloseSlashMenu
    } else if app.is_loading {
        if app.pausable && !app.paused {
            EscapeAction::PauseCommand
        } else if app.pausable && app.paused {
            // Cancel request from second ESC needs debounce
            // to avoid terminal key-repeat overwriting pause message.
            const PAUSE_DEBOUNCE_MS: u128 = 300;
            let enough_time = app.paused_at
                .map(|t| t.elapsed().as_millis() >= PAUSE_DEBOUNCE_MS)
                .unwrap_or(true);
            if enough_time {
                EscapeAction::CancelRequest
            } else {
                EscapeAction::Noop
            }
        } else {
            EscapeAction::CancelRequest
        }
    } else if app.queued_draft.is_some() && app.input.is_empty() {
        EscapeAction::DiscardQueuedDraft
    } else if !app.input.is_empty() {
        EscapeAction::ClearInput
    } else {
        EscapeAction::Noop
    }
}

pub(crate) fn select_previous_slash_menu_entry(app: &mut App, entry_count: usize) {
    if entry_count == 0 {
        return;
    }
    let selected = app.slash_menu_selected.min(entry_count.saturating_sub(1));
    app.slash_menu_selected = (selected + entry_count - 1) % entry_count;
}

pub(crate) fn select_next_slash_menu_entry(app: &mut App, entry_count: usize) {
    if entry_count == 0 {
        return;
    }
    let selected = app.slash_menu_selected.min(entry_count.saturating_sub(1));
    app.slash_menu_selected = (selected + 1) % entry_count;
}

pub(crate) fn handle_composer_history_arrow(
    app: &mut App,
    key: KeyEvent,
    slash_menu_open: bool,
    mention_menu_open: bool,
) -> bool {
    if slash_menu_open || mention_menu_open {
        return false;
    }
    if key.modifiers.contains(KeyModifiers::ALT) || key.modifiers.contains(KeyModifiers::SUPER) {
        return false;
    }

    let scroll_transcript = app.composer_arrows_scroll && !app.input.contains('\n');

    match key.code {
        KeyCode::Up => {
            if scroll_transcript {
                app.scroll_up(COMPOSER_ARROW_SCROLL_LINES);
            } else {
                app.vim_move_up();
            }
            true
        }
        KeyCode::Down => {
            if scroll_transcript {
                app.scroll_down(COMPOSER_ARROW_SCROLL_LINES);
            } else {
                app.vim_move_down();
            }
            true
        }
        KeyCode::PageUp => {
            app.scroll_up(10);
            true
        }
        KeyCode::PageDown => {
            app.scroll_down(10);
            true
        }
        _ => false,
    }
}

pub(crate) fn is_word_cursor_modifier(modifiers: KeyModifiers) -> bool {
    modifiers.contains(KeyModifiers::CONTROL) || modifiers.contains(KeyModifiers::ALT)
}

pub(crate) fn is_composer_newline_key(key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Char('j') => key.modifiers.contains(KeyModifiers::CONTROL),
        KeyCode::Enter => {
            key.modifiers.contains(KeyModifiers::ALT)
                || (key.modifiers.contains(KeyModifiers::SHIFT)
                    && !key.modifiers.contains(KeyModifiers::CONTROL))
        }
        _ => false,
    }
}

pub(crate) fn handle_history_search_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            let _ = app.accept_history_search();
        }
        KeyCode::Esc => {
            app.cancel_history_search();
        }
        KeyCode::Char('c') | KeyCode::Char('C')
            if key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            app.cancel_history_search();
        }
        KeyCode::Backspace => {
            app.history_search_backspace();
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            while app
                .history_search_query()
                .is_some_and(|query| !query.is_empty())
            {
                app.history_search_backspace();
            }
        }
        KeyCode::Up => {
            app.history_search_select_previous();
        }
        KeyCode::Down => {
            app.history_search_select_next();
        }
        KeyCode::Char(ch)
            if key.modifiers.is_empty()
                || key.modifiers == KeyModifiers::SHIFT
                || key.modifiers == KeyModifiers::NONE =>
        {
            app.history_search_insert_char(ch);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_app(pausable: bool, paused: bool, is_loading: bool) -> App {
        let mut app = App::new(
            crate::tui::app::TuiOptions {
                model: "test".to_string(),
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
            },
            &crate::config::Config::default(),
        );
        app.pausable = pausable;
        app.paused = paused;
        app.is_loading = is_loading;
        app
    }

    #[test]
    fn test_pause_command_returned_when_pausable_and_loading() {
        let app = make_app(true, false, true);
        let action = next_escape_action(&app, false);
        assert!(matches!(action, EscapeAction::PauseCommand),
            "expected PauseCommand when pausable+loading, got {action:?}");
    }

    #[test]
    fn test_cancel_returned_when_already_paused() {
        let app = make_app(true, true, true);
        let action = next_escape_action(&app, false);
        assert!(matches!(action, EscapeAction::CancelRequest),
            "expected CancelRequest when already paused, got {action:?}");
    }

    #[test]
    fn test_close_slash_menu_takes_priority() {
        let app = make_app(true, false, true);
        let action = next_escape_action(&app, true);
        assert!(matches!(action, EscapeAction::CloseSlashMenu),
            "expected CloseSlashMenu when slash menu open, got {action:?}");
    }

    #[test]
    fn test_normal_cancel_when_not_pausable() {
        let app = make_app(false, false, true);
        let action = next_escape_action(&app, false);
        assert!(matches!(action, EscapeAction::CancelRequest),
            "expected CancelRequest when loading+not pausable, got {action:?}");
    }

    #[test]
    fn test_discard_draft_when_paused() {
        let mut app = make_app(true, true, false);
        app.queued_draft = Some("draft".to_string());
        let action = next_escape_action(&app, false);
        assert!(matches!(action, EscapeAction::DiscardQueuedDraft),
            "expected DiscardQueuedDraft when draft queued and not loading, got {action:?}");
    }

    #[test]
    fn test_clear_input_when_not_loading() {
        let mut app = make_app(true, false, false);
        app.input = "hello".to_string();
        let action = next_escape_action(&app, false);
        assert!(matches!(action, EscapeAction::ClearInput),
            "expected ClearInput when not loading and has input, got {action:?}");
    }
}