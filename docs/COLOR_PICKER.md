# ColorSwatch / Color Picker Implementation

This document describes the implementation details, design decisions, and lessons learned while building the ColorSwatch widget.

## Overview

The ColorSwatch widget provides a full-featured color picker with:
- Clickable color preview swatch
- Hex text input (supports #RGB, #RRGGBB, #RRGGBBAA formats)
- CSS named color support (140 colors: "red", "coral", "darkblue", etc.)
- Popup color picker with:
  - 2D saturation/brightness canvas (HSV model)
  - Hue slider (0-359°)
  - Alpha slider (optional)
  - RGB component sliders
  - Old/New color comparison with hex values

## Color Model: HSV vs HSL

The 2D color canvas uses the **HSV** (Hue, Saturation, Value) color model, not HSL:

| Model | Vertical Axis | At Top-Right | At Bottom |
|-------|--------------|--------------|-----------|
| HSL | Lightness (0-100%) | White | Black |
| HSV | Value/Brightness (0-100%) | Pure saturated color | Black |

HSV was chosen because it matches the Photoshop-style picker that users expect:
- **Top-right corner**: Pure saturated hue (S=100%, V=100%)
- **Top-left corner**: White (S=0%, V=100%)
- **Bottom edge**: Black (V=0%, regardless of S)
- **Right edge**: Transitions from black (bottom) through the color to fully saturated (top)

### Color Conversion Utilities

The `src/utils/color.rs` module provides:
- `Rgb`, `Rgba`, `Hsl`, `Hsv` structs
- Conversions: `Rgb::to_hsl()`, `Rgb::to_hsv()`, `Hsl::to_rgb()`, `Hsv::to_rgb()`
- Hex parsing: `Rgb::from_hex()`, `Rgba::from_hex()`
- Named colors: `named_color_to_rgb()` with 140 CSS colors

## GPUI Drag System for Sliders

All sliders use GPUI's drag system for proper mouse capture during drag operations.

### The Problem with `on_mouse_event`

Initially, we tried using `window.on_mouse_event()` to track mouse movement:

```rust
// DON'T DO THIS - causes crashes!
window.on_mouse_event(move |event: &MouseMoveEvent, phase, cx| {
    // This handler ACCUMULATES on every render frame!
});
```

**This caused crashes** because each render frame added a new handler, eventually exhausting resources.

### The Solution: GPUI's Drag API

Use `on_drag()` and `on_drag_move()` instead:

```rust
// Drag state type
#[derive(Clone)]
struct SliderDrag {
    origin: Rc<Cell<f32>>,
    width: Rc<Cell<f32>>,
}

// Empty view for drag visualization (we don't need a visible ghost)
struct EmptyDragView;
impl Render for EmptyDragView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_0()
    }
}

// Usage in element
div()
    .on_drag(
        SliderDrag { origin: origin.clone(), width: width.clone() },
        |_drag, _offset, _window, cx| cx.new(|_| EmptyDragView),
    )
    .on_drag_move(cx.listener(|this, event: &DragMoveEvent<SliderDrag>, _window, cx| {
        let x: f32 = event.event.position.x.into();
        let drag = event.drag(cx);  // Note: method call, not field access!
        this.handle_slider_move(x, drag.origin.get(), drag.width.get(), cx);
    }))
```

**Important**: Access drag state via `event.drag(cx)` method, not `event.drag` field (which is private).

## Slider Handle Positioning

### The Measurement Problem

Slider widths need to be measured at runtime because they use `flex_1()` for responsive layout. However, `Rc<Cell<>>` values created during render don't persist between frames:

```rust
// DON'T DO THIS - creates new Cell every frame!
.when(is_picker_open, |parent| {
    let slider_width = Rc::new(Cell::new(200.0));  // Lost after this frame!
    // ...
})
```

**Solution**: Store measurement values in the struct:

```rust
pub struct ColorSwatch {
    // ...
    hue_slider_width: Rc<Cell<f32>>,
    hue_slider_origin: Rc<Cell<f32>>,
    // ...
}
```

Then clone and use them in render:
```rust
let hue_width = self.hue_slider_width.clone();
let hue_width_for_paint = hue_width.clone();
```

The canvas prepaint callback updates the stored value:
```rust
canvas(
    move |bounds, _window, _cx| {
        hue_width_for_paint.set(bounds.size.width.into());
        bounds
    },
    // ...
)
```

### Handle Width Calculation

In GPUI, `.border_1()` does **not** add to the element's layout width. Only the content width matters for positioning:

```rust
let handle_width = 4.0f32;  // Content width only, NOT 4 + 2 for border
let handle_x = (value / max_value) * (slider_width - handle_width);
```

The handle element:
```rust
div()
    .absolute()
    .left(px(handle_x))
    .w(px(handle_width))  // 4px content
    .border_1()           // Border drawn inside, doesn't affect layout
```

### Consistent Input/Output Formulas

The handle position formula (display) must match the value calculation (input):

**Display** (value → position):
```rust
let handle_x = (value / max_value) * (slider_width - handle_width);
```

**Input** (position → value):
```rust
let usable_width = slider_width - handle_width;
let rel_x = (click_x - origin_x - handle_width / 2.0).clamp(0.0, usable_width);
let value = (rel_x / usable_width) * max_value;
```

The `- handle_width / 2.0` centers the handle on the click position.

## Hue Slider Range

Hue is capped at 359° instead of 360° to prevent visual wrap-around:

```rust
let h = (rel_x / usable_width) * 359.0;  // Not 360!
```

Since 360° = 0° = red, allowing 360 would cause the handle to jump from the right edge back to the left edge visually.

## Alpha Channel Visualization

For transparency visualization, a checkerboard pattern is drawn behind any element that can have alpha:

```rust
fn render_checkerboard() -> impl IntoElement {
    canvas(
        |bounds, _window, _cx| bounds,
        |bounds, _prepaint_result, window, _cx| {
            let cell_size = 8.0f32;
            let light = rgb(0xFFFFFF);
            let dark = rgb(0xCCCCCC);
            // Paint alternating cells...
        },
    )
    .size_full()
    .absolute()
}
```

Used for:
- Main color preview swatch
- Alpha slider track
- Old/New color comparison boxes

## Popup Positioning

The popup uses GPUI's `deferred()` + `anchored()` pattern for correct z-ordering:

```rust
deferred(
    anchored()
        .anchor(Corner::TopLeft)
        .child(
            div()
                .absolute()
                .top(px(4.))  // Small gap below the swatch
                // ... popup content
        )
)
```

## Event Flow

1. User clicks color swatch → `open_picker()`
2. User interacts with controls → `update_from_hsv()` or `update_from_rgb()`
3. Internal state updates: `current_rgb`, `current_hsl`, `current_hsv`, `current_alpha`
4. `sync_value()` formats hex string and emits `ColorSwatchEvent::Change`
5. User clicks outside → `close_picker()`

## Files

| File | Purpose |
|------|---------|
| `src/widgets/color_swatch.rs` | Main widget implementation |
| `src/utils/color.rs` | Color types and conversions |
| `src/utils/mod.rs` | Re-exports color utilities |
| `src/widgets/mod.rs` | Widget exports |
