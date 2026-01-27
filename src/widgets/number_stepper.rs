//! Number stepper widget
//!
//! A numeric input with increment/decrement buttons. Supports min/max constraints,
//! step size, value resolution, and display precision.
//!
//! # Features
//!
//! - **Double-click** the value display to edit directly
//! - **Click and drag** horizontally on the value to scrub/adjust smoothly
//!   - Hold **Shift** for fast adjustment (5x speed)
//!   - Hold **Alt/Option** for slow/fine adjustment (0.1x speed)
//! - **Up/Down arrow keys** to increment/decrement when focused
//! - **Value resolution**: Values snap to multiples of resolution relative to min
//! - **Display precision**: Control how many decimal places are shown
//!
//! # Example
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::NumberStepper;
//!
//! let stepper = cx.new(|cx| {
//!     NumberStepper::new(cx)
//!         .with_value(50.0)
//!         .min(0.0)
//!         .max(100.0)
//!         .step(5.0)
//!         .resolution(0.25)      // Values snap to 0.25 increments
//!         .display_precision(2)  // Show 2 decimal places
//! });
//!
//! // Subscribe to changes
//! cx.subscribe(&stepper, |this, _stepper, event: &NumberStepperEvent, cx| {
//!     if let NumberStepperEvent::Change(value) = event {
//!         println!("Value: {}", value);
//!     }
//! }).detach();
//! ```

use std::time::{Duration, Instant};

use gpui::prelude::*;
use gpui::*;

use crate::theme::{get_theme_or, Theme};
use super::focus_navigation::{FocusNext, FocusPrev};

// Actions for text editing mode
actions!(
    ccf_number_stepper,
    [
        CommitEdit,
        CancelEdit,
    ]
);

/// Register keybindings for NumberStepper edit mode
///
/// Call this at application startup if you want Enter/Escape to work in edit mode:
/// ```ignore
/// ccf_gpui_widgets::widgets::number_stepper::register_keybindings(cx);
/// ```
pub fn register_keybindings(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("enter", CommitEdit, Some("CcfNumberStepperEdit")),
        KeyBinding::new("escape", CancelEdit, Some("CcfNumberStepperEdit")),
    ]);
}

/// Events emitted by NumberStepper
#[derive(Clone, Debug)]
pub enum NumberStepperEvent {
    /// Value changed
    Change(f64),
}

/// Marker type for number scrubbing drag operations
#[derive(Clone)]
struct NumberDragState;

/// Empty view used as drag visual (we don't want a visible drag indicator)
struct EmptyDragView;

impl Render for EmptyDragView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<'_, Self>) -> impl IntoElement {
        div().size_0()
    }
}

/// Number stepper widget with +/- buttons
pub struct NumberStepper {
    value: f64,
    min: Option<f64>,
    max: Option<f64>,
    step: Option<f64>,
    /// Value resolution - values snap to (min + n * resolution) where n is an integer
    resolution: Option<f64>,
    /// Number of decimal places to display (value is rounded for display only)
    display_precision: Option<usize>,
    focus_handle: FocusHandle,
    custom_theme: Option<Theme>,

    // Drag sensitivity: value change per pixel of mouse movement
    /// Normal drag: value change per pixel (no modifier key)
    value_per_pixel_normal: f64,
    /// Fast drag: value change per pixel (Shift held)
    value_per_pixel_fast: f64,
    /// Slow/fine drag: value change per pixel (Alt/Option held)
    value_per_pixel_slow: f64,

    // Edit mode state
    /// Whether we're in text editing mode
    editing: bool,
    /// The text being edited
    edit_buffer: String,
    /// Cursor position in the edit buffer
    edit_cursor: usize,
    /// Whether focus-out subscription has been set up
    focus_out_subscribed: bool,
    /// Cursor blink state
    cursor_last_moved: Instant,
    /// Whether blink timer is active
    blink_timer_active: bool,

    // Drag state
    /// Whether we're currently dragging
    dragging: bool,
    /// Starting x position of drag
    drag_start_x: f32,
    /// Value when drag started
    drag_start_value: f64,
}

