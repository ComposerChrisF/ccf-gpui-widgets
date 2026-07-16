# bug-0038: NumberStepper `with_value` skips normalization — out-of-range initial value, and `with_value(f64::NAN)` bricks the widget permanently

**Severity:** medium
**Fix side:** CODE
**Component:** src/widgets/number_stepper.rs:194-198 (vs. `set_value` at 378-385)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

`with_value` stores the raw value while `set_value` normalizes — the builder/setter asymmetry the project conventions warn about:

```rust
// src/widgets/number_stepper.rs:194-198
pub fn with_value(mut self, value: f64) -> Self {
    self.value = value;
    self
}
```

Two consequences:

1. `.with_value(500.0).min(0.0).max(100.0)` renders `500` until the first interaction snaps it to `100` with a surprise `Change(100.0)`.  The initial state violates the widget’s own constraints (contrast Slider, whose builders clamp).
2. `.with_value(f64::NAN)` is permanent: every mutation path guards with `(self.value - new).abs() > f64::EPSILON`, which is `false` when `self.value` is NaN (all NaN comparisons are false).  Buttons, keys, drag, `set_value`, and even committing a valid typed number (`5` parses fine, but `(NaN - 5.0).abs() > f64::EPSILON` is `false`) can never assign a new value.  The display shows `NaN` forever.

## Reproduction (test-ready)

1. `NumberStepper::new(cx).with_value(f64::NAN).min(0.0).max(100.0)` (needs GPUI test context for `cx`).
2. Call `set_value(5.0, cx)` — plain entity test; the interactive paths (click `+`, press Up, double-click and type `5` then Enter) need GPUI window/dispatch machinery and fail the same way.
3. Expected: some finite value, with a `Change` event.
4. Actual: value remains NaN, no events, unrecoverable.

Out-of-range variant:

1. `NumberStepper::new(cx).with_value(500.0).min(0.0).max(100.0)`.
2. `value()` returns `500.0` — outside the widget’s own bounds — until the first interaction, which snaps to `100.0` and emits a surprise `Change(100.0)`.

## Suggested Fix

Two parts:

- Have `with_value` (src/widgets/number_stepper.rs:194-198) reject NaN (e.g., `if value.is_nan() { return self; }`) and normalize lazily at first render — or eagerly if min/max are already set.
- Make the change-guards NaN-safe in `set_value`, `adjust_value`, `update_drag`, and `apply_edit_value`:

```rust
if self.value.is_nan() || (self.value - new_value).abs() > f64::EPSILON {
```

## Why the Fix Works

A NaN can then neither enter the stored value nor — if it somehow does — block the comparison that would replace it, so the widget always has an escape path to a finite value.

## Fix Ordering / Related

Same normalize/clamp code paths as bug-0037, bug-0039, and bug-0070 — fix all four as one coordinated pass.  The NaN-safe guard in `update_drag` textually overlaps bug-0034’s re-anchoring change.
