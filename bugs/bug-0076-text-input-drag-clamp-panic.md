# bug-0076: `f32::clamp` panic in `handle_drag_move` when the input is narrower than its padding

**Severity:** low
**Fix side:** CODE
**Component:** `src/widgets/text_input.rs:587` (context `566-593`); `src/widgets/password_input.rs:414` (context `399-420`)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

Both drag handlers compute:

```rust
// src/widgets/text_input.rs:571-587 (password_input.rs:402-414 is identical)
let relative_x = mouse_x - self.content_origin_x;
let padding = 2.0;
let actual_visible = self.visible_width - padding;
// ...
let clamped_x = relative_x.clamp(0.0, actual_visible);
```

`visible_width` is taken from live canvas bounds (`text_input.rs:1192-1199`), so a flex-crushed input 0–2 px wide yields `actual_visible < 0.0`, and `f32::clamp` panics — its `assert!(min <= max)` is active in release builds — as soon as a drag begins.  The neighboring functions defend against exactly this: `ensure_cursor_visible` and `apply_auto_scroll` both apply `.max(0.0)` to their derived bounds (e.g. `let max_scroll = (content_width - actual_visible).max(0.0);`, `password_input.rs:440`).  Only the drag path is unguarded — in both widgets.

## Reproduction (test-ready)

Panic condition in isolation (plain demonstration): `1.0_f32.clamp(0.0, -1.5)` panics with `min > max, or either was NaN` — which is exactly the call `handle_drag_move` makes when `visible_width < 2.0`.

Widget-level (needs GPUI test context with layout): lay out a `TextInput` in a container squeezed to ≤2 px width (e.g. a shrinking flex row), mouse-down inside its 1–2 px hitbox, and move the mouse.  `handle_drag_move` reaches `relative_x.clamp(0.0, actual_visible)` with `actual_visible` negative and panics, aborting the app.

## Suggested Fix

Guard the derived bound in both `handle_drag_move` implementations (`text_input.rs:573`, `password_input.rs:404`):

```rust
let actual_visible = (self.visible_width - padding).max(0.0);
```

## Why the Fix Works

It guarantees `min <= max` for the clamp under all layouts, degrading gracefully — the cursor pins to position 0 in a zero-width input — instead of aborting the app.

## Fix Ordering / Related

None — independent.  The identical defect exists in both widgets; fix both in one pass.
