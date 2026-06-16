# PR/Issue Evidence Preparation — FEAT-003

**Prepared for:** Phase 8 (Final Checkpoint) consumption
**Date:** 2026-06-16
**Branch:** `release/v0.8.60`
**HEAD:** `6175b7cf`

This document provides the evidence structure and closure criteria for the
final PR body and issue updates. Phase 8 will fill in live check results and
commit SHAs.

---

## 1. Final PR Body Template

### Title

> FEAT-003: Completion cleanup and full validation for EPIC-001

### Body

```
## Summary

Closes EPIC-001 ([#2870](https://github.com/Hmbown/CodeWhale/issues/2870))
and [#2791](https://github.com/Hmbown/CodeWhale/issues/2791) by removing
temporary migration scaffolding from Layers 4-5, running full workspace
validation, and updating the central command architecture document.

### Changes

1. **Temporary-code inventory** (Phase 2): Walked the codebase for every
   bridge, compatibility loader, and duplicate dispatch path. 0 production
   artifacts found — all candidates were either permanent exceptions (documented
   with in-code comments) or deferred test-only functions.

2. **Permanent-exception documentation** (Phase 3): Added explicit FEAT-003
   references in 9 source files:
   - `commands/mod.rs` — `"jihua"`/`"zidong"` legacy aliases, `"set"`/`"deepseek"`
     migration hints
   - `user_registry.rs` — load dependency on `user_commands.rs`
   - `user_commands.rs` — module-level permanent role documentation
   - 6 group `mod.rs` files — `#[allow(clippy::module_inception)]` rationale

3. **Deferred-items tracking** (Phase 3): Created
   `deferred-items-tracking.md` for 2 test-only functions
   (`try_dispatch_user_command()`, `load_user_commands()`) with migration
   strategy — ~15 tests to migrate to `user_registry` APIs.

4. **Architecture documentation** (Phase 4): Created
   `docs/architecture/command-dispatch.md` with:
   - Final Mermaid architecture diagram
   - Module declarations and interface boundaries for every command component
   - Dispatch precedence description (5-step flow)
   - Group-owned built-in command table (8 groups)
   - UI integration (palette, slash completion, shadowing logic)
   - Permanent exceptions table
   - Test/evidence matrix
   - Landed layer cross-references
   - Issue closure sequencing

### Validation Evidence

| Check | Result |
|-------|--------|
| `cargo fmt --all -- --check` | ⬜ |
| `cargo check -p codewhale-tui` | ⬜ |
| `cargo clippy -p codewhale-tui -- -D warnings` | ⬜ |
| `cargo test -p codewhale-tui commands::` | ⬜ |
| `cargo test -p codewhale-tui command_palette` | ⬜ |
| `cargo test -p codewhale-tui slash_completion` | ⬜ |
| `cargo test --workspace` | ⬜ |

### Landed Layers

| Layer | Feature | PR | Description |
|-------|---------|----|-------------|
| Layer 1 | — | #2871 | Command-surface cleanup and neutral shared extraction |
| Layer 2 | — | #2878 | Command parity harness |
| — | — | #2887 | Supporting acceptance-test harness |
| Layer 3 | — | #2888 | Registry and parser helper extraction |
| Layer 4 | FEAT-001 | [PR ref] | Group-owned built-in command files under `commands/groups/` |
| Layer 5 | FEAT-002 | [PR ref] | Dedicated `UserCommandRegistry` boundary |
| Layer 6 | FEAT-003 | This PR | Completion cleanup, validation, architecture docs |

### Unrelated Failures

[Isolate any unrelated test failures here with the pattern:
- FAIL: `test_name` — known flaky test, pre-existing; not caused by this PR.
  Tracked at [issue link].]

### Architecture Document

See [`docs/architecture/command-dispatch.md`](./docs/architecture/command-dispatch.md)
for the final command dispatch architecture, module boundaries, and diagram.

### Deferred Follow-Up

- Issue: [link] "Migrate user_commands test functions to user_registry APIs"
  — ~15 tests to migrate, labeled `cleanup`, `tests`.
```

---

## 2. Issue #2870 — Final Layer Status Update

### Title (update comment, not title)

> Update: All 6 command-boundary layers completed as of FEAT-003

### Body (comment)

```
## Summary

EPIC-001 command dispatch and ownership refactor is complete.

### Layer Completion Status

| Layer | Feature | Status | PR |
|-------|---------|--------|----|
| Layer 1: Command-surface cleanup | — | ✅ Completed | #2871 |
| Layer 2: Command parity harness | — | ✅ Completed | #2878 |
| Supporting: Acceptance-test harness | — | ✅ Completed | #2887 |
| Layer 3: Registry and parser helpers | — | ✅ Completed | #2888 |
| Layer 4: Group-owned built-in files | FEAT-001 | ✅ Completed | [PR ref] |
| Layer 5: User commands boundary | FEAT-002 | ✅ Completed | [PR ref] |
| Layer 6: Cleanup, validation, closure | FEAT-003 | ✅ Completed | [PR ref] |

### Final Architecture

The final command architecture is documented in:
`docs/architecture/command-dispatch.md`

It covers:
- Module declarations and interface boundaries
- 5-step dispatch precedence flow
- 8 group-owned built-in command areas
- User command registry boundary
- Command palette and slash completion integration
- Permanent exceptions and deferred follow-up

### Residual Follow-Up

The following cleanup item is deferred to a separate issue:
- "Migrate user_commands test functions to user_registry APIs" — ~15 tests
  using `#[cfg(test)]` functions in `user_commands.rs` should use
  `UserCommandRegistry` APIs directly.

### Closing

This issue tracks the overall EPIC. The final layer (Layer 6 / FEAT-003) has
landed. Closing this issue in favor of individual component issues.
```

---

## 3. Issue #2791 — Closure Criteria Checklist

Close #2791 **only after** all of the following are verified:

- [ ] Full workspace validation passes: `cargo test --workspace`
- [ ] All command-specific tests pass: `commands::`, `command_palette`, `slash_completion`
- [ ] No undocumented temporary bridges, compatibility loaders, or duplicate dispatch paths remain
- [ ] Central architecture document `docs/architecture/command-dispatch.md` exists and reflects the final state
- [ ] Any residual follow-up is moved to a separate issue (not hidden inside this issue)
- [ ] The final PR body lists validation evidence and links each landed layer

### Closure Comment Template

```
## Closure

The command dispatch and ownership refactor is complete.

All 6 layers have landed (Layers 1-3 via PRs #2871, #2878, #2888; Layers 4-6 via
FEAT-001, FEAT-002, FEAT-003). The final architecture is documented in
`docs/architecture/command-dispatch.md`.

### Validation

- `cargo test --workspace` [fill in Phase 8 result]
- `cargo test -p codewhale-tui commands::` [fill in Phase 8 result]
- `cargo test -p codewhale-tui command_palette` [fill in Phase 8 result]
- `cargo test -p codewhale-tui slash_completion` [fill in Phase 8 result]

### Residual Follow-Up

Created separate issue [link] for "Migrate user_commands test functions to
user_registry APIs" (cleanup, tests).

Closes #2791.
```
