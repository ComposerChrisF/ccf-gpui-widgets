# bug-0056: ProgressBar `Complete` fires at most once per widget lifetime — reaching 100% a second time emits nothing

**Severity:** medium
**Fix side:** CODE
**Component:** src/widgets/progress_bar.rs:159-174 (flag cleared only at 177-188)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

`completed_emitted` is set on the first arrival at max and cleared only by `reset()` or `set_indeterminate()` (lines 177-188) — not when the value moves back below max via `set_value`:

```rust
// src/widgets/progress_bar.rs:164-171
if !self.completed_emitted && (clamped - self.max).abs() < f64::EPSILON {
    // Only emit if we weren't already at max
    if old_value.is_none_or(|v| (v - self.max).abs() >= f64::EPSILON) {
        self.completed_emitted = true;
        cx.emit(ProgressBarEvent::Complete);
    }
}
```

A consumer driving two sequential phases with `set_value(0.0); ...; set_value(1.0)` — a natural way to restart, and equivalent in effect to `reset()` — gets `Complete` for phase 1 and silence for phase 2.  The inner `old_value` check already prevents re-emitting while sitting at max, so the outer lifetime latch is redundant for its stated purpose (`only once`) and only serves to swallow legitimate later completions.

## Reproduction (test-ready)

Needs GPUI entity/test context for `cx.emit` and subscription; otherwise plain method calls:

1. `let pb = cx.new(|_| ProgressBar::new());` and subscribe to its events.
2. `pb.set_value(1.0, cx)` → `Complete` emitted (correct).
3. `pb.set_value(0.0, cx); pb.set_value(1.0, cx)`.
4. Expected: `Complete` again — progress genuinely re-reached 100%.
5. Actual: no event.

## Suggested Fix

Clear `completed_emitted` in `set_value` (src/widgets/progress_bar.rs:159-174) whenever the clamped value is below max:

```rust
if (clamped - self.max).abs() >= f64::EPSILON {
    self.completed_emitted = false;
}
```

The existing `old_value` guard already prevents duplicate emissions while at max.  If once-per-lifetime really is the intended semantics, document it on the event variant and on `set_value` instead.

## Why the Fix Works

The flag then means “currently at max and already announced”, which emits exactly once per arrival at 100% — the semantics both TODO.md (`Complete event when reaching 100%`) and the module’s doc example imply.

## Fix Ordering / Related

Coordinate with bug-0055 — same `set_value` region (the notify guard, and possibly a new `Change(f64)` emission, land in the same lines).
