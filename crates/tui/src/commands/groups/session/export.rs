//! `/export` command — export conversation to Markdown.

use std::fmt::Write;
use std::path::PathBuf;

use crate::tui::app::App;
use crate::tui::history::{HistoryCell, ToolCell};

use super::CommandResult;

/// Export conversation to markdown
pub fn export(app: &mut App, path: Option<&str>) -> CommandResult {
    let export_path = path.map_or_else(
        || {
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            PathBuf::from(format!("chat_export_{timestamp}.md"))
        },
        PathBuf::from,
    );

    let mut content = String::new();
    content.push_str("# Chat Export\n\n");
    let _ = write!(
        content,
        "**Model:** {}\n**Workspace:** {}\n**Date:** {}\n\n---\n\n",
        app.model,
        app.workspace.display(),
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    );

    for cell in &app.history {
        let (role, body) = match cell {
            HistoryCell::User { content } => ("**You:**", content.clone()),
            HistoryCell::Assistant { content, .. } => ("**Assistant:**", content.clone()),
            HistoryCell::System { content } => ("*System:*", content.clone()),
            HistoryCell::Error { message, severity } => match severity {
                crate::error_taxonomy::ErrorSeverity::Warning => {
                    ("**Warning:**", message.clone())
                }
                crate::error_taxonomy::ErrorSeverity::Info => ("*Info:*", message.clone()),
                _ => ("**Error:**", message.clone()),
            },
            HistoryCell::Thinking { content, .. } => ("*Thinking:*", content.clone()),
            HistoryCell::Tool(tool) => ("**Tool:**", render_tool_cell(tool, 80)),
            HistoryCell::SubAgent(sub) => ("**Sub-agent:**", render_subagent_cell(sub, 80)),
            HistoryCell::ArchivedContext {
                level,
                range,
                summary,
                ..
            } => (
                "**Archived Context:**",
                format!("L{level} [{range}]: {summary}"),
            ),
        };

        let _ = write!(content, "{}\n\n{}\n\n---\n\n", role, body.trim());
    }

    match std::fs::write(&export_path, content) {
        Ok(()) => CommandResult::message(format!("Exported to {}", export_path.display())),
        Err(e) => CommandResult::error(format!("Failed to export: {e}")),
    }
}

fn render_tool_cell(tool: &ToolCell, width: u16) -> String {
    tool.lines(width)
        .into_iter()
        .map(line_to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_subagent_cell(cell: &crate::tui::history::SubAgentCell, width: u16) -> String {
    cell.lines(width)
        .into_iter()
        .map(line_to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

fn line_to_string(line: ratatui::text::Line<'static>) -> String {
    line.spans
        .into_iter()
        .map(|span| span.content.to_string())
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
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
    fn test_export_crees_markdown_file() {
        let tmpdir = TempDir::new().unwrap();
        let mut app = create_test_app_with_tmpdir(&tmpdir);
        app.history.push(HistoryCell::User {
            content: "Hello".to_string(),
        });
        app.history.push(HistoryCell::Assistant {
            content: "Hi there".to_string(),
            streaming: false,
        });

        let export_path = tmpdir.path().join("export.md");
        let result = export(&mut app, Some(export_path.to_str().unwrap()));
        assert!(result.message.is_some());
        let msg = result.message.unwrap();
        assert!(msg.contains("Exported to"));
        assert!(export_path.exists());

        let content = std::fs::read_to_string(&export_path).unwrap();
        assert!(content.contains("# Chat Export"));
        assert!(content.contains("**Model:**"));
        assert!(content.contains("**You:**"));
        assert!(content.contains("**Assistant:**"));
    }

    #[test]
    fn test_export_with_default_path() {
        let tmpdir = TempDir::new().unwrap();
        let mut app = create_test_app_with_tmpdir(&tmpdir);
        let result = export(&mut app, None);
        assert!(result.message.is_some());
        // Should create file with timestamp name in current dir
        let entries: Vec<_> = std::fs::read_dir(".")
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("chat_export_"))
            .collect();
        // Clean up
        for entry in &entries {
            let _ = std::fs::remove_file(entry.path());
        }
        assert!(!entries.is_empty() || result.message.unwrap().contains("Exported to"));
    }
}
