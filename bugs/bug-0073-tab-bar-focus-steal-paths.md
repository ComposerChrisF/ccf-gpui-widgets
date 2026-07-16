# bug-0073: TabBar steals focus on filler-area clicks, aborted clicks, and right-clicks

**Severity:** medium
**Fix side:** CODE
**Component:** `src/widgets/tab_bar.rs:280` (`.track_focus(&self.focus_handle)`), `src/widgets/tab_bar.rs:327-353` (mouse handlers), `src/widgets/tab_bar.rs:409-422` (filler area)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

The 2026-02-02 fix arc (`TODO.md:192-194`) claims: “TabBar: Mouse clicks no longer steal focus … Captures previous focus on mouse_down, restores it after tab selection … Blurs if nothing was previously focused.”  The restore dance only runs when a left-click **completes on a tab**.  But—verified against vendored gpui 0.2.2—any element with a tracked focus handle registers a mouse-down listener that transfers focus to itself on _every_ mouse-down of _any_ button, when the hitbox is hovered and `!window.default_prevented()` (`div.rs:2025-2038`), and the `TabBar` container tracks focus unconditionally (`tab_bar.rs:280`).  Three paths grab focus with no restore:

1. **Filler-area clicks.**  The `flex_1()` filler right of the tabs (`tab_bar.rs:409-422`) and the padding fillers have no `on_mouse_down`/`on_click`, so no tab handler fires; the container’s auto-focus keeps focus permanently.  The selected tab even lights up its focus ring (`show_focus = is_selected && is_focused && enabled`, `tab_bar.rs:319`).
2. **Aborted click.**  Mouse-down on a tab, drag off, release: gpui clears the pending click without firing `on_click` (`div.rs:2224-2231`), so `previous_focus` is captured (`tab_bar.rs:329`) but never restored—and the stale `Option<FocusHandle>` is retained indefinitely.
3. **Right-click.**  gpui’s click machinery is left-button-only (`div.rs:2139`), so `on_click` never runs; only the `MouseButton::Left` handler stores `previous_focus` (`tab_bar.rs:327-332`); the auto-focus grab still fires (it ignores the button).  Requesting a context menu (`tab_bar.rs:346-353`) silently defocuses e.g. a `TextInput`.

## Reproduction (test-ready)

All three need GPUI test context (window plus mouse dispatch).  Expected in all three: focus unchanged.

1. Focus a `TextInput`; left-click the empty area right of the tabs.
   Actual: `window.focused(cx)` is now the `TabBar` handle; the `TextInput` cursor stops blinking; the selected tab shows a focus ring.
2. Focus a `TextInput`; mouse-down on a tab, move off past the drag threshold, release.
   Actual: same wrong result, plus `tab_bar.previous_focus` is `Some(stale)`.
3. Focus a `TextInput`; right-click a tab.
   Actual: `ContextMenu` is emitted _and_ focus moves to the `TabBar`.

## Suggested Fix

Suppress gpui’s auto-focus at its source: on the `TabBar` container, register a mouse-down listener covering all buttons (`on_any_mouse_down`, or `on_mouse_down` for Left and Right) that calls `window.prevent_default()`.

The dispatch order makes this safe.  The auto-focus listener registers first (`div.rs:2025-2038`), user mouse-down listeners register after it (`div.rs:2040`), and the bubble phase dispatches in reverse registration order (`window.rs:3704`)—so the widget’s listener runs first and the auto-focus listener’s `!window.default_prevented()` check skips the focus transfer.  The click machinery does not check `default_prevented` (`div.rs:2137-2145`), so tab selection via `on_click` still works.

The `previous_focus` capture/restore/blur dance (`tab_bar.rs:327-345`) then becomes dead and should be removed, along with the `previous_focus` field.  Keyboard focusability via Tab (`tab_stop`) is unaffected.

## Why the Fix Works

It removes the focus grab at its source instead of undoing it after the fact, covering every abort path by construction—filler areas, aborted clicks, and right-clicks never had a restore path, and now they never need one.

## Fix Ordering / Related

- bug-0068 (`SidebarNav` focus-policy decision) should be resolved first so both navigation widgets land on one policy; if click-to-focus wins there, this fix inverts.
- bug-0022 uses the same `prevent_default` mechanism for disabled widgets—land the mechanism consistently.
- bug-0065 edits the same `TabBar` click handler (no-op `Change` guard); coordinate the `TabBar` edits.
