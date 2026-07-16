# bug-0048: All path existence probes are two-state — I/O errors classify as “absent”, and special files misclassify entirely

**Severity:** low
**Fix side:** CODE
**Component:** `src/utils/path.rs:175` (`current.exists()`); `src/widgets/file_picker.rs:165,171,178,184` (`is_dir` / `is_file` / `exists` in `validate_file_path`), `src/widgets/file_picker.rs:391` (`!parent.exists()` in `directory_to_create`), `src/widgets/file_picker.rs:454,467,505` (display), `src/widgets/file_picker.rs:751` (`path.is_file()` in the drop handler); `src/widgets/directory_picker.rs:111-117, 382-388, 521-523`
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

Every existence probe in the path utilities and both pickers is a bare `bool`—`exists()`, `is_dir()`, `is_file()`—which folds every I/O error (`EACCES`, `EIO`, an unmounted volume) into `false`.  The governing principle is Chris’s portfolio rule `positive-evidence-of-absence`: a probe that gates a Valid/Invalid verdict or a will-create decision must distinguish Present / Absent / Unknown, and only a provable `NotFound` may be read as absent.  Here, “could not look” is silently laundered into “not there”.

Consequences:

- **Unreadable or unmounted locations:** a path under an unreadable directory or an unmounted volume displays red `path does not exist` and `is_valid()` returns `false` even though the path exists.  In Save mode with `MissingDirectories::Create`, `validate()` returns `WillCreatePath` and `directory_to_create()` returns a directory that actually exists but merely failed to stat—inviting the consumer to `create_dir_all` over it.
- **Special files:** verified live that an existing FIFO yields `Open` → `PathDoesNotExist` (it exists) and `Save` → `Valid` with the display showing `file will be created` (writing would block on, or write into, the FIFO).

Nothing in this crate deletes, so no destruction gate is involved, but `RepeatableFilePicker::directories_to_create()` output feeds consumer-side creation.  `SECURITY.md:43-50` explicitly disclaims path validation as a security boundary, which is what keeps this at low severity.

## Reproduction (test-ready)

FIFO case—plain unit test plus one setup command:

```bash
mkdir -p /tmp/bug_0048 && mkfifo /tmp/bug_0048/fifo
```

```rust
let p = "/tmp/bug_0048/fifo";
assert!(std::path::Path::new(p).exists()); // the FIFO exists
assert_eq!(
    validate_file_path(p, &FileMode::Open, &MissingDirectories::Error),
    FilePickerValidation::PathDoesNotExist   // passes today — the bug
);
assert_eq!(
    validate_file_path(p, &FileMode::Save, &MissingDirectories::Error),
    FilePickerValidation::Valid              // passes today — the bug
);
```

`EACCES` case: `chmod 000` a parent directory and validate a child path—the result is `PathDoesNotExist` / `ParentDoesNotExist` although nothing is known to be absent.  (Skip this assertion when running as root, which bypasses permission checks.)

## Suggested Fix

- Replace the bare-`bool` probes with `std::fs::symlink_metadata` / `metadata`, matching `ErrorKind::NotFound` explicitly and propagating everything else as a third state.
- Add an `Unknown` (or `Unreadable`) variant to both `FilePickerValidation` and `DirectoryPickerValidation`, displayed as a warning rather than an error.
- Make `directory_to_create()` return `None` (or an error) when the probe errors, rather than treating the error as “missing”.
- In `Open` mode, handle a `FileType` that is neither file nor directory as its own case instead of falling into `PathDoesNotExist`.

## Why the Fix Works

Three-state probes stop laundering “could not look” into “not there”: the UI reports uncertainty as uncertainty, and consumers are never told to create a directory that merely failed to stat.

## Fix Ordering / Related

Land bug-0053 first—the three-state probes build on the same shared resolution pipeline (`parse_path` feeding validation).  Coordinate with bug-0026 (the parent `is_dir()` check) and bug-0020 (the new display branch), both of which edit probes this rework replaces.
