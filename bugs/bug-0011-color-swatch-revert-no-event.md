# bug-0011: ColorSwatch cancel/ESC and “Old”-swatch revert never emit `Change` — subscribers left stale, permanently so with an alpha-bearing original

**Severity:** medium
**Fix side:** CODE
**Component:** src/widgets/color_swatch.rs:447-452 (`cancel_picker`), 1054-1056 (“Old”-swatch click), 327-345 (`set_value_internal`)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

During picking, every slider move emits a live `Change` event (`sync_value`, line `428`).  But both revert paths — ESC via `cancel_picker` (lines `447-452`) and clicking the “Old” swatch (lines `1054-1056`) — restore the original value through `set_value_internal`, which emits nothing itself.

Today, subscribers get the reverted value only by _accident_, through the programmatic echo of bug-0008 (`set_value_internal` → `hex_input.set_value` → `TextInputEvent::Change` → `handle_input_change` → emit).  And that accidental rescue fails outright when `original_value` is 8-digit (`"#RRGGBBAA"`): the echoed `handle_input_change` cannot parse it (bug-0007), so no event is emitted at all _and_ `input_is_valid` is set `false`.

Net result with an alpha-bearing original: the widget displays the original color with a spurious error border, while every subscriber still holds the last mid-drag color.  Note also that fixing bug-0008’s echo alone makes the 6-digit case stale too — the revert paths need their own emission.

## Reproduction (test-ready)

Needs GPUI test context (picker interaction plus subscription):

1. Build `ColorSwatch::new(cx).with_alpha(true)` with value `#FF000080`; subscribe to its events.
2. Open the picker; drag the hue slider — the subscriber receives live events, e.g. `Change("#00FF8080")`.
3. Press ESC.
4. Expected: a final `Change("#FF000080")`, and `is_input_valid() == true`.
5. Actual: no further event — the subscriber’s last known value remains `#00FF8080` — and `is_input_valid() == false`.

Same sequence with a click on the “Old” swatch in place of ESC.

## Suggested Fix

In `cancel_picker` and the “Old”-swatch click handler, call the public `set_value` (carrying the changed-check from bug-0013) instead of `set_value_internal`.

## Why the Fix Works

Subscribers that consumed live preview events are guaranteed a terminal event restoring the true value, and the changed-check skips the emission when nothing was changed before cancel — so the revert emits exactly when the value actually moves back.

## Fix Ordering / Related

Depends on bug-0013 (`set_value_internal` returning whether the value changed; `set_value` emitting only then).  bug-0008 (echo reentrancy guard) must land before or with this fix — once the echo is gone, the accidental rescue disappears for 6-digit values too, so this emission becomes the only one.  bug-0007 explains why the 8-digit case fails even today.
