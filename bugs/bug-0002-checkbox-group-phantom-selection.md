# bug-0002: CheckboxGroup silently retains selected values that are not in `choices`

**Severity:** low
**Fix side:** CODE
**Component:** `src/widgets/checkbox_group.rs:81-84` (`with_selected`), `src/widgets/checkbox_group.rs:113-117` (`set_selected`), `src/widgets/checkbox_group.rs:101-105` (`get_selected`)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

Both selection entry points accept arbitrary strings without checking membership in `self.choices`:

```rust
// with_selected (checkbox_group.rs:81-84)
pub fn with_selected(mut self, selected: Vec<String>) -> Self {
    self.selected = selected.into_iter().collect();
    self
}

// set_selected (checkbox_group.rs:113-117)
pub fn set_selected(&mut self, selected: Vec<String>, cx: &mut Context<Self>) {
    self.selected = selected.into_iter().collect();
    cx.emit(CheckboxGroupEvent::Change(self.get_selected()));
    cx.notify();
}
```

A value absent from `choices` is never rendered (the render loop iterates `choices`), yet it is reported forever by `get_selected()` (`checkbox_group.rs:101-105`), by `is_selected()`, and in every `Change` payload—`toggle_choice` (`checkbox_group.rs:137-144`) emits `Change(self.get_selected())`, which carries the phantom value along.  `RadioGroup`, by contrast, validates membership before accepting a selection (`radio_group.rs:129` in `with_selected_value`, `radio_group.rs:140` in `set_selected_value`, `radio_group.rs:234` in `set_selected`).  The UI and the data model disagree invisibly: no checkbox appears checked, but the widget claims a selection.

## Reproduction (test-ready)

Steps 1–3 need only a GPUI entity context (`cx.new`) to construct the widget—no window or event dispatch.  Step 4 needs a subscription (still entity-level, no window).

1. Build the widget:

   ```rust
   let group = cx.new(|cx| {
       CheckboxGroup::new(cx)
           .choices(vec!["Red".into(), "Blue".into()])
           .with_selected(vec!["Bogus".into()])
   });
   ```

2. Call `get_selected()`.
   Actual: `["Bogus"]`.  Expected: `[]` (phantom value rejected or excluded from reports).
3. Call `is_selected("Bogus")`.
   Actual: `true`.  Expected: `false`.
4. Subscribe to the widget, then toggle `"Red"` (programmatically via `set_selected(vec!["Bogus".into(), "Red".into()], cx)`, or with a window via a space keypress on the highlighted choice—the keypress route needs GPUI test context).
   Actual: `Change(["Bogus", "Red"])`.  Expected: `Change(["Red"])`.

## Suggested Fix

In `with_selected` (`checkbox_group.rs:81-84`) and `set_selected` (`checkbox_group.rs:113-117`), retain only values present in `self.choices`:

```rust
self.selected = selected
    .into_iter()
    .filter(|v| self.choices.contains(v))
    .collect();
```

Note the builder-ordering interaction in the doc comment: `choices()` must be called before `with_selected()`, because filtering against a not-yet-populated `choices` list drops everything.

## Why the Fix Works

The event payload and the getters then describe exactly what the widget displays, matching `RadioGroup`’s validation stance—a consumer can no longer receive a `Change` payload containing a value the user cannot see or toggle.

## Fix Ordering / Related

- bug-0065 edits the same `set_selected` method (missing no-op guard); coordinate the two changes.
- bug-0066 is the cross-widget decision on out-of-range/membership semantics; if that decision lands on a stance other than ignore-invalid, this fix should follow it.
