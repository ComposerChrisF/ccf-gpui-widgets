# bug-0032: NumberStepper click-away commit steals focus back from the widget the user clicked

**Severity:** medium
**Fix side:** CODE
**Component:** src/widgets/number_stepper.rs:153-155, 459-464, 556-559 (Blur source: src/widgets/text_input.rs:820-827)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

The embedded TextInput’s `Blur` event (emitted via `cx.on_focus_out`, text_input.rs:820-827) routes to `commit_edit`:

```rust
// src/widgets/number_stepper.rs:153-155
TextInputEvent::Blur if this.editing => {
    this.commit_edit(cx);
}
```

`commit_edit` unconditionally sets `pending_refocus = true` (lines 459-464), and the next render then runs:

```rust
// src/widgets/number_stepper.rs:556-559
if self.pending_refocus {
    self.pending_refocus = false;
    self.focus_handle.focus(window);
}
```

For Enter, Escape, and Tab this “stay on the stepper” refocus is intended.  But `Blur` fires precisely when the user has moved focus _elsewhere_ (clicked another input).  The sequence: click widget B → B focused → focus-out fires on the edit input → `commit_edit` → `pending_refocus` set plus notify → the stepper re-renders and calls `focus(window)`, yanking focus off widget B.  TODO.md claims `click-away to commit` — the commit does happen, but the click-away target loses its focus.

The mechanism is fully traced in source; an end-to-end GPUI test should confirm the final focus ordering.

## Reproduction (test-ready)

Needs GPUI test context (window focus machinery and render cycle):

1. Build a view containing two widgets: a `NumberStepper` and a `TextInput`.
2. Double-click the stepper’s value display to enter edit mode.
3. Click the `TextInput` and start typing.
4. Expected: the stepper’s value commits, and the `TextInput` keeps focus.
5. Actual: the value commits, and focus returns to the stepper on the next frame.

## Suggested Fix

Split the `Blur` path from `commit_edit`.  In the subscription at src/widgets/number_stepper.rs:153-155, handle `TextInputEvent::Blur` with:

```rust
this.apply_edit_value(cx);
this.editing = false;
cx.notify();
```

— i.e., commit without setting `pending_refocus`.  Keep `pending_refocus` for the Enter/Escape/Tab paths only.

## Why the Fix Works

The refocus is then performed only when the user’s action semantically means “stay here” (Enter, Escape, Tab), while a focus transfer initiated by the user is respected.

## Fix Ordering / Related

Coordinate with bug-0033 — its fix relies on the `TextInputEvent::Blur if this.editing` guard whose path this bug splits, and both edit the same commit/blur interplay; fix in the same pass.  bug-0035 touches the same `apply_edit_value` routine.
