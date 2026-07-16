# bug-0077: `set_value` emits Change and destroys cursor/selection even when the value is unchanged

**Severity:** medium
**Fix side:** CODE
**Component:** `src/widgets/text_input.rs:392-397` (`TextInput::set_value`); `src/widgets/password_input.rs:223-228` (`set_value`), `231-238` (`set_value_secret`)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

`CLAUDE.md`’s Widget API Pattern spec guards the setter — `if self.value != value { ... emit ... }`.  None of the three text setters do:

```rust
// src/widgets/text_input.rs:392-397
pub fn set_value(&mut self, value: &str, cx: &mut Context<Self>) {
    self.core.set_content(value);
    self.scroll_offset = 0.0;
    cx.emit(TextInputEvent::Change);
    cx.notify();
}
```

`PasswordInput::set_value` (`password_input.rs:223-228`) and `set_value_secret` (`231-238`) are the same shape.  Two harms:

1. **Echo loops never terminate.**  A consumer that echoes the value back from its `Change` handler (`Change` → `set_value(same)`) creates an unterminated `Change` ping-pong; the spec’s equality guard exists precisely to break that cycle.  This is the enabling defect behind ColorSwatch’s echo loop (bug-0008).
2. **Editing state is destroyed by a no-op set.**  Even a single redundant echo resets the cursor to the end, clears the selection, and zeroes `scroll_offset` while the user is typing mid-string, because `set_content` unconditionally moves the cursor.

## Reproduction (test-ready)

Needs a GPUI entity context for event observation; no window or render machinery:

1. Subscribe to the input’s events with a counter.  Call `input.update(cx, |i, cx| i.set_value("abc", cx));` twice.
2. Expected: the second call is silent (value unchanged).  Actual: a `Change` event fires both times.

Editing-state variant (drive `core` directly in a unit test, or type via GPUI window dispatch):

1. Content `abcd`, cursor moved to position 2 (arrow-left twice).
2. A `Change` subscriber calls `set_value` with the identical content.
3. Expected: cursor stays at position 2.  Actual: cursor jumps to the end and any selection is gone.

## Suggested Fix

Early-return before mutating, in all three setters:

```rust
pub fn set_value(&mut self, value: &str, cx: &mut Context<Self>) {
    if self.core.content() == value {
        return;
    }
    self.core.set_content(value);
    self.scroll_offset = 0.0;
    cx.emit(TextInputEvent::Change);
    cx.notify();
}
```

For `set_value_secret`, compare against `secret.expose_secret()` before mutating.

## Why the Fix Works

It matches the spec’s documented setter pattern, terminates echo loops after one round trip, and leaves cursor/selection/scroll untouched on no-op sets.

## Fix Ordering / Related

This is the enabling defect behind bug-0008’s echo loop in ColorSwatch — bug-0008’s reentrancy guard and this equality guard defend the same cycle from opposite ends; land this with or before that pass (bug-0008 in turn gates bug-0007, bug-0011, and bug-0013).  Same phantom-`Change` class as bug-0044 (unguarded paste) and bug-0065 (selection widgets).
