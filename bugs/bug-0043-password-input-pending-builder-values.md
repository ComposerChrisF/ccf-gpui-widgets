# bug-0043: PasswordInput builder values deferred to first render — `value()` lies, pre-render `set_value` is silently reverted

**Severity:** medium
**Fix side:** CODE
**Component:** `src/widgets/password_input.rs:172-175` (`with_value`), `192-199` (`apply_pending`), `644-645` (call site in `render`); also `165-168` (`placeholder`)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

`with_value` only stores the value for later:

```rust
// src/widgets/password_input.rs:170-175
#[must_use]
pub fn with_value(mut self, value: impl Into<String>) -> Self {
    self.pending_value = Some(value.into());
    self
}
```

The content is actually applied by `apply_pending()` (`password_input.rs:192-199`) at the top of the first `render()` (`password_input.rs:644-645`).  `placeholder` works the same way via `pending_placeholder`.  Two consequences:

1. **The getter lies before first render.**  `cx.new(|cx| PasswordInput::new(cx).with_value("secret")).read(cx).value(cx)` returns the empty string until the widget has rendered once.
2. **A pre-render `set_value` is silently reverted.**  If the consumer calls `set_value("b", cx)` after construction but before the first render, `apply_pending` then executes `self.core.set_content(&"a")` — reverting to the builder value with **no `Change` event** — desynchronizing the consumer’s model from the widget.

The deferral buys nothing: `TextInput::with_value` (`text_input.rs:241-245`) applies immediately via `self.core.set_content(&text)`, and nothing in `EditingCore`/`SensitiveString` needs a window or a render pass.

## Reproduction (test-ready)

Needs a GPUI entity context (e.g. a test `App`); no window or render machinery except where noted:

1. `let p = cx.new(|cx| PasswordInput::new(cx).with_value("a"));`
2. Getter lie, observable immediately: `p.read(cx).value(cx)` (with `secure-password`: `.expose_secret()`) returns `""` — expected `"a"`.
3. `p.update(cx, |p, cx| p.set_value("b", cx));` — still before any render.  The value now reads `"b"`.
4. Render the widget once (needs GPUI window/render).  Now `p.read(cx).value(cx)` returns `"a"` — expected `"b"` — and no `Change` event accompanied the reversion.

## Suggested Fix

Apply values directly in the builders and delete the deferral machinery:

- `with_value` (`password_input.rs:172-175`): `self.core.set_content(&value.into());`
- `placeholder` (`password_input.rs:165-168`): `self.placeholder = Some(text.into());`
- Delete the `pending_value` and `pending_placeholder` fields, `apply_pending` (`192-199`), and its call site in `render` (`644-645`).

This is exactly `TextInput`’s existing shape (`text_input.rs:234-245`).

## Why the Fix Works

Builder state becomes immediately observable and there is no later overwrite step, so no window of time exists in which getters lie or setters get clobbered.

## Fix Ordering / Related

None — independent.  (bug-0042 concerns the same widget’s event surface but different machinery.)
