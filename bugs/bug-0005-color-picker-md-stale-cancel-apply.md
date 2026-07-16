# bug-0005: COLOR_PICKER.md predates the Cancel/Apply workflow — overview and Event Flow describe removed behavior

**Severity:** medium
**Fix side:** SPEC (docs)
**Component:** docs/COLOR_PICKER.md:7-16 (overview), 223-229 (Event Flow)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

> **THE DOCS ARE THE PROBLEM — the code is correct and consumed by clui.  Do NOT change code to match the docs.**

## Description

`docs/COLOR_PICKER.md` describes the picker as it existed before commit `6772468` (“Improve ColorPicker popover UX with Cancel/Apply workflow”).  `git show 6772468 --stat` confirms that commit touched only `TODO.md` and `src/widgets/color_swatch.rs` — never `COLOR_PICKER.md` — so the design doc silently fell behind.

What the doc says:

- The overview bullet list (lines 7-16) enumerates the popup contents (canvas, hue slider, alpha slider, RGB sliders, old/new comparison) with **no mention of the Cancel/Apply buttons, the ESC-revert behavior, or the clickable “Old” swatch**.
- “Event Flow” step 5 (line 229) says: “User clicks outside → `close_picker()`” — i.e. clicking outside merely closes, with no commit/revert semantics.

What the code does (all in `src/widgets/color_swatch.rs`):

- Keybindings (lines 49-62): `escape` → `ClosePicker` action, `enter` → `ApplyPicker` action, in the `CcfColorPicker` context.
- `cancel_picker()` (lines 446-452) reverts to `original_value` and closes; `apply_picker()` (lines 454-459) keeps the current value and closes; plain `close_picker()` (lines 440-444) still exists.
- **ESC reverts to the original color and closes; Enter applies and closes; clicking outside now _applies_ the current color** (not a bare close).
- Cancel and Apply buttons exist at the bottom of the picker.
- The “Old” color swatch is clickable to reset to the original color (lines 1052-1056: `on_click` → `set_value_internal(&this.original_value…)`).

`TODO.md:311-318` (“ColorPicker Popover UX Improvements (2026-02-04)”) records the new behavior accurately — the design doc is the only stale surface.

## Reproduction (test-ready)

Documentation bug — verification by comparison:

1. Read `docs/COLOR_PICKER.md:223-229`; step 5 claims clicking outside merely closes.
2. Read `src/widgets/color_swatch.rs:440-459` — three distinct exits exist (`close`, `cancel`-revert, `apply`), and the outside-click path applies.
3. Read `src/widgets/color_swatch.rs:49-62` — ESC and Enter are bound to cancel/apply semantics; the doc mentions neither.
4. Behavioral confirmation (needs GPUI test context or the gallery): open the picker, drag the hue slider, press ESC — the swatch reverts to the original color, which the doc nowhere predicts.

Expected: the design doc describes the shipped interaction model.  Actual: it describes the pre-`6772468` model.

## Suggested Fix

In `docs/COLOR_PICKER.md`:

- **Lines 7-16 (overview)**: add bullets for the Cancel/Apply buttons, the ESC-revert / Enter-apply keybindings, and the clickable “Old” swatch (resets to the original color).
- **Lines 223-229 (Event Flow)**: rewrite step 5 into the three real exits — ESC or Cancel button → `cancel_picker()` (revert to original, close); Enter or Apply button → `apply_picker()` (keep current value, close); click outside → applies the current color and closes.  Mention the `ApplyPicker` action added alongside the existing `ClosePicker`.

Source of truth for the wording: `src/widgets/color_swatch.rs:440-459` and `TODO.md:311-318`.

## Why the Fix Works

The design doc regains parity with the shipped interaction model, so anyone (or any agent) extending the picker from the doc no longer designs against removed click-outside semantics.

## Fix Ordering / Related

- Independent of the code-side ColorSwatch bugs, but if **bug-0011** (revert emits no event) changes revert semantics, reflect that in the same doc pass.
- The “140 colors” claim at `docs/COLOR_PICKER.md:10` is owned by **bug-0023**.
