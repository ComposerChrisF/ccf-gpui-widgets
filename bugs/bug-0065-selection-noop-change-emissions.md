# bug-0065: Several selection paths emit `Change` when the selection did not change

**Severity:** low
**Fix side:** CODE
**Component:** `src/widgets/checkbox_group.rs:113-117`; `src/widgets/dropdown.rs:391-397`; `src/widgets/radio_group.rs:373-379`; `src/widgets/tab_bar.rs:333-345` and `217-258`; `src/widgets/sidebar_nav.rs:301-307` and `201-242`
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

One contract, several sites: the library’s own convention is that a `Change` event fires only when the value actually changes.  The guarded implementations establish the contract—`Checkbox::set_checked` (`checkbox.rs:103-109`), `ToggleSwitch::set_on` (`toggle_switch.rs:123-129`), `Dropdown::set_selected_index` (`dropdown.rs:158-167`), `RadioGroup::set_selected` (`radio_group.rs:233-242`), `SegmentedControl::set_selected` (`segmented_control.rs:296-305`), `Collapsible::set_collapsed` (`collapsible.rs:153-159`), and `RadioGroup`’s keyboard path `select_by_index` (`radio_group.rs:276-283`) all compare before emitting.  The following sites do not:

1. **`CheckboxGroup::set_selected`** (`checkbox_group.rs:113-117`)—the only sibling setter with no guard at all:

   ```rust
   pub fn set_selected(&mut self, selected: Vec<String>, cx: &mut Context<Self>) {
       self.selected = selected.into_iter().collect();
       cx.emit(CheckboxGroupEvent::Change(self.get_selected()));
       cx.notify();
   }
   ```

2. **`Dropdown` menu-item mouse-down** (`dropdown.rs:391-397`)—clicking the _already selected_ choice emits `Change` (plus the expected `Close`); the guarded path is `set_selected_index` (`dropdown.rs:158-167`), which the mouse handler bypasses:

   ```rust
   cx.listener(move |dropdown, _event, _window, cx| {
       dropdown.selected_index = i;
       dropdown.is_open = false;
       cx.emit(DropdownEvent::Change(choice_clone.clone()));
       cx.emit(DropdownEvent::Close);
       cx.notify();
   }),
   ```

3. **`RadioGroup` item click** (`radio_group.rs:373-379`)—clicking the already-selected radio emits `Change`, whereas the widget’s own keyboard path `select_by_index` (`radio_group.rs:276-283`) explicitly guards `if self.selected != *item`.

4. **`TabBar`/`SidebarNav` re-click** (`tab_bar.rs:333-345`, `sidebar_nav.rs:301-307`)—clicking the active tab/item re-emits `Change`.

5. **`TabBar`/`SidebarNav` single-item arrow wrap** (`tab_bar.rs:217-236` and `239-258`, `sidebar_nav.rs:201-220` and `223-242`)—with exactly one tab/item, Left/Right (Up/Down) arrows emit `Change` for the unchanged selection: `current_index >= len - 1` wraps to `new_index == 0 == current_index`, and the handler emits anyway.

A consumer echoing `Change` back into a setter (common two-way binding) gets an extra event per cycle from site 1; sites 2–5 make mouse and keyboard interaction observably inconsistent with the widgets’ own guarded paths.

## Reproduction (test-ready)

Site 1 needs only a GPUI entity context plus a subscription; sites 2–5 need GPUI test context (window plus mouse/key dispatch).

1. `CheckboxGroup`: subscribe, then call `group.set_selected(vec!["Red".into()], cx)` twice.
   Actual: two `Change(["Red"])` events.  Expected: one.
2. `RadioGroup` with `choices(["A", "B"])`, `"A"` selected: click item `"A"`.
   Actual: `Change(A)` emitted.  Expected (matching the widget’s own keyboard path): no event.
3. `Dropdown`: open the menu, click the currently selected row.
   Actual: `Change` and `Close` both emitted.  Expected: `Close` only.
4. `TabBar` with a single tab, focused: press Right.
   Actual: `Change` emitted for the unchanged selection.  Expected: no event.

## Suggested Fix

Per-site checklist:

- [ ] `checkbox_group.rs:113-117`—build the new `HashSet`, compare with `self.selected`, and only assign/emit/notify on inequality.
- [ ] `dropdown.rs:391-397`—wrap the assignment and `Change` emission in `if dropdown.selected_index != i { … }`; keep `is_open = false` and the `Close` emission unconditional.
- [ ] `radio_group.rs:373-379`—wrap the assignment and emission in `if radio_group.selected != item_clone { … }`; keep the focus and `highlight_index` updates unconditional.
- [ ] `tab_bar.rs:217-236` and `239-258` (`select_previous`/`select_next`)—return early when `new_index == current_index` (equivalently, when `len < 2`).
- [ ] `sidebar_nav.rs:201-220` and `223-242`—same early return.
- [ ] `tab_bar.rs:333-345` and `sidebar_nav.rs:301-307` (re-click of the active tab/item)—guard the emission on inequality.  Whether clicking the active tab should re-emit is a design choice; if the re-emit is kept deliberately, document it on `TabBarEvent`/`SidebarNavEvent` instead of guarding.

## Why the Fix Works

Each site adopts the identical compare-before-emit guard the library already uses everywhere else, so `Change` regains a single meaning—the value changed—across setters, keyboard paths, and mouse paths.

## Fix Ordering / Related

- bug-0002 edits the same `CheckboxGroup::set_selected` (membership filtering); coordinate the two changes.
- bug-0066 is the decision on out-of-range/membership semantics in these same setters; landing it first avoids editing the guards twice.
- bug-0073 rewrites the same `TabBar` click handler (focus-steal fix); coordinate the `TabBar` edits.
