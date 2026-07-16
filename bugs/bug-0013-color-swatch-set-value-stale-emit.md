# bug-0013: ColorSwatch `set_value` emits `Change` unconditionally — including a stale value on parse failure and redundant events on no-op

**Severity:** medium
**Fix side:** CODE
**Component:** src/widgets/color_swatch.rs:316-319 (`set_value`), 327-345 (`set_value_internal`); `handle_input_commit` has the same unconditional emit
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

CLAUDE.md’s form-widget convention is that `set_*` emits when the value actually changes.  `set_value` (lines `316-319`) instead calls `set_value_internal` — which leaves _all_ state untouched when `parse_color_alpha` fails (lines `327-345`) — and then emits `ColorSwatchEvent::Change(self.value.clone())` regardless.  Three consequences:

1. **Stale-value event:** `set_value("garbage", cx)` emits a `Change` carrying the _old_ value — subscribers see a change that never happened.
2. **Redundant events:** calling `set_value` with the current value emits anyway.
3. **Double events on success:** a successful set also triggers the bug-0008 echo (`set_value_internal` → `hex_input.set_value` → `handle_input_change` → emit), so subscribers get two events per programmatic set.

`handle_input_commit` (Enter/Blur) has the same shape: it emits even when the committed value is unchanged.

## Reproduction (test-ready)

Needs a GPUI `Context` and a subscription; no window or dispatch required:

1. Build a `ColorSwatch` (default value `#000000`) and subscribe.
2. Call `swatch.set_value("not-a-color", cx)`.
3. Expected: no event.  Actual: one `Change("#000000")` fires.
4. Call `set_value("#FF0000", cx)` twice.
5. Expected: no event after the second call.  Actual: events fire after both (and, via the echo, twice per call).

## Suggested Fix

Make `set_value_internal` return `bool` — whether the parsed value differed from the previous `self.value` — and have `set_value` emit only on `true`.  Apply the same changed-check before the emits in `handle_input_commit`.

## Why the Fix Works

Emission then tracks actual state change, matching the documented convention; the stale-value event and the no-op duplicates disappear, and (together with the bug-0008 guard) each programmatic set produces exactly one event.

## Fix Ordering / Related

bug-0008 (echo reentrancy guard) must land before or with this fix — it removes the duplicate emission path this fix cannot reach.  bug-0011’s revert fix consumes the `set_value_internal → bool` changed-check introduced here, so land this one first or together.
