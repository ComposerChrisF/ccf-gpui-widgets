# bug-0034: NumberStepper drag applies modifier sensitivity to the entire accumulated delta — value jumps when Shift/Alt changes mid-drag

**Severity:** medium
**Fix side:** CODE
**Component:** src/widgets/number_stepper.rs:514-533
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

`update_drag` picks `value_per_pixel` from the _current_ modifiers and multiplies it by the _total_ delta from drag start:

```rust
// src/widgets/number_stepper.rs:531-533 (sensitivity selection at 514-529)
let delta_pixels = (x - self.drag_start_x) as f64;
let new_value =
    self.normalize_value(self.drag_start_value + delta_pixels * value_per_pixel);
```

Sensitivity changes are therefore retroactive.  Verified: drag 100 px unmodified (value-per-pixel 0.5) → value goes up by 50; press Shift and move 1 px more → `101 * 2.5 = 252.5` — an instant jump of roughly 200.  Releasing Shift snaps the value back; pressing Alt collapses the entire accumulated change to one tenth.  This defeats the documented purpose of Alt (module header, line 13: `Hold **Alt/Option** for slow/fine adjustment (0.1x speed)`) — engaging fine mode after coarse positioning throws away the coarse position.

## Reproduction (test-ready)

The arithmetic is checkable by driving `start_drag`/`update_drag` directly (crate-internal — they are private methods); full mouse simulation would need GPUI drag-event machinery.

1. `NumberStepper::new(cx).min(0.0).max(1000.0).manual_drag_sensitivity().drag_sensitivities(0.5, 2.5, 0.05)` — auto-scale off for determinism (needs GPUI test context for `cx`).
2. `start_drag(0.0)` (mouse-down at x = 0).
3. `update_drag(100.0, &modifiers_none, cx)` → value 50.0.
4. `update_drag(101.0, &modifiers_shift, cx)`.
5. Expected: approximately `50 + 2.5 = 52.5` (Shift sensitivity applied to the 1 px moved while Shift is held).
6. Actual: `252.5` (`101 * 2.5`).

## Suggested Fix

Re-anchor on modifier change in `update_drag` (src/widgets/number_stepper.rs:514-533): store the last-seen modifiers in the drag state; when they differ from the event’s modifiers, set `drag_start_x = x` and `drag_start_value = self.value` before computing the delta.

## Why the Fix Works

Each sensitivity then applies only to movement made while it is active, so the value is continuous across modifier transitions — the standard scrubbing behavior.

## Fix Ordering / Related

None — independent.  (Note: bug-0038’s NaN-safe change-guard also edits `update_drag`, so expect textual overlap if fixed in parallel.)
