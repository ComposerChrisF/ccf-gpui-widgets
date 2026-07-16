# bug-0033: NumberStepper `+`/`-` buttons silently discard the typed text while editing

**Severity:** medium
**Fix side:** CODE
**Component:** src/widgets/number_stepper.rs:713-716 and 750-753
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

Both button click handlers begin with:

```rust
// src/widgets/number_stepper.rs:713-716 (decrement) and 750-753 (increment)
if stepper.editing {
    // Set editing to false before anything that could trigger blur
    stepper.editing = false;
}
```

This is deliberately placed so that the subsequent `focus_handle.focus(window)` — which blurs the embedded TextInput — reaches the `TextInputEvent::Blur if this.editing` guard (line 153) with `editing == false`.  The effect: the click-away commit is _suppressed_, the typed text is thrown away, and the increment/decrement is applied to the stale pre-edit value.

Per the TODO.md spec (`Double-click to edit value as text, Enter/Tab/click-away to commit`), clicking away should commit — and the `+`/`-` buttons are the most adjacent “away” there is.  A user who double-clicks, types `50`, and clicks `+` expects `51` (or at minimum expects the `50` to survive); they get pre-edit-value plus one.

## Reproduction (test-ready)

Needs GPUI test context (click and blur dispatch):

1. Create a `NumberStepper` with value `10`.
2. Double-click the value display to enter edit mode; type `50`.
3. Click the `+` button.
4. Expected: value is `51` (commit the edit, then step).
5. Actual: value is `11`, and the `50` is gone with no event ever referencing it.

## Suggested Fix

In both handlers (src/widgets/number_stepper.rs:713-716 and 750-753), replace `stepper.editing = false;` with:

```rust
stepper.apply_edit_value(cx);
stepper.editing = false;
```

so the typed value is committed before the step is applied.  The existing guard ordering still prevents the subsequent `Blur` from double-committing.

## Why the Fix Works

It preserves the anti-double-commit ordering the comment is protecting while honoring the click-away-commits contract; the step then operates on the value the user can actually see in the edit box.

## Fix Ordering / Related

Coordinate with bug-0032 — it splits the `Blur` path that this fix’s guard interplay relies on; fix in the same pass.  bug-0035 touches the same `apply_edit_value` routine.
