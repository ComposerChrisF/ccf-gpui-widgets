# bug-0075: Disabled TextInput stays in the tab order and traps keyboard focus

**Severity:** high
**Fix side:** CODE
**Component:** `src/widgets/text_input.rs:212` (handle creation), `914-915` (render `track_focus`/`tab_stop`), `1054-1065` (`FocusNext`/`FocusPrev` handlers), `1067-1070` (`on_key_down` enabled guard)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

The spec (`TODO.md:185`, 2026-02-01) says disabled widgets are “removed from tab order, interactions blocked”.  `TextInput` fails both halves.

**Half 1: the disabled input remains a tab stop.**  The handle is created as `focus_handle: cx.focus_handle().tab_stop(true)` (`text_input.rs:212`), and render does:

```rust
// src/widgets/text_input.rs:911-915
div()
    .id("ccf_text_input")
    .key_context("CcfTextInput")
    .track_focus(&focus_handle)
    .tab_stop(enabled)
```

Per the repo’s own `docs/TAB_STOP_SOLUTIONS.md` (line 93: `.tab_stop(true)  // Does NOT work with track_focus()`) and the vendored gpui 0.2.2 source (`div.rs:1584-1599`: the div-level `tab_stop` is only applied when `tracked_focus_handle.is_none()`; `tab_stop.rs:85` reads `focus_handle.tab_stop` from the tracked handle), the div-level `.tab_stop(enabled)` is a no-op whenever `track_focus` is present.  The handle’s flag stays `true` from construction, so the disabled input remains a tab stop.

**Half 2: once focused, focus is trapped.**  Both Tab escape routes are blocked when disabled:

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

gpui sets `propagate_event = false` **before** invoking a bubble-phase action listener (vendored `window.rs:4056`: `Actions stop propagation by default`), so the early return still consumes the Tab action without navigating.  The `on_key_down` fallback checks `if !this.enabled { return; }` (`text_input.rs:1068`) before its tab-handling branch, so it cannot help either.  The user can only escape with the mouse.

`PasswordInput` does both parts correctly: it sets the flag on the handle each render — `self.input_focus_handle.clone().tab_stop(enabled)` (`password_input.rs:649-650`), effective because `FocusHandle::tab_stop` writes into the shared `FocusMap` (gpui `window.rs:323-329`) — and it runs `handle_tab_navigation(event, window)` **before** the enabled check (`password_input.rs:905-908`).

## Reproduction (test-ready)

Needs GPUI test context (three widgets, keybinding registration, Tab dispatch):

1. Build a view with `TextInput::new(cx).with_enabled(false)` between two enabled widgets; call `register_all_keybindings(cx)`.
2. Press Tab from the first widget.  Expected: the disabled input is skipped.  Actual: focus lands on it.
3. Press Tab again.  Expected: focus moves on.  Actual: nothing happens — focus is trapped, and only the mouse can escape.
4. Alternate repro for the trap alone: focus an enabled `TextInput`, call `set_enabled(false, cx)`, press Tab — same trap.

## Suggested Fix

In `render`, mirror `PasswordInput`:

```rust
let focus_handle = self.focus_handle.clone().tab_stop(enabled);
```

and drop the no-op div-level `.tab_stop(enabled)` (`text_input.rs:915`).  Additionally, move the tab branch in `on_key_down` (`text_input.rs:1072-1089`) above the enabled guard — or call `handle_tab_navigation` first, as `PasswordInput` does — and remove (or make tab-navigation-only) the enabled guards in the `FocusNext`/`FocusPrev` handlers (`1054-1065`), so Tab always navigates even while the widget is disabled.

## Why the Fix Works

Setting the flag on the handle updates the shared `FocusMap`, which is what `TabStopMap::insert` actually reads, so disabled inputs get skipped; letting Tab handling run when disabled removes the trap for the programmatic-disable case, where focus is already inside the now-disabled widget.

## Fix Ordering / Related

The fix should match the corrected widget template from bug-0003 (handle-level `tab_stop`; the current `CLAUDE.md` template omits it entirely).  bug-0024 modifies the same `FocusNext`/`FocusPrev` handlers (`emit_tab_events` branch) — coordinate the two edits in one pass.  bug-0022 (disabled widgets steal focus) is the same policy family across other widgets.
