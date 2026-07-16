# bug-0045: PasswordInput visibility toggle double-fires on keyboard Enter/Space — net no-op

**Severity:** high
**Fix side:** CODE
**Component:** `src/widgets/password_input.rs:1093-1104` (toggle button’s `on_key_down` + `on_click`)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

The toggle button handles keyboard activation twice:

```rust
// src/widgets/password_input.rs:1093-1104
.on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
    if handle_tab_navigation(event, window) {
        return;
    }
    if matches!(event.keystroke.key.as_str(), "enter" | "space") {
        if !this.enabled { return; }
        this.toggle_visibility(cx);
    }
}))
.when(enabled, |d| d.on_click(cx.listener(|this, _event, _window, cx| {
    this.toggle_visibility(cx);
})))
```

gpui 0.2.2 synthesizes a click for focused elements with click listeners on Enter/Space **key-up** (vendored gpui `div.rs:2179-2209`: `Press enter, space to trigger click, when the element is focused`).  So one key press toggles at keydown (the `on_key_down` branch) and again at keyup (the synthetic click): visibility flips and flips back — the button appears dead to keyboard users, and holding the key flickers the masking via key-repeat.

This is exactly the double-trigger class that `focus_navigation.rs` documents in its own code (lines 101–113: `CRITICAL BUG WARNING - DO NOT REMOVE action_just_handled MECHANISM` — “pressing Space/Enter once would add/remove TWO entries!”) and that `TODO.md:176` records fixing for Collapsible (“Fixed double-toggle on Space/Enter (was responding to both keydown and synthetic click)”, 2026-02-01).  PasswordInput’s toggle was never given the fix.

## Reproduction (test-ready)

Needs GPUI test context (focus + key dispatch):

1. Tab to the toggle button; record `is_password_visible()`.
2. Press Space once.
3. Expected: `is_password_visible()` flips once.  Actual: `toggle_visibility` runs twice — once at keydown via `on_key_down`, once at keyup via the synthetic click — so the final state is unchanged.
4. Control: a mouse click toggles correctly (once).

## Suggested Fix

Delete the `"enter" | "space"` branch from the toggle’s `on_key_down` (`password_input.rs:1097-1100`) and rely solely on `on_click` — gpui’s synthetic click already provides keyboard activation for focused elements.  (The `on_click` is wrapped in `.when(enabled, ...)`, so the disabled case remains inert after the deletion.)  Alternative, if a keydown-time response is wanted: adopt the `action_just_handled` flag pattern from `focus_navigation.rs`.

## Why the Fix Works

With only one activation path — the click listener, fired once per press whether by mouse or synthesized from key-up — each press toggles exactly once.

## Fix Ordering / Related

Same defect class as bug-0016 (dialog Enter double-fire, currently latent); the fixes are independent.
