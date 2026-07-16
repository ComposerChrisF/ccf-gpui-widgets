# bug-0012: ColorSwatch RGB sliders use handle width 6.0 for input math but 4.0 for display — clicking where the handle is drawn yields a different value

**Severity:** low
**Fix side:** CODE
**Component:** src/widgets/color_swatch.rs:1188 (`handle_visual_width = 6.0`), 1250-1252 (display uses 4.0), 1271 and 1297 (input uses 6.0); spec: docs/COLOR_PICKER.md:137-169 (correct — do not change)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

The R/G/B component sliders position their handle with one constant and convert clicks with another:

```rust
// src/widgets/color_swatch.rs:1188
let handle_visual_width = 6.0f32; // 4px content + 2px border

// display (1250-1252): handle_width = 4.0 → x = (value/max) * (w - 4)
// input   (1271, 1297): usable_width = slider_w - handle_visual_width
//                       value = (x - origin - 3).clamp(0, w - 6) / (w - 6) * max
```

docs/COLOR_PICKER.md:137-169 is explicit on both points: the handle position formula (display) _must match_ the value calculation (input), and the border does **not** count — it records the lesson that GPUI’s `.border_1()` does not affect layout width (“NOT 4 + 2 for border”).  So the input side’s `6.0` is the wrong side; the display’s `4.0` matches the spec and matches the hue and alpha sliders, which use `4.0` consistently on both sides (display `519`/`867`, input `546`/`954`).

With the two formulas different, clicking exactly where the handle is drawn computes a different value than the handle represents — about 2 px, or roughly 2.6 units of 255, of drift at the right end of the track.

## Reproduction (test-ready)

Pure arithmetic — unit-testable without any GPUI machinery:

1. Take slider width `w = 200` and value `128`.
2. Display position: `128/255 * (200 - 4) = 98.4`.
3. Feed `x = origin + 98.4` into the mouse-down conversion: `(98.4 - 3).clamp(0, 194) / 194 * 255 ≈ 125.4` → `125` after `as u8`.
4. Expected: the value you clicked on — `128`, give or take `as u8` rounding.
5. Actual: `125`.

## Suggested Fix

Change `handle_visual_width` (line `1188`) to `4.0` — or better, replace both uses with the same `handle_width = 4.0` constant the display closure uses, shared the way the hue and alpha sliders already share theirs.

## Why the Fix Works

With identical constants, the position→value and value→position maps are exact inverses (modulo `as u8`), which is precisely what docs/COLOR_PICKER.md:137-169 demands.

## Fix Ordering / Related

None — independent.
