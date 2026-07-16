# bug-0035: NumberStepper edit mode round-trips through display precision — an untouched edit silently changes the stored value

**Severity:** medium
**Fix side:** CODE
**Component:** src/widgets/number_stepper.rs:441-443 (seed) and 467-477 (commit)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

`enter_edit_mode` seeds the embedded TextInput with the _display-rounded_ string:

```rust
// src/widgets/number_stepper.rs:441-443
let formatted = self.format_value();
self.edit_input.update(cx, |input, cx| {
    input.set_value(&formatted, cx);
});
```

and `apply_edit_value` (lines 467-477) parses whatever is in the box and commits it whenever it differs from the stored value by more than epsilon.  With `display_precision(2)` and a stored value of `1.23456`, merely double-clicking and then clicking away (Blur commits via lines 153-155) or pressing Enter replaces `1.23456` with `1.23` and emits `Change(1.23)` — though the user typed nothing.

This contradicts the field’s own documentation (lines 233-234): `The displayed value is rounded to this many decimal places.  This is independent of the actual stored value.`

## Reproduction (test-ready)

Needs GPUI test context (edit mode drives the embedded TextInput entity); `enter_edit_mode` plus `commit_edit` can be called directly in a crate-internal test without simulating the mouse.

1. `NumberStepper::new(cx).with_value(0.0).display_precision(2)`, then `set_value(1.23456, cx)` (or reach `1.23456` via drag).
2. Enter edit mode (double-click the value display), then press Enter — touching nothing in between.
3. Expected: value still `1.23456`, no event.
4. Actual: value is `1.23`, and `Change(1.23)` is emitted.

## Suggested Fix

Either:

- Seed the edit box with the full-precision value — `format_display_value(self.value, None)` — in `enter_edit_mode` (src/widgets/number_stepper.rs:441); or
- Track whether the TextInput content was actually modified during the edit and skip the commit in `apply_edit_value` when untouched.

## Why the Fix Works

The round-trip then parses back to exactly the stored value (or the commit is skipped entirely), so an untouched edit is a no-op and display rounding stays display-only, as documented.

## Fix Ordering / Related

Independent, but shares `apply_edit_value` and the commit paths with bug-0032 and bug-0033 — coordinate if fixed in the same pass.
