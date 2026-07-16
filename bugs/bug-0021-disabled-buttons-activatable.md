# bug-0021: Disabled primary/danger buttons do not block interaction—they are clickable, click-focusable, and keyboard-activatable

**Severity:** medium
**Fix side:** CODE
**Component:** `src/widgets/button.rs:69-105` (`primary_button`), `src/widgets/button.rs:196-233` (`danger_button`); doc example at `src/widgets/button.rs:12-15`
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

When `enabled == false` the factories change only styling and tab order:

```rust
.focusable()
.tab_stop(enabled) // Disabled buttons are not tab stops
// ...
.when(!enabled, |d| {
    d.bg(rgb(theme.disabled_bg))
        .text_color(rgb(theme.disabled_text))
        .cursor_default()
})
// ...
.focus(|d| d.border_color(rgb(theme.border_focus_on_color)))
```

Nothing blocks activation, and the crate’s own documented usage attaches the handler unconditionally (`primary_button("run_btn", "Run", enabled, cx).on_click(...)`, doc example at `src/widgets/button.rs:12-15`).

In gpui 0.2.2: `.focusable()` auto-creates a focus handle (gpui `div.rs:1584-1599`); any focusable element is focused on mousedown (gpui `div.rs:2025-2031`); mouse clicks fire click listeners with no enabled concept; and once focused, Enter/Space **keyup** fires click listeners too (gpui `div.rs:2178-2207`).  So a disabled button:

- fires `on_click` on a mouse click;
- grabs focus on mousedown, showing the focus ring, since the `.focus(|d| ...)` styling at lines 104 and 231 is unconditional;
- then activates on Enter/Space.

Contrast: the repeatable buttons in `src/widgets/focus_navigation.rs` get this right by gating the handlers (`if enabled { button = button.on_action(...).on_click(...) }`, lines 194–204 and 302–312).

## Reproduction (test-ready)

Needs GPUI test context (mouse and key dispatch).

1. In a view, build `primary_button("b", "Run", false, cx).on_click(cx.listener(|_, _, _, _| panic!("activated")))`.
2. Mouse-click the gray button.  Actual: the handler fires.  Expected: no activation.
3. Click it again (it takes focus and shows the focus ring), then press Space.  Actual: the handler fires again.  Expected: no focus and no activation, per the disabled-widget convention.

## Suggested Fix

In the `enabled`-aware factories (`primary_button`, `danger_button`), make focusability conditional:

```rust
.when(enabled, |d| d.focusable())
```

which also removes the focus-ring-on-click artifact.  Then either:

- **(a) recommended:** change the factory signatures to accept the click handler and attach it only when enabled, matching the repeatable-button helpers in `focus_navigation.rs` (apply to all three factories for a consistent API); or
- **(b) minimum:** document loudly that callers must gate `.on_click` on `enabled`, and fix the doc example at `src/widgets/button.rs:12-15`.

Option (a) is the robust fix; the current API cannot block a caller-attached listener.

## Why the Fix Works

Without a tracked focus handle, the mousedown focus transfer and the Enter/Space keyup click never engage; with the handler attached only when enabled, the mouse path is closed too.

## Fix Ordering / Related

No hard ordering.  bug-0001 deletes dead keybinding code in the same file—land together to avoid churn.  bug-0022 covers the disabled-widgets-and-focus principle in other widgets; its resolution (gated by the bug-0068 focus-policy decision) may share the approach chosen here.
