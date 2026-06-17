# Command Extraction Contract — EPIC-002

**Related EPIC:** [EPIC-002 — Command Single Responsibility Extraction](https://github.com/Hmbown/CodeWhale/issues/2870)
**Related FEAT:** Layer 4.0 — Baseline and Contract
**Last updated:** 2026-06-17
**Branch:** `release/v0.8.60`

## Overview

This document defines the command-module ownership contract, registration
responsibilities, presentation-surface contracts, and acceptance criteria for
EPIC-002 command extraction. It serves as the cross-FEAT reference for every
extraction layer (Layers 4.1 through 4.4).

After EPIC-001 established group-owned command areas, this EPIC moves each
command inside a group into a focused single-responsibility module. The final
shape is layered ownership:

- Top-level command boundary registers command groups only.
- Each group registers only the commands it owns.
- Each command module owns only that command's registration, aliases,
  help metadata, dispatch entry, and implementation.
- Shared behavior lives in neutral support modules outside individual
  command modules.

## Dispatch Flow (Precedence Order)

The `execute()` function in `crates/tui/src/commands/mod.rs` follows this
strict precedence. This order must be preserved by every extraction FEAT:

| Step | Stage | Handler | Details |
|------|-------|---------|---------|
| 1 | **User-defined commands** | `user_registry::try_dispatch()` | Highest precedence. User markdown commands override any built-in with the same name or alias. |
| 2 | **Legacy backward-compatible aliases** | `groups::config::dispatch()` | `/jihua` -> `/mode plan`, `/zidong` -> `/mode yolo`. Permanent aliases predating group-owned structure. |
| 3 | **Built-in command registry** | `registry().get()` -> `command_object.execute()` | All registered built-in commands resolved by canonical name or alias. |
| 4 | **Legacy migration hints** | `commands/mod.rs` match arms | `/set` and `/deepseek` return explanatory error messages. Permanently excluded from registry and autocomplete. |
| 5 | **Skills fallback** | `groups::skills::run_skill_by_name()` | Lowest precedence. Falls through to unknown-command suggestions. |

## Command-Module Contract

Every extracted command module must satisfy this contract:

### Required Items

| Contract Item | Requirement | Verified By |
|---------------|-------------|-------------|
| Canonical command name | The primary slash command name (ASCII lowercase, no slash prefix). | `command_registry_metadata_is_complete_and_palette_safe` test |
| Aliases | Alternate names accepted by parser, palette, or completion. Stored without `/` prefix. | `command_info_resolves_canonical_names_and_aliases` test |
| Help metadata | `CommandInfo` with `name`, `aliases`, `usage`, `description_id: MessageId`. | `every_registered_command_has_a_help_topic` test |
| Dispatch entry | The handler invoked by command dispatch, registered via group's `CommandGroup::commands()`. | `every_registered_command_dispatches_to_a_handler` test |
| Implementation | The command-specific business logic, owned entirely by the command module. | Code review |
| Tests | Focused tests or existing Gherkin coverage proving parity. | Test count per extraction layer |
| Ownership boundary | Clear evidence that behavior for the command is not split across unrelated modules. | Static ownership check (Layer 4.4) |

### Prohibited Patterns

- Silent registration side effects outside the documented registration flow.
  Command modules must not register themselves or modify the global registry
  outside the group's `CommandGroup::commands()` return.
- Duplicate command names or aliases across modules.
  The `command_registry_has_unique_names_and_aliases` test catches this.
- Command-specific presentation logic in palette, completion, or help code.
  Those surfaces consume metadata from `command_infos()` only.

### Target Module Layout

For a command named `<cmd>` in group `<group>`:

```
crates/tui/src/commands/groups/<group>/<cmd>.rs
  ├── CommandInfo static (name, aliases, usage, description_id)
  ├── pub fn dispatch(app: &mut App, args: Option<&str>) -> CommandResult
  ├── pub fn handler_name(...)  // optional helper functions
  └── #[cfg(test)] mod tests { ... }  // focused unit tests
```

## Group Registration Responsibilities

| Owner | Responsibility | Key File |
|-------|---------------|----------|
| Top-level command registry | Registers command groups and global registry wiring only. Never contains command-specific implementation logic. | `crates/tui/src/commands/mod.rs` |
| Command group module | Owns the list of commands belonging to that group via `CommandGroup::commands()`. Must not duplicate commands from other groups. | `crates/tui/src/commands/groups/<group>/mod.rs` |
| Command module | Owns command metadata, aliases, dispatch entry, and implementation. | `crates/tui/src/commands/groups/<group>/<cmd>.rs` |
| Parser / Dispatcher | Owns parsing and routing behavior (`execute()` in `commands/mod.rs`). Never contains command-specific business logic. | `crates/tui/src/commands/mod.rs` (`execute()`) |
| Palette integration | Consumes command metadata from `command_infos()`. Target contract: no command-specific hardcoding in extracted state. Current exception: `command_runs_directly()` hardcodes ~30 command names to decide Execute vs Insert behavior — later FEATs must either eliminate this or move the dispatch-time-flag into `CommandInfo`. | `crates/tui/src/tui/command_palette.rs` (`build_entries()`, `command_runs_directly()`) |
| Slash completion | Consumes command metadata from `command_infos()`. Never hardcodes command-specific facts. | `crates/tui/src/tui/widgets/mod.rs` (`slash_completion_hints()`) |

Top-level registration must not contain command-specific implementation logic.
Group modules may aggregate command modules, but must not duplicate command
metadata from the command module.

## Key Data Shapes

```rust
// CommandInfo - static metadata per command
// Source: crates/tui/src/commands/traits.rs
pub struct CommandInfo {
    pub name: &'static str,              // Canonical slash name, ASCII lowercase
    pub aliases: &'static [&'static str], // Alternate names (stored without '/')
    pub usage: &'static str,             // Usage string starting with "/<name>"
    pub description_id: MessageId,       // Localized description key
}

// Command trait
pub trait Command: Send + Sync {
    fn info(&self) -> &'static CommandInfo;
    fn execute(&self, app: &mut App, args: Option<&str>) -> CommandResult;
}

// CommandGroup trait
pub trait CommandGroup: Send + Sync {
    fn commands(&self) -> Vec<Box<dyn Command>>;
}

// CommandRegistry
pub struct CommandRegistry {
    commands: Vec<Box<dyn Command>>,
    name_to_index: HashMap<&'static str, usize>,
}

// UserCommandMetadata (user_registry.rs)
pub struct UserCommandMetadata {
    pub name: String,
    pub body: String,
    pub description: Option<String>,
    pub argument_hint: Option<String>,
    pub allowed_tools: Vec<String>,
    pub pausable: bool,
    pub aliases: Vec<String>,
    pub hidden: bool,
}
```

## User-Command Precedence Matrix

User commands are always checked before built-in commands. The exact behavior
varies by presentation surface:

| Scenario | Dispatch | Palette | Completion | Help |
|----------|----------|---------|------------|------|
| **A. User cmd shadows built-in canonical name** | User command runs | Built-in excluded | Built-in entirely hidden | Built-in dispatch is shadowed |
| **B. User cmd shadows built-in alias only** | User command runs | Built-in still included | Built-in visible via canonical name | Built-in metadata always shown |
| **C. Hidden user cmd shadows nothing** | Runs directly | Excluded (hidden filter) | Excluded (hidden filter) | N/A |
| **D. Hidden user cmd shadows canonical name** | User command runs | Both excluded | Both excluded | Built-in dispatch is shadowed |
| **E. User cmd shadows retired command** | User command runs | User included (built-in not in palette) | User included (built-in excluded) | N/A |
| **F. No shadowing** | Built-in dispatches normally | Built-in appears | Built-in appears in all phases | Built-in help shown normally |
| **G. User cmd shadows name via alias** | User command runs | Built-in still included | Built-in visible via canonical name | Built-in metadata always shown |

**Enforcement code paths:**
- Dispatch precedence: `commands/mod.rs:execute()` line ~50
- Palette shadowing: `command_palette.rs:build_entries()` user-registry check
- Completion shadowing: `widgets/mod.rs:builtin_visible_for_completion_match()`
- Help text: dispatch-level shadowing only; built-in handler reads `command_infos()`

## Canonical Rule: Single Source of Truth

Palette, slash completion, and help text always consume built-in command
metadata from `command_infos()` (which reads from `registry().infos()` via
`build_registry()`). No presentation surface hardcodes command-specific names,
aliases, descriptions, or usage text.

**Current exception — `command_runs_directly()`:** The palette function
`command_runs_directly()` at `command_palette.rs:434-470` hardcodes ~30 command
names to decide whether a palette action should execute immediately or insert
the command text for the user to edit. This is pre-extraction legacy behavior;
the ideal contract would move a "runs-directly" flag into `CommandInfo`, but
that change is deferred to a later extraction FEAT. Until then, this function
is the only palette-specific command list, and Layer 4.1-4.4 extraction FEATs
must ensure that any command added to or removed from this list is documented
in the extraction PR.

Note: the hardcoded names in `command_runs_directly()` affect only the palette
*action* (Execute vs Insert Text), not the palette *entry* visibility or
metadata — those still come from `command_infos()`.

| Surface | Source of Truth | Consumer Code Path |
|---------|----------------|-------------------|
| Palette entries (built-in) | `commands::command_infos()` | `command_palette.rs:build_entries()` |
| Palette entries (user) | `user_registry::registry_for_workspace()` | `command_palette.rs:build_entries()` |
| Completion entries (built-in) | `commands::command_infos()` | `widgets/mod.rs:slash_completion_hints()` |
| Completion entries (user) | `user_registry::registry_for_workspace()` | `widgets/mod.rs:slash_completion_hints()` |
| Help text | `commands::get_command_info(topic)` | `groups/core/core.rs:help()` |

**Contract for extraction FEATs:** Extracted command modules must define their
`CommandInfo` static in the command module itself. The group `mod.rs` must not
duplicate or override metadata. The palette, completion, and help surfaces will
automatically consume the metadata through `command_infos()` — no per-command
changes needed in presentation code.

## Current Architecture Reference

The current command dispatch architecture (post EPIC-001) is documented in
[command-dispatch.md](./command-dispatch.md). That document covers:

- Module boundaries for every command component
- The 5-step dispatch precedence flow
- 8 group-owned built-in command areas
- User command registry boundary
- Permanent exceptions and deferred follow-up

Extraction FEATs should read `command-dispatch.md` as the baseline reference
for the current architecture before moving any command implementation.

## Existing Group Command Inventory

All group memberships verified from live `CommandGroup::commands()` source:

| Group | File | Commands |
|-------|------|----------|
| **Core** | `crates/tui/src/commands/groups/core/mod.rs` | anchor, help, clear, exit, model, models, provider, queue, stash, hooks, subagents, agent, swarm, links, feedback, hf, home, workspace, profile, rlm, translate, voice, voicesend, voicecontrol |
| **Session** | `crates/tui/src/commands/groups/session/mod.rs` | rename, save, fork, new, sessions, load, compact, purge, relay, export |
| **Config** | `crates/tui/src/commands/groups/config/mod.rs` | config, sidebar, settings, status, statusline, mode, theme, verbose, trust, logout, slop |
| **Debug** | `crates/tui/src/commands/groups/debug/mod.rs` | tokens, cost, balance, cache, change, system, context, edit, diff, undo, retry |
| **Project** | `crates/tui/src/commands/groups/project/mod.rs` | init, lsp, share, goal |
| **Skills** | `crates/tui/src/commands/groups/skills/mod.rs` | skills, skill, review, restore |
| **Memory** | `crates/tui/src/commands/groups/memory/mod.rs` | note, memory |
| **Utility** | `crates/tui/src/commands/groups/utility/mod.rs` | attach, task, jobs, mcp, network, plugins |

## Parity Expectations for Extraction FEATs

Every extraction FEAT (Layers 4.1-4.4) must satisfy these parity requirements:

| # | Expectation | Verification |
|---|-------------|-------------|
| P1 | `CommandInfo` + `MessageId` defined in extracted command module, not in group `mod.rs` | Code review + existing metadata tests |
| P2 | Extracted command appears in `command_infos()` after registration | `command_palette_has_one_entry_for_every_registered_command` |
| P3 | Extracted command appears in slash completion for all 3 matching phases | `slash_completion_hints_include_links_and_config` |
| P4 | Extracted command resolves via `/help <command>` with correct metadata | `every_registered_command_has_a_help_topic` |
| P5 | Palette entry formatting matches `CommandInfo` methods | `command_registry_metadata_is_complete_and_palette_safe` |
| P6 | User-command shadowing preserved (canonical and alias) | Explicit shadowing tests in palette and completion |
| P7 | No new duplicate names or aliases | `command_registry_has_unique_names_and_aliases` |
| P8 | Retired commands `/set`, `/deepseek` remain excluded | Existing exclusion tests |
| P9 | Hidden user commands stay runnable but non-discoverable | `hidden_user_commands_*` tests |
| P10 | Unknown command suggestions remain stable | `unknown_command_suggests_nearest_match` and fallback tests |
| P11 | Palette action (Execute vs Insert Text) matches pre-extraction baseline | Code review of extracted command vs `command_runs_directly()` list |

## Baseline Gherkin Scenarios

The following Gherkin scenario skeletons define the EPIC acceptance coverage
that later extraction FEATs should implement as real feature files. Layer 4.0
defines the skeletons and maps existing unit-test coverage.

### AT-001: Acceptance Harness Is Available

**Purpose:** Proves Gherkin/Cucumber infrastructure is configured and discovers
feature files. Implemented in Layer 4.0.

- Feature file: `crates/tui/tests/features/epic_acceptance_harness.feature`
- Runner: `crates/tui/tests/epic_acceptance_harness.rs`
- Status: Implemented, passes 3/3 steps

### AT-002: Representative Built-In Commands Still Dispatch

```gherkin
Feature: Built-in command dispatch

  Scenario Outline: Representative built-in commands dispatch from
                    every command group
    Given a clean CodeWhale workspace
    And the runtime is using mocked provider/tool behavior
    When the user runs "<command>"
    Then the command succeeds
    And the observed behavior matches the pre-extraction baseline
      for "<command>"
    Examples:
      | group   | command  |
      | core    | /help    |
      | session | /relay   |
      | config  | /config  |
      | debug   | /tokens  |
      | project | /hunt    |
      | memory  | /note    |
      | skills  | /skills  |
      | utility | /mcp     |
```

Existing coverage: `every_registered_command_dispatches_to_a_handler` and
`representative_command_groups_keep_dispatch_surfaces` in `commands/mod.rs`.
Gherkin implementation deferred to Layer 4.1.

### AT-003: Built-In Aliases Still Dispatch

Existing coverage: `every_command_alias_dispatches_to_a_handler` in
`commands/mod.rs` covers all declared aliases. Gherkin implementation deferred
to Layer 4.1.

### AT-004: Help, Palette, And Completion See Extracted Commands

Existing coverage:
- `every_registered_command_has_a_help_topic` (commands/mod.rs)
- `command_registry_metadata_is_complete_and_palette_safe` (commands/mod.rs)
- `command_palette_has_one_entry_for_every_registered_command` (command_palette.rs)
- 17 `slash_completion_hints_*` tests (widgets/mod.rs)

Gherkin implementation deferred to Layer 4.4.

### AT-005: User Commands Keep Precedence Over Built-Ins

Existing coverage:
- `user_command_shadows_builtin_before_group_dispatch` (canonical shadowing)
- `dispatch_prefers_user_alias_over_builtin_alias` (alias shadowing)
- `dispatch_prefers_user_command_over_builtin_with_same_name`
- `removed_user_command_reloads_and_falls_back_to_builtin`

Gherkin implementation deferred to Layer 4.4.

### AT-006: Hidden User Commands Stay Hidden But Runnable

Existing coverage:
- `hidden_user_commands_still_dispatch_directly` (user_registry.rs)
- `command_palette_excludes_hidden_user_commands` (command_palette.rs)
- `slash_completion_hints_exclude_hidden_user_commands` (widgets/mod.rs)

Gherkin implementation deferred to Layer 4.4.

### AT-007: Unknown Command Suggestions Remain Stable

Existing coverage:
- `unknown_command_suggests_nearest_match` (commands/mod.rs)
- `unknown_command_without_close_match_keeps_help_guidance` (commands/mod.rs)

Gherkin implementation deferred to Layer 4.4.

### AT-008: No Duplicate Command Or Alias Registration

Existing coverage: `command_registry_has_unique_names_and_aliases`
(commands/mod.rs). This is a registry-integrity check, not a user workflow;
Rust unit test is the correct coverage.

### AT-009: Command Ownership Contract Is Enforced

The ownership contract is defined in this document and the companion
[command-dispatch.md](./command-dispatch.md). Enforcement is manual review
during extraction PRs. Static ownership check deferred to Layer 4.4.

### AT-010: Temporary Migration Paths Are Removed Or Documented

No migrations exist yet (no extraction has occurred). Inventory template
prepared. Final cleanup deferred to Layer 4.4.

### AT-011: Final Validation Evidence Is Complete

| Field | Value |
|-------|-------|
| Target branch/commit | `release/v0.8.60` / `<final Layer 4.0 commit hash>` |
| Gherkin command | `cargo test -p codewhale-tui --test epic_acceptance_harness -- --test-threads=1` |
| Gherkin result | 3/3 steps pass |
| Focused command tests | 36 inline + 19 user_commands + 12 user_registry |
| Command palette tests | 17 tests |
| Slash completion tests | 17 tests |
| Harness availability | AT-001 feature file + runner committed |
| Workspace validation | `cargo check` passes cleanly |

## EPIC-002 Issue Checklist

When updating the GitHub [EPIC-style issue](https://github.com/Hmbown/CodeWhale/issues/2870),
use the following checklist format:

- [ ] Layer 4.0: Command Extraction Contract and Baseline **⬜**
  - [ ] Target branch/commit: `release/v0.8.60` / `<final Layer 4.0 commit hash>`
  - [ ] Gherkin acceptance harness smoke test
  - [ ] Command-module contract documented in architecture docs
  - [ ] Acceptance traceability mapped to existing tests
  - [ ] EPIC issue checklist prepared

- [ ] Layer 4.1: Core and Session Command Extraction **⬜**
  - [ ] Representative commands extracted to focused modules
  - [ ] Aliases, metadata, dispatch preserved
  - [ ] Palette/completion/help verify extracted commands visible
  - [ ] AT-002/AT-003 Gherkin scenarios implemented
  - [ ] Per-FEAT validation gate recorded

- [ ] Layer 4.2: Config and Debug Command Extraction **⬜**
  - [ ] Config commands extracted to focused modules
  - [ ] Debug commands extracted to focused modules
  - [ ] Palette/completion/help verify extracted commands visible
  - [ ] AT-004 Gherkin scenario implemented
  - [ ] Per-FEAT validation gate recorded

- [ ] Layer 4.3: Project, Memory, Skills, and Utility Extraction **⬜**
  - [ ] Project commands extracted to focused modules
  - [ ] Memory commands extracted to focused modules
  - [ ] Skills commands extracted to focused modules
  - [ ] Utility commands extracted to focused modules
  - [ ] Palette/completion/help verify extracted commands visible
  - [ ] Per-FEAT validation gate recorded

- [ ] Layer 4.4: Registry Cleanup, Documentation, and Full Validation **⬜**
  - [ ] Temporary adapters removed or documented as permanent
  - [ ] AT-005, AT-006, AT-007 Gherkin scenarios implemented
  - [ ] AT-008 duplicate registration check green
  - [ ] AT-009 ownership contract enforced (static check)
  - [ ] AT-010 migration cleanup inventory complete
  - [ ] AT-011 final validation evidence recorded

- [ ] EPIC acceptance gate: All acceptance tests green on final target branch
  with recorded evidence **⬜**

## Evidence Template

For each extraction layer PR, record validation evidence in this format:

```text
Validation evidence for Layer 4.x

Target branch/commit:
Gherkin command:
Gherkin result:
Focused command tests:
Command palette tests:
Slash completion tests:
Duplicate registration check (AT-008):
Ownership-contract check (AT-009):
Migration-cleanup inventory (AT-010):
Full workspace validation:
Known unrelated failures:
```

## Related Documents

- [ARCHITECTURE.md](../../docs/ARCHITECTURE.md) — overall system architecture
- [command-dispatch.md](./command-dispatch.md) — current dispatch architecture
  (post EPIC-001)
- [#2870](https://github.com/Hmbown/CodeWhale/issues/2870) — EPIC tracking issue
- [#2791](https://github.com/Hmbown/CodeWhale/issues/2791) — Original refactor issue
- [#2851](https://github.com/Hmbown/CodeWhale/pull/2851) — Proof/Reference PR
- [#2887](https://github.com/Hmbown/CodeWhale/pull/2887) — Acceptance harness PR
- [#3278](https://github.com/Hmbown/CodeWhale/pull/3278) — EPIC-001 closure/replay PR
