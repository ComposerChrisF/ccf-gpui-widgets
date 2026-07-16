# bug-0072: `StringItem::id()` collides for distinct values, violating its own uniqueness contract

**Severity:** low
**Fix side:** CODE
**Component:** `src/widgets/selection.rs:109-113`
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

The `SelectionItem` trait requires uniqueness (`selection.rs:54`: `/// A unique element ID for this item (used for click handling)`), but the implementation is not injective:

```rust
// src/widgets/selection.rs:109-113
fn id(&self) -> ElementId {
    // Create stable ID from value - lowercase with underscores
    let id_str = format!("item_{}", self.value.to_lowercase().replace(' ', "_"));
    ElementId::Name(id_str.into())
}
```

`StringItem::new("Foo Bar")`, `StringItem::new("foo bar")`, and `StringItem::new("foo_bar")` all yield `item_foo_bar`.  Two such items in one `RadioGroup`, `SegmentedControl`, or `TabBar` render sibling elements with identical `ElementId`s, so GPUI element state (hover/click/active tracking) is shared between them.

## Reproduction (test-ready)

Plain unit test (no GPUI machinery):

```rust
let a = StringItem::new("A B");
let b = StringItem::new("a_b");
assert_ne!(a.id(), b.id()); // fails today: both are ElementId::Name("item_a_b")
```

Widget-level (needs GPUI test context to observe): `RadioGroup::new(vec![StringItem::new("A B"), StringItem::new("a_b")], ...)` — the two options share hover/click element state.

## Suggested Fix

Drop the normalization at `selection.rs:109-113`:

```rust
fn id(&self) -> ElementId {
    ElementId::Name(format!("item_{}", self.value).into())
}
```

`ElementId::Name` accepts arbitrary strings, including spaces and mixed case.

## Why the Fix Works

The identity function on distinct values is injective; the lowercasing and space replacement served no requirement of `ElementId`, so removing them restores uniqueness with no loss.

## Fix Ordering / Related

None — independent.
