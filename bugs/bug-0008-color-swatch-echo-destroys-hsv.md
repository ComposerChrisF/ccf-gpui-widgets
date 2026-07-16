# bug-0008: ColorSwatch echo re-derives HSV from the hex string, destroying picker hue and saturation at black/white/gray

**Severity:** high
**Fix side:** CODE
**Component:** src/widgets/color_swatch.rs:186-201 (input subscription), 348-362 (`handle_input_change`), 415-430 (`sync_value`); src/widgets/text_input.rs:392-397 (`set_value` emits unconditionally)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

Every picker interaction ends in `sync_value` (lines `415-430`), which calls `hex_input.set_value(&self.value)`.  `TextInput::set_value` unconditionally emits `TextInputEvent::Change` (text_input.rs:395), and ColorSwatch’s own subscription (lines `186-201`) cannot tell this programmatic echo from user typing — so it runs `handle_input_change`, which re-derives `self.current_hsv = rgb.to_hsv()` from the 8-bit hex round-trip (line `353`).

That destroys exactly the state the code is written to protect.  `update_from_hsv` (lines `405-412`) deliberately stores HSV to preserve hue — its doc comment says it is “preserving the hue value … to keep the hue slider consistent” — and the echo throws that away one event-cycle later.  Verified arithmetic:

```text
Hsv(200, 50, 0)  → "#000000" → to_hsv() = (0, 0, 0)      hue AND saturation lost
Hsv(200, 0, 50)  → "#808080" → to_hsv() = (0, 0, 50.2)   hue lost
```

Consequences in the open picker: dragging the S/V crosshair to the bottom edge (V = 0) snaps the canvas background (`Hsv::new(hue, 100, 100)`, line `732`) to red and jumps the hue handle to 0; dragging back up yields red, not the original hue.  Worse, with a black color the hue slider is fully inoperable: `handle_hue_at_position(h)` → `update_from_hsv(h, 0, 0)` → black → echo resets `h` to 0 on every attempt.

## Reproduction (test-ready)

Unit-testable without a window (a GPUI `Context` for the entity is enough; no event dispatch needed):

1. Build a `ColorSwatch` and call `update_from_hsv(217.0, 78.0, 0.0, cx)`.
2. Simulate the subscriber by calling `handle_input_change("#000000", cx)` — this is exactly what the echo delivers.
3. Expected: `current_hsv.h == 217.0` (hue preserved, per the `update_from_hsv` doc).
4. Actual: `current_hsv.h == 0.0`.

In-app: open the picker on `#3B82F6`; drag the S/V crosshair to the bottom edge.  The canvas snaps to red, the hue handle jumps to 0, and dragging back up yields red instead of blue.

## Suggested Fix

Add a `syncing_input: bool` field.  Set it around every internal `hex_input.update(… set_value …)` — in `sync_value`, `set_value_internal`, `handle_input_commit`, and the `needs_input_sync` first-render block — and early-return from the `TextInputEvent::Change` arm of the subscription while it is set.

## Why the Fix Works

The subscriber then reacts only to genuine user edits.  Programmatic writes no longer round-trip through the 8-bit hex string, so the stored HSV (and the `input_is_valid` flag of bug-0007) survive intact.

## Fix Ordering / Related

Must land before or with bug-0007, bug-0011, and bug-0013 — their fixes assume the echo is gone.  bug-0077 (`TextInput::set_value` emitting unconditionally) is the enabling defect on the `TextInput` side: fixing either helps, fixing both is correct.
