# bug-0019: ConfirmationDialog steals focus back from its own buttons on every render—Tab navigation inside the dialog is broken

**Severity:** high
**Fix side:** CODE
**Component:** `src/widgets/confirmation_dialog.rs:236-242` (render-time focus grab); also `src/widgets/confirmation_dialog.rs:299-307` (the Tab handler this defeats)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

The `render` method grabs focus whenever the dialog-box handle is not _exactly_ focused (`src/widgets/confirmation_dialog.rs:236-242`):

```rust
// Save the current focus and focus the dialog when it first renders
if !focus_handle.is_focused(window) {
    // Save the currently focused element before we take focus
    if self.previous_focus.is_none() {
        self.previous_focus = window.focused(cx);
    }
    focus_handle.focus(window);
}
```

In gpui 0.2.2, `FocusHandle::is_focused` is an exact match (`window.focus == Some(id)`, gpui `window.rs:237`), not a containment check, and `Window::focus()` calls `self.refresh()` (gpui `window.rs:1386-1394`), which re-renders every view.  The dialog’s buttons (`dialog_primary`, `dialog_secondary`, `dialog_tertiary`) are separate tab stops with their own anonymous `.focusable()` handles.  The sequence:

1. The user presses Tab; the dialog’s `on_key_down` handler calls `window.focus_next()`, moving focus to a button.
2. The focus change triggers a redraw.
3. `render()` runs, sees `is_focused() == false`, and yanks focus back to the dialog box.

Additionally, `dispatch_key_event` draws the window before dispatching (gpui `window.rs:3731-3733`), so by the time any Enter/Space keyup would reach a button, focus is already back on the dialog box.

Result: the dialog’s buttons can never hold keyboard focus; keyboard activation of Secondary/Tertiary is impossible (Enter always fires `Primary` via the dialog-level `on_key_up`, lines 339–342).  The dialog’s own `on_key_down` Tab handler (lines 299–307) is therefore useless.

## Reproduction (test-ready)

Needs GPUI test context (window, focus, key dispatch).

1. Mount a dialog: `ConfirmationDialog::new("Confirm", "Are you sure?", cx).primary_label("Yes").secondary_label("No")`.  The dialog box takes focus on first render.
2. Press Tab (focus moves to a button; the resulting refresh re-runs `render`).
3. Observe on the next frame that `window.focused(cx)` is the dialog-box handle again, not the button.
4. Press and release Enter.

Expected: focus stays on the button after step 2, and Enter activates the focused button (`Secondary` for “No”).
Actual: focus snaps back to the dialog box, and `ConfirmationDialogEvent::Primary` is emitted even though the user tabbed to “No”.

## Suggested Fix

Replace the exact-match guard at `src/widgets/confirmation_dialog.rs:236` with containment plus a one-shot flag:

- Minimum: `if !focus_handle.contains_focused(window, cx) { ... }`.  `contains_focused` (gpui `window.rs:351`) returns true when a descendant (a dialog button) is focused, so re-renders no longer re-focus the dialog box.
- Better: take focus only on the first render—add a `has_taken_focus: bool` field—since `contains_focused` alone would still re-steal focus the consumer moved intentionally elsewhere.

## Why the Fix Works

With containment (or a one-shot grab), a focused dialog button no longer trips the render-time guard, so Tab/Shift-Tab plus Enter/Space on individual buttons work as documented.

## Fix Ordering / Related

**MUST be fixed in the same pass as bug-0016.**  Today bug-0016 (Enter on a focused dialog button double-fires `Secondary` and `Primary`) is masked precisely because buttons can never hold focus; fixing this bug alone exposes it.  Also coordinate with bug-0017 and bug-0018, which change the same `on_key_up` handler.
