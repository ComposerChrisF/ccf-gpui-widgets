# bug-0014: ColorSwatch `with_enabled(false)` does not disable the inner hex input unless `with_value` was also called

**Severity:** medium
**Fix side:** CODE
**Component:** src/widgets/color_swatch.rs:266-269 (`with_enabled`), 562-570 (sync block), 245 (only `with_value` arms it)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

`with_enabled` (lines `266-269`) sets only the widget’s own `enabled` flag.  The enabled state is propagated to the child hex `TextInput` only inside the first-render `needs_input_sync` block (lines `562-570`) — and `needs_input_sync` is set only by `with_value` (line `245`).

So `ColorSwatch::new(cx).with_enabled(false)` leaves the child `TextInput` fully interactive: the swatch click handlers check `this.enabled`, but the text input was constructed enabled and nothing ever calls `input.set_enabled(false, …)` on this path.  The user can type into a “disabled” widget, changing the color and emitting `Change` events.

The programmatic path is fine — `set_enabled` (lines `304-313`) propagates correctly.  Only the builder path is broken, and only when `with_value` is absent.

## Reproduction (test-ready)

Needs GPUI test context (typing into the child input):

1. Build `ColorSwatch::new(cx).with_enabled(false)` — deliberately _without_ `with_value`.
2. Type `#FF0000` in the hex field.
3. Expected: input inert — no state change, no event.
4. Actual: the text is accepted, the preview updates, and `Change("#FF0000")` is emitted.

## Suggested Fix

Set `self.needs_input_sync = true;` in `with_enabled` (lines `266-269`), treating the flag as “builder state pending sync” rather than “value pending sync”.

## Why the Fix Works

The existing first-render sync block already writes _both_ value and enabled state to the child input — it just never runs unless `with_value` armed it.  Arming it from `with_enabled` closes the gap with no new plumbing.

## Fix Ordering / Related

bug-0009 (`placeholder()` dead builder) piggybacks on this same sync block — fix the two together.
