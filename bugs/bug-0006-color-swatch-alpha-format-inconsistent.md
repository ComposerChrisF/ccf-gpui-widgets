# bug-0006: ColorSwatch value formatting loses alpha — live typing and the README’s own builder ordering each produce a value/render mismatch

**Severity:** medium
**Fix side:** CODE
**Component:** src/widgets/color_swatch.rs:355 and 239-243 (broken sites); 334-338, 371-375, 417-423 (correct sites); README.md:195-199
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

Five sites in `color_swatch.rs` format an `Rgb` (plus optional alpha) into `self.value`.  Three of them (`set_value_internal` at `334-338`, `handle_input_commit` at `371-375`, `sync_value` at `417-423`) carry the alpha-aware branch:

```rust
self.value = if self.with_alpha && rgba.a != 255 {
    format!("#{:02X}{:02X}{:02X}{:02X}", rgba.r, rgba.g, rgba.b, rgba.a)
} else {
    format!("#{:02X}{:02X}{:02X}", rgba.r, rgba.g, rgba.b)
};
```

The other two sites are missing it, producing two distinct symptoms of the same absent shared formatter:

**Symptom A — live typing drops alpha from `value` while `current_alpha` keeps it.**  `handle_input_change` (line `355`) formats unconditionally without alpha:

```rust
// src/widgets/color_swatch.rs:355 — the only live formatting site with no alpha branch
self.value = format!("#{:02X}{:02X}{:02X}", rgb.r, rgb.g, rgb.b);
```

When the user types a valid 6-digit hex or named color while `with_alpha` is on and `current_alpha != 255`, `self.value` becomes the alpha-less string and `Change("#RRGGBB")` is emitted, but `current_alpha` is not touched.  `parse_display_color` (lines `462-466`) still renders with the old alpha, so the swatch shows a translucent color while `value()` and the emitted event claim an opaque one.

**Symptom B — `.with_value("#RRGGBBAA").with_alpha(true)` silently strips alpha from the value string.**  `with_value` (lines `239-243`) formats `self.value` conditionally on `self.with_alpha` — which is still `false` when `with_value` is called first.  That is exactly the ordering the README’s own example shows (README.md:195-199).  An initial `"#FF000080"` produces `value == "#FF0000"` while `current_alpha == 128` is retained, so `value()` and the first `Change` event report an opaque color while the swatch renders translucent.  The alpha silently reappears in the string after the first picker interaction, because `sync_value` re-checks the flag.

## Reproduction (test-ready)

**Symptom B — plain construction test (no window or dispatch needed beyond a GPUI `Context`):**

1. Build `ColorSwatch::new(cx).with_value("#FF000080").with_alpha(true)`.
2. Expected: `value() == "#FF000080"` (or alpha forced to 255 — but not a half state).
3. Actual: `value() == "#FF0000"` and `alpha() == 128`.

**Symptom A — the state transition is directly callable:**

1. Build `ColorSwatch::new(cx).with_alpha(true)` and put it in state `#FF000080` (alpha 128).
2. Type `#00FF00` into the hex field without pressing Enter (or call `handle_input_change("#00FF00", cx)` directly).
3. Expected: value string, emitted event, and rendered preview agree — either `#00FF0080` (alpha kept) or alpha reset to 255 everywhere.
4. Actual: `value() == "#00FF00"`, `Change("#00FF00")` emitted, `alpha() == 128`, and the preview renders 50%-transparent green.

## Suggested Fix

Extract one shared helper, e.g. `format_value(rgb: Rgb, alpha: u8, with_alpha: bool) -> String`, and use it at all five formatting sites (`239-243`, `334-338`, `355`, `371-375`, `417-423`).  Then:

1. In `handle_input_change`, decide explicitly whether live-typing a 6-digit hex resets alpha to 255 (the commit path in `handle_input_commit` already does) and make the rendered preview match the decision.
2. Make `with_alpha` re-format: after setting the flag, recompute `self.value` from `current_rgb`/`current_alpha` with the shared formatter, and set `self.needs_input_sync = true` so the child input picks up the corrected string on first render.

## Why the Fix Works

One formatter means the value string, the rendered color, and the emitted event can no longer disagree about alpha; re-formatting inside `with_alpha` makes builder-method ordering irrelevant, so the README’s example becomes correct as written.

## Fix Ordering / Related

`handle_input_change` is also reworked by bug-0007 (its parser must become alpha-aware) and its spurious invocations come from the echo in bug-0008 — coordinate the three so line `355` is only rewritten once.  Symptom B is otherwise independent.
