//! Slider widget
//!
//! A horizontal slider for selecting numeric values within a range.
//! Supports mouse dragging, keyboard navigation, and optional value display.
//!
//! # Features
//!
//! - **Click** anywhere on the track to set the value
//! - **Click and drag** the thumb or track to adjust the value smoothly
//!   - Hold **Shift** for fast adjustment (10x step)
//!   - Hold **Alt/Option** for slow/fine adjustment (0.1x step)
//! - **Arrow keys** to increment/decrement when focused
//! - **Home/End** keys to jump to min/max values
//!
//! # Example
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::Slider;
//!
//! let slider = cx.new(|cx| {
//!     Slider::new(cx)
//!         .with_value(50.0)
//!         .min(0.0)
//!         .max(100.0)
//!         .step(1.0)
//!         .show_value(true)
//! });
//!
//! // Subscribe to changes
//! cx.subscribe(&slider, |this, _slider, event: &SliderEvent, cx| {
//!     match event {
//!         SliderEvent::Change(value) => println!("Value: {}", value),
//!         SliderEvent::ChangeComplete => println!("Drag ended"),
//!     }
//! }).detach();
//! ```

use std::cell::Cell;
use std::rc::Rc;

use gpui::prelude::*;
use gpui::*;

use crate::theme::{get_theme_or, Theme};
use super::focus_navigation::{FocusNext, FocusPrev};

/// Events emitted by Slider
#[derive(Clone, Debug)]
pub enum SliderEvent {
    /// Value changed during interaction
    Change(f64),
    /// Interaction (drag) completed
    ChangeComplete,
}

/// Marker type for slider drag operations
#[derive(Clone)]
struct SliderDragState;

/// Empty view used as drag visual (we don't want a visible drag indicator)
struct EmptyDragView;

impl Render for EmptyDragView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<'_, Self>) -> impl IntoElement {
        div().size_0()
    }
}

/// Slider widget for selecting numeric values
pub struct Slider {
    value: f64,
    min: f64,
    max: f64,
    step: Option<f64>,
    focus_handle: FocusHandle,
    custom_theme: Option<Theme>,
    show_value: bool,
    /// Display precision (decimal places)
    display_precision: Option<usize>,

    // Measured track dimensions
    track_origin: Rc<Cell<f32>>,
    track_width: Rc<Cell<f32>>,

    // Drag state
    dragging: bool,
}

impl EventEmitter<SliderEvent> for Slider {}

