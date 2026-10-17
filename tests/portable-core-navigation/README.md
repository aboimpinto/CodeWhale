# Core navigation normal-library proof

This unpublished workspace package mounts the **actual production navigation
root**, all ten leaf modules and all13 registrations. It contains no substitute
App, copied implementation, test-only root or host adapter. Shared metadata,
results/actions, facets and nested payloads are normal dependencies.

Run sequentially from the repository root:

```sh
python3 -B scripts/test_check_command_core_navigation_proof.py
sh scripts/check-portable-core-navigation.sh
```

The proof checks the declared complete source closure and real host inventory
positions, an all-feature/all-target **normal** dependency graph, a no-default
normal-library build, and nonzero tests from the actual portable leaves.
Rejection controls cover omitted/reordered registrations, metadata drift,
selected-leaf/stub/test-only wrappers, conditional production modules and
unreviewed include macros, host imports (even in an omitted owned leaf), test-only
dependency substitution and unapproved transitive packages.

The approved graph is deliberately explicit: command-contract/protocol data;
serde/JSON/indexed containers and their derive/parser support; chrono/UUID and
existing platform/timezone/random primitives, including their wasm/Windows
support. These are inherited shared data dependencies, not permission to add
TUI, application runtime, configuration, HTTP, terminal or credential-storage
services. New normal packages require source review, not automatic approval.

Host registration, concrete filesystem/view/config effects, real-App tests and
voice recording/transcription remain outside the movable slice. The rest of core
is not claimed portable by this package: the top-level `core` frontier remains
pending for its other27 registrations. Physical movement is a later feature.