impl EventEmitter<NumberStepperEvent> for NumberStepper {}

impl Focusable for NumberStepper {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl NumberStepper {
    /// Create a new number stepper
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            value: 0.0,
            min: None,
            max: None,
            step: None,
            resolution: None,
            display_precision: None,
            focus_handle: cx.focus_handle().tab_stop(true),
            custom_theme: None,
            value_per_pixel_normal: 0.5,
            value_per_pixel_fast: 2.5,   // 5x normal
            value_per_pixel_slow: 0.05,  // 0.1x normal
            editing: false,
            edit_buffer: String::new(),
            edit_cursor: 0,
            focus_out_subscribed: false,
            cursor_last_moved: Instant::now(),
            blink_timer_active: false,
            dragging: false,
            drag_start_x: 0.0,
            drag_start_value: 0.0,
        }
    }

    /// Set initial value (builder pattern)
    pub fn with_value(mut self, value: f64) -> Self {
        self.value = value;
        self
    }

    /// Set minimum value (builder pattern)
    pub fn min(mut self, min: f64) -> Self {
        self.min = Some(min);
        self
    }

    /// Set maximum value (builder pattern)
    pub fn max(mut self, max: f64) -> Self {
        self.max = Some(max);
        self
    }

    /// Set step value for +/- buttons (builder pattern)
    pub fn step(mut self, step: f64) -> Self {
        self.step = Some(step);
        self
    }

    /// Set value resolution (builder pattern)
    ///
    /// Values will snap to (min + n * resolution) where n is an integer.
    /// For example, with min=0.5 and resolution=0.25, valid values are 0.5, 0.75, 1.0, 1.25, etc.
    pub fn resolution(mut self, resolution: f64) -> Self {
        self.resolution = Some(resolution);
        self
    }

    /// Set display precision - number of decimal places to show (builder pattern)
    ///
    /// The displayed value is rounded to this many decimal places.
    /// This is independent of the actual stored value.
    pub fn display_precision(mut self, precision: usize) -> Self {
        self.display_precision = Some(precision);
        self
    }

    /// Set custom theme (builder pattern)
    pub fn theme(mut self, theme: Theme) -> Self {
        self.custom_theme = Some(theme);
        self
    }

    /// Set drag sensitivities as value change per pixel (builder pattern)
    ///
    /// Each parameter specifies how much the value changes per pixel of mouse movement:
    /// - `normal`: Value per pixel with no modifier key
    /// - `fast`: Value per pixel when Shift is held
    /// - `slow`: Value per pixel when Alt/Option is held
    ///
    /// Example for an integer 0-100 range:
    /// ```ignore
    /// .drag_sensitivities(1.0, 10.0, 1.0)  // 1, 10, or 1 unit per pixel
    /// ```
    ///
    /// Example for a float with fine control:
    /// ```ignore
    /// .drag_sensitivities(1.0, 2.0, 0.1)  // 10 pixels of slow drag = 1.0 change
    /// ```
    pub fn drag_sensitivities(mut self, normal: f64, fast: f64, slow: f64) -> Self {
        self.value_per_pixel_normal = normal;
        self.value_per_pixel_fast = fast;
        self.value_per_pixel_slow = slow;
        self
    }

    /// Set normal drag sensitivity, scaling fast (5x) and slow (0.1x) proportionally
    pub fn drag_sensitivity(mut self, value_per_pixel: f64) -> Self {
        self.value_per_pixel_normal = value_per_pixel;
        self.value_per_pixel_fast = value_per_pixel * 5.0;
        self.value_per_pixel_slow = value_per_pixel * 0.1;
        self
    }

    /// Get the focus handle
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
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
            cx.emit(NumberStepperEvent::Change(self.value));
            cx.notify();
        }
    }

    /// Format the value for display, applying display_precision rounding
    fn format_value(&self) -> String {
        match self.display_precision {
            Some(p) => {
                // Round to display precision
                let multiplier = 10_f64.powi(p as i32);
                let rounded = (self.value * multiplier).round() / multiplier;
                format!("{:.prec$}", rounded, prec = p)
            }
            None => {
                // Auto-format: show integer if whole number, otherwise trim trailing zeros
                if self.value.fract() == 0.0 {
                    format!("{:.0}", self.value)
                } else {
                    let s = format!("{}", self.value);
                    s.trim_end_matches('0').trim_end_matches('.').to_string()
                }
            }
        }
    }

    /// Snap value to resolution and clamp to min/max range
    fn normalize_value(&self, value: f64) -> f64 {
        let min = self.min.unwrap_or(f64::NEG_INFINITY);
        let max = self.max.unwrap_or(f64::INFINITY);

        // First snap to resolution if specified
        let snapped = if let Some(resolution) = self.resolution {
            if resolution > 0.0 {
                // Round to nearest multiple of resolution relative to min
                let offset = value - min;
                let n = (offset / resolution).round();
                min + n * resolution
            } else {
                value
            }
        } else {
            value
        };

        // Then clamp to range
        snapped.clamp(min, max)
    }

    fn increment(&mut self, multiplier: f64, cx: &mut Context<Self>) {
        let step = self.step.unwrap_or(1.0) * multiplier;
        let new_value = self.normalize_value(self.value + step);
        if (self.value - new_value).abs() > f64::EPSILON {
            self.value = new_value;
            cx.emit(NumberStepperEvent::Change(self.value));
            cx.notify();
        }
    }

    fn decrement(&mut self, multiplier: f64, cx: &mut Context<Self>) {
        let step = self.step.unwrap_or(1.0) * multiplier;
        let new_value = self.normalize_value(self.value - step);
        if (self.value - new_value).abs() > f64::EPSILON {
            self.value = new_value;
            cx.emit(NumberStepperEvent::Change(self.value));
            cx.notify();
        }
    }

    // ===== Edit mode methods =====

    /// Enter text editing mode
    fn enter_edit_mode(&mut self, cx: &mut Context<Self>) {
        self.editing = true;
        self.edit_buffer = self.format_value();
        self.edit_cursor = self.edit_buffer.len();
        self.cursor_last_moved = Instant::now();
        cx.notify();
    }

    /// Exit text editing mode without committing changes
    fn cancel_edit(&mut self, cx: &mut Context<Self>) {
        self.editing = false;
        self.edit_buffer.clear();
        self.edit_cursor = 0;
        self.blink_timer_active = false;
        cx.notify();
    }

    /// Commit the edited text value
    fn commit_edit(&mut self, cx: &mut Context<Self>) {
        if let Ok(parsed) = self.edit_buffer.trim().parse::<f64>() {
            let normalized = self.normalize_value(parsed);
            if (self.value - normalized).abs() > f64::EPSILON {
                self.value = normalized;
                cx.emit(NumberStepperEvent::Change(self.value));
            }
        }
        self.editing = false;
        self.edit_buffer.clear();
        self.edit_cursor = 0;
        self.blink_timer_active = false;
        cx.notify();
    }

    /// Reset cursor blink timer
    fn reset_cursor_blink(&mut self) {
        self.cursor_last_moved = Instant::now();
    }

    /// Check if cursor should be visible based on blink cycle
    fn is_cursor_visible(&self) -> bool {
        let elapsed = self.cursor_last_moved.elapsed();
        let blink_period = Duration::from_millis(530);
        let cycle_position = elapsed.as_millis() % (blink_period.as_millis() * 2);
        cycle_position < blink_period.as_millis()
    }

    /// Insert character at cursor position
    fn insert_char(&mut self, ch: &str, cx: &mut Context<Self>) {
        // Only allow valid numeric characters
        let valid = ch.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '-');
        if !valid {
            return;
        }
        self.edit_buffer.insert_str(self.edit_cursor, ch);
        self.edit_cursor += ch.len();
        self.reset_cursor_blink();
        cx.notify();
    }

    /// Delete character before cursor
    fn delete_backward(&mut self, cx: &mut Context<Self>) {
        if self.edit_cursor > 0 {
            let prev = self.edit_buffer[..self.edit_cursor]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.edit_buffer.replace_range(prev..self.edit_cursor, "");
            self.edit_cursor = prev;
            self.reset_cursor_blink();
            cx.notify();
        }
    }

    /// Delete character after cursor
    fn delete_forward(&mut self, cx: &mut Context<Self>) {
        if self.edit_cursor < self.edit_buffer.len() {
            let next = self.edit_buffer[self.edit_cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.edit_cursor + i)
                .unwrap_or(self.edit_buffer.len());
            self.edit_buffer.replace_range(self.edit_cursor..next, "");
            self.reset_cursor_blink();
            cx.notify();
        }
    }

    /// Move cursor left
    fn move_cursor_left(&mut self, cx: &mut Context<Self>) {
        if self.edit_cursor > 0 {
            self.edit_cursor = self.edit_buffer[..self.edit_cursor]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.reset_cursor_blink();
            cx.notify();
        }
    }

    /// Move cursor right
    fn move_cursor_right(&mut self, cx: &mut Context<Self>) {
        if self.edit_cursor < self.edit_buffer.len() {
            self.edit_cursor = self.edit_buffer[self.edit_cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.edit_cursor + i)
                .unwrap_or(self.edit_buffer.len());
            self.reset_cursor_blink();
            cx.notify();
        }
    }

    /// Move cursor to start
    fn move_cursor_to_start(&mut self, cx: &mut Context<Self>) {
        self.edit_cursor = 0;
        self.reset_cursor_blink();
        cx.notify();
    }

    /// Move cursor to end
    fn move_cursor_to_end(&mut self, cx: &mut Context<Self>) {
        self.edit_cursor = self.edit_buffer.len();
        self.reset_cursor_blink();
        cx.notify();
    }

    // ===== Drag methods =====

    /// Start drag scrubbing
    fn start_drag(&mut self, x: f32) {
        self.dragging = true;
        self.drag_start_x = x;
        self.drag_start_value = self.value;
    }

    /// Update value based on drag delta with modifier-based sensitivity
    fn update_drag(&mut self, x: f32, modifiers: &Modifiers, cx: &mut Context<Self>) {
        if !self.dragging {
            return;
        }

        // Select value-per-pixel based on modifier keys
        let value_per_pixel = if modifiers.shift {
            self.value_per_pixel_fast
        } else if modifiers.alt {
            self.value_per_pixel_slow
        } else {
            self.value_per_pixel_normal
        };

        let delta_pixels = (x - self.drag_start_x) as f64;
        let new_value = self.normalize_value(self.drag_start_value + delta_pixels * value_per_pixel);
        if (self.value - new_value).abs() > f64::EPSILON {
            self.value = new_value;
            cx.emit(NumberStepperEvent::Change(self.value));
            cx.notify();
        }
    }

    /// End drag scrubbing
    fn end_drag(&mut self) {
        self.dragging = false;
    }
}

