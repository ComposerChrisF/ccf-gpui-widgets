# bug-0023: Color-count claims disagree across the docs — and none of them is right

**Severity:** low
**Fix side:** SPEC (docs)
**Component:** CLAUDE.md:253; src/lib.rs:122; src/theme.rs:22, 51; TODO.md:306; docs/COLOR_PICKER.md:10, 39; src/widgets/color_swatch.rs:10; docs/ccf-pgui-widgets-README.md:216
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

> **THE DOCS ARE THE PROBLEM — the code is correct and consumed by clui.  Do NOT change code to match the docs.**

## Description

The documentation makes three different numeric claims about the theme’s color-field count, and a fourth about the named-color table — and every one of them is wrong:

- `CLAUDE.md:253`: “The `Theme` struct contains ~30 semantic color fields”.
- `src/lib.rs:122` (crate doc): “all ~30 fields have `with_*` methods”.
- `src/theme.rs:22` and `:51`: “derives all 52 theme colors from 7 seeds” — sitting directly above the 49-field struct.
- `TODO.md:306`: “`Theme::from_palette(palette)` derives all 52 theme colors from 7 seeds”.
- `docs/COLOR_PICKER.md:10` and `:39`, `src/widgets/color_swatch.rs:10`, and `docs/ccf-pgui-widgets-README.md:216`: “140 CSS named colors”.

Ground truth, verified by count against the working tree:

- `Theme` has exactly **49** `pub … : u32` fields.
- `theme.rs` has **56** `pub fn with_*` builders (every field covered, plus compound helpers).
- `NAMED_COLORS` (`src/utils/color.rs:397`) has **148** entries — the full CSS named-color list.  The code is right; the docs undercount it.

So “~30” is stale (the struct grew), “52” matches no count in the code (not fields, not builders), and “140” miscounts a table whose real length is 148.

## Reproduction (test-ready)

Pure counting — no GPUI machinery:

```bash
awk '/^pub struct Theme/,/^}/' src/theme.rs | grep -c 'pub .*: u32'    # 49
grep -c 'pub fn with_' src/theme.rs                                     # 56
awk 'NR>=398 && /^\];/{exit} /^[[:space:]]*\(/{c++} END{print c+0}' src/utils/color.rs   # 148
grep -n '~30\|52 theme colors\|all 52\|140' CLAUDE.md src/lib.rs src/theme.rs TODO.md docs/COLOR_PICKER.md src/widgets/color_swatch.rs docs/ccf-pgui-widgets-README.md
```

Expected: doc claims match the counts.  Actual: three mutually inconsistent claims (~30, 52, 140), none matching the code (49, 148).

## Suggested Fix

Standardize on the true counts in all locations listed in **Component**:

- Field count: **49** (or “~50” where an approximation reads better) — `CLAUDE.md:253`, `src/lib.rs:122`, `src/theme.rs:22` and `:51`, `TODO.md:306`.
- Named colors: **148** — `docs/COLOR_PICKER.md:10` and `:39`, `src/widgets/color_swatch.rs:10` (module doc), `docs/ccf-pgui-widgets-README.md:216`.

Secondary occurrence spotted while verifying: `TODO.md:77` says “`with_*` builder methods for all 30+ fields” — technically true but stale; update in the same pass if convenient.

**Alternative flagged for Chris:** if “52” was a design _target_ (three more derived colors intended from the palette work), the other resolution is to add 3 fields — but nothing in `TODO.md` or the palette code indicates a missing trio, so the recommendation is the doc fix.  Note the counts will drift again; wording like “49 fields (as of v0.1.2)” or an approximation keeps the claim honest longer.

## Why the Fix Works

Every count becomes a projection of a mechanically countable code fact, and all documents agree — removing the current situation where a reader gets three different answers depending on which doc they open.

## Fix Ordering / Related

- `docs/ccf-pgui-widgets-README.md:216` sits in the file **bug-0057** rewrites and **bug-0058** renames — fold this one-word change into whichever pass touches that file first.
- `CLAUDE.md:253` sits in the file **bug-0003**/**bug-0004** rewrite — same consolidation applies.
