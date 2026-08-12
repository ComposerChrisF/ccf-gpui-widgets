# bug-0068: SidebarNav focus policy contradicts TabBar — clicks focus the sidebar container

**Severity:** low
**Fix side:** CODE (decided 2026-08-12; see Decision below)
**Component:** `src/widgets/sidebar_nav.rs:264` (`.track_focus`), `src/widgets/sidebar_nav.rs:301-307` (item click handler)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

> **Decision (Chris, 2026-08-12): Option A — no-steal.**  Clicks never move focus; apply the `prevent_default` mouse-down suppression to SidebarNav — the same mechanism bug-0073 and bug-0022 use.

## Description

Both `TabBar` and `SidebarNav` are classified “navigation widgets”.  `TabBar` got a dedicated fix arc so that “mouse clicks no longer steal focus” (`TODO.md:192-194`).  `SidebarNav` has no such protection: its item click handler (`sidebar_nav.rs:301-307`) carefully avoids calling `focus()`—but gpui’s auto-focus grabs focus on every mouse-down anyway (verified against vendored gpui 0.2.2, `div.rs:2025-2038`: any element with a tracked focus handle transfers focus to itself on every mouse-down when the hitbox is hovered and `!window.default_prevented()`), because the container tracks focus unconditionally (`sidebar_nav.rs:264`).  There is no capture/restore dance at all.

Net effect: clicking a sidebar item defocuses whatever the user had focused, and the focus ring appears on the selected item (`show_focus = is_selected && is_focused && enabled`, `sidebar_nav.rs:290`).  Either this is intended (a click-then-arrow-keys workflow, which the ring plausibly supports) and `TabBar`’s no-steal behavior is the odd one out, or it is the same bug `TabBar` already tried to fix.

## Reproduction (test-ready)

Needs GPUI test context (window plus mouse dispatch).

1. Focus a `TextInput`; click a `SidebarNav` item.
2. Actual: the sidebar container takes focus (`window.focused(cx)` is the sidebar’s handle; the ring shows on the selected item); the `TextInput` blurs.
3. Contrast: `TabBar` under the same interaction restores the previous focus (or blurs) by design.

## Suggested Fix

**Question for Chris:** one focus policy for the two navigation widgets—which?

- **Option A (no-steal).**  Clicks never move focus.  Apply the same `prevent_default` mouse-down suppression proposed for `TabBar` in bug-0073 to `SidebarNav`’s container.  Keyboard focus still arrives via Tab (`tab_stop`), so the click-then-arrows workflow remains available after a Tab, just not from a bare click.
- **Option B (click-to-focus).**  Clicking a nav widget deliberately focuses it so arrow keys work immediately.  Then `TabBar`’s capture/restore/blur dance (`tab_bar.rs:327-345`) is the anomaly and should be removed, and the behavior should be documented on both widgets.

**Reviewer recommendation: Option A**, for consistency with the intent `TabBar` already recorded in its 2026-02-02 fix arc—a nav widget silently stealing focus from a form field is the behavior that arc calls a bug, and `SidebarNav` currently does exactly that.  The click-then-arrows workflow is plausibly intentional, though, which is why this is a decision rather than a fix: if Chris wants Option B, the work inverts to removing `TabBar`’s dance and documenting click-to-focus.

## Why the Fix Works

Whichever option is chosen, both navigation widgets end up with one documented focus policy, and the `prevent_default` mechanism (or its removal) lands once, consistently.

## Fix Ordering / Related

- **This decision gates the shared implementation mechanism used by bug-0073 (`TabBar` focus-steal paths) and bug-0022 (disabled widgets steal focus)**—all three use the same `prevent_default` mouse-down suppression.  Resolve this first.
