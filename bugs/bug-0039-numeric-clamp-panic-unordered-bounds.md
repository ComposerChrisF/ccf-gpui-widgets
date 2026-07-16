# bug-0039: Reachable `f64::clamp` panics — unordered min/max bounds panic Slider, ProgressBar, and NumberStepper

**Severity:** high
**Fix side:** CODE
**Component:** src/widgets/slider.rs:129-133 and 136-141; src/widgets/progress_bar.rs:85-91 and 93-100; src/widgets/number_stepper.rs:412
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

`std::f64::clamp` panics when `min > max` (or either is NaN).  The Slider and ProgressBar builders clamp eagerly against the _current_ other bound:

```rust
// src/widgets/slider.rs:129-133 (progress_bar.rs:85-91 is the same shape)
pub fn min(mut self, min: f64) -> Self {
    self.min = min;
    self.value = self.value.clamp(self.min, self.max);
    self
}
```

Slider defaults to `0..100` and ProgressBar to `0..1`, so the perfectly reasonable builder chains `Slider::new(cx).min(200.0).max(400.0)` and `ProgressBar::new().min(5.0).max(10.0)` panic _inside_ `min()` before `max()` ever runs — `0.0.clamp(200.0, 100.0)` and `0.0.clamp(5.0, 1.0)` respectively.

NumberStepper’s builders store `Option`s without clamping, so `.min(10.0).max(5.0)` constructs fine — instead `normalize_value`’s `snapped.clamp(min, max)` (number_stepper.rs:412) panics later, on the first button click, keypress, drag, or edit-commit.  That is a panic reachable from user input.  NaN passed to `min()`/`max()` panics the same way on any of the three widgets.

Panic verified, message: `min > max, or either was NaN. min = 5.0, max = 1.0`.

## Reproduction (test-ready)

1. `ProgressBar::new().min(5.0).max(10.0)` — panics inside `min()`.  Plain unit test; no GPUI context needed.
2. `cx.new(|cx| Slider::new(cx).min(200.0).max(400.0))` — panics inside `min()`.  Needs GPUI test context for `cx`.
3. `NumberStepper::new(cx).min(10.0).max(5.0)` — constructs fine; then `set_value(7.0, cx)` (or the first `+` click / Up keypress / drag / edit-commit) panics in `normalize_value`.  Needs GPUI test context.

Expected in all three cases: a defined, non-panicking result (a degraded-but-defined value, or the bound treated as unset).  Actual: library panic.

## Suggested Fix

Never call `f64::clamp` with potentially unordered bounds.  Either:

- Clamp via `value.max(lo).min(hi)` after normalizing `lo = min.min(max)`, `hi = min.max(max)`; or
- Preferably, defer clamping out of the individual builders entirely — clamp lazily in `normalize_value` (NumberStepper), `percentage` (ProgressBar), and the Slider equivalent, using the ordered pair — and treat NaN bounds as unset.

## Why the Fix Works

`value.max(lo).min(hi)` with ordered bounds never asserts, so a transiently inverted range (mid-builder-chain) or a permanently inverted one degrades to a defined value instead of a library panic.  Deferring the clamp additionally removes the builder-order trap altogether, since no constraint is applied until all constraints are known.

## Fix Ordering / Related

Same normalize/clamp code paths as bug-0037, bug-0038, and bug-0070 — fix all four as one coordinated pass.  The lazy-clamp rework described here largely resolves bug-0070 as a side effect.
