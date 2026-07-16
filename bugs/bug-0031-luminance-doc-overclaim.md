# bug-0031: `luminance()` doc claims ITU-R BT.709 relative luminance but the code computes luma on gamma-encoded sRGB

**Severity:** low
**Fix side:** SPEC (docs)
**Component:** src/utils/color.rs:575-584 (`luminance` doc comment), 586-589 (`is_dark`); in-crate consumer: src/theme.rs:482 (`Theme::from_palette` mode detection)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

> **THE DOCS ARE THE PROBLEM — the code is correct and consumed by clui.  Do NOT change code to match the docs.**

## Description

The doc comment on `luminance()` says it “Uses the ITU-R BT.709 formula for relative luminance”, but the coefficients are applied to gamma-encoded channel values without linearization.  That computes Rec. 709 _luma_ (Y′), not relative luminance.  Verified: `luminance(0x808080)` returns `0.502`, whereas the true relative luminance of `#808080` is `0.216`.

The behavior itself is defensible: `is_dark`’s `0.5` threshold sits at perceptual mid-gray, which is arguably the _desired_ behavior for theme-mode detection — its only in-crate use (`Theme::from_palette`, theme.rs:482).  But anyone using `luminance()` for WCAG contrast math will get wrong numbers while the doc tells them it is the standard formula.  The overclaim, not the computation, is the defect.

## Reproduction (test-ready)

Pure unit test:

```rust
// Actual behavior (luma of gamma-encoded sRGB):
assert!((luminance(0x808080) - 0.502).abs() < 0.001);
// The documented (linearized) formula would yield 0.2159 for the same input.
```

## Suggested Fix

Rewrite the doc comment at src/utils/color.rs:575-584 to describe what the code does: luma of gamma-encoded sRGB — a perceptual approximation, explicitly **not** WCAG relative luminance — and note that `is_dark`’s `0.5` threshold is calibrated to this scale (perceptual mid-gray).  This doc-only fix is safe and complete on its own.

**Optional behavior change — needs Chris’s decision, NOT part of this fix:** linearizing to true WCAG relative luminance would flip `is_dark`’s behavior around the threshold (mid-gray would flip from “light” to “dark”), which changes `Theme::from_palette`’s mode detection.  Pursue that only if WCAG contrast math becomes a crate goal, and re-tune the `is_dark` threshold in the same pass.

## Why the Fix Works

The doc then matches the implementation, so a consumer reaching for WCAG contrast math is warned off instead of misled — with zero behavior change for the existing, correctly-calibrated `is_dark`/`from_palette` path.

## Fix Ordering / Related

The doc-only fix is independent.  If the optional linearization is ever chosen, coordinate with bug-0027 (`Theme::from_palette` text scale) and re-verify both theme modes.
