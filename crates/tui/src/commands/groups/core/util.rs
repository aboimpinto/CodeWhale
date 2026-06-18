//! Shared utility helpers for core group commands.
//!
//! This module holds genuinely cross-command helpers that are used by
//! multiple focused command modules in the core group. Keeping them in
//! a neutral support module prevents duplication and keeps command
//! modules focused on their own responsibility.
//!
//! FEAT-005 cross-phase contract: `util` is the single neutral support
//! location for shared helpers. No command-specific logic should be
//! hidden here.

use crate::tui::app::App;

/// Parse an optional depth-prefixed argument.
///
/// Accepts `[N] <text>` where N is an optional 0-3 depth value.
/// Defaults to `default_depth` when no numeric prefix is present.
///
/// Used by: rlm, agent, swarm
pub(in crate::commands) fn parse_depth_prefixed_arg(
    arg: Option<&str>,
    default_depth: u32,
) -> Result<(u32, Option<&str>), String> {
    let Some(raw) = arg.map(str::trim).filter(|raw| !raw.is_empty()) else {
        return Ok((default_depth, None));
    };
    let mut parts = raw.splitn(2, char::is_whitespace);
    let first = parts.next().unwrap_or_default();
    if first.chars().all(|ch| ch.is_ascii_digit()) {
        let depth: u32 = first
            .parse()
            .map_err(|_| "Depth must be an integer from 0 to 3".to_string())?;
        if depth > 3 {
            return Err("Depth must be between 0 and 3".to_string());
        }
        Ok((depth, parts.next().map(str::trim)))
    } else {
        Ok((default_depth, Some(raw)))
    }
}

/// Check whether an input string resolves to an existing file in the
/// workspace.
///
/// Used by: rlm
pub(in crate::commands) fn resolves_to_existing_file(app: &App, input: &str) -> bool {
    let path = std::path::Path::new(input);
    let candidate = if path.is_absolute() {
        path.to_path_buf()
    } else {
        app.workspace.join(path)
    };
    candidate.is_file()
}
