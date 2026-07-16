# bug-0017: Escape emits Primary (confirms the dialog) whenever there is no secondary button—including Danger style

**Severity:** medium
**Fix side:** CODE
**Component:** `src/widgets/confirmation_dialog.rs:330-338` (escape arm), vs the module doc at `src/widgets/confirmation_dialog.rs:8-12`
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

The escape arm of the dialog-level `on_key_up` handler (`src/widgets/confirmation_dialog.rs:330-338`):

```rust
"escape" => {
    // Escape: triggers secondary if exists, otherwise primary (for Info)
    if has_secondary {
        dialog.emit_button(DialogButton::Secondary, window, cx);
    } else {
        dialog.emit_button(DialogButton::Primary, window, cx);
    }
}
```

The fallback is intended `(for Info)` per the comment, but it applies to every style.  The module doc (lines 8–12) says of Danger: “Escape triggers secondary.  Enter does NOT trigger primary (must click explicitly).”  Yet a `DialogStyle::Danger` dialog built without `secondary_label()`—which the API permits; `new()` defaults `secondary_label: None`—will emit **`Primary`, the destructive action, on Escape**, the one key users press to mean “get me out of here”.  Enter is carefully gated for Danger (`"enter" if !is_danger`, line 339), but Escape is not.

The same doc/code mismatch exists, less dangerously, for Default/Warning without a secondary button: the doc says Escape triggers secondary, but the code confirms `Primary`.

## Reproduction (test-ready)

Needs GPUI test context (key dispatch).

1. Mount `ConfirmationDialog::new("Delete?", "This cannot be undone.", cx).style(DialogStyle::Danger).primary_label("Delete")` (no secondary label) and subscribe to `ConfirmationDialogEvent`.
2. Let the dialog take focus.
3. Press and release Escape.

Expected: nothing (or a non-destructive dismissal), per the Danger documentation.
Actual: `ConfirmationDialogEvent::Primary` is emitted.

## Suggested Fix

Restrict the fallback to Info at `src/widgets/confirmation_dialog.rs:330-338`:

```rust
"escape" => {
    if has_secondary {
        dialog.emit_button(DialogButton::Secondary, window, cx);
    } else if is_info {
        dialog.emit_button(DialogButton::Primary, window, cx);
    }
}
```

Danger (and arguably Default/Warning) with no secondary button should ignore Escape.

## Why the Fix Works

Escape can then never trigger the style-colored destructive action, and the Info convenience (single-button dismiss) is preserved exactly where the comment says it was intended.

## Fix Ordering / Related

Coordinate with bug-0016 and bug-0018, which modify the same `on_key_up` handler, and with the bug-0019 pass that reworks dialog focus.  No hard ordering beyond editing the same code.
