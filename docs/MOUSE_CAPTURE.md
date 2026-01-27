# Mouse Capture in GPUI

This document explains how mouse capture works in GPUI and how it's used in the `NumberStepper` widget for drag-to-scrub functionality.

## The Problem

By default, mouse events in GPUI only fire when the mouse is within an element's bounds. This is problematic for drag operations like value scrubbing, where users expect to:
1. Click and hold on a control
2. Drag the mouse (potentially far outside the control)
3. Have the value continue updating based on mouse movement
4. Release the mouse anywhere to end the drag

## GPUI's Drag System

GPUI provides a drag-and-drop system that solves this problem. When a drag is initiated, GPUI tracks the mouse globally until the drag ends.

### Key APIs

#### `on_drag<T>(value: T, constructor: Fn) -> Self`

Initiates a drag operation when the user clicks and drags on an element.

- `value: T` - A value of any type that identifies this drag operation
- `constructor` - Creates a view to render as the "drag ghost" (visual feedback)

```rust
.on_drag(MyDragState, |_state, _position, _window, cx| {
    cx.new(|_| MyDragView)
})
```

#### `on_drag_move<T>(listener: Fn(&DragMoveEvent<T>)) -> Self`

Called for **all** mouse move events while a drag of type `T` is active, even when the mouse is outside the element's bounds.

```rust
.on_drag_move(cx.listener(|this, event: &DragMoveEvent<MyDragState>, _window, cx| {
    let mouse_position = event.event.position;
    let modifiers = event.event.modifiers;
    // Update state based on mouse position
}))
```

#### `on_mouse_up_out(button, listener) -> Self`

Called when the mouse button is released outside the element's bounds. Essential for ending drag operations cleanly.

```rust
.on_mouse_up_out(MouseButton::Left, cx.listener(|this, _event, _window, _cx| {
    this.end_drag();
}))
```

### DragMoveEvent Structure

```rust
pub struct DragMoveEvent<T> {
    /// The underlying mouse move event (position, modifiers, etc.)
    pub event: MouseMoveEvent,
    /// The bounds of the element that initiated the drag
    pub bounds: Bounds<Pixels>,
    // ... internal fields
}
```

## NumberStepper Implementation

The `NumberStepper` widget uses this system for drag-to-scrub value adjustment:

### Drag State

A marker type identifies the drag operation:

```rust
#[derive(Clone)]
struct NumberDragState;
```

### Empty Drag View

Since we don't want a visible "drag ghost", we use an invisible view:

```rust
struct EmptyDragView;

impl Render for EmptyDragView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<'_, Self>) -> impl IntoElement {
        div().size_0()  // Zero-size invisible element
    }
}
```

### Wiring It Up

```rust
div()
    .id("value_display")
    // Track internal drag state on mouse down
    .on_mouse_down(MouseButton::Left, cx.listener(|this, event, window, cx| {
        let x: f32 = event.position.x.into();
        this.start_drag(x);  // Record start position and value
    }))
    // Initiate GPUI drag for mouse capture
    .on_drag(NumberDragState, |_state, _position, _window, cx| {
        cx.new(|_| EmptyDragView)
    })
    // Handle drag movement (works outside element bounds!)
    .on_drag_move(cx.listener(|this, event: &DragMoveEvent<NumberDragState>, _window, cx| {
        if this.dragging {
            let x: f32 = event.event.position.x.into();
            let modifiers = &event.event.modifiers;
            this.update_drag(x, modifiers, cx);
        }
    }))
    // End drag on mouse up (inside element)
    .on_mouse_up(MouseButton::Left, cx.listener(|this, _event, _window, _cx| {
        this.end_drag();
    }))
    // End drag on mouse up (outside element)
    .on_mouse_up_out(MouseButton::Left, cx.listener(|this, _event, _window, _cx| {
        this.end_drag();
    }))
```

### Drag Sensitivity

The `NumberStepper` supports three drag speeds controlled by modifier keys:

| Modifier | Speed | Use Case |
|----------|-------|----------|
| None | Normal | General adjustment |
| Shift | Fast | Large value changes |
| Alt/Option | Slow | Fine-grained precision |

Sensitivity values represent **value change per pixel of mouse movement**:

```rust
// Example: Integer stepper (0-100 range)
NumberStepper::new(cx)
    .drag_sensitivities(
        1.0,   // Normal: 1 unit per pixel
        10.0,  // Fast: 10 units per pixel (with Shift)
        1.0    // Slow: 1 unit per pixel (with Alt)
    )

// Example: Float stepper with fine control
NumberStepper::new(cx)
    .drag_sensitivities(
        1.0,   // Normal: 1.0 per pixel
        2.0,   // Fast: 2.0 per pixel
        0.1    // Slow: 0.1 per pixel (10 pixels = 1.0 change)
    )
```

The value calculation:
```rust
let delta_pixels = current_x - start_x;
let value_change = delta_pixels * sensitivity;
let new_value = start_value + value_change;
```

## Best Practices

1. **Always pair `on_mouse_up` with `on_mouse_up_out`** to ensure drags end cleanly regardless of where the mouse is released.

2. **Use a marker type for simple drags** when you don't need to pass data through the drag system.

3. **Keep drag views minimal** if you don't need visual feedback during the drag.

4. **Check `dragging` state in `on_drag_move`** since the callback may fire even after you've logically ended the drag.

5. **Access modifiers from the event** to support modifier-based behavior changes during drag.
