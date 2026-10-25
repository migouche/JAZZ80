# JAZZ80 - Hardening, Technical Debt Reduction & Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
> 
> **Execution handoff:** After review, I recommend **Subagent-driven development**. The tasks have tight interface coupling (especially around CPU tables, opcode parts, assembler AST, and the `Workspace`/`DebugSession` split) and a mistake in any of the "REPLACE 1-4" items will cascade through the rest of the codebase. The cost of an independent review between tasks far outweighs the context cost here.

**Goal:** Systematically harden JAZZ80 (Rust 2024, egui 0.37.0, Z80 emulator/assembler/debugger) by fixing correctness bugs, completing the four critical REPLACE items, collapsing `gui/mod.rs` and `components/devices.rs` into testable modules, eliminating cognitive complexity, and moving towards self-documenting code with minimal comments.

**Architecture:** Enforce separation of concerns (pure models/services/platform/UI), introduce explicit state machines/enums where boolean soup exists, replace table/tuple boilerplate with declarative data, move I/O out of `ui()`, and make persistence schema-versioned. Prefer composition and newtypes over growing god-objects.

**Tech Stack:** Rust 2024, eframe/egui 0.37.0, rfd, wasm-bindgen/web-sys, serde, tracing, rstest, egui_kittest.

**Spec:** This plan is derived from the full codebase audit performed against the current tree (HEAD `bf8ad6e`, `cargo clippy --all-targets --all-features` clean). `docs/ui.png` could not be visually inspected and findings are based on source.

**Plan saved to:** `docs/superpowers/plans/2026-10-24-jazz80-hardening-refactor.md`

## Global Constraints

- **No behavior changes without tests.** Every correctness fix must have a regression test that fails before the fix and passes after.
- **Self-documenting over commented.** Remove `test_log!`/`test_callback` from production paths. Replace mystery tuples/booleans with named types/enums.
- **Preserve platform parity.** Native and WASM must both compile and behave correctly. Respect `#[cfg(target_arch = "wasm32")]` and `#[cfg(not(target_arch = "wasm32"))]` boundaries.
- **Don't break existing tests.** `cargo test --all-targets` must pass at the end of every phase (and preferably after each task).
- **Minimal diffs per task.** Keep tasks reviewable. Follow existing style (formatting, imports, module layout).
- **Persistence compatibility.** Any change to `Z80App` serialization must be backwards compatible via `#[serde(default)]` and/or a new `PersistedWorkspace` with a `version` field.
- **egui 0.37 API.** Use `egui::Panel`, `Window::show` accepting `&mut Ui`, `ui.close()`, etc. as they are used currently.
- **Clippy clean.** `cargo clippy --all-targets --all-features -- -D warnings` must remain clean.

## Review Focus (spec-implied inputs most likely to bite)

1. **CB rotate flags/target register** – Ensure `RLC/RRC/RL/RR/SLA/SRA/SLL/SRL` on `A` and memory update F correctly and do *not* overwrite the destination register with flags. (Test must assert both the result register and F.)
2. **Level-triggered INT** – `INT` must fire at most once per assertion, be cleared after service, and not create an interrupt storm when `EI` is executed. (Test pulse behavior.)
3. **Tab focus/history isolation** – Switching tabs must not share `TextEdit` undo history. (Test via widget id strategy or behavior.)
4. **Persisted state corruption** – Loading old/wonky serialized state with `active_tab` out of range or missing fields must not panic. (Test load with clamped indices/defaults.)
5. **Breakpoint sync ordering** – Toggling breakpoints and pressing F5 in the same frame must use the latest breakpoints. (Test UI action ordering.)

---

## Phase 0: Critical Correctness Bugs (P0) 

These are small, high-impact fixes. Do each with a regression test.

### Task 0.1: Fix CB rotate writing flags to register A

**Files:**
- Modify: `src/cpu/mod.rs:1449-1468`
- Test: `src/cpu/tests/instructions/cb/rot.rs` (extend)

**Interfaces:**
- Consumes: `alu::rot::*`, `Z80A::set_register/get_register`, flag calculations
- Produces: Correct CB rotate behavior for all registers including `A` and memory

