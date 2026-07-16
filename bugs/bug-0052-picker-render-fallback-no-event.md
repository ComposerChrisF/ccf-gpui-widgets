# bug-0052: Render-time focus-loss fallback commits the edited text without emitting `Change`

**Severity:** medium
**Fix side:** CODE
**Component:** `src/widgets/file_picker.rs:671-680`; `src/widgets/directory_picker.rs:443-452`
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

Both pickers carry a render-time fallback for “focus lost during editing” (`src/widgets/file_picker.rs:671-680`; identical at `src/widgets/directory_picker.rs:443-452`):

```rust
// Handle focus lost during editing
if self.is_editing {
    if let Some(edit_state) = &self.edit_state {
        if !edit_state.read(cx).focus_handle().is_focused(window) {
            self.is_editing = false;
            let text = edit_state.read(cx).content().to_string();
            let path_info = parse_path(&text);
            self.value = path_info.full_path_string();
        }
    }
}
```

It mutates `self.value` with **no** `FilePickerEvent::Change` / `DirectoryPickerEvent::Change`—violating the project convention that value changes emit their `Change` event (and `CLAUDE.md`: emit events so subscribers see updated state).

Worse, it poisons the later `Blur` handler: when `TextInputEvent::Blur` does arrive for the same focus loss, the commit arm computes the same `new_value`, finds `this.value != new_value` now `false`, and never emits.  So the event is not merely late—it is lost entirely.  The parent `RepeatableFilePicker`, whose `Change(Vec<String>)` is driven solely by the child’s `Change` event (`src/widgets/repeatable_file_picker.rs:278-283`), and every other subscriber silently miss the new value.

Reachability caveat: hitting this fallback requires a render to land between the focus change and the focus-out dispatch (e.g. another entity’s `cx.notify()` scheduling a frame in that window).  Confirming that ordering needs a GPUI dynamic test; the convention violation itself is fully traced in source.

## Reproduction (test-ready)

Needs GPUI test context (window, focus, frame pumping).

1. Create a `FilePicker`; subscribe to `FilePickerEvent`.  Enter edit mode and modify the text.
2. Force a render in which the `TextInput`’s focus handle is no longer focused but its `Blur` event has not yet been processed (e.g. move window focus and trigger `cx.notify()` on another entity before the focus-out dispatch runs).
3. Observe `value()` changed with no `Change` emitted.
4. Let the `Blur` event arrive: it also emits nothing, because the values now compare equal.

Expected: exactly one `Change` carrying the committed value.
Actual: zero `Change` events; subscribers hold a stale value.

## Suggested Fix

Make the fallback mirror the subscription arm—compare, assign, and emit when different (emitting from render’s `Context<Self>` is legal):

```rust
let new_value = path_info.full_path_string();
if self.value != new_value {
    self.value = new_value;
    cx.emit(FilePickerEvent::Change(self.value.clone()));
}
```

Alternatively, drop the fallback entirely and rely on the `on_focus_out`-driven `Blur`, which is guaranteed to fire.

## Why the Fix Works

Every path that mutates `value` then emits the same event, so subscribers can never observe a stale value regardless of which of the two commit paths wins the race.

## Fix Ordering / Related

Fix in the same pass as bug-0049.  Once bug-0049 guards the `Blur` commit arm on `is_editing`, this fallback (which clears `is_editing`) suppresses the later `Blur` commit entirely, so the fallback’s own emission becomes the only notification path—the two changes must land together so both commit paths carry the same compare-assign-emit shape.
