# bug-0069: Slider module docs claim Shift 10x / Alt 0.1x drag modifiers that do not exist anywhere in the code

**Severity:** low
**Fix side:** SPEC (docs)
**Component:** src/widgets/slider.rs:9-11 (vs. drag at 443-453 and keyboard at 483-494)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

> **THE DOCS ARE THE PROBLEM — the code is correct and consumed by clui.  Do NOT change code to match the docs.**

## Description

The module header states:

```rust
// src/widgets/slider.rs:9-11
//! - **Click and drag** the thumb or track to adjust the value smoothly
//!   - Hold **Shift** for fast adjustment (10x step)
//!   - Hold **Alt/Option** for slow/fine adjustment (0.1x step)
```

The drag path is purely positional — `set_value_from_position` (lines 263-279) never reads modifiers — so Shift/Alt during drag do nothing.  `modifiers.alt` is not consulted anywhere in the file, so there is no 0.1x behavior on keyboard either.  Shift 10x exists only for the arrow keys (lines 483-494) — which matches TODO.md line 340 (`Keyboard support: Left/Right arrows, Shift for 10x step`).

## Reproduction (test-ready)

1. Read the module doc, then hold Alt and drag the thumb — or hold Alt and press Right.
2. Expected per the doc: 0.1x fine adjustment.
3. Actual: identical to unmodified.
4. Hold Shift and drag: expected 10x per the doc; actual: identical to unmodified (Shift affects arrow keys only).

## Suggested Fix

Doc edit (the default): in the module header (src/widgets/slider.rs:6-13), move the Shift bullet under the arrow-keys bullet (as `Hold Shift for 10x step`) and delete the Alt bullet.  The drag bullets should describe only positional adjustment.

**Option for Chris:** if fine keyboard stepping is wanted, Alt = 0.1x could instead be implemented on the arrow keys — the same multiplier-selection shape as the NumberStepper keyboard fix in bug-0036.  The phantom _drag_-modifier claim should be deleted either way, since positional dragging has no meaningful sensitivity multiplier.

## Why the Fix Works

The doc then describes exactly the shipped interaction set, and no consumer designs around a nonexistent fine-drag mode.

## Fix Ordering / Related

Independent as a doc-only edit.  If Chris opts to implement Alt on the arrow keys instead, coordinate with bug-0036 (parallel multiplier logic in NumberStepper).
