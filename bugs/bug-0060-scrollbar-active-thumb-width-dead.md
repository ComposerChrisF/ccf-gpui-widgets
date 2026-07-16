# bug-0060: Scrollbar hover/drag “active” thumb width is computed but never applied—`_thumb_width` is discarded

**Severity:** low
**Fix side:** CODE
**Component:** `src/widgets/scrollbar.rs:420-437` (style selection), `src/widgets/scrollbar.rs:460-478` (`thumb_fill_bounds`), `src/widgets/scrollbar.rs:25` (`THUMB_ACTIVE_WIDTH`)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

The style tuple selects `THUMB_ACTIVE_WIDTH` (8 px) when the thumb is hovered or dragged, but the binding is `_thumb_width`—underscored because it is never used (`src/widgets/scrollbar.rs:420-437`):

```rust
let (thumb_bg, track_bg, _thumb_width, inset, radius) =
    if is_dragged || is_hovered_on_thumb {
        (
            self.get_hover_thumb_color(cx),
            self.get_track_color(cx),
            THUMB_ACTIVE_WIDTH,
            THUMB_ACTIVE_INSET,
            THUMB_ACTIVE_RADIUS,
        )
    } else {
        // ... THUMB_WIDTH, THUMB_INSET, THUMB_RADIUS
```

`thumb_fill_bounds` hardcodes `THUMB_WIDTH` (6 px) in all three branches (`src/widgets/scrollbar.rs:464`, `470`, `476`).  So the constant `THUMB_ACTIVE_WIDTH = px(8.)` (line 25) is dead: hovering/dragging changes only color, inset, and radius—the intended thicker thumb never renders.  A side effect: with `THUMB_ACTIVE_RADIUS = px(4.)` applied to a 6 px-wide fill, the active thumb’s corner radii exceed half its width.

## Reproduction (test-ready)

Visual; needs GPUI render context.

1. Render a scrollable with overflowing content so the scrollbar shows.
2. Hover the thumb.

Expected (per the constants): the fill widens to 8 px.
Actual: the fill stays 6 px wide; only color, inset, and radius change.

## Suggested Fix

Rename `_thumb_width` to `thumb_width` and use it in the three `thumb_fill_bounds` branches (`src/widgets/scrollbar.rs:464`, `470`, `476`) in place of `THUMB_WIDTH`.

## Why the Fix Works

The fill rectangle then takes the state-selected width, matching the inset/radius pairing already chosen for the active state (`8 = 12 - 2*2`, exactly as `6 = 12 - 2*3`).

## Fix Ordering / Related

None—independent.  Same file as bug-0061 and bug-0062; coordinate edits to avoid conflicts.