impl Render for NumberStepper {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = get_theme_or(cx, self.custom_theme.as_ref());
        let display_value = self.format_value();
        let focus_handle = self.focus_handle.clone();
        let is_focused = self.focus_handle.is_focused(window);
        let editing = self.editing;

        // Set up focus-out subscription on first render
        if !self.focus_out_subscribed {
            self.focus_out_subscribed = true;
            let focus_handle = self.focus_handle.clone();
            cx.on_focus_out(&focus_handle, window, |this: &mut Self, _event, _window, cx| {
                if this.editing {
                    this.commit_edit(cx);
                }
            }).detach();
        }

        // Set up blink timer when editing
        if editing && is_focused && !self.blink_timer_active {
            self.blink_timer_active = true;
            let entity = cx.entity();
            window.spawn(cx, async move |async_cx| {
                loop {
                    smol::Timer::after(Duration::from_millis(530)).await;
                    let should_continue = async_cx
                        .update_entity(&entity, |this: &mut NumberStepper, cx| {
                            if !this.blink_timer_active || !this.editing {
                                this.blink_timer_active = false;
                                return false;
                            }
                            cx.notify();
                            true
                        })
                        .unwrap_or(false);
                    if !should_continue {
                        break;
                    }
                }
            }).detach();
        }

        // Colors for the unified control
        let bg_color = theme.bg_input;
        let border_color = if is_focused { theme.border_focus } else { theme.border_input };
        let separator_color = theme.text_muted;  // Light color for visibility
        let text_color = theme.text_value;
        let button_text_color = theme.text_value;  // Light color for +/- buttons

