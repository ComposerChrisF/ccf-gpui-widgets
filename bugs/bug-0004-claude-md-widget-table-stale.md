# bug-0004: CLAUDE.md Widget Quick Reference table — 10 rows contradict the actual event enums and builder methods

**Severity:** high
**Fix side:** SPEC (docs)
**Component:** CLAUDE.md:212-238 (preamble at 214, table at 216-238)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

> **THE DOCS ARE THE PROBLEM — the code is correct and consumed by clui.  Do NOT change code to match the docs.**

## Description

The “Widget Quick Reference” table in `CLAUDE.md` (lines 216-238) was verified row-by-row against the widget sources.  Ten rows name events that do not exist, builder methods that do not exist, or wrong payload types.  Because `CLAUDE.md` is loaded into every Claude Code session in this repo, an agent that trusts the table writes non-compiling consumer code (e.g. subscribing to `ProgressBarEvent::Change` or calling `Checkbox::checked()`).

Row-by-row findings (line numbers are `CLAUDE.md` lines):

- **TextInput (218)**: omits the `Tab` and `ShiftTab` event variants (`text_input.rs:147-150`; emitted only when `emit_tab_events(true)` is set).
- **PasswordInput (219)**: claims events `Change, Enter, Escape, Blur, Focus` and builders `placeholder(), select_on_focus()`.  Code (`password_input.rs:54-65`): `Change`, `Enter`, `Blur` only; no `select_on_focus()` exists.  **[GATED on bug-0042 — the correct row depends on whether the event surface is widened.]**
- **Checkbox (222)**: builder listed as `checked()` — the code is `with_checked()` (`checkbox.rs:71`); the gallery uses `with_checked(true)` (`examples/widget_gallery.rs:426`).
- **ToggleSwitch (223)**: claims `Change(bool)` — the code’s event is `Toggle(bool)`.  **[GATED on bug-0079 — the event name itself is under decision.]**
- **Dropdown (224)**: lists `placeholder()` — no such method exists (`dropdown.rs:96-185`).  If the feature is wanted, that is a feature plan, not this bug.
- **CheckboxGroup (226)**: claims `Change(Vec<usize>)` and `with_selected_indices()` — the code is `Change(Vec<String>)` (`checkbox_group.rs:35-38`) and `with_selected(Vec<String>)` (`checkbox_group.rs:81`); no index-based API exists.  clui already matches the code.
- **ProgressBar (229)**: claims `Change(f64)` — the only event is `Complete` (`progress_bar.rs:42-45`); `TODO.md:345` agrees with the code.  **[GATED on bug-0055 — a `Change` event may be added.]**
- **Collapsible (231)**: claims builders `with_expanded(), title()` — the code is `with_collapsed(bool)` (`collapsible.rs:109`), and the title is a required argument of `new(title, cx)` (`collapsible.rs:96`); the `collapsible(bool)` builder is omitted.  Note the `Change(bool)` payload means _collapsed_ — the opposite polarity of the documented `with_expanded`.
- **Scrollable (234)**: `scrollable_vertical` / `scrollable_horizontal` / `scrollable_both` are free functions (`scrollable.rs:456-493`), not builder methods on a widget.
- **FilePicker / DirectoryPicker (236-237)**: claim `Change(PathBuf), Validated` — the code is `Change(String)`, and no `Validated` event exists anywhere (repo-wide grep: zero hits).  **[GATED on bug-0050 — the picker event surface is under decision.]**
- **RepeatableTextInput (238)**: claims `Add, Remove` — the code is `EntryAdded(usize)`, `EntryRemoved(usize)` (`repeatable_text_input.rs:57-64`).
- **Missing rows**: `Tooltip`, `RepeatableFilePicker`, `RepeatableDirectoryPicker` are all exported but absent from the table.  (The false preamble at line 214 — “All widgets follow the pattern `cx.new(|cx| Widget::new(cx).builder_methods())`” — is covered by bug-0003, not here.)

