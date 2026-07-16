# bug-0061: Scrollbar auto-fade never schedules a repaint, and visibility is computed from stale state—one scroll never shows the bar, and an idle bar never hides

**Severity:** medium
**Fix side:** CODE
**Component:** `src/widgets/scrollbar.rs:513-528` (paint ordering), `src/widgets/scrollbar.rs:162-173` (`is_scrollbar_visible`), `src/widgets/scrollbar.rs:558-574` (dead wheel listener), `src/widgets/scrollbar.rs:156-160` and `666` (dead `last_update`)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

Visibility is purely `elapsed < FADE_OUT_DURATION` evaluated inside `paint` (`is_scrollbar_visible`, `src/widgets/scrollbar.rs:162-173`), and nothing ever schedules a repaint for the fade deadline.  Worse, `paint` computes visibility _before_ recording the scroll that just happened (`src/widgets/scrollbar.rs:513-528`):

```rust
let is_visible = self.state.0.get().is_scrollbar_visible() || self.always_visible;

if self.scroll_handle.offset() != self.state.0.get().last_scroll_offset {
    self.state.0.set(
        self.state
            .0
            .get()
            .with_last_scroll(self.scroll_handle.offset(), Some(Instant::now())),
    );
    // Don't notify here - let GPUI handle scroll updates naturally.
    // Calling cx.notify() in paint causes a re-render loop that resets scroll position.
}

if !is_visible && !self.always_visible {
    return;
}
```

Consequences:

1. **A single scroll never shows the bar.**  From the hidden state, one wheel tick updates the offset, gpui notifies the view (gpui `div.rs:2450`), and one draw happens.  That paint records `last_scroll_time = now` but returns early, because `is_visible` was computed from the pre-scroll state—and since no notify or timer is issued, no further frame is scheduled.  During continuous scrolling the bar appears one event late.
2. **The bar never hides while idle.**  After the user stops scrolling, no repaint is scheduled at `last_scroll_time + FADE_OUT_DURATION` (3 seconds), so the bar stays painted indefinitely (until an unrelated event redraws the window), then vanishes abruptly.  There is no fade at all, despite `src/widgets/scrollable.rs:11` and `TODO.md:243` (“Auto-fade after inactivity”).

Two pieces of adjacent dead code compound the confusion:

- The `ScrollWheelEvent` listener at `src/widgets/scrollbar.rs:562-574` is effectively dead: it registers later than the inner div’s scroll handler and therefore runs _earlier_ in gpui’s reverse-order bubble phase (gpui `window.rs:3704-3712`), so it always sees the pre-scroll offset and its `offset != last_scroll_offset` condition is false.
- `with_last_update`/`last_update` (`src/widgets/scrollbar.rs:156-160`, `666`) is written but never read.

## Reproduction (test-ready)

Needs a GPUI dynamic test (or manual run) to observe frames.

1. Wrap tall content in `scrollable_vertical`, render, and let the bar fade (or start fresh so it is hidden).
2. Send exactly one scroll-wheel event, then stop all input.

Expected: the scrollbar appears, then fades about 3 seconds later.
Actual: the scrollbar never appears.  (Variant: if the bar was already visible, keep the mouse still after scrolling—it never hides.)

## Suggested Fix

In `src/widgets/scrollbar.rs`:

1. Move the `is_visible` computation after the `with_last_scroll` update (reorder lines 515–526).
2. When visibility rests on a deadline, schedule a repaint: during paint, if `last_scroll_time` is within `FADE_OUT_DURATION`, spawn or refresh a timer (or use `window.request_animation_frame()`) that notifies at the deadline.  The “re-render loop” the comment at lines 524–525 fears comes from notifying unconditionally, not from a deadline-scheduled notify.
3. Delete the dead wheel listener (lines 558–574) and the `last_update` field with its `with_last_update` helper (lines 156–160, 666).

## Why the Fix Works

Computing visibility after recording activity makes the bar appear on the very frame that scrolled, and a deadline-scheduled notify gives the state machine the one repaint it needs to hide the bar without looping.

## Fix Ordering / Related

None—independent.  Same file as bug-0060 and bug-0062; coordinate edits to avoid conflicts.
