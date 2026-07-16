# bug-0066: Same-named selection setters have divergent out-of-range and membership semantics

**Severity:** low
**Fix side:** DECISION REQUIRED (Chris)
**Component:** `src/widgets/dropdown.rs:117-120` and `158-167`; `src/widgets/radio_group.rs:195-201` and `247-256`; `src/widgets/segmented_control.rs:249-255` and `310-319`; `src/widgets/tab_bar.rs:183-196`; `src/widgets/sidebar_nav.rs:167-180`
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

> **DO NOT FIX without Chris’s decision.  Both a code-side and a doc-side resolution exist; the sections below present the options and a recommendation.**

## Description

The 2026-02-03 unified selection API gave five widgets the same method names, but the methods disagree on what an invalid argument means.

**Out-of-range index.**  `Dropdown::set_selected_index` (`dropdown.rs:158-167`) **clamps** to the last item and emits `Change`:

```rust
let index = index.min(self.choices.len().saturating_sub(1));
```

`Dropdown::with_selected_index` (`dropdown.rs:117-120`) clamps the same way.  `RadioGroup` (`radio_group.rs:195-201`, `247-256`), `SegmentedControl` (`segmented_control.rs:249-255`, `310-319`), `TabBar` (`tab_bar.rs:191-196`), and `SidebarNav` (`sidebar_nav.rs:175-180`) all **silently ignore** an out-of-range index instead (`if let Some(item) = self.items.get(index)`).  Same method name, opposite contracts.

**Membership validation.**  `RadioGroup::set_selected` (`radio_group.rs:233-242`, guard at line 234) and `SegmentedControl::set_selected` (`segmented_control.rs:296-305`) ignore an item not in the list.  `TabBar::set_selected` (`tab_bar.rs:183-186`) and `SidebarNav::set_selected` (`sidebar_nav.rs:167-170`) assign **any** `T` unconditionally (`self.selected = tab;`).  Afterward `selected_index()` (`tab_bar.rs:171-177`) falls back to `unwrap_or(0)`—reporting index 0 while no rendered item is selected—and the next arrow-key press navigates relative to index 0.

The divergence itself is the bug; which semantic wins is the decision.

## Reproduction (test-ready)

Needs a GPUI entity context (`cx.new`); no window or dispatch.

1. `Dropdown` with `choices(["a", "b", "c"])`: call `set_selected_index(99, cx)`.
   Actual: selects `"c"` and emits `Change("c")`.
2. `RadioGroup` with three items: call `set_selected_index(99, cx)`.
   Actual: no-op, no event.  Two widgets, same call shape, opposite outcomes.
3. `TabBar::new(vec![A, B], A, cx)`, then `set_selected(C, cx)` where `C` is not in the tabs.
   Actual: `selected() == C`, `selected_index() == 0`, and no tab renders as selected.

## Suggested Fix

**Question for Chris:** when a selection setter receives an out-of-range index or a non-member item, what is the one contract all five widgets follow?

- **Option A (majority semantic — ignore invalid input).**  Change `Dropdown`’s two clamping methods (`dropdown.rs:117-120`, `158-167`) to `if index < self.choices.len() { … }`, and add membership validation to `TabBar::set_selected` and `SidebarNav::set_selected` mirroring `radio_group.rs:234`.  Document “invalid input is ignored” on all the setters.
- **Option B (document the permissiveness).**  Keep `TabBar`/`SidebarNav` accepting any `T` as a deliberate “none of the tabs selected” state, document it, and make `selected_index()` honest (return `Option<usize>`, or document the `unwrap_or(0)` fallback and fix arrow-key navigation from the phantom state).  `Dropdown`’s clamping would still need reconciling with the ignore-semantics widgets.

**Reviewer recommendation: Option A.**  Clamping manufactures a selection the caller never named and emits a `Change` for it; the unvalidated assign leaves the widget lying through `selected_index() == 0`.  Ignore-invalid is already what three of the five widgets do, so it is the smallest change with the fewest surprises.  If a real “no selection” state is wanted later, it deserves an explicit `Option`-based API rather than a permissive side door.

## Why the Fix Works

One documented contract for one method name—whichever option is chosen—means a consumer (and a doc table) can describe `set_selected_index` once, and `selected_index()` stops reporting an index that nothing renders.

## Fix Ordering / Related

- Decide before bug-0074 lands: the new `TabBar`/`SidebarNav` `with_selected_index` builders must adopt the winning semantic.
- bug-0065 adds no-op guards to these same setters; landing this decision first avoids editing the guards twice.
- bug-0002 (`CheckboxGroup` phantom selection) should follow the same membership stance chosen here.
