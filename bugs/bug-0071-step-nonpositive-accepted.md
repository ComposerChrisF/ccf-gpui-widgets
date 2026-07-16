# bug-0071: `step(0.0)` and negative steps accepted silently — buttons and keys become no-ops or run backwards

**Severity:** low
**Fix side:** CODE
**Component:** src/widgets/number_stepper.rs:216-219 and 415-423; src/widgets/slider.rs:145-148 and 281-289
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

Neither `step()` builder validates its argument.  `adjust_value` computes:

```rust
// src/widgets/slider.rs:281-289 (number_stepper.rs:415-423 is the same shape)
let step = self.step.unwrap_or(1.0) * multiplier * direction;
```

With `step(0.0)`, the `+`/`-` buttons and arrow keys all silently do nothing — the widget appears broken.  With `step(-5.0)`, `increment` _decreases_ the value and `decrement` increases it — direction inverted.

The snapping code defends itself against exactly these values (`if step > 0.0` at slider.rs:243; `resolution > 0.0` at number_stepper.rs:399), but `adjust_value` does not — so the two halves of the same widget disagree about whether a non-positive step is legal.

## Reproduction (test-ready)

Needs GPUI test context for `cx` and keystroke dispatch; the `adjust_value` arithmetic is crate-internal testable without simulating input:

1. `Slider::new(cx).min(0.0).max(100.0).step(-1.0).with_value(50.0)`.
2. Press Right (or call `increment(1.0, cx)` directly).
3. Expected: `51` — or the configuration rejected up front.
4. Actual: `49`.
5. Variant: with `step(0.0)`, Right/Left and `+`/`-` silently do nothing.

## Suggested Fix

In both `step()` builders (src/widgets/slider.rs:145-148 and src/widgets/number_stepper.rs:216-219) and NumberStepper’s `resolution()` (lines 226-229), ignore non-finite or non-positive values — `debug_assert!` plus keep the field `None` — matching the guard the snapping code already applies.

## Why the Fix Works

`adjust_value` then always sees either `None` (default `1.0`) or a positive step, so direction and magnitude are always what the method names promise, and the builder and the snapping code agree on what a legal step is.

## Fix Ordering / Related

Independent, but the `resolution()` guard touches the same builder/normalize territory as the bug-0037/bug-0038/bug-0039/bug-0070 coordinated pass — folding it into that pass is convenient.
