# bug-0051: `PickerMode` is publicly exported but wired to nothing — delete it, or implement the RGB/HSL mode switch it implies?

**Severity:** low
**Fix side:** CODE (feature) (decided 2026-08-12; see Decision below)
**Component:** src/widgets/color_swatch.rs:117-122 (definition); src/widgets/mod.rs:53-56 (public export)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

> **Decision (Chris, 2026-08-12): Option 2 — implement the RGB/HSL slider-mode switch the enum implies.**  Real feature work: a mode field consuming `PickerMode`, a builder, an HSL slider row in the picker popup, and doc updates (docs/COLOR_PICKER.md).  This also settles bug-0080’s wording — the module-doc claim of RGB/HSL modes becomes true rather than deleted.

## Description

`PickerMode { Rgb, Hsl }` is defined at src/widgets/color_swatch.rs:117-122 and publicly exported at src/widgets/mod.rs:53-56 — yet no field, method, or builder consumes it anywhere in the crate or the gallery.  A repo-wide grep finds exactly two occurrences: the definition and the export.

The enum implies a switchable RGB/HSL slider mode that does not exist: docs/COLOR_PICKER.md:15 documents only RGB component sliders, and no doc anywhere mentions `PickerMode`.  (The `color_swatch.rs` module doc, lines 4-5 and 11, also claims RGB/HSL slider modes — that _text_ is handled in bug-0080; this report is only about the exported type.)

As public API, the dead enum is a standing trap: a consumer can write code against it today, and it does nothing.

## Reproduction (test-ready)

Not a runtime bug — verify the dead export by grep:

```bash
grep -rn "PickerMode" src/
# → src/widgets/color_swatch.rs:117-122 (definition)
# → src/widgets/mod.rs:53-56 (export)
# no other occurrences
```

## Suggested Fix

**Question for Chris:** is an HSL slider mode still planned for the ColorSwatch picker?

**Option 1 — delete `PickerMode`** (the definition and the `mod.rs` export).  Technically a breaking API change, but the enum is wired to nothing, so no consumer can be using it meaningfully.  Verify clui with `cd ../clui && cargo check && cargo test` — it can only break if clui names the type, in which case that reference is itself dead code.

**Option 2 — implement the RGB/HSL slider-mode switch the enum implies**, and record the intent as a TODO.md entry until the feature lands.  This is real feature work: a mode field, a builder, an HSL slider row, and doc updates (docs/COLOR_PICKER.md currently documents RGB sliders only).

**Reviewer recommendation: Option 1 — delete now.**  It is cheapest before anyone depends on the type, and nothing in the crate’s docs promises the feature.  If HSL sliders are wanted later, the enum is trivial to reintroduce alongside the actual implementation.

## Why the Fix Works

Either resolution removes the lie: Option 1 makes the public API match the implemented feature set; Option 2 makes the feature set match the API.

## Fix Ordering / Related

Gates the wording of bug-0080’s module-doc correction — whether the “RGB and HSL modes” claim is deleted or becomes a documented feature depends on this decision.
