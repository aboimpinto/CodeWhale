//! EPIC acceptance harness smoke test.
//!
//! Proves that the Gherkin/Cucumber infrastructure is available and functional
//! on the target branch. This is the only Gherkin scenario implemented in
//! FEAT-004 (Layer 4.0). All other EPIC acceptance scenarios are defined as
//! baseline skeletons and will be implemented by later extraction layers.
//!
//! AT-001: Acceptance Harness Is Available.
//!   Given the final EPIC target branch is checked out
//!   When the acceptance test command is executed
//!   Then the Gherkin runner discovers the EPIC acceptance scenarios
//!   And the runner exits successfully

use cucumber::{World as _, given, then, when, writer::Stats as _};

const FEATURE_NAME: &str = "EPIC acceptance harness";
const FEATURE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/features/epic_acceptance_harness.feature"
);
const SMOKE_SCENARIO: &str =
    "Gherkin acceptance tests can run on the target branch";

#[derive(Debug, Default, cucumber::World)]
struct EpicAcceptanceWorld;

#[given("the acceptance harness is available")]
fn acceptance_harness_available(_world: &mut EpicAcceptanceWorld) {}

#[when("the runner discovers EPIC scenarios")]
fn runner_discovers_epic_scenarios(_world: &mut EpicAcceptanceWorld) {}

#[then("the runner exits successfully")]
fn runner_exits_successfully(_world: &mut EpicAcceptanceWorld) {}

#[tokio::test(flavor = "current_thread")]
async fn acceptance_harness_smoke_test() {
    let writer = EpicAcceptanceWorld::cucumber()
        .fail_on_skipped()
        .with_default_cli()
        .filter_run(FEATURE_PATH, move |feature, _, scenario| {
            feature.name == FEATURE_NAME && scenario.name == SMOKE_SCENARIO
        })
        .await;
    assert_eq!(
        writer.failed_steps(),
        0,
        "scenario failed: {SMOKE_SCENARIO}"
    );
    assert_eq!(
        writer.skipped_steps(),
        0,
        "scenario skipped steps: {SMOKE_SCENARIO}"
    );
    assert_eq!(
        writer.passed_steps(),
        3,
        "scenario did not run: {SMOKE_SCENARIO}"
    );
}
