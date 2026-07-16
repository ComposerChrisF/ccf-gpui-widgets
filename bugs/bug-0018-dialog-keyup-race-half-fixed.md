# bug-0018: Dialog keyup dismissal only half-fixes the documented race—a keydown-launched dialog is instantly dismissed by the launching key’s keyup

**Severity:** medium
**Fix side:** CODE
**Component:** `src/widgets/confirmation_dialog.rs:308-345`; `TODO.md:179-180`
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

The comment at `src/widgets/confirmation_dialog.rs:308-311` (and `TODO.md:180`) claims the keyup design prevents the keydown-launch race:

```rust
// Dismissal actions respond on keyup to avoid race conditions when
// the dialog is launched by a keydown - if we dismissed on keydown,
// the keyup would fire on the restored-focus element and potentially
// re-launch the dialog
.on_key_up(cx.listener(move |dialog, event: &KeyUpEvent, window, cx| {
```

It prevents only the _re-launch_ direction (dismiss-on-keydown → keyup fires on the restored launcher).  The mirror image is wide open: gpui dispatches action bindings and `on_key_down` on keydown, and `dispatch_key_event` **draws the window before dispatching the next event** (gpui `window.rs:3731`: `if self.invalidator.is_dirty() { self.draw(cx)... }`).  So if an app opens the dialog from a keydown path (an action binding or `on_key_down`—e.g. `enter` bound to open a confirm dialog), the dialog is rendered and focused before the user’s finger comes up; the launching key’s **keyup** then dispatches along the dialog’s focus path into `.on_key_up` (line 312).  If that key is `enter` (→ `Primary`, lines 339–342), `escape` (→ `Secondary`/`Primary`, lines 331–337), or any `map_key` mapping (lines 316–327), the dialog is confirmed or dismissed by the same keystroke that opened it.

Dialogs launched by gpui button clicks are safe only by accident: gpui’s keyboard click happens to fire on keyup (gpui `div.rs:2180-2183`).

## Reproduction (test-ready)

Needs GPUI test context.  The dispatch mechanics are fully traced in source; an end-to-end GPUI test should confirm the event ordering.

1. Build a root view with an `on_key_down` handler that mounts a Default-style `ConfirmationDialog` when `event.keystroke.key == "enter"`; subscribe to `ConfirmationDialogEvent`.
2. Press and release Enter once.
3. Trace: the keydown mounts the dialog; the pre-dispatch draw focuses it; the Enter keyup reaches the dialog’s `.on_key_up`, hits the `"enter" if !is_danger` arm, and emits `Primary`.

Expected: the dialog stays open awaiting a fresh keypress.
Actual: one Enter press opens and instantly confirms the dialog.

## Suggested Fix

Track keydowns seen while the dialog owns focus:

1. Add a `keys_seen_down: HashSet<String>` field to `ConfirmationDialog`.
2. In the existing `on_key_down` handler (`src/widgets/confirmation_dialog.rs:299-307`), record `event.keystroke.key` into the set.
3. In `on_key_up`, only act when `keys_seen_down.remove(&key)` returns `true`.

Keep the keyup timing—it correctly prevents the re-launch direction.

## Why the Fix Works

A keyup can then only dismiss if the _matching keydown_ occurred after the dialog took focus, which excludes the launching keystroke in both directions while leaving normal operation untouched.

## Fix Ordering / Related

Coordinate with bug-0016 and bug-0017 (same `on_key_up` handler) and with the bug-0019 focus rework (same render/dispatch path).  No hard ordering beyond editing the same code.
