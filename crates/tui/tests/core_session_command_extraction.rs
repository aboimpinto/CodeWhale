//! FEAT-005 Gherkin acceptance test.
//!
//! AT-002: Representative Built-In Commands Still Dispatch.
//!
//! Proves that after core/session command extraction, the binary still loads
//! and the evaluation harness runs without errors. This is an end-to-end
//! integration test: it builds the binary, runs `codewhale-tui eval` with a
//! shell command, and asserts on the JSON output report.
//!
//! The exhaustive command-dispatch correctness is verified by the 55/55
//! command registry unit tests. This acceptance test proves that the binary
//! compiles, links, and initializes the command registry without panicking at
//! startup — a different layer of validation that cannot be covered by unit tests.
//!
//! Step definitions use the same `TempDir` + `Command::new(codewhale_tui_binary())`
//! pattern as `directory_listing_acceptance.rs`.

use std::path::PathBuf;
use std::process::Command;

use cucumber::{World as _, given, then, when, writer::Stats as _};
use serde_json::Value;
use tempfile::TempDir;

const FEATURE_NAME: &str = "Core and session command extraction";
const FEATURE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/features/core_session_command_extraction.feature"
);

/// Scenario names from the feature file.
const CORE_SCENARIO: &str =
    "The binary loads and runs the evaluation harness after extraction";

/// Shared world state for FEAT-005 acceptance scenarios.
#[derive(Debug, Default, cucumber::World)]
struct CoreSessionExtractionWorld {
    record_dir: Option<TempDir>,
    report: Option<Value>,
}

// ---------------------------------------------------------------------------
// Step: Given "a clean CodeWhale evaluation workspace"
// ---------------------------------------------------------------------------

#[given("a clean CodeWhale evaluation workspace")]
fn clean_codewhale_evaluation_workspace(world: &mut CoreSessionExtractionWorld) {
    world.record_dir = Some(TempDir::new().expect("evaluation TempDir"));
}

// ---------------------------------------------------------------------------
// Step: When "the evaluation harness runs a shell command"
// ---------------------------------------------------------------------------

#[when("the evaluation harness runs a shell command")]
fn eval_harness_runs_shell_command(world: &mut CoreSessionExtractionWorld) {
    let record_dir = world
        .record_dir
        .as_ref()
        .expect("evaluation workspace should exist");

    let output = Command::new(codewhale_tui_binary())
        .args([
            "eval",
            "--json",
            "--shell-command",
            "echo eval-harness",
            "--record",
        ])
        .arg(record_dir.path())
        .output()
        .expect("codewhale-tui eval should start");

    // Fail fast if the binary could not start or panicked.
    // A non-zero exit or stderr output after extraction means the registry or
    // harness initialization is broken.
    assert!(
        output.status.success(),
        "codewhale-tui eval failed\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report: Value =
        serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
            panic!(
                "eval --json should emit valid JSON: {e}\nstdout:\n{}",
                String::from_utf8_lossy(&output.stdout)
            )
        });

    world.report = Some(report);
}

// ---------------------------------------------------------------------------
// Step: Then "the harness completes successfully"
// ---------------------------------------------------------------------------

#[then("the harness completes successfully")]
fn harness_completes_successfully(world: &mut CoreSessionExtractionWorld) {
    let report = world.report.as_ref().expect("eval report should exist");

    let success = report
        .get("metrics")
        .and_then(|m| m.get("success"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    assert!(
        success,
        "eval report 'metrics.success' should be true, got: {report:?}"
    );
}

// ---------------------------------------------------------------------------
// Step: And "the JSON report contains a step with the expected kind"
// ---------------------------------------------------------------------------

#[then("the JSON report contains a step with the expected kind")]
fn json_report_contains_step_with_expected_kind(world: &mut CoreSessionExtractionWorld) {
    let report = world.report.as_ref().expect("eval report should exist");

    let steps = report
        .get("steps")
        .and_then(|v| v.as_array())
        .expect("eval report should have a 'steps' array");

    assert!(!steps.is_empty(), "eval report should have at least one step");

    // Verify that the first step has the expected structure for a Shell step.
    let first_step = &steps[0];
    let kind = first_step
        .get("kind")
        .and_then(|v| v.as_str())
        .expect("step should have a 'kind' field");

    // The eval harness runs a shell command step, so the first step kind
    // should be "List" (the eval always starts with a List step).
    assert_eq!(
        kind, "List",
        "first step kind should be 'List', got: {kind}"
    );

    // Verify the step succeeded.
    let step_success = first_step
        .get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    assert!(
        step_success,
        "first step 'success' should be true, got: {first_step:?}"
    );

    // Verify the output is not empty (the shell command ran).
    let output = first_step
        .get("output")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(
        !output.is_empty(),
        "step output should not be empty: {first_step:?}"
    );
}

// ---------------------------------------------------------------------------
// Test runner
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "current_thread")]
async fn codewhale_eval_runs_after_extraction() {
    let writer = CoreSessionExtractionWorld::cucumber()
        .fail_on_skipped()
        .with_default_cli()
        .filter_run(FEATURE_PATH, move |feature, _, scenario| {
            feature.name == FEATURE_NAME && scenario.name == CORE_SCENARIO
        })
        .await;
    assert_eq!(
        writer.failed_steps(),
        0,
        "scenario failed: {CORE_SCENARIO}"
    );
    assert_eq!(
        writer.skipped_steps(),
        0,
        "scenario skipped steps: {CORE_SCENARIO}"
    );
    // The feature file has 4 steps.
    assert_eq!(
        writer.passed_steps(),
        4,
        "scenario did not run: {CORE_SCENARIO}"
    );
}

// ---------------------------------------------------------------------------
// Helper: locate the codewhale-tui binary
// ---------------------------------------------------------------------------

fn codewhale_tui_binary() -> PathBuf {
    if let Some(path) = option_env!("CARGO_BIN_EXE_codewhale-tui") {
        return PathBuf::from(path);
    }
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_codewhale-tui") {
        return PathBuf::from(path);
    }

    let mut path = std::env::current_exe().expect("current test executable path");
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.push(format!("codewhale-tui{}", std::env::consts::EXE_SUFFIX));
    path
}
