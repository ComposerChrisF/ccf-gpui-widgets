# bug-0007: ColorSwatch flags its own alpha values as invalid — error border on every picker interaction when alpha is not 255

**Severity:** medium
**Fix side:** CODE
**Component:** src/widgets/color_swatch.rs:350-360 (`handle_input_change`), 417-421 (`sync_value`), 645-649 (error border); src/utils/color.rs:565-567 and 45 (`parse_color` is RGB-only)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

This is the same programmatic-echo mechanism as bug-0008, surfacing as a different symptom.  With `with_alpha(true)` and `current_alpha != 255`, every picker interaction ends in `sync_value` (lines `417-421`), which writes `"#RRGGBBAA"` into the hex `TextInput`.  `TextInput::set_value` unconditionally emits `TextInputEvent::Change` (bug-0077), and ColorSwatch’s subscription delivers that echo to `handle_input_change` (lines `350-360`) — which parses with `parse_color`.

`parse_color` has no 4/8-digit path: `Rgb::from_hex` accepts byte lengths 3 and 6 only (color.rs:45), then falls through to named-color lookup (color.rs:565-567).  The 8-digit string therefore returns `None`, `input_is_valid` is set `false`, and the red error border (lines `645-649`) renders — for a string the widget itself produced one event-cycle earlier.

The same parser hole affects genuine typing: entering `#FF000080` by hand shows the error border on every keystroke, _including the completed 8-digit string_, until Enter or Blur reaches `handle_input_commit` — which correctly uses `parse_color_alpha`.

Additionally, when the echoed value _does_ parse (alpha 255), line `357` emits a second `ColorSwatchEvent::Change`, so subscribers receive two events per slider tick.

## Reproduction (test-ready)

The parser mismatch is unit-testable with just a GPUI `Context`:

1. Build `ColorSwatch::new(cx).with_alpha(true)`.
2. Call `handle_input_change("#00000080", cx)`.
3. Expected: `is_input_valid() == true` — `parse_color_alpha("#00000080")` succeeds, and the widget’s docs promise the `#RRGGBBAA` grammar.
4. Actual: `is_input_valid() == false`.

End-to-end (needs GPUI test context with the picker and input subscription live):

1. Build `ColorSwatch::new(cx).with_alpha(true)`; open the picker.
2. Drag the alpha slider to the midpoint.
3. Expected: value becomes `#00000080`, `is_input_valid() == true`, exactly one `Change` per tick.
4. Actual: `is_input_valid() == false` and the error border renders; with alpha at 255 instead, two `Change` events fire per tick.

## Suggested Fix

Two parts:

1. The reentrancy guard from bug-0008 (`syncing_input` flag) — this stops the self-inflicted echo case entirely.
2. In `handle_input_change`, parse with `parse_color_alpha` (mirroring `handle_input_commit`) and keep alpha in the formatted value, so hand-typed 8-digit hex previews live instead of erroring until commit.

## Why the Fix Works

Part 1 removes the echo, so the widget never re-parses its own output.  Part 2 makes the live-preview parser accept the same grammar (`#RGB`, `#RRGGBB`, `#RRGGBBAA`) that the widget’s documentation promises and that the commit parser already accepts.

## Fix Ordering / Related

bug-0008 (echo reentrancy guard) must land before or with this fix — part 1 _is_ that guard.  Part 2 rewrites the same lines as bug-0006’s shared formatter; coordinate so `handle_input_change` is reworked once.  bug-0077 is the enabling defect on the `TextInput` side (unconditional emit from `set_value`).
