# bug-0037: NumberStepper `resolution()` without `min()` makes the widget permanently inert (NaN poisoning)

**Severity:** high
**Fix side:** CODE
**Component:** src/widgets/number_stepper.rs:393-412
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

`normalize_value` snaps relative to `min`, and `min` defaults to `f64::NEG_INFINITY` when unset:

```rust
// src/widgets/number_stepper.rs:393-412
fn normalize_value(&self, value: f64) -> f64 {
    let min = self.min.unwrap_or(f64::NEG_INFINITY);
    let max = self.max.unwrap_or(f64::INFINITY);

    // First snap to resolution if specified
    let snapped = if let Some(resolution) = self.resolution {
        if resolution > 0.0 {
            // Round to nearest multiple of resolution relative to min
            let offset = value - min;
            let n = (offset / resolution).round();
            min + n * resolution
        } else {
            value
        }
    } else {
        value
    };

    // Then clamp to range
    snapped.clamp(min, max)
}
```

With a resolution set but no min, the snapping arithmetic degenerates: `value - (-inf)` is `+inf`, `n` is `+inf`, and `-inf + inf * resolution` is NaN.  The final `NaN.clamp(min, max)` returns NaN, so `normalize_value` returns NaN for _every_ input.

Every mutation path then compares `(self.value - NaN).abs() > f64::EPSILON`, which is `false` (all NaN comparisons are false), so `set_value` (lines 378-385), `adjust_value` (buttons and keyboard, lines 415-423), `update_drag` (lines 531-538), and `apply_edit_value` (lines 467-477) all silently no-op.  The widget renders normally, but nothing can ever change its value, and no `Change` event is ever emitted.

Verified numerically: `normalize(5.0, None, Some(100.0), Some(0.25))` evaluates to NaN, and the change-check evaluates false.

## Reproduction (test-ready)

The core arithmetic is pure and needs no GPUI machinery:

```rust
let min = f64::NEG_INFINITY;           // self.min.unwrap_or(f64::NEG_INFINITY)
let offset = 5.0 - min;                // +inf
let n = (offset / 0.25).round();       // +inf
let snapped = min + n * 0.25;          // -inf + inf = NaN
assert!(snapped.is_nan());             // this is what normalize_value returns
assert!(!((0.0_f64 - snapped).abs() > f64::EPSILON)); // so the change-guard never passes
```

Widget-level repro:

1. Construct `NumberStepper::new(cx).resolution(0.25)` with no `.min()` (needs GPUI test context for `cx`).
2. Call `set_value(5.0, cx)` — or click `+`, press Up, drag, or double-click-edit `7` and press Enter (the interactive paths need GPUI window/dispatch machinery; `set_value` alone reproduces it).
3. Expected: value becomes `5.0` and `Change` fires.
4. Actual: value stays `0.0` forever; no event is ever emitted.

## Suggested Fix

In `normalize_value` (src/widgets/number_stepper.rs:393-412), snap relative to a _finite_ anchor when `min` is unset — `self.min.unwrap_or(0.0)` — keeping the `NEG_INFINITY`/`INFINITY` defaults only for the final `clamp`.

## Why the Fix Works

A finite anchor makes `offset / resolution` finite, so snapping produces multiples of `min + n * resolution` exactly as the builder documents, and the final clamp still applies whatever bounds actually exist.

## Fix Ordering / Related

Same normalize/clamp code paths as bug-0038, bug-0039, and bug-0070 — fix all four as one coordinated pass (bug-0039’s lazy-clamp rework touches the same `snapped.clamp(min, max)` line at number_stepper.rs:412).