**Steps:**

- [ ] **Step 1: Write failing tests for rot flags and target register (including A)**
- [ ] **Step 2: Run tests to verify they fail**
- [ ] **Step 3: Remove the stray write and fix `rot`**
- [ ] **Step 4: Run tests to verify they pass**
- [ ] **Step 5: Commit**

### Task 0.2: Make INT a pulse / clear int_pending after service

**Files:**
- Modify: `src/cpu/mod.rs` (add `clear_interrupt`, adjust `handle_int`/`is_int_line_active`)
- Modify: `src/gui/mod.rs` (INT button behavior)
- Modify: `src/gui/exec/*` if using `Command::SetInterrupt` (or add `PulseInterrupt`)
- Test: `src/cpu/tests/interrupts.rs` or new test

**Steps:**

- [ ] **Step 1: Write failing tests**
- [ ] **Step 2: Run to fail**
- [ ] **Step 3: Implement pulse semantics**
- [ ] **Step 4: Update UI**
- [ ] **Step 5: Run tests**
- [ ] **Step 6: Commit**

### Task 0.3: Isolate tab TextEdit undo history

**Files:**
- Modify: `src/gui/mod.rs` (editor rendering)
- Modify: `src/gui/mod.rs` (EditorTab if needed)

**Steps:**

- [ ] **Step 1: Add stable tab id**
- [ ] **Step 2: Wrap editor in `ui.push_id`**
- [ ] **Step 3: Test behavior**
- [ ] **Step 4: Commit**

### Task 0.4: Clamp active_tab on load and harden indexing

**Files:**
- Modify: `src/gui/mod.rs` (`new`, `default`, access sites)
- Test: unit test for load

**Steps:**

- [ ] **Step 1: Clamp on restore**
- [ ] **Step 2: Prefer safe access where easy**
- [ ] **Step 3: Test**
- [ ] **Step 4: Commit**

### Task 0.5: Sync breakpoints before action dispatch in same frame

**Files:**
- Modify: `src/gui/mod.rs` (`ui` flow)

**Steps:**

- [ ] **Step 1: Move sync earlier**
- [ ] **Step 2: Verify**
- [ ] **Step 3: Commit**

**Phase 0 gate:** `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings` pass.

---

## Phase 1: Critical REPLACE 1-4 (Highest Priority)

### Task 1.1: REPLACE 1 - Declarative table macro for enum conversions
### Task 1.2: REPLACE 2 - Named opcode parts struct
### Task 1.3: REPLACE 3 - Snapshot memory storage
### Task 1.4: REPLACE 4 - Two-pass assembler with Statement enum

**Phase 1 gate:** All REPLACE 1-4 done, tests pass, clippy clean.

---

## Phase 2: GUI Architecture - Split god object
### Task 2.1: Extract pure models
### Task 2.2: Split UI into modules
### Task 2.3: Move platform I/O out of ui()

**Phase 2 gate:** `gui/mod.rs` dramatically smaller, `ui()` is a coordinator, all tests pass.

---

## Phase 3: Devices Refactor
### Task 3.1: Trait defaults + per-device modules + safer locking

**Phase 3 gate:** Devices refactored, tests pass.

---

## Phase 4: Theme, Highlighting, Formatting
### Task 4.1: Unify themes with tokens
### Task 4.2: Optimize highlighting
### Task 4.3: Move Data column formatting to assembly time

**Phase 4 gate:** Tests pass.

---

## Phase 5: Cleanup & Polish
### Task 5.1: Remove `test_log!` from production CPU
### Task 5.2: UI polish
### Task 5.3: Misc

**Final gate:** `cargo test`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --release` pass.

## Execution Notes

- **Start with Phase 0.** Bugs are independent and unblock confidence.
- **Do REPLACE 1-4 early (Phase 1).** They reduce complexity before the big GUI split.
- **Split GUI incrementally.** Keep app compiling after each subtask.
- **Tests first for fixes.** Follow TDD for P0 items.
- **Don't over-comment.** Remove misleading comments like the rot one.

**Plan complete.** Using subagent-driven development.