impl Focusable for Slider {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Slider {
    /// Create a new slider
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            value: 0.0,
            min: 0.0,
            max: 100.0,
            step: None,
            focus_handle: cx.focus_handle().tab_stop(true),
            custom_theme: None,
            show_value: false,
            display_precision: None,
            track_origin: Rc::new(Cell::new(0.0)),
            track_width: Rc::new(Cell::new(0.0)),
            dragging: false,
        }
    }

    /// Set initial value (builder pattern)
    #[must_use]
    pub fn with_value(mut self, value: f64) -> Self {
        self.value = value.clamp(self.min, self.max);
        self
    }

    /// Set minimum value (builder pattern)
    #[must_use]
    pub fn min(mut self, min: f64) -> Self {
        self.min = min;
        self.value = self.value.clamp(self.min, self.max);
        self
    }

    /// Set maximum value (builder pattern)
    #[must_use]
    pub fn max(mut self, max: f64) -> Self {
        self.max = max;
        self.value = self.value.clamp(self.min, self.max);
        self
    }

    /// Set step value (builder pattern)
    #[must_use]
    pub fn step(mut self, step: f64) -> Self {
        self.step = Some(step);
        self
    }

    /// Show value display (builder pattern)
    #[must_use]
    pub fn show_value(mut self, show: bool) -> Self {
        self.show_value = show;
        self
    }

    /// Set display precision (builder pattern)
    #[must_use]
    pub fn display_precision(mut self, precision: usize) -> Self {
        self.display_precision = Some(precision);
        self
    }

    /// Set custom theme (builder pattern)
    #[must_use]
    pub fn theme(mut self, theme: Theme) -> Self {
        self.custom_theme = Some(theme);
        self
    }

    /// Get the current value
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Set value programmatically
    pub fn set_value(&mut self, value: f64, cx: &mut Context<Self>) {
        let normalized = self.normalize_value(value);
        if (self.value - normalized).abs() > f64::EPSILON {
            self.value = normalized;
            cx.emit(SliderEvent::Change(self.value));
            cx.notify();
        }
    }

    /// Get the focus handle
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    /// Calculate the percentage (0.0-1.0) of the current value within the range
    fn percentage(&self) -> f64 {
        if (self.max - self.min).abs() < f64::EPSILON {
            0.0
        } else {
            (self.value - self.min) / (self.max - self.min)
        }
    }

    /// Snap value to step and clamp to range
    fn normalize_value(&self, value: f64) -> f64 {
        let snapped = if let Some(step) = self.step {
            if step > 0.0 {
                let offset = value - self.min;
                let n = (offset / step).round();
                self.min + n * step
            } else {
                value
            }
        } else {
            value
        };

        snapped.clamp(self.min, self.max)
    }

    /// Format value for display
    fn format_value(&self) -> String {
        match self.display_precision {
            Some(p) => format!("{:.prec$}", self.value, prec = p),
            None => {
                if self.value.fract() == 0.0 {
                    format!("{:.0}", self.value)
                } else {
                    let s = format!("{}", self.value);
                    s.trim_end_matches('0').trim_end_matches('.').to_string()
                }
            }
        }
    }

    /// Set value from pixel position on track
    fn set_value_from_position(&mut self, x: f32, cx: &mut Context<Self>) {
        let track_origin = self.track_origin.get();
        let track_width = self.track_width.get();

        if track_width > 0.0 {
            let relative_x = (x - track_origin).clamp(0.0, track_width);
            let percentage = (relative_x / track_width) as f64;
            let raw_value = self.min + percentage * (self.max - self.min);
            let normalized = self.normalize_value(raw_value);

            if (self.value - normalized).abs() > f64::EPSILON {
                self.value = normalized;
                cx.emit(SliderEvent::Change(self.value));
                cx.notify();
            }
        }
    }

    fn adjust_value(&mut self, direction: f64, multiplier: f64, cx: &mut Context<Self>) {
        let step = self.step.unwrap_or(1.0) * multiplier * direction;
        let new_value = self.normalize_value(self.value + step);
        if (self.value - new_value).abs() > f64::EPSILON {
            self.value = new_value;
            cx.emit(SliderEvent::Change(self.value));
            cx.notify();
        }
    }

    fn increment(&mut self, multiplier: f64, cx: &mut Context<Self>) {
        self.adjust_value(1.0, multiplier, cx);
    }

    fn decrement(&mut self, multiplier: f64, cx: &mut Context<Self>) {
        self.adjust_value(-1.0, multiplier, cx);
    }

    fn go_to_min(&mut self, cx: &mut Context<Self>) {
        if (self.value - self.min).abs() > f64::EPSILON {
            self.value = self.min;
            cx.emit(SliderEvent::Change(self.value));
            cx.notify();
        }
    }

    fn go_to_max(&mut self, cx: &mut Context<Self>) {
        if (self.value - self.max).abs() > f64::EPSILON {
            self.value = self.max;
            cx.emit(SliderEvent::Change(self.value));
            cx.notify();
        }
    }

    fn start_drag(&mut self) {
        self.dragging = true;
    }

    fn end_drag(&mut self, cx: &mut Context<Self>) {
        if self.dragging {
            self.dragging = false;
            cx.emit(SliderEvent::ChangeComplete);
        }
    }
}

impl Render for Slider {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = get_theme_or(cx, self.custom_theme.as_ref());
        let focus_handle = self.focus_handle.clone();
        let is_focused = self.focus_handle.is_focused(window);
        let percentage = self.percentage();
        let show_value = self.show_value;
        let display_value = self.format_value();

        // Dimensions
        let track_height = 6.0;
        let thumb_size = 16.0;

        // Clone for closures
        let track_origin = self.track_origin.clone();
        let track_width = self.track_width.clone();

