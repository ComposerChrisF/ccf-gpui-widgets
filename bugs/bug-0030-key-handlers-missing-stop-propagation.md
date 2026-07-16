# bug-0030: Handled keys are not stopped from propagating in five widgets

**Severity:** low
**Fix side:** CODE
**Component:** `src/widgets/radio_group.rs:302-335`, `src/widgets/checkbox_group.rs:163-195`, `src/widgets/checkbox.rs:164-176`, `src/widgets/toggle_switch.rs:231-243`, `src/widgets/collapsible.rs:240-254`—contrast `src/widgets/segmented_control.rs:382, 391, 397`
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

`SegmentedControl`’s key handler calls `cx.stop_propagation()` after consuming each handled key—`left` (`segmented_control.rs:382`), `right` (`segmented_control.rs:391`), and `space`/`enter` (`segmented_control.rs:397`)—added deliberately in commit `9abc743`.  The other five widgets consume `up`/`down`/`space`/`enter` in their `on_key_down` handlers without stopping propagation, so a keystroke the focused widget has already handled continues to bubble to ancestor `on_key_down` listeners.  Any consumer container that also reacts to arrows, space, or enter (scroll regions, form-level enter handling) will double-handle an event that a focused `RadioGroup`, `Checkbox`, etc. already consumed.

The affected handlers:

- `radio_group.rs:302-335`—match arms `"up"`, `"down"`, `"space"`.
- `checkbox_group.rs:163-195`—match arms `"up"`, `"down"`, `"space"`.
- `checkbox.rs:164-176`—the `matches!(…, "space" | "enter")` branch calling `toggle`.
- `toggle_switch.rs:231-243`—the `matches!(…, "space" | "enter")` branch calling `toggle`.
- `collapsible.rs:240-254`—match arms `"down"`, `"up"`, `"space" | "enter"`.

## Reproduction (test-ready)

The mechanism is fully traced in source; a dynamic end-to-end GPUI test (a real key event dispatched through nested `on_key_down` listeners) is recommended to pin the behavior, since propagation is a runtime property of gpui’s dispatch.

1. Nest a `RadioGroup` inside a parent `div` with an `.on_key_down` listener that records the key (needs GPUI test context).
2. Focus the radio group; press the Down arrow.
3. Actual: the selection moves **and** the parent listener records the key.
4. Expected (and `SegmentedControl`’s actual behavior under the same setup): the parent never sees the consumed key.

## Suggested Fix

Add `cx.stop_propagation()` in each arm that actually handles the key, exactly as `segmented_control.rs:382/391/397` does.  Leave the `_ => {}` arms and all unhandled keys propagating—including Tab, which is handled by `handle_tab_navigation` before the match and must keep its existing behavior.

## Why the Fix Works

It is the same one-line pattern the project already adopted for `SegmentedControl` for this precise reason; scoping it to the handled arms keeps Tab and unrelated keys flowing to ancestors.

## Fix Ordering / Related

None—independent.
