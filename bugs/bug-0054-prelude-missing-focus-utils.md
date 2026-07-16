# bug-0054: Prelude omits `with_focus_actions`, one of the four documented widget utilities

**Severity:** low
**Fix side:** CODE
**Component:** `src/lib.rs:96-100` (crate docs, Utilities list) vs `src/lib.rs:141-164` (prelude); `README.md:105`
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

The crate docs’ “Utilities” section lists four helpers (`src/lib.rs:97-100`; `README.md:105` likewise):

```text
- primary_button      - Blue/accent styled button
- secondary_button    - Gray styled button
- danger_button       - Red styled button
- with_focus_actions  - Add Tab/Shift-Tab focus navigation to elements
```

The prelude (`src/lib.rs:141-164`) re-exports the three button functions but not `with_focus_actions`—nor `EnabledCursorExt`, `FocusNext`, or `FocusPrev`, which `with_focus_actions`-style code typically needs.  The symbol is exported from `widgets` (`src/widgets/mod.rs:63-66`), so this is an inconsistency in the curated prelude rather than a missing symbol—but a consumer using `use ccf_gpui_widgets::prelude::*` gets three of the four documented utilities.

## Reproduction (test-ready)

Compile-time check; no GPUI window machinery needed:

```rust
use ccf_gpui_widgets::prelude::*;

// with_focus_actions(div(), cx);                          // error: cannot find function
// ccf_gpui_widgets::widgets::with_focus_actions(div(), cx);  // resolves fine
```

Expected: all four documented utilities resolve from the prelude.
Actual: `with_focus_actions` is unresolved; only the `widgets::` path works.

## Suggested Fix

Recommended: add `with_focus_actions` (and plausibly `EnabledCursorExt`, `FocusNext`, and `FocusPrev`) to the prelude re-export list at `src/lib.rs:143-154`.

Alternative, if the prelude curation is deliberate: trim the docs’ Utilities list (`src/lib.rs:100`, `README.md:105`) so the docs stop advertising a helper the prelude does not deliver.

## Why the Fix Works

It aligns the prelude with the documented utility set, so the four advertised helpers import uniformly.

## Fix Ordering / Related

None—independent.  If the doc-trim alternative is chosen instead, coordinate with bug-0057 (stale README examples), which rewrites the same README region.
