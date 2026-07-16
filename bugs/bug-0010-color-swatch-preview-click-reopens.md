# bug-0010: Clicking the ColorSwatch preview while the picker is open reopens it instead of closing — and silently overwrites `original_value`, breaking a later ESC revert

**Severity:** low
**Fix side:** CODE
**Component:** src/widgets/color_swatch.rs:628-637 (preview `on_click` toggle), 1125-1127 (picker `on_mouse_down_out` → `apply_picker`), 433-438 (`open_picker` resets `original_value`)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

The preview swatch sits outside the popup.  Pressing the mouse on it therefore first fires the popup’s `on_mouse_down_out` handler (lines `1125-1127`), which calls `apply_picker()` and sets `is_picker_open = false`.  On mouse-up, the preview’s `on_click` toggle (lines `628-637`) runs, sees `is_picker_open == false`, and calls `open_picker` — which re-stamps `original_value = self.value` (lines `433-438`).

Net effect:

1. The `close_picker` branch of the toggle is unreachable in practice — the picker cannot be closed by clicking the swatch.
2. After a mid-pick swatch click, ESC “reverts” to the mid-pick color rather than the true original.  TODO.md and docs/COLOR_PICKER.md promise that the “Old” swatch and ESC restore the color from when the picker opened; this gesture silently breaks that contract.

The mechanism is fully traced in source (mouse-down-out fires on press, `on_click` on release), but the mousedown-before-click event ordering is a GPUI runtime property — an end-to-end GPUI test should confirm it before the fix lands.

## Reproduction (test-ready)

Needs GPUI test context with real mouse-event dispatch (the whole bug is an event-ordering interaction):

1. Open the picker on `#FF0000`.
2. Drag the hue slider to green.
3. Click the preview swatch once.
4. Press ESC.
5. Expected: the click in step 3 closes the picker (or at minimum leaves `original_value == "#FF0000"` so ESC restores red).
6. Actual (per static trace): the picker is still open, `original_value` is now the green value, and ESC keeps green.

## Suggested Fix

Suppress the reopen: add a `picker_closed_by_outside_click: bool` field, set it in the popup’s `on_mouse_down_out` handler when it closes the picker, and have the preview’s `on_click` consume it (early-return and clear) instead of toggling.

## Why the Fix Works

The two handlers stop double-processing the same gesture: the mouse-down closes, the click is recognized as the tail of that same gesture and does nothing.  The toggle behaves as intended again, and `original_value` is only stamped on a genuine open — keeping the ESC/“Old”-swatch revert contract intact.

## Fix Ordering / Related

Related to bug-0011 (the ESC/“Old”-swatch revert paths themselves fail to emit) — both defend the same revert contract but are independently fixable.  Confirm the mousedown-before-click ordering dynamically before implementing; that is the one link in the trace not verified at runtime.
