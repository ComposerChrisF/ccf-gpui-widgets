# bug-0036: NumberStepper keyboard step ignores the configurable multipliers — Shift hardcoded to 10.0, Alt fine step missing entirely

**Severity:** medium
**Fix side:** CODE
**Component:** src/widgets/number_stepper.rs:783-793 (vs. button handlers at 717-724 and 754-761)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

Button clicks use the configured `stepper.step_large_multiplier` / `stepper.step_small_multiplier` (lines 717-724 and 754-761), per TODO.md (2026-02-02): `Configurable step multipliers via .step_multipliers(small, large)`, `Default multipliers: Alt/Option = 0.1x, Shift = 10x, Normal = 1x`.  The key handler instead does:

```rust
// src/widgets/number_stepper.rs:783-787
let multiplier = if event.keystroke.modifiers.shift {
    10.0
} else {
    1.0
};
```

The configured large multiplier is ignored, and `modifiers.alt` is not consulted at all, so Alt+Up/Down has no fine-step counterpart.  A stepper configured with `.step_multipliers(0.01, 100.0)` steps by a factor of 100 on Shift+click but a factor of 10 on Shift+Up, and by a factor of 0.01 on Alt+click but a factor of 1 on Alt+Up.

## Reproduction (test-ready)

Needs GPUI test context (keystroke dispatch to the `on_key_down` handler):

1. `NumberStepper::new(cx).step(1.0).step_multipliers(0.5, 4.0)`.
2. Shift+click `+` → value increases by 4 (correct).
3. Shift+Up → value increases by 10.  Expected: 4.
4. Alt+Up → value increases by 1.  Expected: 0.5.

## Suggested Fix

Replicate the button logic in the key handler (src/widgets/number_stepper.rs:783-787):

```rust
let multiplier = if event.keystroke.modifiers.shift {
    stepper.step_large_multiplier
} else if event.keystroke.modifiers.alt {
    stepper.step_small_multiplier
} else {
    1.0
};
```

## Why the Fix Works

All three mutation surfaces (buttons, keyboard, and their modifiers) then agree on one configuration, which is what `.step_multipliers()` promises.

## Fix Ordering / Related

None — independent.  (Slider’s keyboard handler shares the same hardcoded-10.0 pattern but has no configurable multipliers; the slider-side issue is doc drift, filed as bug-0069.)
