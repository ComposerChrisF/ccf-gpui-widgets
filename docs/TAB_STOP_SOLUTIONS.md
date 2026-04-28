# Tab Navigation Fix for GPUI Widgets

This document describes the Tab navigation bug discovered in ccf-gpui-widgets and the correct solution.

## The Problem

Tab key navigation between widgets was not working in the widget gallery example, even though:
- All widgets had `.tab_stop(true)` in their render methods
- All widgets had `.track_focus(&focus_handle)`
- All widgets had `.on_action()` handlers for `FocusNext`/`FocusPrev` actions
- The keybindings were correctly registered via `register_all_keybindings(cx)`

## Investigation Summary

### What Should Happen

1. User presses Tab
2. GPUI dispatches `FocusNext` action to the focused widget
3. Widget's `.on_action()` handler calls `window.focus_next()`
4. `window.focus_next()` uses `TabStopMap::next()` to find the next focusable element
5. Focus moves to the next widget

### What Was Actually Happening

Step 4 was failing because `TabStopMap::next()` skips entries where `tab_stop == false`. All widgets were being registered with `tab_stop: false` despite calling `.tab_stop(true)` on the div element.

## Root Cause Analysis

The bug is in the interaction between GPUI's `.track_focus()` and `.tab_stop()` methods on div elements.

### The Widget Pattern (Before Fix)

```rust
impl TextInput {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),  // Creates handle with tab_stop: false
            // ...
        }
    }
}

impl Render for TextInput {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        div()
            .track_focus(&self.focus_handle)  // Stores handle in tracked_focus_handle
            .tab_stop(true)                    // Sets interactivity().tab_stop = true
            // ...
    }
}
```

### Why This Doesn't Work

Looking at GPUI's `div.rs` (lines 1584-1599):

```rust
if self.focusable
    && self.tracked_focus_handle.is_none()  // <-- This check!
    && let Some(element_state) = element_state.as_mut()
{
    let mut handle = element_state
        .focus_handle
        .get_or_insert_with(|| cx.focus_handle())
        .clone()
        .tab_stop(self.tab_stop);  // Only applies tab_stop here

    // ...
    self.tracked_focus_handle = Some(handle);
}
```

The code that applies `interactivity().tab_stop` to the `FocusHandle` **only runs when `tracked_focus_handle.is_none()`**.

When you call `.track_focus(&focus_handle)`, it sets `tracked_focus_handle = Some(handle)`, so the tab_stop application code is skipped entirely.

### The Sequence of Events

1. `cx.focus_handle()` creates a `FocusHandle` with `tab_stop: false` (default)
2. `.track_focus(&focus_handle)` sets `self.tracked_focus_handle = Some(handle)`
3. `.tab_stop(true)` sets `self.interactivity().tab_stop = true` (but this only affects the div, not the already-tracked handle)
4. During paint, GPUI checks `if tracked_focus_handle.is_none()` - this is FALSE
5. The focus_handle is inserted into `TabStopMap` with its original `tab_stop: false`
6. `TabStopMap::next()` skips this entry because `tab_stop == false`

## Failed/Ineffective Solutions

### 1. Calling `.tab_stop(true)` on the Div

```rust
div()
    .track_focus(&self.focus_handle)
    .tab_stop(true)  // Does NOT work with track_focus()
```

This doesn't work because the div's `tab_stop` setting is only applied to handles created by the div itself, not to externally-tracked handles.

### 2. Handling Tab in `on_key_down`

The widgets also had fallback Tab handling:

```rust
.on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
    if event.keystroke.key == "tab" {
        window.focus_next();  // Still calls TabStopMap::next()
        return;
    }
    // ...
}))
```

This doesn't help because `window.focus_next()` still uses `TabStopMap`, which has the same problem.

### 3. Using Both Action and KeyDown Handlers

Having both `.on_action()` for `FocusNext` and `.on_key_down()` for Tab key doesn't help because:
- When the action is handled, GPUI sets `propagate_event = false`
- The `on_key_down` handler is never called
- Both ultimately call `window.focus_next()` which has the same `TabStopMap` issue

## The Correct Solution

Set `tab_stop(true)` directly on the `FocusHandle` when creating it:

```rust
impl TextInput {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle().tab_stop(true),  // Set here!
            // ...
        }
    }
}
```

This ensures the `FocusHandle` has `tab_stop: true` from the start, so when it's registered in `TabStopMap`, it's correctly marked as a tab stop.

### Files Modified

All widget files needed this fix:

- `src/widgets/text_input.rs`
- `src/widgets/checkbox.rs`
- `src/widgets/dropdown.rs`
- `src/widgets/number_stepper.rs`
- `src/widgets/radio_group.rs`
- `src/widgets/checkbox_group.rs`
- `src/widgets/color_swatch.rs`
- `src/widgets/file_picker.rs`
- `src/widgets/directory_picker.rs`

## Key Takeaways

1. **GPUI's `.tab_stop(true)` on divs only works for auto-created focus handles**, not for handles provided via `.track_focus()`.

2. **When using `.track_focus()`, you must set `tab_stop(true)` on the `FocusHandle` itself**, not on the div element.

3. **The correct pattern for focusable widgets is:**

```rust
// In new():
focus_handle: cx.focus_handle().tab_stop(true),

// In render():
div()
    .track_focus(&self.focus_handle)
    // .tab_stop(true) is optional here - it doesn't hurt but doesn't help either
```

## GPUI Version

This behavior was observed in GPUI 0.2.2. Future versions may change how `.track_focus()` and `.tab_stop()` interact.

## Date

2026-01-26
