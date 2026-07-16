# bug-0074: TODO claims universal `with_selected_index()`; TabBar and SidebarNav do not have it

**Severity:** low
**Fix side:** CODE
**Component:** `TODO.md:328` vs `src/widgets/tab_bar.rs:125-259` and `src/widgets/sidebar_nav.rs:110-243`
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

`TODO.md:328` (Unified Selection API arc, 2026-02-03) claims: “Added universal index-based selection: `selected_index()`, `set_selected_index()`, `with_selected_index()`.”  `RadioGroup` (`radio_group.rs:195`) and `SegmentedControl` (`segmented_control.rs:249`) have all three.  `TabBar` (`tab_bar.rs:125-259`) and `SidebarNav` (`sidebar_nav.rs:110-243`) have only the first two—there is no `with_selected_index` builder in either; initial selection goes through the constructor (`new(tabs, selected, cx)` / `new(items, selected, cx)`).  The claimed uniformity does not hold.

## Reproduction (test-ready)

Compile-time check—no GPUI runtime needed:

```rust
let bar = TabBar::new(tabs, sel, cx).with_selected_index(1);
```

Actual: compile error—no method named `with_selected_index` on `TabBar<T>`.  Expected per `TODO.md:328`: compiles, selecting the tab at index 1.

## Suggested Fix

**Recommended:** add the trivial builder to both widgets—about four lines each, validating the index exactly like their `set_selected_index` (`tab_bar.rs:191-196`, `sidebar_nav.rs:175-180`) and emitting nothing:

```rust
/// Set selected by index (builder pattern)
#[must_use]
pub fn with_selected_index(mut self, index: usize) -> Self {
    if let Some(tab) = self.tabs.get(index).cloned() {
        self.selected = tab;
    }
    self
}
```

(Same shape in `sidebar_nav.rs` with `items`/`item`.)

**Alternative:** reword the `TODO.md`/`DONE.md` claim to scope “universal” to `RadioGroup` and `SegmentedControl`.  This costs nothing in code but leaves the API asymmetric across the four `SelectionItem` widgets.

The reviewer recommends adding the builders: it restores the claimed uniformity, and the constructor-only path forces consumers who only know an index to do the `tabs[i].clone()` dance themselves.

## Why the Fix Works

Both widgets gain the same builder their siblings already have, with the same validation semantics as their own `set_selected_index`, making the documented claim true.

## Fix Ordering / Related

- bug-0066 (out-of-range semantics decision) should land first: the new builders must adopt whichever invalid-index semantic wins there (the sketch above uses the current ignore-invalid majority).
- bug-0078 (`TODO.md` stale references) is the home for the rewording if the alternative is chosen instead.
