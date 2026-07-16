# bug-0041: PasswordInput never emits Blur when focus leaves via the toggle button

**Severity:** medium
**Fix side:** CODE
**Component:** `src/widgets/password_input.rs:656-667`
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

`PasswordInput` has two focus nodes — the text field (`input_focus_handle`) and the visibility toggle (`toggle_focus_handle`) — but only one focus-out subscription, on the input handle:

```rust
// src/widgets/password_input.rs:656-667
if !self.focus_out_subscribed {
    self.focus_out_subscribed = true;
    let input_fh = self.input_focus_handle.clone();
    cx.on_focus_out(&input_fh, window, |this: &mut Self, _event, window, cx| {
        // Only blur if neither input nor toggle has focus
        if !this.input_focus_handle.is_focused(window)
            && !this.toggle_focus_handle.is_focused(window)
        {
            this.on_blur(cx);
        }
    })
    .detach();
}
```

gpui’s `on_focus_out` fires when the given focus handle **or one of its descendants** loses focus (vendored gpui 0.2.2 `context.rs:625`).  The toggle handle is a sibling focus node, not a descendant of the input.  So the focus path input → toggle fires the callback but is correctly suppressed by the guard (the toggle is focused); when focus then leaves the toggle for anywhere else, the input handle is not the one losing focus, so nothing fires — `PasswordInputEvent::Blur` is never emitted for that exit path.

## Reproduction (test-ready)

Needs GPUI test context (two focusable widgets, Tab dispatch):

1. Subscribe to `PasswordInputEvent`; place a `PasswordInput` before another focusable widget.
2. Focus the password field.
3. Press Tab: focus moves to the toggle button.  No `Blur` — correct, focus is still inside the widget.
4. Press Tab again: focus moves to the next widget.
5. Expected: exactly one `Blur`.  Actual: none — and none ever arrives until the input field itself is refocused and blurred again.

## Suggested Fix

Add a second `cx.on_focus_out` subscription on the toggle handle, with the identical neither-focused guard, alongside the existing one at `password_input.rs:656-667`:

```rust
let toggle_fh = self.toggle_focus_handle.clone();
cx.on_focus_out(&toggle_fh, window, |this: &mut Self, _event, window, cx| {
    if !this.input_focus_handle.is_focused(window)
        && !this.toggle_focus_handle.is_focused(window)
    {
        this.on_blur(cx);
    }
})
.detach();
```

## Why the Fix Works

With both handles subscribed, any exit from the widget pair triggers at least one callback, and the shared neither-focused guard ensures `Blur` fires exactly once, exactly when neither part holds focus.

## Fix Ordering / Related

bug-0042 (PasswordInput event-surface decision) touches the same `on_blur` machinery — Escape currently emits `Blur` too.  This fix is required under every option there; land it with or after bug-0042’s resolution so the `Blur` semantics settle once.
