# bug-0024: `emit_tab_events(true)` is dead under the documented keybinding setup

**Severity:** medium
**Fix side:** CODE
**Component:** `src/widgets/text_input.rs:1054-1065` (`FocusNext`/`FocusPrev` action handlers) vs. `src/widgets/text_input.rs:1072-1089` (`on_key_down` Tab logic); consumer: `src/widgets/number_stepper.rs:143, 156-159`
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

`register_all_keybindings` registers `KeyBinding::new("tab", FocusNext, None)` and `KeyBinding::new("shift-tab", FocusPrev, None)` (`src/widgets/focus_navigation.rs:27-29`).  gpui 0.2.2 dispatches matched key bindings **before** `on_key_down` listeners (vendored gpui `window.rs:3834-3848`: the bindings loop runs first, and the path that reaches `on_key_down` — `finish_dispatch_key_event` — only runs if the action propagated).  TextInput’s own action handlers consume the actions unconditionally:

```rust
// src/widgets/text_input.rs:1054-1065
.on_action(cx.listener(|this, _: &FocusNext, window, _cx| {
    if !this.enabled {
        return;
    }
    window.focus_next();
}))
.on_action(cx.listener(|this, _: &FocusPrev, window, _cx| {
    if !this.enabled {
        return;
    }
    window.focus_prev();
}))
```

The `emit_tab_events` logic lives only in `on_key_down` (`text_input.rs:1072-1089`), which never sees the Tab key while those bindings exist.  So the builder’s documented contract (`text_input.rs:308-316`: pressing Tab or Shift+Tab emits `TextInputEvent::Tab` or `TextInputEvent::ShiftTab`, “allowing the parent to handle focus navigation”) never happens in any app that calls `register_all_keybindings` — the README-documented setup.

Concrete victim in this repo: `NumberStepper` builds its embedded editor with `.emit_tab_events(true)` — the comment reads `// Let NumberStepper handle Tab` (`number_stepper.rs:143`) — and maps `TextInputEvent::Tab | TextInputEvent::ShiftTab => commit_edit` with the comment `// Treat Tab same as Enter - commit and stay on stepper` (`number_stepper.rs:156-159`).  Instead, Tab moves focus away; the commit only happens as a side effect of the `Blur` event.

## Reproduction (test-ready)

Needs GPUI test context (keybinding registration, focus, key dispatch):

1. Call `register_all_keybindings(cx)`; create `TextInput::new(cx).emit_tab_events(true)`; subscribe for `TextInputEvent`.
2. Focus the input; dispatch a Tab keystroke.
3. Expected: `TextInputEvent::Tab` is emitted and focus stays on the input.  Actual: no event is emitted and focus moves to the next widget.

## Suggested Fix

Branch on the flag in the action handlers (`text_input.rs:1054-1065`), mirroring the `on_key_down` logic:

```rust
.on_action(cx.listener(|this, _: &FocusNext, window, cx| {
    if !this.enabled {
        return;
    }
    if this.emit_tab_events {
        cx.emit(TextInputEvent::Tab);
    } else {
        window.focus_next();
    }
}))
```

and `TextInputEvent::ShiftTab` in the `FocusPrev` handler.  Keep the existing `on_key_down` branch for apps that do not register the bindings.

## Why the Fix Works

The action path is the one that actually runs under the standard setup; making it respect `emit_tab_events` restores the documented behavior on both dispatch paths.

## Fix Ordering / Related

bug-0075 modifies the same `FocusNext`/`FocusPrev` handlers (enabled-guard change for the disabled-focus trap) — coordinate the two edits in one pass.
