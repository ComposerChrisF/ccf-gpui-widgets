# bug-0059: Scrollables created in a loop share one scroll state—call-site-derived `ElementId`s collide

**Severity:** low
**Fix side:** CODE
**Component:** `src/widgets/scrollable.rs:144-167` (`new_with_location` ID generation); `src/widgets/scrollable.rs:203` (`.id()` escape hatch); `src/widgets/scrollable.rs:222` (`with_element_state`); module doc at `src/widgets/scrollable.rs:105-113`
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

`new_with_location` builds the element ID solely from the caller’s file/line/column (`src/widgets/scrollable.rs:151-156`):

```rust
let id = ElementId::Name(SharedString::from(format!(
    "scrollable-{}:{}:{}",
    location.file(),
    location.line(),
    location.column()
)));
```

Any code constructing scrollables in a loop or iterator—`items.iter().map(|it| scrollable_vertical(render_item(it)))`—produces siblings with identical `ElementId`s and therefore identical `GlobalElementId`s, so `with_element_state` (`src/widgets/scrollable.rs:222`) hands them all the _same_ `ScrollViewState`: one shared `ScrollHandle` and `ScrollbarState`.  Scrolling one entangles all of them.

The `.id()` escape hatch exists (`src/widgets/scrollable.rs:203`), but neither the module doc (lines 105–113) nor the factory docs warn that per-instance IDs are required in loops.

## Reproduction (test-ready)

Needs a GPUI dynamic test to demonstrate live (element state is only materialized during draw); the ID collision itself is visible from the code.

1. In one render, build three siblings from a single `map` call site:

```rust
.children((0..3).map(|_| scrollable_vertical(tall_content()).into_any_element()))
```

2. Scroll the first list.
3. Observe the other two.

Expected: independent scroll positions.
Actual: the other two jump to the same offset (same `ScrollHandle` from the shared element state).

## Suggested Fix

Document the constraint prominently and, better, make collisions impossible:

- Strongest: require an explicit ID—make the factories take `id: impl Into<ElementId>`.  (Breaking API change; coordinate with the `clui` consumer.)
- Minimum: document “when constructing scrollables in a loop you MUST call `.id(...)` with a unique value” in the module doc (`src/widgets/scrollable.rs:105-113`) and on all three factory functions (`scrollable_vertical`, `scrollable_horizontal`, `scrollable_both`, plus the `Scrollable::vertical`/`horizontal`/`both` constructors).

## Why the Fix Works

Distinct `ElementId`s give distinct `GlobalElementId`s, so each instance gets its own `ScrollViewState`/`ScrollHandle`—the invariant the element-state design assumes.

## Fix Ordering / Related

None—independent.  Same widget family as bug-0060, bug-0061, and bug-0062 (scrollbar internals), but no shared code path.
