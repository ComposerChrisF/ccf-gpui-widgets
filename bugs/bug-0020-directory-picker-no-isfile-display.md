# bug-0020: DirectoryPicker display has no is-a-file case — a file path renders as calmly valid while `validate()` returns `IsFile`

**Severity:** medium
**Fix side:** CODE
**Component:** `src/widgets/directory_picker.rs:281-332` (`compute_path_display`); mirror-image pattern already implemented at `src/widgets/file_picker.rs:454-465`
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

`validate_directory_path` distinguishes the file case (`src/widgets/directory_picker.rs:113-114`):

```rust
} else if path.is_file() {
    DirectoryPickerValidation::IsFile
```

but `compute_path_display` (`src/widgets/directory_picker.rs:281-332`) branches only on `path_info.fully_exists()`.  An existing file fully exists, so the entire path is rendered in `theme.text_muted` with no highlight and no explanation—visually indistinguishable from a valid directory.

`FilePicker` implements exactly the mirror-image case (“file expected, but path is a directory”, `src/widgets/file_picker.rs:454-465`) with the final segment in `theme.warning` and an explanation line.  `DirectoryPicker` is missing the counterpart, so a consumer form shows no problem while `is_valid()` returns `false` and the form refuses to proceed—the user gets a silent dead end.

## Reproduction (test-ready)

Needs GPUI test context for construction and render; the validation half is plain.

1. `DirectoryPicker::new(cx).with_value("/etc/hosts")`—or `set_value` any existing file path.  Note the drag-drop handler (`src/widgets/directory_picker.rs:519-537`) substitutes the parent directory for a dropped file, so typing or `set_value` is the reproduction path, not drag-drop.
2. Render the widget and inspect the computed path display.

Expected: the final segment highlighted (warning color) with an explanation such as `directory expected, but path is a file`.
Actual: the whole path renders muted with no explanation, while `picker.validate()` returns `IsFile` and `is_valid()` returns `false`.

## Suggested Fix

At the top of `DirectoryPicker::compute_path_display` (after the empty check at `src/widgets/directory_picker.rs:289-291`), mirror the FilePicker pattern:

```rust
if path_info.full_path.is_file() {
    if let Some(parent) = path_info.full_path.parent() {
        info.add_segment(&parent.to_string_lossy(), theme.text_muted);
    }
    if let Some(name) = path_info.full_path.file_name() {
        info.add_path_prefix(&name.to_string_lossy(), color_or_muted(theme.warning));
    }
    if show_message {
        info.set_explanation("directory expected, but path is a file", theme.warning);
    }
    return info;
}
```

## Why the Fix Works

It gives the display parity with the validation enum—every `IsFile` verdict now has a visible counterpart—using the exact segment/prefix/explanation pattern already proven in `FilePicker`.

## Fix Ordering / Related

Coordinate with bug-0048: the three-state probe rework replaces the display-side `is_file()` / `exists()` probes, and this new branch should adopt whatever probe shape that fix lands.  Also related to bug-0046 (`add_path_prefix` separator behavior), which this branch calls.
