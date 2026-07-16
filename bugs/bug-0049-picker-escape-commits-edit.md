# bug-0049: ESC in picker edit mode commits the abandoned text instead of canceling

**Severity:** high
**Fix side:** CODE
**Component:** `src/widgets/file_picker.rs:559-584` (commit arm) and `src/widgets/file_picker.rs:657-661` (refocus); identical in `src/widgets/directory_picker.rs:343-368` and `src/widgets/directory_picker.rs:429-433`
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

Pressing Escape while editing a `FilePicker` or `DirectoryPicker` path is supposed to cancel the edit.  Instead, the abandoned text is committed—with a `FilePickerEvent::Change` / `DirectoryPickerEvent::Change` emission—because the Escape path itself triggers the `Blur` that the commit arm listens for.

The `TextInput` subscription created in `start_editing` commits on `Enter | Blur` unconditionally and cancels on `Escape` (`src/widgets/file_picker.rs:559-584`):

```rust
TextInputEvent::Enter | TextInputEvent::Blur => {
    this.is_editing = false;
    let text = edit_state.read(cx).content().to_string();
    let path_info = parse_path(&text);
    let new_value = path_info.full_path_string();
    if this.value != new_value {
        this.value = new_value;
        cx.emit(FilePickerEvent::Change(this.value.clone()));
    }
    cx.notify();
}
TextInputEvent::Escape => {
    // Cancel editing and refocus the picker
    this.is_editing = false;
    this.pending_refocus = true;
    cx.notify();
}
```

But `TextInput` emits `Blur` from a `cx.on_focus_out` subscription whenever its focus handle loses focus (`src/widgets/text_input.rs:820-828`, dispatching to `on_blur` at `src/widgets/text_input.rs:469-476`), and the Escape path itself forces exactly that focus loss: `pending_refocus` makes the next render call `self.edit_button_focus_handle.focus(window)` (`src/widgets/file_picker.rs:657-661`), moving focus off the still-alive `edit_state`—which is never cleared, and whose content is never reset.  The resulting `Blur` event runs the commit arm with the edited text.  Net effect: ESC commits instead of canceling, and emits a spurious `Change`.

The bug hides whenever ESC is pressed without modifying the text: then `new_value == this.value`, so nothing is emitted and nothing visibly changes.  That is presumably how the `TODO.md:81` item (“ESC key returns focus to picker”) was verified as working.

`DirectoryPicker` has the identical structure at `src/widgets/directory_picker.rs:343-368` (subscription) and `src/widgets/directory_picker.rs:429-433` (refocus).

## Reproduction (test-ready)

Needs GPUI test context (window, focus, and event dispatch).

1. Create a `FilePicker` with value `/tmp/a.txt`; subscribe to `FilePickerEvent`.
2. Enter edit mode (click the field, or invoke the edit path so `start_editing` runs).
3. Type so the `TextInput` content becomes `/tmp/b.txt`.
4. Dispatch the `Escape` action to the `TextInput`.
5. Pump two frames: the refocus happens on the next render, and the focus-out dispatch produces `TextInputEvent::Blur` after that.

Expected: `value()` is still `/tmp/a.txt`; no `Change` event is emitted.
Actual: after the refocus frame, `TextInputEvent::Blur` fires, `value()` becomes `/private/tmp/b.txt` (the parsed and canonicalized form), and `FilePickerEvent::Change` is emitted.

Repeat for `DirectoryPicker` with a directory path.

## Suggested Fix

Guard the commit arm on `is_editing`, in both pickers:

```rust
TextInputEvent::Enter | TextInputEvent::Blur => {
    if !this.is_editing {
        return;
    }
    this.is_editing = false;
    // ... rest unchanged ...
}
```

Optionally also set `this.edit_state = None` in the `Escape` arm so a canceled editor can never be read again.

## Why the Fix Works

`is_editing` is `true` exactly while a commit is legitimate (Enter while editing, click-away Blur while editing); ESC sets it `false` before the refocus-induced `Blur` arrives, so the stale `Blur` becomes a no-op.  The same guard also suppresses the redundant second commit that currently follows Enter, when the subsequent focus loss emits `Blur` again.

## Fix Ordering / Related

Fix in the same pass as bug-0052 (the render-time focus-loss fallback in the same two files).  Both are commit paths through the same machinery; with this guard in place, whenever the render fallback runs first it sets `is_editing = false` and the later `Blur` commit is suppressed entirely, so bug-0052’s missing `Change` emission must be fixed together with this guard for subscribers to be notified at all.