**Git evidence:** the table was added in commit `6d4beb9` (“Reorganize documentation by widget category and add quick reference”) with several rows already wrong at introduction (verified via `git show 6d4beb9:CLAUDE.md`, lines 213-226) — it was written aspirationally, not from code.  Subsequent refactors `b59e29c`, `5b331d8`, `87e592a`, and `466746c` never corrected it; `87e592a` and `466746c` updated _other_ parts of `CLAUDE.md` but not these rows.

## Reproduction (test-ready)

Documentation bug — verification is by comparison, no GPUI machinery needed:

1. Read the table at `CLAUDE.md:216-238`.
2. For each claimed event, grep the enum:

   ```bash
   grep -n 'pub enum' src/widgets/password_input.rs src/widgets/progress_bar.rs \
     src/widgets/checkbox_group.rs src/widgets/toggle_switch.rs \
     src/widgets/repeatable_text_input.rs src/widgets/file_picker.rs
   grep -rn 'Validated' src/   # zero hits
   ```

3. For each claimed builder, grep the method: `grep -n 'pub fn checked\|pub fn with_checked' src/widgets/checkbox.rs` (only `with_checked` exists); `grep -n 'pub fn placeholder' src/widgets/dropdown.rs` (no hits); `grep -n 'pub fn with_expanded\|pub fn with_collapsed' src/widgets/collapsible.rs` (only `with_collapsed`).
4. Expected: the table matches the enums and method signatures.  Actual: the ten rows above do not.

## Suggested Fix

Rewrite the affected rows in `CLAUDE.md:216-238` from the source enums and builders exactly.  Corrected rows that can be applied **immediately**:

| Widget | Event Type | Key Builder Methods |
|--------|-----------|---------------------|
| `TextInput` | `Change, Enter, Escape, Blur, Focus, Tab, ShiftTab` (last two only with `emit_tab_events(true)`) | `placeholder(), select_on_focus(), with_value()` |
| `Checkbox` | `Change(bool)` | `with_checked(), label()` |
| `Dropdown` | `Change(String), Open, Close` | `choices(), with_selected_index()` |
| `CheckboxGroup` | `Change(Vec<String>)` | `choices(), with_selected()` |
| `Collapsible` | `Change(bool)` (payload = collapsed) | `new(title, cx), with_collapsed(), collapsible()` |
| `Scrollable` | (no events) | free functions: `scrollable_vertical(), scrollable_horizontal(), scrollable_both()` |
| `RepeatableTextInput` | `Change(Vec<String>), EntryAdded(usize), EntryRemoved(usize)` | `with_values(), placeholder()` |

Also add the missing `Tooltip`, `RepeatableFilePicker`, and `RepeatableDirectoryPicker` rows.

The four **decision-gated rows must wait** — do not rewrite them until their decisions land, then write whatever was decided:

- `PasswordInput` → bug-0042 (current code truth: `Change, Enter, Blur`; `placeholder(), with_value()`).
- `ToggleSwitch` → bug-0079 (current code truth: `Toggle(bool)`).
- `ProgressBar` → bug-0055 (current code truth: `Complete`; builders `with_value(), indeterminate(), min(), max()`).
- `FilePicker` / `DirectoryPicker` → bug-0050 (current code truth: `Change(String)`; no `Validated`).

Do **not** change the code to match the table — the code side is self-consistent, tested, and consumed by clui.

## Why the Fix Works

The table becomes a faithful projection of the shipped enums and builders, so an agent or consumer copying from it produces compiling code; deferring the four gated rows prevents writing them twice.

## Fix Ordering / Related

- Gated rows wait on decisions **bug-0042** (PasswordInput events), **bug-0055** (ProgressBar `Change`), **bug-0050** (picker events), **bug-0079** (ToggleSwitch event name); all other rows can be fixed immediately.
- **bug-0028** (getter-naming decision) gates parts of this table only if renames are chosen there.
- **bug-0003** owns the false preamble at `CLAUDE.md:214` and the other `CLAUDE.md` drift; fix both files in one pass to avoid a half-updated document.
- **bug-0057** is the same class of rot in the README — the corrected event names must match across both.
