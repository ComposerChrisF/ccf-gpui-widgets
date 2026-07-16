# bug-0080: Module-doc examples that cannot compile, hidden behind `ignore` fences — wrong builder names and phantom HSL claims

**Severity:** low
**Fix side:** SPEC (docs)
**Component:** src/widgets/dropdown.rs:17; src/widgets/checkbox_group.rs:14; src/widgets/color_swatch.rs:4-5, 11, 24
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

> **THE DOCS ARE THE PROBLEM — the code is correct and consumed by clui.  Do NOT change code to match the docs.**

## Description

Three widget module-doc examples call methods that do not exist as written.  All three sit in ```` ```ignore ```` fences, so `cargo test` never compiles them and the rot is invisible to CI.  These are copy-paste surfaces — the first thing a consumer or agent copies when using the widget.

**(a) `dropdown.rs:17`** — the example builds:

```rust
Dropdown::new(cx)
    .choices(vec!["Option 1".to_string(), "Option 2".to_string()])
    .selected_index(0)
```

`selected_index()` is the **getter** (`&self`, returns the index); the builder is `with_selected_index()`.  Copied verbatim, the chain fails to compile (the getter takes no argument and does not return `Self`).

**(b) `checkbox_group.rs:14`** — the example calls:

```rust
CheckboxGroup::new(cx)
    .choices(vec![…])
    .selected(vec!["Red".to_string(), "Blue".to_string()])
```

No `selected()` method exists at all; the builder is `with_selected()` (`checkbox_group.rs:81`).

**(c) `color_swatch.rs:24`** — the example calls:

```rust
ColorSwatch::new(cx)
    .value("#3b82f6")
    .with_alpha(true)
```

`value()` is the `&self` **getter**; the builder is `with_value()`.

**(d) `color_swatch.rs:4-5` and `:11`** — the module doc claims the picker opens “with RGB/HSL sliders” and describes a “Color picker popup with RGB and HSL modes”.  No HSL sliders and no RGB/HSL mode switch exist — the popup has RGB component sliders only (plus the HSV canvas and hue slider; `docs/COLOR_PICKER.md:12` correctly documents the canvas as HSV saturation/brightness).  The related dead `PickerMode` export is **bug-0051**.  While correcting these lines, note the doc’s “2D saturation/lightness selector” wording should be “saturation/brightness (HSV)” for the same reason.

## Reproduction (test-ready)

1. Change any of the three fences from ```` ```ignore ```` to ```` ```no_run ```` (temporarily, do not commit) and run `cargo test --doc` — each fails to compile with “method not found” / wrong-signature errors as described.
2. Or verify by signature:

   ```bash
   grep -n 'pub fn selected_index\|pub fn with_selected_index' src/widgets/dropdown.rs
   grep -n 'pub fn selected\|pub fn with_selected' src/widgets/checkbox_group.rs
   grep -n 'pub fn value\|pub fn with_value' src/widgets/color_swatch.rs
   grep -n 'Hsl\|hsl' src/widgets/color_swatch.rs   # no HSL slider/mode UI, only color-math imports
   ```

Expected: examples compile as written (modulo the GPUI context the `ignore` legitimately elides).  Actual: three wrong method names; one feature claim with no implementation behind it.

## Suggested Fix

- `src/widgets/dropdown.rs:17`: `.selected_index(0)` → `.with_selected_index(0)`.
- `src/widgets/checkbox_group.rs:14`: `.selected(vec![…])` → `.with_selected(vec![…])`.
- `src/widgets/color_swatch.rs:24`: `.value("#3b82f6")` → `.with_value("#3b82f6")`.
- `src/widgets/color_swatch.rs:4-5, 11`: delete or correct the HSL claims — the popup offers an HSV canvas, hue slider, optional alpha slider, and RGB component sliders; there is no HSL mode.  Fix “saturation/lightness” → “saturation/brightness (HSV)” in the same pass.

These are doc-comment edits only; rustdoc renders them, so run `cargo doc` afterward to confirm formatting.

## Why the Fix Works

The first example a consumer copies compiles on first paste, and the color-swatch doc stops promising a mode the widget never had — with method names verified against the shipped signatures rather than aspirational ones.

## Fix Ordering / Related

- **bug-0051** owns the dead `PickerMode` export that the HSL-mode claim in (d) orbits — resolve together for a consistent story.
- The identical `140 colors` claim at `color_swatch.rs:10` is owned by **bug-0023**.
- Same class of rot as **bug-0057** (README examples) — a follow-up sweep could convert `ignore` fences to `no_run` where GPUI types permit, so future drift fails `cargo test --doc` instead of hiding.
