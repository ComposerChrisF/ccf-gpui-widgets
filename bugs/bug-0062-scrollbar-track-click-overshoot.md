# bug-0062: Scrollbar track click uses the wrong scale factor—overshoots by `content / (content - viewport)`

**Severity:** medium
**Fix side:** CODE
**Component:** `src/widgets/scrollbar.rs:590-614` (track-click handler), vs the drag path at `src/widgets/scrollbar.rs:651` and `656`, and the prepaint thumb mapping at `src/widgets/scrollbar.rs:387-388`
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

The track-click handler computes a target offset with `-scroll_area_size * percentage` (`src/widgets/scrollbar.rs:590-614`):

```rust
let percentage = if is_vertical {
    (event.position.y - thumb_size / 2. - bounds.origin.y)
        / (bounds.size.height - thumb_size)
} else {
    (event.position.x - thumb_size / 2. - bounds.origin.x)
        / (bounds.size.width - thumb_size)
}
.min(1.);

if is_vertical {
    scroll_handle.set_offset(point(
        offset.x,
        (-scroll_area_size * percentage)
            .clamp(safe_range.start, safe_range.end),
    ));
```

But the maximum scroll offset is `-(scroll_area_size - container_size)`—which is exactly `safe_range.start` (line 576).  The drag path does it correctly: `(-(scroll_area_size - container_size) * percentage)` (lines 651, 656).  So a track click overshoots by the factor `scroll_area / (scroll_area - container)`, and every click in the last `container / scroll_area` fraction of the track slams to max scroll.

Two secondary inconsistencies in the same handler:

- For a horizontal bar in `Both` mode, the percentage denominator omits `margin_end` (`bounds.size.width - thumb_size`, line 597) while the drag path includes it (`- margin_end`, line 644) and the thumb-position mapping in prepaint reserves it (line 388).
- The denominators use the inset-adjusted thumb length (`state.thumb_size`, set at line 495 from `thumb_end - thumb_start - inset * 2`, line 439) while the thumb-position mapping in prepaint (lines 387–388) uses the un-inset length, so thumb and mouse diverge by up to `2 * inset` (6 px).

## Reproduction (test-ready)

The arithmetic is checkable on paper (worked example below); live confirmation needs a GPUI test.  Vertical, viewport 200 px, content 1000 px:

1. Thumb raw length = `max(200/1000 * 200, 48)` = 48 px; visual thumb = 42 px (`state.thumb_size`, inset-adjusted).
2. Click the track at `y = 100` relative to the track top: `percentage = (100 - 21 - 0) / (200 - 42) = 0.5`.
3. The code sets offset = `-1000 * 0.5 = -500`, then clamps into `[-800, 0]`.

Expected (thumb centered at the click, matching the drag/prepaint mapping): offset ≈ `-800 * 0.5 = -400`.
Actual: `-500`—and any click at `y >= ~148` (`percentage >= 0.8`) jumps straight to `-800` (max).

## Suggested Fix

Use the same formula as the drag path at `src/widgets/scrollbar.rs:604` and `609`:

```rust
(-(scroll_area_size - container_size) * percentage)
```

and use the same denominator as the drag path for horizontal bars (`bounds.size.width - thumb_size - margin_end`).  Ideally compute all three mappings (prepaint thumb position, track click, drag) from one shared helper so they cannot diverge again.

## Why the Fix Works

With `percentage` in `[0, 1]` mapped linearly onto the full legal offset range, the click mapping is precisely the inverse of the thumb-position mapping, so the thumb lands where the user clicked.

## Fix Ordering / Related

None—independent.  Same file as bug-0060 and bug-0061; coordinate edits to avoid conflicts.
