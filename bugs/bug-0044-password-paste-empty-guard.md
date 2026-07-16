# bug-0044: PasswordInput paste lacks the empty-text guard — deletes the selection and emits spurious Change

**Severity:** low
**Fix side:** CODE
**Component:** `src/widgets/password_input.rs:511-521` (`paste`); contrast `src/widgets/text_input.rs:446-448`; enabling defect `src/widgets/editing_core.rs:214-219` (`insert_text`)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

`TextInput`’s `paste` returns early when the cleaned/filtered text is empty (`text_input.rs:446-448`).  `PasswordInput`’s does not:

```rust
// src/widgets/password_input.rs:511-521
fn paste(&mut self, cx: &mut Context<Self>) {
    if let Some(clipboard) = cx.read_from_clipboard() {
        if let Some(text) = clipboard.text() {
            let clean_text = text.replace(['\n', '\r'], "");
            self.core.insert_text(&clean_text);
            self.reset_cursor_blink();
            self.emit_change(cx);
            cx.notify();
        }
    }
}
```

The enabling defect is `EditingCore::insert_text` (`editing_core.rs:214-219`): it first calls `delete_selection()` and then unconditionally returns `true` — its doc comment `Returns true if content changed` is wrong for empty input:

```rust
pub fn insert_text(&mut self, text: &str) -> bool {
    self.delete_selection();
    self.content.insert_str(self.cursor, text);
    self.cursor += text.len();
    true
}
```

So pasting a clipboard containing only newlines (`"\n"`) deletes the selected password text and inserts nothing; and with no selection, a `Change` event fires with identical content.

## Reproduction (test-ready)

Core-level (plain unit test, no GPUI):

1. Create an `EditingCore`, set content to `"secret"`, call `select_all()`, then `insert_text("")`.
2. Expected: content unchanged.  Actual: content is `""` (the selection was deleted), and the call returned `true`.

Widget-level (needs GPUI test context, clipboard):

1. Put `"\n"` on the clipboard.  In a `PasswordInput` containing `"secret"`, select all, dispatch the `Paste` action (cmd-v).
2. Expected (`TextInput` behavior): no-op.  Actual: content becomes `""` and `Change("")` is emitted.
3. With no selection and a clipboard that is empty after newline stripping: `Change` fires although content is unchanged.

## Suggested Fix

Add the guard after the newline strip in `password_input.rs:511-521`, mirroring `text_input.rs:446-448`:

```rust
let clean_text = text.replace(['\n', '\r'], "");
if clean_text.is_empty() {
    return;
}
```

Optionally also make `insert_text` honest (`editing_core.rs:214-219`): `let deleted = self.delete_selection(); ... deleted || !text.is_empty()` — so the return value matches its documented contract for every caller.

## Why the Fix Works

It restores parity with `TextInput`’s guarded paste path, so empty pastes can neither destroy a selection nor fire phantom `Change` events.

## Fix Ordering / Related

Same phantom-`Change` class as bug-0077 (unguarded `set_value`); the fixes are independent.
