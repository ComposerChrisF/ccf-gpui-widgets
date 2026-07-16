# bug-0027: `Theme::from_palette` text hierarchy collapses to pure white in dark mode and runs inverted in light mode

**Severity:** high
**Fix side:** CODE
**Component:** src/theme.rs:485 (`opposite` pole), 521-528 (text scale)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

The derived text scale mixes `palette.text` toward `opposite`:

```rust
// src/theme.rs:485
let opposite = if dark_mode { 0xFFFFFF } else { 0x000000 };

// src/theme.rs:521-528
let text_primary = palette.text;
let text_label = mix(palette.text, opposite, 0.07);
// … through …
let text_icon = mix(palette.text, opposite, 0.60);
```

Muting text means fading it _toward the background_, and this pole is exactly backwards — white in dark mode, black in light mode.  Verified numerically with the exact `mix` implementation:

- **Dark mode — hierarchy collapses.**  `Palette::dark().text = 0xffffff`, so `mix(0xffffff, 0xFFFFFF, r) = 0xffffff` for every ratio.  `text_label`, `text_muted`, `text_placeholder`, `text_dimmed`, and `text_icon` are all identical pure white.  The hand-made `Theme::dark()` uses `0xeeeeee`/`0xaaaaaa`/`0x999999`/`0x888888`/`0x666666`.  Placeholder text becomes indistinguishable from entered text.
- **Light mode — hierarchy inverts.**  `mix(0x1a1a1a, 0x000000, 0.60) = 0x0a0a0a` — “muted” text comes out _darker_ (higher contrast) than primary, while the hand preset goes lighter (`0x777777`, `0xaaaaaa`).

## Reproduction (test-ready)

Plain unit test, no GPUI machinery:

```rust
let t = Theme::from_palette(Palette::dark());
assert_ne!(t.text_muted, t.text_primary); // FAILS — both are 0xffffff

let t = Theme::from_palette(Palette::light());
// text_muted (0x0a0a0a) is darker than text_primary (0x1a1a1a) — inverted
// versus the hand preset direction (which goes lighter: 0x777777, 0xaaaaaa)
```

## Suggested Fix

Mix the text scale toward the background: in lines `521-528`, replace `opposite` with `palette.bg` (e.g. `mix(palette.text, palette.bg, 0.33)` for `text_muted`).  `opposite` is used nowhere else and can be deleted (line `485`).

## Why the Fix Works

`mix(0xffffff, 0x1e1e1e, 0.33) ≈ 0xb5b5b5` and `mix(0x1a1a1a, 0xf5f5f5, 0.33) ≈ 0x626262` — the derived tiers land close to the hand-tuned presets in both modes, restoring a real hierarchy in dark mode and the correct direction in light mode.

## Fix Ordering / Related

Independent.  One caution: bug-0031’s _optional_ (not default) linearization of `luminance()` would change `from_palette`’s dark-mode detection (theme.rs:482) — if that option is ever taken, re-verify this function’s output in both modes.
