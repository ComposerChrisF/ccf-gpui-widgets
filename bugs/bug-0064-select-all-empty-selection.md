# bug-0064: `select_all()` on empty content leaves a zero-width `Some((0, 0))` selection

**Severity:** low
**Fix side:** CODE
**Component:** `src/widgets/editing_core.rs:418-422` (`select_all`); consumer effect at `src/widgets/text_input.rs:1109-1113` (mouse-down restore path)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

Everywhere else in `EditingCore`, an empty selection is normalized to `None` — both `update_selection` and `set_selection` (`editing_core.rs:425-434`) collapse `start == end` to `None`.  `select_all` does not:

```rust
// src/widgets/editing_core.rs:417-422
/// Select all text
pub fn select_all(&mut self) {
    self.selection_anchor = Some(0);
    self.cursor = self.content.len();
    self.selection = Some((0, self.content.len()));
}
```

With empty content this is `Some((0, 0))`, and nothing clears it: `delete_selection` (`editing_core.rs:200-210`) and `insert_text` skip zero-width ranges but leave `selection` set.  The stale value then persists across subsequent typing.

Consumer effect: `TextInput`’s mouse-down restore path keys off `this.core.selection().is_some()` (`text_input.rs:1109-1113` — “if clicking to restore focus and there’s a selection to restore, just restore focus without changing cursor/selection”).  After cmd-A on an empty input, blur, then click-to-refocus, the click’s cursor positioning is swallowed — even after the user has typed content in between, because the stale `Some((0, 0))` survives `insert_text`.  (`PasswordInput` has the same restore path at `password_input.rs:920-928`.)

## Reproduction (test-ready)

Plain unit test (no GPUI):

```rust
let mut core = EditingCore::new();
core.select_all();
assert_eq!(core.selection(), None); // fails today: Some((0, 0))
core.insert_text("hello");
assert_eq!(core.selection(), None); // also fails: the stale Some((0, 0)) survives
```

Widget-level (needs GPUI test context): focus an empty `TextInput`, press cmd-A, type `hello`, Tab away, click in the middle of the text.  Expected: cursor lands at the click position.  Actual: the first click only restores focus (the restore path is taken); a second click is needed.

## Suggested Fix

In `select_all` (`editing_core.rs:418-422`), normalize the empty case the same way `set_selection` does:

```rust
pub fn select_all(&mut self) {
    if self.content.is_empty() {
        self.selection = None;
        self.selection_anchor = None;
        self.cursor = 0;
    } else {
        self.selection_anchor = Some(0);
        self.cursor = self.content.len();
        self.selection = Some((0, self.content.len()));
    }
}
```

## Why the Fix Works

It restores the empty-selection-means-`None` invariant that the rest of the state machine assumes, so the restore-selection heuristic only triggers when there is actually something to restore.

## Fix Ordering / Related

None — independent.