        // Build track element with filled portion and thumb
        let track_element = div()
            .id("ccf_slider_track")
            .relative()
            .flex_1()
            .h(px(thumb_size)) // Height includes thumb space
            .cursor_pointer()
            // Canvas to measure track position and dimensions
            .child(
                canvas(
                    {
                        let origin = track_origin.clone();
                        let width = track_width.clone();
                        move |bounds, _window, _cx| {
                            origin.set(bounds.origin.x.into());
                            width.set(bounds.size.width.into());
                            bounds
                        }
                    },
                    |_, _, _, _| {},
                )
                .size_full()
                .absolute()
            )
            // Track background (centered vertically)
            .child(
                div()
                    .absolute()
                    .top(px((thumb_size - track_height) / 2.0))
                    .left_0()
                    .right_0()
                    .h(px(track_height))
                    .rounded_full()
                    .bg(rgb(theme.bg_input))
            )
            // Filled portion
            .child(
                div()
                    .absolute()
                    .top(px((thumb_size - track_height) / 2.0))
                    .left_0()
                    .w(relative(percentage as f32))
                    .h(px(track_height))
                    .rounded_full()
                    .bg(rgb(theme.primary))
            )
            // Thumb
            .child(
                div()
                    .absolute()
                    .top_0()
                    // Position thumb so its center is at the value position
                    .left(relative(percentage as f32))
                    .ml(px(-(thumb_size / 2.0)))
                    .w(px(thumb_size))
                    .h(px(thumb_size))
                    .rounded_full()
                    .bg(rgb(theme.bg_white))
                    .border_2()
                    .border_color(rgb(theme.primary))
                    .shadow_sm()
            )
            // Mouse down starts drag
            .on_mouse_down(MouseButton::Left, cx.listener(|slider, event: &MouseDownEvent, window, cx| {
                slider.focus_handle.focus(window);
                slider.start_drag();
                let x: f32 = event.position.x.into();
                slider.set_value_from_position(x, cx);
            }))
            // Initiate drag
            .on_drag(SliderDragState, |_state, _position, _window, cx| {
                cx.new(|_| EmptyDragView)
            })
            // Track drag movement
            .on_drag_move(cx.listener(|slider, event: &DragMoveEvent<SliderDragState>, _window, cx| {
                if slider.dragging {
                    let x: f32 = event.event.position.x.into();
                    slider.set_value_from_position(x, cx);
                }
            }))
            // End drag on mouse up
            .on_mouse_up(MouseButton::Left, cx.listener(|slider, _event: &MouseUpEvent, _window, cx| {
                slider.end_drag(cx);
            }))
            .on_mouse_up_out(MouseButton::Left, cx.listener(|slider, _event: &MouseUpEvent, _window, cx| {
                slider.end_drag(cx);
            }));

        div()
            .id("ccf_slider")
            .track_focus(&focus_handle)
            .tab_stop(true)
            // Focus navigation
            .on_action(cx.listener(|_this, _: &FocusNext, window, _cx| {
                window.focus_next();
            }))
            .on_action(cx.listener(|_this, _: &FocusPrev, window, _cx| {
                window.focus_prev();
            }))
            .on_key_down(cx.listener(|slider, event: &KeyDownEvent, window, cx| {
                let multiplier = if event.keystroke.modifiers.shift { 10.0 } else { 1.0 };
                match event.keystroke.key.as_str() {
                    "tab" => {
                        if event.keystroke.modifiers.shift {
                            window.focus_prev();
                        } else {
                            window.focus_next();
                        }
                    }
                    "left" => slider.decrement(multiplier, cx),
                    "right" => slider.increment(multiplier, cx),
                    "home" => slider.go_to_min(cx),
                    "end" => slider.go_to_max(cx),
                    _ => {}
                }
            }))
            .flex()
            .flex_row()
            .gap_3()
            .items_center()
            .w_full()
            .py_1()
            .px_1()
            .rounded_sm()
            .border_2()
            .border_color(if is_focused { rgb(theme.border_focus) } else { rgba(0x00000000) })
            .child(track_element)
            .when(show_value, |d| {
                d.child(
                    div()
                        .min_w(px(40.0))
                        .text_sm()
                        .text_color(rgb(theme.text_value))
                        .text_right()
                        .child(display_value)
                )
            })
    }
}
