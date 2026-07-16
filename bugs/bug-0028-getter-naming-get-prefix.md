# bug-0028: Getter naming — `get_*` prefix contradicts the documented convention, deliberately in two widgets, accidentally in one

**Severity:** low
**Fix side:** DECISION REQUIRED (Chris)
**Component:** CLAUDE.md:202-206 (naming convention); src/widgets/number_stepper.rs:352-375; src/widgets/slider.rs:183-201; src/widgets/checkbox_group.rs:101-105
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

> **DO NOT FIX without Chris’s decision.  Both a code-side and a doc-side resolution exist; the sections below present the options and a recommendation.**

## Description

`CLAUDE.md`’s naming convention (line 205) says: “Getters: direct name (e.g., `value()`, `is_checked()`)”.  Three widgets violate it, in two distinct ways:

**1.  Deliberate, collision-driven (NumberStepper, Slider).**  `NumberStepper` exposes `get_min()`, `get_max()`, `get_step()`, `get_resolution()`, `get_display_precision()` (`number_stepper.rs:352-375`), and `Slider` exposes `get_min()`, `get_max()`, `get_step()`, `get_display_precision()` (`slider.rs:183-201`).  The cause is a real collision: the direct names are already taken by the **builders** `min()`, `max()`, `step()` — the convention’s own rule for configuration builders (“direct name (e.g., `placeholder`, `theme`, `min`, `max`)”) consumed the getter names.  The choice is recorded verbatim in `TODO.md:188`: “Added missing getters: NumberStepper (`get_min/max/step/resolution/display_precision`), Slider (`get_min/max/step/display_precision`)” — so this is deliberate, and the _docs_ never absorbed the exception.

**2.  Accidental (CheckboxGroup).**  `CheckboxGroup::get_selected()` (`checkbox_group.rs:101-105`) uses the prefix where every sibling selection widget uses `selected()` / `selected_value()`.  Here the collision rationale does **not** apply — the builder is `with_selected()` (`checkbox_group.rs:81`), so the direct name `selected()` is free.  This one could simply be renamed.

## Reproduction (test-ready)

Comparison only — no GPUI machinery:

```bash
grep -n 'Getters: direct name' CLAUDE.md                       # line 205
grep -n 'pub fn get_' src/widgets/number_stepper.rs            # 353-373
grep -n 'pub fn get_' src/widgets/slider.rs                    # 184-199
grep -n 'pub fn get_selected\|pub fn with_selected' src/widgets/checkbox_group.rs   # 101 vs 81
grep -rn 'pub fn selected' src/widgets/radio_group.rs src/widgets/tab_bar.rs        # siblings use direct names
```

Expected: getters follow the documented convention or the convention documents the exception.  Actual: neither.

## Suggested Fix

**Question for Chris:** how should the getter-naming collision be resolved?

**Option (a) — amend the docs, minimal rename.**  Add the exception to `CLAUDE.md`’s naming convention: “when the direct name is taken by a builder, getters use `get_*`” (which blesses NumberStepper and Slider as-is).  Rename only the accidental case: `CheckboxGroup::get_selected()` → `selected()`, keeping `get_selected()` as a `#[deprecated]` alias for one release so clui migrates at leisure.

**Option (b) — rename the builders, free the getter names.**  Rename builders `min()`/`max()`/`step()` → `with_min()`/`with_max()`/`with_step()` across NumberStepper and Slider, then give getters the direct names.  This is **breaking for clui** (and any consumer), touches many call sites, and also contradicts the convention’s existing rule that configuration builders use direct names — so it trades one documented inconsistency for another unless that rule changes too.

**Recommendation: (a)** — smallest blast radius.  The `get_*` prefix in NumberStepper/Slider is a reasonable, recorded response to a genuine collision; documenting it costs one sentence, while (b) breaks a consumer to fix a cosmetic issue.  The only code change under (a) is the `CheckboxGroup` rename, where no collision ever justified the prefix.

## Why the Fix Works

Under (a), the convention becomes accurate (exceptions documented, accidents removed) without breaking clui; the deprecated alias makes the one rename non-breaking in practice.

## Fix Ordering / Related

- This decision **gates parts of bug-0003 and bug-0004** (the `CLAUDE.md` naming-convention text and quick-reference rows) if renames are chosen — land the decision before or with those doc rewrites.
- Under (a), the `CheckboxGroup` rename is a small CODE change plus a doc touch; coordinate with **bug-0002** (CheckboxGroup phantom selection) since both edit the same file.
