# bug-0026: Save-mode validation reports `Valid` when the “parent directory” is an existing file

**Severity:** medium
**Fix side:** CODE
**Component:** `src/widgets/file_picker.rs:181-187` (`validate_file_path`), `src/widgets/file_picker.rs:505` (display)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

Save-mode validation asks only whether the parent path _exists_, not whether it is a directory (`src/widgets/file_picker.rs:181-187`):

```rust
// Check if parent directory exists
let parent_exists = path
    .parent()
    .is_some_and(|p| p.exists() || p.as_os_str().is_empty());

if parent_exists {
    FilePickerValidation::Valid
```

`p.exists()` is `true` when the parent is a _file_.  Verified live: `validate_file_path("<tmp>/existing_file.txt/newfile.txt", Save, Error)` returns `Valid`, and `compute_path_display` (line 505, the same `p.exists()` probe) shows `file will be created` in green.  Actually creating that file fails with `ENOTDIR`.  `directory_to_create()` (line 391, gated on `!parent.exists()`) consistently returns `None`, so nothing even attempts to repair the situation—the picker certifies an impossible save target.

## Reproduction (test-ready)

Plain unit test on the pure function—no GPUI machinery needed:

```rust
#[test]
fn save_mode_parent_is_file_is_not_valid() {
    let dir = std::env::temp_dir().join("bug_0026_repro");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("f.txt");
    std::fs::write(&file, b"x").unwrap();

    let input = format!("{}/new.txt", file.display()); // parent is a FILE
    let v = validate_file_path(&input, &FileMode::Save, &MissingDirectories::Error);
    assert_ne!(v, FilePickerValidation::Valid); // fails today: v == Valid
}
```

Expected: an invalid state (parent is not a directory).
Actual: `Valid`.

## Suggested Fix

Use `is_dir()` where “can contain a new file” is the real question:

- `src/widgets/file_picker.rs:184`: `p.is_dir() || p.as_os_str().is_empty()`
- `src/widgets/file_picker.rs:505`: `p.is_dir()`

For the exists-but-not-a-directory case, returning `ParentDoesNotExist` is serviceable; a dedicated variant (e.g. `ParentNotADirectory`) would be clearer if the enum is being touched anyway.

## Why the Fix Works

“The file can be created here” requires the parent to be a directory, not merely an existing inode; `is_dir()` states that requirement directly, and the display probe changes in lockstep so the two can never disagree.

## Fix Ordering / Related

Coordinate with bug-0048 (the three-state probe rework replaces these same `exists()` / `is_dir()` calls—this fix should adopt whatever probe shape that lands) and bug-0053 (which changes what string reaches `validate_file_path`).  The `is_dir()` semantics are compatible with both.
