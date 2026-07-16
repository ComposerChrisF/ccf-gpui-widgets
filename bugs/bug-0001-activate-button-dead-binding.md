# bug-0001: `ccf_button::ActivateButton` keybinding is registered but the action is never handled

**Severity:** low
**Fix side:** CODE
**Component:** `src/widgets/button.rs:29-43`; registration wired at `src/widgets/mod.rs:130`
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

`src/widgets/button.rs:29-43` defines and binds an action:

```rust
// Actions for button activation
actions!(ccf_button, [ActivateButton]);

pub fn register_keybindings(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("enter", ActivateButton, Some("CcfButton")),
        KeyBinding::new("space", ActivateButton, Some("CcfButton")),
    ]);
}
```

`register_keybindings` is wired into `register_all_keybindings` via `src/widgets/mod.rs:130`, so every consumer app registers these bindings.  But a crate-wide grep shows **no** `on_action` handler for `ccf_button::ActivateButton` anywhere—the button factories handle only `FocusNext`/`FocusPrev` (`src/widgets/button.rs:75-80`, `132-137`, `202-207`).  The type is not re-exported either (`src/widgets/mod.rs:46-49`), so consumers cannot handle it themselves.

Keyboard activation of these buttons actually works solely through gpui’s built-in Enter/Space keyup click (gpui `div.rs:2178-2207`); the dispatched action goes unhandled and gpui falls through harmlessly (verified: unhandled bindings continue to `finish_dispatch_key_event`, gpui `window.rs:3835-3848`).

This also makes the comment at `src/widgets/focus_navigation.rs:107-111` (`GPUI fires BOTH: 1. on_action() - from the ActivateButton keybinding ...`) wrong as applied to `CcfButton` buttons—it is accurate only for `CcfRepeatableButton`.

## Reproduction (test-ready)

No user-visible misbehavior today; the evidence is a grep:

```bash
grep -rn "on_action" src/ | grep "ActivateButton"
```

The only handlers found are for `ccf_repeatable_text_input::ActivateButton` and the picker variants; nothing handles `ccf_button::ActivateButton`.

## Suggested Fix

Delete the `ccf_button` action, its two bindings (the whole of `src/widgets/button.rs:29-43`), and the `register_button_keybindings` call at `src/widgets/mod.rs:130` (plus its re-export at `mod.rs:47`)—or wire the action to a real handler if a non-click activation path is wanted.  Deleting is safe because activation already rides gpui’s built-in keyboard click.  While there, correct the `focus_navigation.rs:107-111` comment so it claims the double-fire only for `CcfRepeatableButton`.

## Why the Fix Works

It removes a binding that promises behavior no code delivers, eliminating the trap for future maintainers who assume the action does the activating.

## Fix Ordering / Related

No hard ordering.  bug-0021 modifies the same factories in `src/widgets/button.rs`—landing the two together avoids churn.
