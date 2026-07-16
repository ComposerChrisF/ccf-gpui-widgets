# bug-0078: TODO.md stale references — pre-move doc paths, long-completed checkboxes, and an overstated known issue

**Severity:** low
**Fix side:** SPEC (docs)
**Component:** TODO.md:362, 406-407, 421, 425, 442
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

> **THE DOCS ARE THE PROBLEM — the code is correct and consumed by clui.  Do NOT change code to match the docs.**

**Scope note:** the coordinating session is about to restructure `TODO.md` with a prioritized bug queue at the top.  This report deliberately covers only the specific stale references below, all of which live in the Future-Enhancements / Known-Issues / Resolved-Issues sections and survive that restructure.

## Description

**(a) Two doc references predate the `docs/` consolidation.**  Commit `ee0935c` moved the documentation under `docs/`, but:

- `TODO.md:425` says “See `GPUI-LAYOUT-PATTERNS.md` for details” (no directory) — the file is `docs/GPUI-LAYOUT-PATTERNS.md`.
- `TODO.md:442` says “See `doc/TAB_STOP_SOLUTIONS.md` for full details” (pre-move path, and singular `doc/`) — the file is `docs/TAB_STOP_SOLUTIONS.md`.

**(b) Completed work still shown as open.**

- Line 362: `- [ ] Set up GitHub repository` — the repo and CI both exist: `git remote -v` shows `github.com/ComposerChrisF/ccf-gpui-widgets`, `.github/workflows/ci.yml` is committed, and commits `911a6e6` / `4473d18` are CI maintenance.
- Lines 406-407: `- [ ] Example application` and `- [ ] Widget showcase/gallery` — `examples/widget_gallery.rs` exists, is registered in `Cargo.toml:42-44`, and roughly 20 other TODO entries describe improving it.
- By contrast, line 361 (`- [ ] Publish to crates.io`) genuinely remains open — leave it.

**(c) Known Issue #2 overstates the failure mode.**  Line 421 says FilePicker/DirectoryPicker “require the `file-picker` feature flag and won’t compile without it”.  Verified: `cargo check` with default features passes — the widgets are cfg-gated out cleanly, so the _library_ always compiles.  What actually fails is consumer code that names those widgets without enabling the feature.  As written, the issue reads like a build defect; it should describe the intended feature gate.

## Reproduction (test-ready)

```bash
ls GPUI-LAYOUT-PATTERNS.md doc/TAB_STOP_SOLUTIONS.md      # both: No such file
ls docs/GPUI-LAYOUT-PATTERNS.md docs/TAB_STOP_SOLUTIONS.md # both exist
git remote -v && ls .github/workflows/ci.yml               # repo + CI exist (line 362 says otherwise)
ls examples/widget_gallery.rs && grep -n 'widget_gallery' Cargo.toml   # lines 406-407 say otherwise
cargo check                                                 # passes with default features (line 421 says otherwise)
```

Expected: paths resolve, checkboxes reflect reality, the known issue describes actual behavior.  Actual: all five spots are stale.

## Suggested Fix

In `TODO.md`:

- **Line 425**: `GPUI-LAYOUT-PATTERNS.md` → `docs/GPUI-LAYOUT-PATTERNS.md`.
- **Line 442**: `doc/TAB_STOP_SOLUTIONS.md` → `docs/TAB_STOP_SOLUTIONS.md`.
- **Line 362**: check the box (or delete the line) — done.
- **Lines 406-407**: check both boxes — the gallery satisfies both entries (keep line 361 open).
- **Line 421**: reword to describe the feature gate, e.g.: “FilePicker/DirectoryPicker are gated behind the `file-picker` feature; the library compiles cleanly without it, but consumer code referencing these widgets requires the feature to be enabled.”

## Why the Fix Works

The two references resolve again, the Publishing/Documentation sections stop advertising finished work as open, and Known Issue #2 stops describing working cfg-gating as a compilation defect.

## Fix Ordering / Related

- Coordinate with the coordinating session’s `TODO.md` restructure — apply these edits after (or as part of) that pass so they land in the surviving sections.
- The tab-stop material referenced at line 442 is load-bearing for **bug-0003** (a) and **bug-0075**; fixing the path keeps that chain navigable.
