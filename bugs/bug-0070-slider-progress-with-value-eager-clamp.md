# bug-0070: Slider/ProgressBar `with_value` clamps against the not-yet-configured default range — documented builder order silently loses the initial value

**Severity:** medium
**Fix side:** CODE
**Component:** src/widgets/slider.rs:121-125; src/widgets/progress_bar.rs:77-81
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

`with_value` clamps eagerly against whatever the bounds happen to be at that point in the chain:

```rust
// src/widgets/slider.rs:121-125 (progress_bar.rs:77-81 is the same shape)
pub fn with_value(mut self, value: f64) -> Self {
    self.value = value.clamp(self.min, self.max);
    self
}
```

The README, the module docs, and CLAUDE.md’s quick reference all put `with_value()` _first_ in the chain — `.with_value(50.0).min(0.0).max(100.0)` — which works only because `50` happens to fit the `0..100` default.  `Slider::new(cx).with_value(150.0).min(0.0).max(200.0)` clamps `150` to `100` inside `with_value` (defaults `0..100`), and the later `max(200.0)` does not restore it — the slider starts at `100`, silently.  ProgressBar likewise: `.with_value(50.0).max(100.0)` yields value `1.0`, not `50.0` (default max is `1.0`).  No error, no event — just a wrong initial value.

## Reproduction (test-ready)

1. `ProgressBar::new().with_value(50.0).max(100.0)` then `pb.value()`.  Expected `Some(50.0)`; actual `Some(1.0)`.  Plain unit test — no GPUI context needed.
2. `Slider::new(cx).with_value(150.0).min(0.0).max(200.0)` then `slider.value()`.  Expected `150.0`; actual `100.0`.  Needs GPUI test context for `cx`.

## Suggested Fix

Store the requested initial value unclamped in `with_value` and clamp once, lazily — at first render, or inside `percentage` (ProgressBar) / `normalize_value` (Slider), using ordered bounds per bug-0039.  Alternative: clamp in `with_value` only if the resulting value would change, and document that builder order matters — but the lazy approach is preferred, since it also resolves bug-0039’s builder-ordering panic for these two widgets.

## Why the Fix Works

The constraint is applied after all constraints are known, so builder-call order stops mattering — matching how a declarative builder API reads.

## Fix Ordering / Related

Same normalize/clamp code paths as bug-0037, bug-0038, and bug-0039 — fix all four as one coordinated pass.  The lazy-clamp rework proposed in bug-0039 largely resolves this bug.
