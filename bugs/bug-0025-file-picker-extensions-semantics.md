# bug-0025: `extensions()` is enforced only in the native dialog — typing, drag-drop, and `set_value` accept any extension

**Severity:** low
**Fix side:** SPEC (docs) (decided 2026-08-12; see Decision below)
**Component:** `src/widgets/file_picker.rs:624-628` (sole use of `self.extensions`), vs `src/widgets/file_picker.rs:151-199` (`validate_file_path`, no extension parameter) and `src/widgets/file_picker.rs:749-760` (drop handler, no extension check)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

> **Decision (Chris, 2026-08-12): option (a) — `extensions()` is a dialog-only convenience filter.**  Document that in the doc comment, the README, and the CLAUDE.md table; no behavior change.

## Description

The `extensions` field is used in exactly one place—as a filter on the native dialog (`src/widgets/file_picker.rs:624-628`):

```rust
if !extensions.is_empty() {
    let ext_refs: Vec<&str> =
        extensions.iter().map(|s| s.as_str()).collect();
    dialog = dialog.add_filter("Files", &ext_refs);
}
```

Every other input path ignores it: `validate_file_path` (`src/widgets/file_picker.rs:151-199`) takes no extension parameter, and the drag-drop handler (`src/widgets/file_picker.rs:749-760`) does no extension check.  A picker configured with `extensions(vec!["json"])` therefore reports `Valid` and `is_valid() == true` for `photo.png` entered by typing, drag-drop, or `set_value`—and `RepeatableFilePicker::is_all_valid()` inherits the same answer.

Whether the filter is _meant_ as validation is unspecified: `CLAUDE.md:236` lists `extensions()` as a key builder method right next to the fictional `Validated` event (see bug-0050), and neither the README nor the doc comments say “dialog-only”.  Note also that case handling is currently delegated to `rfd` inside the dialog, while nothing else compares extensions at all—so multi-dot names (`archive.tar.gz`), dotfiles, and case variants are simply never examined anywhere.

## Reproduction (test-ready)

GPUI context needed only to construct the entity; the methods under test are plain.

1. `FilePicker::new(cx).extensions(vec!["json".into()]).with_value("/etc/hosts")`.
2. `picker.validate()` → `Valid` (Open mode, the file exists); `is_valid()` → `true`.

Expected under an enforcing reading: an invalid-extension state.
Actual: `Valid`—the extension list plays no part outside the dialog.

## Suggested Fix

**The question for Chris:** is `extensions()` a dialog-only convenience filter, or a validation constraint that every input path (typed, dropped, programmatic) must satisfy?

- **Option (a) — document dialog-only:** state in the `extensions()` doc comment, the README, and the `CLAUDE.md` table that the filter applies only to the native dialog.  No behavior change; cheap; non-breaking.
- **Option (b) — enforce in validation:** add an `InvalidExtension` variant to `FilePickerValidation`, checked case-insensitively against the final `Path::extension()` in `validate_file_path`—which then needs a signature change to accept the extension list, rippling to `validate()`, the display, and the public standalone-function API.

**Recommendation:** lean option (a) unless clui actually needs enforcement.  It is non-breaking and makes the current behavior explicit, whereas option (b) is a real feature with API ripple and open sub-decisions of its own (case sensitivity, multi-dot names, dotfiles).  Either resolution removes the unspecified middle ground, which is the actual bug.

## Why the Fix Works

Both options make the filter’s scope explicit, so a consumer can no longer assume dialog-level filtering implies validation-level enforcement.

## Fix Ordering / Related

Feeds the same doc rewrite as bug-0050 and bug-0004 (the corrected widget table must describe `extensions()` accurately, whichever way this lands).  If option (b) is chosen, coordinate with bug-0053 (what string reaches `validate_file_path`) and bug-0048 (both validation enums are gaining variants there).
