# bug-0055: ProgressBar never emits `Change(f64)`, contradicting the CLAUDE.md widget table

**Severity:** medium
**Fix side:** SPEC (docs) + minor CODE (notify guard) (decided 2026-08-12; see Decision below)
**Component:** src/widgets/progress_bar.rs:41-45 and 158-174; CLAUDE.md:229
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

> **Decision (Chris, 2026-08-12): option (b) — ProgressBar is a display widget; correct the CLAUDE.md table row to `Complete`.**  The `cx.notify()` no-op guard in `set_value` is added either way.

## Description

CLAUDE.md’s Widget Quick Reference (line 229) specifies:

```
| ProgressBar | Change(f64) | with_value(), indeterminate() |
```

The code’s event enum contains only `Complete`:

```rust
// src/widgets/progress_bar.rs:41-45
pub enum ProgressBarEvent {
    /// Progress reached 100%
    Complete,
}
```

and `set_value` (lines 158-174) emits nothing on ordinary value changes.  Under the project convention that form widgets’ `set_*` methods emit events, a consumer subscribing for `Change(f64)` per the table gets a compile error — no such variant exists.  TODO.md line 345 only claims the `Complete` event, so the table is likely the wrong side — but one of the two must change.

Separately, and regardless of the decision: `set_value` calls `cx.notify()` unconditionally even when the clamped value is identical to the old one, unlike every other widget’s guarded setter.  That guard should be added either way.

## Reproduction (test-ready)

1. Compile check: write `if let ProgressBarEvent::Change(v) = event` in a subscriber, per the CLAUDE.md table.  Does not compile — the variant does not exist.
2. Behavior check (needs GPUI entity/test context for subscription): `set_value(0.5, cx)` with a subscriber attached — no event of any kind arrives for non-max values.

## Suggested Fix

**The question for Chris:** is ProgressBar a _form widget_, whose `set_value` should emit `Change(f64)` — or a _display widget_, so the CLAUDE.md table row should be corrected to `Complete`?

Options:

- **(a) Code side:** add `Change(f64)` to `ProgressBarEvent`, emitted from `set_value` when the clamped value differs from the old one, and guard the unconditional `cx.notify()` — treating ProgressBar as a form widget.
- **(b) Doc side:** correct the CLAUDE.md table row (line 229) to `Complete` — treating ProgressBar as consumer-driven display state that needs no echo of its own `set_value` calls (the same reasoning as the navigation-widget convention).

**Reviewer recommendation: (b).**  The table was written aspirationally (see the git evidence collected in bug-0004), TODO.md agrees with the code, and a progress bar is display state driven by the consumer, who does not need an event echoing its own call.  But the choice affects the public API and is Chris’s.

Either way, guard the `cx.notify()` in `set_value` so a no-op call does not trigger a re-render.

## Why the Fix Works

Either resolution makes the documented and shipped event surfaces agree, so consumers can code against the table without compile errors or silent non-events; the notify guard restores the guarded-setter convention every other widget follows.

## Fix Ordering / Related

This decision gates the doc-table rewrite bug-0004 (the corrected table must reflect the decided event name/payload).  Coordinate with bug-0056, which edits the same `set_value` region of progress_bar.rs.
