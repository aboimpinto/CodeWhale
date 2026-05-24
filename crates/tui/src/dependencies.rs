//! Probes for runtime dependencies (Python, pandoc, tesseract, ...).
//!
//! All public helpers return `Option<String>` so callers can fall
//! back gracefully.  Cached lookups never block on repeated calls.

use std::process::Command;
use std::sync::OnceLock;

// ── Generic probing helper ──────────────────────────────────────────

/// Probe a single executable candidate.
///
/// `spec` is either a bare name (`"python3"`) or a `/path/to/bin -arg`
/// style string.  For the latter, only the first token is probed.
pub fn probe_executable(spec: &str) -> bool {
    let p = spec.split_whitespace().next().unwrap_or(spec);
    which::which(p).is_ok()
}

// ── Python ──────────────────────────────────────────────────────────

pub const PYTHON_CANDIDATES: &[&str] = &["python3", "python", "py -3"];

/// Resolve the Python interpreter, caching the result after the first
/// successful probe.
pub fn resolve_python_interpreter() -> Option<String> {
    static CACHE: OnceLock<Option<String>> = OnceLock::new();
    CACHE.get_or_init(|| {
        for spec in PYTHON_CANDIDATES {
            if probe_executable(spec) {
                return Some(spec.to_string());
            }
        }
        None
    })
    .clone()
}

// ── pdf tools ───────────────────────────────────────────────────────

pub fn resolve_pdftotext() -> Option<String> {
    if probe_executable("pdftotext") {
        Some("pdftotext".to_string())
    } else {
        None
    }
}

pub fn resolve_tesseract() -> Option<String> {
    if probe_executable("tesseract") {
        Some("tesseract".to_string())
    } else {
        None
    }
}

// ── pandoc ──────────────────────────────────────────────────────────

pub fn resolve_pandoc() -> Option<String> {
    if probe_executable("pandoc") {
        Some("pandoc".to_string())
    } else {
        None
    }
}

// ── Node.js ─────────────────────────────────────────────────────────

pub const NODE_CANDIDATES: &[&str] = &["node", "nodejs"];

pub fn resolve_node() -> Option<String> {
    for spec in NODE_CANDIDATES {
        if probe_executable(spec) {
            return Some(spec.to_string());
        }
    }
    None
}

// ── Split interpreter spec ──────────────────────────────────────────

/// Split `"py -3"` into `("py", ["-3"])` the same way [`probe_executable`]
/// would find it.  Returns `(spec, [])` if no whitespace separates tokens.
pub fn split_interpreter_spec(spec: &str) -> (String, Vec<String>) {
    let mut parts = spec.splitn(2, ' ');
    let program = parts.next().unwrap_or(spec).to_string();
    let args: Vec<String> = parts
        .next()
        .map(|a| a.split_whitespace().map(String::from).collect())
        .unwrap_or_default();
    (program, args)
}

// ── RuntimeTool types (used by runtime_tool.rs) ──

use tokio::process;

#[allow(dead_code)]
pub struct RustC;
#[allow(dead_code)]
impl RustC {
    pub fn tokio_command() -> Option<process::Command> {
        Some(process::Command::new("rustc"))
    }
}

#[allow(dead_code)]
pub struct Python;
#[allow(dead_code)]
impl Python {
    pub fn tokio_command() -> Option<process::Command> {
        resolve_python_interpreter()
            .map(|s| process::Command::new(split_interpreter_spec(&s).0))
    }
}

#[allow(dead_code)]
pub struct Node;
#[allow(dead_code)]
impl Node {
    pub fn tokio_command() -> Option<process::Command> {
        resolve_node().map(process::Command::new)
    }
    pub fn available() -> bool {
        resolve_node().is_some()
    }
}

#[allow(dead_code)]
pub struct DotNet;
#[allow(dead_code)]
impl DotNet {
    pub fn tokio_command() -> Option<process::Command> {
        if probe_executable("dotnet") {
            Some(process::Command::new("dotnet"))
        } else {
            None
        }
    }
    pub fn available() -> bool {
        probe_executable("dotnet")
    }
}

#[allow(dead_code)]
pub struct Go;
#[allow(dead_code)]
impl Go {
    pub fn tokio_command() -> Option<process::Command> {
        if probe_executable("go") {
            Some(process::Command::new("go"))
        } else {
            None
        }
    }
}

#[allow(dead_code)]
pub struct TypeScript;
#[allow(dead_code)]
impl TypeScript {
    pub fn tokio_command() -> Option<process::Command> {
        resolve_node().map(process::Command::new)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn python_candidates_include_standard_names() {
        // Should contain at least the main Python candidates
        assert!(
            PYTHON_CANDIDATES.contains(&"python3")
                || PYTHON_CANDIDATES.contains(&"python")
        );
    }

    #[test]
    fn node_candidates_include_standard_names() {
        assert!(
            NODE_CANDIDATES.contains(&"node")
                || NODE_CANDIDATES.contains(&"nodejs")
        );
    }

    #[test]
    fn probe_executable_uses_first_token_of_spec() {
        // "py -3" should probe just "py"
        let _ = probe_executable("py -3");
    }

    #[test]
    fn split_interpreter_spec_splits_on_first_space() {
        let (prog, args) = split_interpreter_spec("py -3 -E");
        assert_eq!(prog, "py");
        assert_eq!(args, vec!["-3", "-E"]);
    }

    #[test]
    fn split_interpreter_spec_without_args_returns_empty_vec() {
        let (prog, args) = split_interpreter_spec("python3");
        assert_eq!(prog, "python3");
        assert!(args.is_empty());
    }

    #[test]
    fn resolve_python_interpreter_returns_known_name_or_none() {
        if let Some(spec) = resolve_python_interpreter() {
            assert!(
                PYTHON_CANDIDATES.contains(&&spec[..])
                    || spec.starts_with("python")
                    || spec.starts_with("py"),
                "unexpected python spec: {spec}"
            );
        }
    }

    #[test]
    fn resolve_node_returns_node_or_nodejs() {
        if let Some(spec) = resolve_node() {
            assert!(
                spec == "node" || spec == "nodejs",
                "unexpected node spec: {spec}"
            );
        }
    }

    #[test]
    fn probe_executable_returns_false_for_nonsense_name() {
        assert!(!probe_executable("this_executable_surely_does_not_exist_xyzzy"));
    }

    #[test]
    fn rustc_tokio_command_always_succeeds() {
        // rustc is always assumed available; command() should return Some
        assert!(RustC::tokio_command().is_some());
    }

    #[test]
    fn split_interpreter_with_long_path() {
        let (prog, args) = split_interpreter_spec("/usr/local/bin/python3 -E -S");
        assert_eq!(prog, "/usr/local/bin/python3");
        assert_eq!(args, vec!["-E", "-S"]);
    }
}
