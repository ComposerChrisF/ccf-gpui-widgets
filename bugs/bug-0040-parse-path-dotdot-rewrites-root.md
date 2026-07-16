# bug-0040: `..` after a nonexistent path component silently rewrites the path to filesystem root

**Severity:** medium
**Fix side:** CODE
**Component:** `src/utils/path.rs:170-196` (`find_existing_prefix`); consumed at commit by `src/widgets/file_picker.rs:566-571` and `src/widgets/directory_picker.rs:349-356`
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

`find_existing_prefix` walks a path upward with `file_name()` / `parent()` to find the longest existing prefix (`src/utils/path.rs:179-190`):

```rust
match current.file_name() {
    Some(component) => {
        suffix_components.push(component.to_os_string());
        current = current
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("/"));
    }
    None => {
        return (PathBuf::from("/"), build_suffix(&suffix_components));
    }
}
```

`Path::file_name()` returns `None` when the path ends in `..`.  When a `..` sits inside the non-existing region of the path (so the `current.exists()` check at `src/utils/path.rs:175` never resolved it), the walk eventually reaches it and hits the `None` arm—which discards _everything_ before the `..` and fabricates `/` as the existing prefix.

Verified live:

- `parse_path("<tmp>/nodir_xyz/../file.txt")` → `full_path == "/file.txt"`
- `parse_path("<tmp>/nodir_xyz/..")` → `full_path == "/"` with `fully_exists() == true`

Because both pickers commit `path_info.full_path_string()` as the new value on Enter/Blur (`src/widgets/file_picker.rs:566-571`, `src/widgets/directory_picker.rs:349-356`), a user who types a `..` after a nonexistent component gets the path silently rewritten to a different absolute location at root—and in Save mode the consumer will then write to `/file.txt`.

A `..` in the _existing_ region is unaffected: the OS resolves it during `exists()` / `canonicalize()`.

Secondary issue in the same function: the hardcoded `PathBuf::from("/")` (lines 185 and 188) is a Unix-only assumption.

## Reproduction (test-ready)

Plain unit test on the pure function—no GPUI machinery needed:

```rust
#[test]
fn dotdot_in_missing_region_preserves_prefix() {
    let existing_dir = std::env::temp_dir().canonicalize().unwrap(); // exists
    let input = format!("{}/nodir/../file.txt", existing_dir.display()); // `nodir` does not exist
    let info = parse_path(&input);
    let expected = existing_dir.join("file.txt");
    assert_eq!(info.full_path, expected); // fails today: full_path == "/file.txt"
}
```

Expected: `full_path == <existing_dir>/file.txt` (or, at worst, the typed components preserved unresolved).
Actual: `full_path == "/file.txt"`.

## Suggested Fix

In `parse_path` (`src/utils/path.rs:83-129`), lexically normalize the absolute path before calling `find_existing_prefix`: fold over `Path::components()`, pushing `Component::Normal` entries, popping the last pushed entry on `Component::ParentDir`, keeping `RootDir`/`Prefix`, and dropping `CurDir`.  After that, `file_name()` can only return `None` at a genuine root, so the walk in `find_existing_prefix` visits real ancestors only.

Alternative (smaller but weaker): in the `None` arm, stop and return the remaining `current` unresolved as part of the suffix rather than fabricating `/`.

## Why the Fix Works

After lexical normalization no interior `..` / `.` remains, so the parent walk only ever visits real ancestors of the input, and the throw-away-the-prefix arm becomes unreachable except at a genuine root.

## Fix Ordering / Related

- bug-0047: the trailing-`..` case is also what lets a missing path report `fully_exists() == true` at `/`; this fix removes that route, but bug-0047’s empty-input case needs its own one-line fix.
- bug-0046: paths whose existing prefix degenerates to `/` are the main real-world trigger for the double-slash display; the fixes are independent, but the reproductions overlap.
