# bug-0022: Disabled widgets still steal focus on mouse-down

**Severity:** medium
**Fix side:** CODE
**Component:** `src/widgets/dropdown.rs:285`, `src/widgets/radio_group.rs:298`, `src/widgets/checkbox_group.rs:159`, `src/widgets/checkbox.rs:160`, `src/widgets/toggle_switch.rs:227`, `src/widgets/segmented_control.rs:361`, `src/widgets/tab_bar.rs:280`, `src/widgets/sidebar_nav.rs:264`, `src/widgets/collapsible.rs:236`
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

The disabled-state contract (`TODO.md:185`: “grayed styling, no hover effects, removed from tab order, interactions blocked”) is enforced in three places—tab order (`.tab_stop(enabled)`), key handlers (`if !self.enabled { return; }`), and click handlers (attached only inside `.when(enabled, …)`)—but `.track_focus(&focus_handle)` is unconditional in all nine widgets listed above.

Verified against vendored gpui 0.2.2: any element with a tracked focus handle registers a mouse-down listener that transfers focus to itself on every mouse-down of any button, whenever the hitbox is hovered and `!window.default_prevented()` (`div.rs:2025-2038`).  So clicking a disabled `Checkbox` (or any of the nine) silently steals focus from the currently focused widget.

Worse, the stolen focus is _invisible_: every focus ring is gated on `is_focused && enabled` (e.g. `checkbox.rs:186`), so the user sees their `TextInput` defocus with no ring appearing anywhere—and subsequent keystrokes go nowhere, because every key handler early-returns on `!enabled`.

## Reproduction (test-ready)

Needs GPUI test context (window plus mouse-event dispatch).

1. Place a `TextInput` and a `Checkbox::new(cx).with_enabled(false)` in a window.
2. Focus the `TextInput`; confirm `window.focused(cx)` is its handle.
3. Dispatch a mouse-down (and up) on the disabled checkbox.
4. Actual: `window.focused(cx)` becomes the checkbox’s handle; the `TextInput` blurs; subsequent typing is swallowed (the checkbox’s key handler early-returns on `!enabled`, and the text input is no longer focused).  No focus ring is visible anywhere.
5. Expected: the click is inert; focus stays on the `TextInput`.

The same repro applies to each of the other eight widgets.

## Suggested Fix

In each of the nine widgets, when `!enabled`, attach a mouse-down listener that calls `window.prevent_default()`:

```rust
.when(!enabled, |d| {
    d.on_any_mouse_down(|_, window, _| window.prevent_default())
})
```

This works because of gpui’s dispatch order: the auto-focus listener registers first (`div.rs:2025-2038`), user mouse-down listeners register after it (`div.rs:2040`), and the bubble phase dispatches in reverse registration order (`window.rs:3704`)—so the widget’s own listener runs first, and the auto-focus listener’s `!window.default_prevented()` check then skips the focus transfer.

Alternative considered: apply `track_focus` only when enabled.  Rejected because it changes focus-retention semantics when an already-focused widget is disabled at runtime; the `prevent_default` approach leaves the enabled path untouched.

Scope note: `TextInput`/`PasswordInput` and the button factories have their own disabled-interaction gaps tracked separately (bug-0075, bug-0021); this report covers the nine selection/navigation widgets listed above.

## Why the Fix Works

`default_prevented()` is exactly the escape hatch gpui provides for its click-to-focus default; blocking it when disabled completes the “interactions blocked” contract for the mouse as well as the keyboard, without touching enabled-path behavior.  Keyboard focusability is already handled by `.tab_stop(enabled)`.

## Fix Ordering / Related

- bug-0068 (nav-widget focus-policy decision) gates the implementation approach shared by this bug and bug-0073—all three use the same `prevent_default` mouse-down suppression; resolve bug-0068 first so the mechanism lands once, consistently.
- bug-0073 applies the identical mechanism to `TabBar`’s enabled path.
- bug-0075 (`TextInput` disabled gaps) and bug-0021 (disabled buttons) cover the widgets excluded here.
