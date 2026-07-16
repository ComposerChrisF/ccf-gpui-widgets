# bug-0058: README symlink target misspells the crate name — “pgui” for gpui

**Severity:** low
**Fix side:** SPEC (docs)
**Component:** README.md (committed symlink, git mode 120000); docs/ccf-pgui-widgets-README.md (the misnamed target)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

> **THE DOCS ARE THE PROBLEM — the code is correct and consumed by clui.  Do NOT change code to match the docs.**

## Description

`README.md` is a committed symlink — `git ls-tree HEAD README.md` shows mode `120000` pointing at `docs/ccf-pgui-widgets-README.md`.  The symlink arrangement itself is sound (one canonical file, no duplicate-drift risk), but the target filename misspells the crate: “pgui” where the crate is ccf-**gpui**-widgets.

The typo was born in commit `ee0935c` (“Consolidate documentation under docs/ directory”), whose message repeats it — “replace README.md with a symlink to docs/ccf-pgui-widgets-README.md so the canonical README lives with the rest of the documentation” — confirming an accidental transposition rather than a deliberate name.

A repo-wide grep for `pgui` matches only the filename itself (and the symlink that carries it), so nothing else references the misspelled path.

## Reproduction (test-ready)

```bash
git ls-tree HEAD README.md        # 120000 blob …  README.md  (symlink)
readlink README.md                # docs/ccf-pgui-widgets-README.md
grep -rn 'pgui' --exclude-dir=.git --exclude-dir=bugs .   # only the filename/symlink
```

Expected: the README filename spells the crate name.  Actual: `ccf-pgui-widgets-README.md`.

## Suggested Fix

In a single commit (both halves must move together, or `README.md` dangles):

```bash
git mv docs/ccf-pgui-widgets-README.md docs/ccf-gpui-widgets-README.md
ln -sf docs/ccf-gpui-widgets-README.md README.md
git add README.md
```

Cargo resolves the symlink when packaging, so crates.io publication is unaffected by either the current state or the rename.

## Why the Fix Works

The canonical README keeps its single-copy home under `docs/` and regains a correctly spelled name; recreating the symlink in the same commit means no revision ever has a dangling `README.md`.

## Fix Ordering / Related

- **bug-0057** (README examples stale) and **bug-0023** (color-count claim at line 216) edit this same file — do this rename first or in the same pass so their edits land in the surviving filename.
- Bug reports in this directory refer to the file by its current (typo) name; after the rename, those references describe history, not a live path.
