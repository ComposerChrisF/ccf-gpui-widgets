# bug-0050: Picker and repeatable-input event surfaces do not match the documented table — fix docs, or implement the documented events?

**Severity:** medium
**Fix side:** SPEC (docs) (decided 2026-08-12; see Decision below)
**Component:** `CLAUDE.md:236-238` vs `src/widgets/file_picker.rs:119-123`, `src/widgets/directory_picker.rs:76-80`, `src/widgets/repeatable_text_input.rs:56-64`
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

> **Decision (Chris, 2026-08-12): option (a) — correct the docs to the code.**  Table rows become `Change(String)` with no `Validated` variant; repeatable-input events documented as `EntryAdded(usize)` / `EntryRemoved(usize)`.

## Description

The widget quick-reference table in `CLAUDE.md:236-238` claims:

```
| FilePicker | Change(PathBuf), Validated | mode(), extensions(), placeholder() |
| DirectoryPicker | Change(PathBuf), Validated | placeholder() |
| RepeatableTextInput | Change(Vec<String>), Add, Remove | with_values(), placeholder() |
```

The code disagrees on every event column:

- `FilePickerEvent { Change(String) }` (`src/widgets/file_picker.rs:119-123`)—payload is `String`, not `PathBuf`, and there is no `Validated` variant.
- `DirectoryPickerEvent { Change(String) }` (`src/widgets/directory_picker.rs:76-80`)—same two discrepancies.
- `RepeatableTextInputEvent { Change(Vec<String>), EntryAdded(usize), EntryRemoved(usize) }` (`src/widgets/repeatable_text_input.rs:56-64`)—the events are `EntryAdded` / `EntryRemoved`, not `Add` / `Remove`.

No `Validated` event exists anywhere in `src/` or `docs/` (grep confirms zero hits), and no code path answers the question “when would `Validated` be emitted?”.  It is pure fiction in the spec.  Meanwhile `TODO.md:85-88` records the validation API as deliberately _pull_-style—`validate()` returning a detailed enum, `is_valid()`, and the standalone `validate_file_path()` / `validate_directory_path()` functions—all marked done.

## Reproduction (test-ready)

```bash
grep -rn "Validated" src/    # no output
grep -n "Change(PathBuf)" CLAUDE.md
```

Compare the enum definitions at the cited lines with `CLAUDE.md:236-238`.

## Suggested Fix

**The question for Chris:** should the documentation be corrected to the code, or should the documented push-style event surface be implemented?

- **Option (a) — doc-side (cheap, likely the intent):** correct the `CLAUDE.md` table rows to `Change(String)`, drop `Validated`, and rename `Add` / `Remove` to `EntryAdded(usize)` / `EntryRemoved(usize)`.  No code change; clui unaffected.
- **Option (b) — code-side (features / breaking changes):** implement a `Validated(FilePickerValidation)` / `Validated(DirectoryPickerValidation)` event emitted on validity transitions, and/or switch `Change` payloads to `PathBuf`, and/or rename the repeatable-input events to match the table.  Payload and rename changes are breaking for clui and need a coordinated update there.

**Recommendation:** option (a).  `TODO.md:85-88` records the pull-style validation API as the deliberate design; the push event was never implemented anywhere; and `String` payloads are what clui consumes today.  If a transition event is genuinely wanted later, it can be added as a non-breaking feature after the docs are truthful.

## Why the Fix Works

Either resolution makes the documented event surface match the real one, so a consumer written against the table stops subscribing to events that never fire.

## Fix Ordering / Related

This decision gates bug-0004 (the `CLAUDE.md` widget-table rewrite must reflect the decided event names and payloads).  Together with the other event-surface decisions (bug-0079, bug-0055, bug-0042) it also gates bug-0057 (README example corrections).  bug-0025 (the `extensions()` semantics decision) feeds the same table row.
