# bug-0016: Enter on a focused dialog button double-fires—the button’s event and Primary are both emitted (latent until bug-0019 is fixed)

**Severity:** medium
**Fix side:** CODE
**Component:** `src/widgets/confirmation_dialog.rs:312-345` (dialog-level `on_key_up`), interacting with gpui `div.rs:2178-2207`
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

> **MASKED TODAY—MUST BE FIXED IN THE SAME PASS AS bug-0019.**  This bug cannot currently manifest, because bug-0019 (render-time focus steal) prevents dialog buttons from ever holding focus.  Fixing bug-0019 alone exposes this bug immediately.

## Description

gpui’s built-in keyboard click for a focused element fires on `KeyUpEvent` and does **not** stop propagation (gpui `div.rs:2183-2207`).  The dialog’s `on_key_up` handler lives on an ancestor (the dialog box) and dispatches during the same bubble walk (gpui `window.rs:3855`, `finish_dispatch_key_event` → `dispatch_key_down_up_event`).

So once a dialog button can actually hold focus (i.e., once bug-0019 is fixed), pressing and releasing Enter on the focused `dialog_secondary` button will:

1. Fire the button’s own click → `emit_button(DialogButton::Secondary, ...)`.
2. Bubble to the dialog box’s `on_key_up`, match the `"enter" if !is_danger` arm (`src/widgets/confirmation_dialog.rs:339-342`), and `emit_button(DialogButton::Primary, ...)`.

Subscribers receive contradictory `Secondary` _and_ `Primary` events for one keystroke.  The second `emit_button` also runs after `previous_focus` was already `take()`n by the first (`src/widgets/confirmation_dialog.rs:206`), so its focus-restore branch is a no-op.

## Reproduction (test-ready)

Needs GPUI test context, **and requires bug-0019’s fix to be in place first** (otherwise the button never holds focus and the double-fire cannot occur).

1. Mount `ConfirmationDialog::new("Confirm", "Are you sure?", cx).primary_label("Yes").secondary_label("No")` and subscribe to `ConfirmationDialogEvent`.
2. Press Tab until focus is on the `dialog_secondary` (“No”) button.
3. Press and release Enter; record the emitted events.

Expected: only `Secondary`.
Actual (once bug-0019 is fixed, without this fix): `Secondary` then `Primary`.

## Suggested Fix

In the dialog’s `on_key_up` `"enter"` arm (`src/widgets/confirmation_dialog.rs:339-342`), only emit `Primary` when the dialog box itself is focused—clone the focus handle into the closure and guard:

```rust
"enter" if !is_danger => {
    if focus_handle.is_focused(window) {
        dialog.emit_button(DialogButton::Primary, window, cx);
    }
}
```

Equivalently: skip the default enter/escape handling whenever `window.focused(cx)` is one of the dialog’s buttons, letting the focused button’s own click be the activation.

## Why the Fix Works

The dialog-level Enter fallback exists for the “nothing specific is focused” case; scoping it to exact dialog-box focus removes the overlap while preserving Enter-anywhere → `Primary` when no button is focused.

## Fix Ordering / Related

**MUST land in the same pass as bug-0019**, whose fix un-masks this bug.  Coordinate with bug-0017 and bug-0018, which modify the same `on_key_up` handler.
