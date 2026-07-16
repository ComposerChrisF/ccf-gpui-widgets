# bug-0003: CLAUDE.md drift — widget template reintroduces the tab-stop bug, and four other sections contradict the codebase

**Severity:** medium
**Fix side:** SPEC (docs)
**Component:** CLAUDE.md:51-87 (directory structure), 152-200 (widget template, esp. 164), 214 (table preamble), 285-297 (adding a new widget), 299-306 (testing), 322 (known limitation #3)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

> **THE DOCS ARE THE PROBLEM — the code is correct and consumed by clui.  Do NOT change code to match the docs.**

## Description

Five distinct drifts in `CLAUDE.md`, all of which mislead an agent following the file as instructions:

**(a) The Widget API Pattern template reintroduces a solved bug.**  The template at `CLAUDE.md:164` constructs the focus handle as:

```rust
focus_handle: cx.focus_handle(),
```

— without `.tab_stop(true)`.  This is the exact bug documented in `docs/TAB_STOP_SOLUTIONS.md` and `TODO.md:431-442` (“Tab Navigation Fix (2026-01-26)”): setting `tab_stop` on the div after `.track_focus()` does nothing, so the handle must be created as `cx.focus_handle().tab_stop(true)`.  Every shipped focusable widget does this (e.g. `checkbox.rs:63`).  An agent following the template verbatim produces a widget invisible to Tab navigation.  The template and the “Adding a New Widget” checklist (`CLAUDE.md:285-297`) also omit the `enabled: bool` field (plus `with_enabled()` / `is_enabled()` / `set_enabled()`) that all 17 interactive widgets carry since 2026-02-01 (`TODO.md:182-189`).

**(b) The “Directory Structure” listing (`CLAUDE.md:51-87`) omits six real files:** `src/widgets/cursor_blink.rs`, `src/widgets/editing_core.rs`, `src/widgets/path_display.rs`, `src/widgets/selection.rs`, `src/widgets/sensitive_string.rs`, and `src/utils/color.rs`.

**(c) The “Testing” section (`CLAUDE.md:299-306`) says tests live only in `src/utils/path.rs`.**  Actually there are 63 tests across 8 files: `path.rs:198`, `utils/mod.rs:34`, `color.rs:634` and `:804` (including proptest), `sensitive_string.rs:127`, `directory_picker.rs:845`, `file_picker.rs:1069`, `editing_core.rs:551`.  The picker and sensitive-string tests are feature-gated, so `cargo test --features full` is needed to run them all — the section mentions neither.

**(d) Known Limitation #3 (“No built-in validation”, `CLAUDE.md:322`) is false as stated.**  The pickers ship a validation API — `validate()`, `is_valid()`, `ValidationDisplay`, plus standalone `validate_file_path()` / `validate_directory_path()` (`TODO.md:79-89`) — and `TextInput` has `input_filter()`.  What the limitation _means_ is that there is no general validation framework and path validation is not a security boundary (per `SECURITY.md`); it should say that.

**(e) The table preamble (`CLAUDE.md:214`) — “All widgets follow the pattern: `cx.new(|cx| Widget::new(cx).builder_methods())`” — is false for six widgets.**  `ProgressBar::new()` and `Spinner::new()` take no `cx`; `Collapsible::new(title, cx)`, `ConfirmationDialog::new(title, message, cx)`, `TabBar::new(tabs, selected, cx)`, and `SidebarNav::new(items, selected, cx)` take required arguments before `cx`.

## Reproduction (test-ready)

Documentation bug — verification by comparison:

1. (a) `grep -n 'focus_handle()' src/widgets/checkbox.rs` → line 63 shows `cx.focus_handle().tab_stop(true)`; compare with `CLAUDE.md:164`.  Then read `TODO.md:431-442` for why the difference matters.  Scaffold a widget from the template and Tab through a window with it (needs GPUI test context or the gallery): the widget is skipped.
2. (b) `ls src/widgets/ src/utils/` and diff against `CLAUDE.md:51-87` — six files missing.
3. (c) `cargo test --features full 2>&1 | tail -5` — tests run from 8 files, not 1; `grep -rn 'mod tests' src/ | wc -l` corroborates.
4. (d) `grep -n 'pub fn validate\|ValidationDisplay' src/widgets/file_picker.rs src/widgets/directory_picker.rs` and `grep -n 'pub fn input_filter' src/widgets/text_input.rs` (line 303) — the “no built-in validation” claim contradicts shipped API.
5. (e) `grep -n 'pub fn new' src/widgets/progress_bar.rs src/widgets/spinner.rs src/widgets/collapsible.rs src/widgets/confirmation_dialog.rs src/widgets/tab_bar.rs src/widgets/sidebar_nav.rs` — six signatures contradict the preamble.

## Suggested Fix

All edits in `CLAUDE.md`:

- **(a)** Line 164: change the template to `focus_handle: cx.focus_handle().tab_stop(true),` with a one-line comment pointing at `docs/TAB_STOP_SOLUTIONS.md`.  Add `enabled: bool` (default `true`) plus the `with_enabled()` / `is_enabled()` / `set_enabled()` trio to the template, and add a step for them to “Adding a New Widget” (lines 285-297).
- **(b)** Add the six missing files to the directory listing at lines 51-87 with one-line descriptions.
- **(c)** Rewrite lines 299-306: tests live in 8 files (~63 total), some feature-gated; document `cargo test` and `cargo test --features full`.
- **(d)** Reword line 322 to scope the claim: no general validation framework; pickers provide path-validation UI feedback (`validate()`, `is_valid()`, `ValidationDisplay`) and `TextInput` has `input_filter()`; none of it is a security boundary (see `SECURITY.md`).
- **(e)** Reword line 214 to: most widgets follow `cx.new(|cx| Widget::new(cx).builders())`; `ProgressBar`/`Spinner` need no `cx`, and `Collapsible`, `ConfirmationDialog`, `TabBar`, `SidebarNav` take required constructor arguments.

## Why the Fix Works

The template stops regenerating a documented, already-fixed bug; the structure, testing, and limitation sections stop contradicting observable repo state, so agents following `CLAUDE.md` produce widgets matching the shipped conventions.

## Fix Ordering / Related

- **bug-0004** owns the Widget Quick Reference table rows (this bug owns only the preamble at line 214) — fix both in one `CLAUDE.md` pass.
- **bug-0028** (getter-naming decision) may amend the Naming Convention section; if decided first, fold its outcome into the same pass.
- **bug-0075**’s code fix should match the corrected template from (a) (handle-level `tab_stop`).
- **bug-0023** owns the “~30 semantic color fields” claim at `CLAUDE.md:253`.