        // Build the center value element (without its own border/background)
        let value_element = if editing {
            // Text edit mode
            let edit_buffer = self.edit_buffer.clone();
            let edit_cursor = self.edit_cursor;
            let cursor_visible = self.is_cursor_visible();

            // Calculate cursor position (simple approximation for monospace-ish display)
            let cursor_offset = edit_buffer[..edit_cursor].chars().count() as f32 * 8.0;

            div()
                .id("ccf_number_value")
                .key_context("CcfNumberStepperEdit")
                .track_focus(&focus_handle)
                .px_2()
                .py_1()
                .flex_1()
                .flex()
                .items_center()
                .text_sm()
                .text_color(rgb(text_color))
                .cursor_text()
                .overflow_hidden()
                .relative()
                // Actions for Enter/Escape
                .on_action(cx.listener(|this, _: &CommitEdit, _window, cx| {
                    this.commit_edit(cx);
                }))
                .on_action(cx.listener(|this, _: &CancelEdit, _window, cx| {
                    this.cancel_edit(cx);
                }))
                // Keyboard handling for editing
                .on_key_down(cx.listener(move |stepper, event: &KeyDownEvent, window, cx| {
                    if !stepper.editing {
                        return;
                    }
                    match event.keystroke.key.as_str() {
                        "tab" => {
                            stepper.commit_edit(cx);
                            if event.keystroke.modifiers.shift {
                                window.focus_prev();
                            } else {
                                window.focus_next();
                            }
                        }
                        "left" => stepper.move_cursor_left(cx),
                        "right" => stepper.move_cursor_right(cx),
                        "home" => stepper.move_cursor_to_start(cx),
                        "end" => stepper.move_cursor_to_end(cx),
                        "backspace" => stepper.delete_backward(cx),
                        "delete" => stepper.delete_forward(cx),
                        _ => {
                            // Character input
                            if !event.keystroke.modifiers.alt
                                && !event.keystroke.modifiers.control
                                && !event.keystroke.modifiers.platform
                            {
                                if let Some(ref ch) = event.keystroke.key_char {
                                    stepper.insert_char(ch, cx);
                                }
                            }
                        }
                    }
                }))
                .child(
                    div()
                        .relative()
                        .h_full()
                        .flex()
                        .items_center()
                        // Text content
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(text_color))
                                .whitespace_nowrap()
                                .child(edit_buffer)
                        )
                        // Cursor
                        .when(cursor_visible, |d| {
                            d.child(
                                div()
                                    .absolute()
                                    .top(px(2.))
                                    .bottom(px(2.))
                                    .left(px(cursor_offset))
                                    .w(px(1.))
                                    .bg(rgb(text_color))
                            )
                        })
                )
        } else {
            // Normal display mode
            div()
                .id("ccf_number_value")
                .px_2()
                .py_1()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .text_sm()
                .text_color(rgb(text_color))
                .cursor(CursorStyle::ResizeLeftRight)
                // Double-click to edit, single-click starts drag state tracking
                .on_mouse_down(MouseButton::Left, cx.listener(|stepper, event: &MouseDownEvent, window, cx| {
                    stepper.focus_handle.focus(window);
                    if event.click_count == 2 {
                        // Double-click: enter edit mode
                        stepper.enter_edit_mode(cx);
                    } else {
                        // Single click: record drag start position for on_drag_move
                        let x: f32 = event.position.x.into();
                        stepper.start_drag(x);
                    }
                }))
                // Initiate drag - this enables on_drag_move to track outside element bounds
                .on_drag(NumberDragState, |_state, _position, _window, cx| {
                    cx.new(|_| EmptyDragView)
                })
                // Track drag movement even outside element bounds (Shift=fast, Alt/Option=slow)
                .on_drag_move(cx.listener(|stepper, event: &DragMoveEvent<NumberDragState>, _window, cx| {
                    if stepper.dragging {
                        let x: f32 = event.event.position.x.into();
                        stepper.update_drag(x, &event.event.modifiers, cx);
                    }
                }))
                // End drag on mouse up (inside element)
                .on_mouse_up(MouseButton::Left, cx.listener(|stepper, _event: &MouseUpEvent, _window, _cx| {
                    stepper.end_drag();
                }))
                // End drag on mouse up outside element
                .on_mouse_up_out(MouseButton::Left, cx.listener(|stepper, _event: &MouseUpEvent, _window, _cx| {
                    stepper.end_drag();
                }))
                .child(display_value)
        };

        // Vertical separator element
        let separator = || {
            div()
                .w(px(1.0))
                .h_full()
                .bg(rgb(separator_color))
        };

        // Unified container with all three parts
        div()
            .id("ccf_number_stepper")
            .track_focus(&focus_handle)
            .tab_stop(true)
            // Focus navigation (Tab / Shift+Tab)
            .on_action(cx.listener(|_this, _: &FocusNext, window, _cx| {
                window.focus_next();
            }))
            .on_action(cx.listener(|_this, _: &FocusPrev, window, _cx| {
                window.focus_prev();
            }))
            .on_key_down(cx.listener(|stepper, event: &KeyDownEvent, window, cx| {
                // Don't handle keys when editing (let the value element handle them)
                if stepper.editing {
                    return;
                }
                let multiplier = if event.keystroke.modifiers.shift { 10.0 } else { 1.0 };
                match event.keystroke.key.as_str() {
                    "tab" => {
                        if event.keystroke.modifiers.shift {
                            window.focus_prev();
                        } else {
                            window.focus_next();
                        }
                    }
                    "up" => stepper.increment(multiplier, cx),
                    "down" => stepper.decrement(multiplier, cx),
                    _ => {}
                }
            }))
            // Unified styling - single rounded box
            .flex()
            .flex_row()
            .items_center()
            .h(px(28.0))  // Fixed height for uniform appearance
            .bg(rgb(bg_color))
            .border_1()
            .border_color(rgb(border_color))
            .rounded_md()
            .overflow_hidden()
            // Decrement button
            .child(
                div()
                    .id("ccf_number_decrement")
                    .flex()
                    .items_center()
                    .justify_center()
                    .px_2()
                    .py_1()
                    .cursor_pointer()
                    .text_color(rgb(button_text_color))
                    .hover(|d| d.bg(rgb(theme.bg_hover)))
                    .on_click(cx.listener(|stepper, event: &ClickEvent, window, cx| {
                        stepper.focus_handle.focus(window);
                        if stepper.editing {
                            stepper.cancel_edit(cx);
                        }
                        let multiplier = if event.modifiers().shift { 10.0 } else { 1.0 };
                        stepper.decrement(multiplier, cx);
                    }))
                    .child("−")  // Using proper minus sign
            )
            // Left separator
            .child(separator())
            // Value display
            .child(value_element)
            // Right separator
            .child(separator())
            // Increment button
            .child(
                div()
                    .id("ccf_number_increment")
                    .flex()
                    .items_center()
                    .justify_center()
                    .px_2()
                    .py_1()
                    .cursor_pointer()
                    .text_color(rgb(button_text_color))
                    .hover(|d| d.bg(rgb(theme.bg_hover)))
                    .on_click(cx.listener(|stepper, event: &ClickEvent, window, cx| {
                        stepper.focus_handle.focus(window);
                        if stepper.editing {
                            stepper.cancel_edit(cx);
                        }
                        let multiplier = if event.modifiers().shift { 10.0 } else { 1.0 };
                        stepper.increment(multiplier, cx);
                    }))
                    .child("+")
            )
    }
}
