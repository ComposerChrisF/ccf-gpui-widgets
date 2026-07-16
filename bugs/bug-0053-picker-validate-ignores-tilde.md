# bug-0053: `validate()` and `directory_to_create()` use the raw value while the display expands `~`

**Severity:** medium
**Fix side:** CODE
**Component:** `src/widgets/file_picker.rs:151-199` (`validate_file_path`), `src/widgets/file_picker.rs:383-396` (`directory_to_create`), `src/widgets/file_picker.rs:401-403` (`validate`); `src/widgets/directory_picker.rs:102-118` (`validate_directory_path`), `src/widgets/directory_picker.rs:259-261` (`validate`)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

The two halves of each picker resolve the value differently.  Render calls `parse_path(&self.value)`, which runs `expand_tilde` (`src/utils/path.rs:89`).  But `validate()` calls `validate_file_path(&self.value, ...)` on the _raw_ string (`src/widgets/file_picker.rs:401-403`), and `directory_to_create()` does `let path = Path::new(&self.value);` (`src/widgets/file_picker.rs:388`)—also raw.

For a value set via `with_value("~/…")` or `set_value` (neither normalizes; only the edit-commit path runs the text through `parse_path`), the display shows the path as existing/green while `is_valid()` returns `false`.  Verified live: `parse_path("~").fully_exists() == true`, but `validate_directory_path("~") == PathDoesNotExist`.

Worse, `directory_to_create()` on the value `~/new_dir/out.txt` returns `Some(PathBuf::from("~/new_dir"))`—a literal tilde path.  A consumer that runs `create_dir_all` on the returned paths (the documented purpose: Save mode with `MissingDirectories::Create`, surfaced via `RepeatableFilePicker::directories_to_create`) creates a literal `./~/new_dir` directory in the current working directory—the classic literal-tilde hazard.

## Reproduction (test-ready)

Pure-function repro (plain unit test, requires a `$HOME` that exists):

```rust
assert_eq!(validate_directory_path("~"), DirectoryPickerValidation::PathDoesNotExist); // passes today — the bug
assert!(parse_path("~").fully_exists()); // the display side disagrees
```

Widget-level repro (GPUI context needed only to construct the entity; the methods under test are plain):

1. `FilePicker::new(cx).mode(FileMode::Save).missing_directories(MissingDirectories::Create).with_value("~/new_dir/out.txt")` (with `$HOME/new_dir` not existing).
2. `picker.validate()` → `WillCreatePath` (the raw `~/new_dir` is treated as missing).
3. `picker.directory_to_create()` → `Some(PathBuf::from("~/new_dir"))`.

Expected: `directory_to_create()` returns `Some($HOME/new_dir)`, and `validate()` judges the expanded path.
Actual: both operate on the literal `~` string.

## Suggested Fix

In `FilePicker::validate()`, `DirectoryPicker::validate()`, and `FilePicker::directory_to_create()`, resolve the value first, exactly as the display does:

```rust
let resolved = parse_path(&self.value);
validate_file_path(&resolved.full_path_string(), &self.mode, &self.missing_directories)
```

and in `directory_to_create()`, derive `parent` from `resolved.full_path` instead of `Path::new(&self.value)`.

Alternative—normalizing in `set_value` / `with_value`—would also work, but it changes what `Change` event payloads carry, which is a behavior change for consumers; the resolve-at-use fix is self-contained.

## Why the Fix Works

Display and programmatic validation then share one resolution pipeline, so they cannot disagree, and `directory_to_create()` can never emit a literal-`~` path for a consumer to `create_dir_all`.

## Fix Ordering / Related

Land this before bug-0048: the three-state probe rework builds on the same shared resolution pipeline.  Also coordinate with bug-0026, which edits the same `validate_file_path` parent check.
