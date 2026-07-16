# bug-0015: Cursor-blink timers accumulate on rapid focus cycling

**Severity:** low
**Fix side:** CODE
**Component:** `src/widgets/text_input.rs:840-865`; `src/widgets/password_input.rs:683-708`
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

Both `TextInput` and `PasswordInput` spawn the cursor-blink loop from `render()` when the widget is focused, guarded only by the boolean `blink_timer_active`:

```rust
// src/widgets/text_input.rs:840-865 (password_input.rs:683-708 is identical)
if is_focused && !self.blink_timer_active {
    self.blink_timer_active = true;
    let entity = cx.entity();
    let blink_period = CursorBlink::blink_period();
    window
        .spawn(cx, async move |async_cx| loop {
            smol::Timer::after(blink_period).await;
            let should_continue = async_cx
                .update_entity(&entity, |this, cx| {
                    if !this.blink_timer_active {
                        return false;
                    }
                    cx.notify();
                    true
                })
                .unwrap_or(false);
            if !should_continue {
                break;
            }
        })
        .detach();
}

if !is_focused {
    self.blink_timer_active = false;
}
```

On blur, render sets `blink_timer_active = false`, and the running loop only notices at its next tick — up to 530 ms later (`CursorBlink::blink_period()`).  If the widget is refocused before that tick, the focused render sees `!blink_timer_active`, sets the flag `true`, and spawns a **second** loop — and the first loop’s check `if !this.blink_timer_active { return false; }` now passes again, so it survives.  Every blur→refocus cycle faster than the blink period adds one immortal timer, and each surviving loop issues `cx.notify()` every 530 ms for as long as the widget stays focused.

## Reproduction (test-ready)

Needs GPUI test context (window, render, focus dispatch):

1. Render a focused `TextInput` — loop 1 spawns.
2. Blur it (render sets the flag `false`), then refocus within 530 ms — render spawns loop 2; loop 1 has not ticked yet, so it never observes the `false` flag.
3. Repeat the blur→refocus cycle N times, each within the blink period.
4. Expected: exactly one live blink loop.  Actual: N+1 concurrent loops — observable by instrumenting the loop body with a counter, or by counting `cx.notify()` calls per 530 ms period while the widget stays focused.

## Suggested Fix

Replace the boolean guard with a generation counter, in both files:

1. Add a field `blink_timer_generation: u64` (initialized 0).
2. When spawning: increment it and capture the new value in the task —

```rust
if is_focused && !self.blink_timer_active {
    self.blink_timer_active = true;
    self.blink_timer_generation += 1;
    let my_generation = self.blink_timer_generation;
    // ... in the loop body:
    if this.blink_timer_generation != my_generation || !this.blink_timer_active {
        return false;
    }
}
```

Apply identically to `text_input.rs:840-865` and `password_input.rs:683-708`.

## Why the Fix Works

Each spawn invalidates every earlier task unambiguously — a stale loop sees a generation mismatch on its very next tick and exits — so at most one timer survives regardless of focus-cycle timing.

## Fix Ordering / Related

None — independent.  The identical defect exists in both widgets; fix both in one pass.
