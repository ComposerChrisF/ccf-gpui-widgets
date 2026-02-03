//! Number stepper widget
//!
//! A numeric input with increment/decrement buttons. Supports min/max constraints,
//! step size, value resolution, and display precision.
//!
//! # Features
//!
//! - **Double-click** the value display to edit directly
//! - **Click and drag** horizontally on the value to scrub/adjust smoothly
//!   - Drag sensitivity auto-scales to widget width when min/max are set
//!   - Dragging across the full value display covers the entire range
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

use std::cell::Cell;
use std::rc::Rc;

use gpui::prelude::*;
use gpui::*;

use crate::theme::{get_theme_or, Theme};
use crate::utils::format_display_value;
use super::focus_navigation::{handle_tab_navigation, with_focus_actions, EnabledCursorExt};
use super::text_input::{TextInput, TextInputEvent};

/// Events emitted by NumberStepper
#[derive(Clone, Debug)]
pub enum NumberStepperEvent {
    /// Value changed
    Change(f64),
}

/// Marker type for number scrubbing drag operations
#[doc(hidden)]
#[derive(Clone)]
struct NumberDragState;

/// Empty view used as drag visual (we don't want a visible drag indicator)
#[doc(hidden)]
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
    /// Whether the stepper is enabled
    enabled: bool,

    // Drag sensitivity: value change per pixel of mouse movement
    /// Normal drag: value change per pixel (no modifier key)
    value_per_pixel_normal: f64,
    /// Fast drag: value change per pixel (Shift held)
    value_per_pixel_fast: f64,
    /// Slow/fine drag: value change per pixel (Alt/Option held)
    value_per_pixel_slow: f64,
    /// Whether to auto-scale drag sensitivity based on widget width and value range
    auto_scale_drag: bool,
    /// Measured width of the value display area (for auto-scaling)
    value_display_width: Rc<Cell<f32>>,

    // Edit mode state
    /// Whether we're in text editing mode
    editing: bool,
    /// The embedded text input for editing
    edit_input: Entity<TextInput>,
    /// Value when edit started (for cancel)
    original_value: f64,
    /// Whether we need to refocus the stepper in the next render
    pending_refocus: bool,

    // Drag state
    /// Whether we're currently dragging
    dragging: bool,
    /// Starting x position of drag
    drag_start_x: f32,
    /// Value when drag started
    drag_start_value: f64,

    // Step multipliers for button clicks
    /// Multiplier for small step (Alt/Option + click), default 0.1
    step_small_multiplier: f64,
    /// Multiplier for large step (Shift + click), default 10.0
    step_large_multiplier: f64,
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
        // Create the embedded text input for editing
        let edit_input = cx.new(|cx| {
            TextInput::new(cx)
                .borderless(true)
                .select_on_focus(true)
                .input_filter(|c| c.is_ascii_digit() || c == '.' || c == '-')
                .emit_tab_events(true)  // Let NumberStepper handle Tab
        });

        // Subscribe to TextInput events
        cx.subscribe(&edit_input, |this: &mut Self, _input, event: &TextInputEvent, cx| {
            match event {
                TextInputEvent::Enter => this.commit_edit(cx),
                TextInputEvent::Escape => this.cancel_edit(cx),
                TextInputEvent::Blur => {
                    if this.editing {
                        this.commit_edit(cx);
                    }
                }
                TextInputEvent::Tab | TextInputEvent::ShiftTab => {
                    // Treat Tab same as Enter - commit and stay on stepper
                    this.commit_edit(cx);
                }
                _ => {}
            }
        }).detach();

        Self {
            value: 0.0,
            min: None,
            max: None,
            step: None,
            resolution: None,
            display_precision: None,
            focus_handle: cx.focus_handle().tab_stop(true),
            custom_theme: None,
            enabled: true,
            value_per_pixel_normal: 0.5,
            value_per_pixel_fast: 2.5,   // 5x normal
            value_per_pixel_slow: 0.05,  // 0.1x normal
            auto_scale_drag: true,
            value_display_width: Rc::new(Cell::new(0.0)),
            editing: false,
            edit_input,
            original_value: 0.0,
            pending_refocus: false,
            dragging: false,
            drag_start_x: 0.0,
            drag_start_value: 0.0,
            step_small_multiplier: 0.1,  // Alt/Option = 0.1x step
            step_large_multiplier: 10.0, // Shift = 10x step
        }
    }

    /// Set initial value (builder pattern)
    #[must_use]
    pub fn with_value(mut self, value: f64) -> Self {
        self.value = value;
        self
    }

    /// Set minimum value (builder pattern)
    #[must_use]
    pub fn min(mut self, min: f64) -> Self {
        self.min = Some(min);
        self
    }

    /// Set maximum value (builder pattern)
    #[must_use]
    pub fn max(mut self, max: f64) -> Self {
        self.max = Some(max);
        self
    }

    /// Set step value for +/- buttons (builder pattern)
    #[must_use]
    pub fn step(mut self, step: f64) -> Self {
        self.step = Some(step);
        self
    }

    /// Set value resolution (builder pattern)
    ///
    /// Values will snap to (min + n * resolution) where n is an integer.
    /// For example, with min=0.5 and resolution=0.25, valid values are 0.5, 0.75, 1.0, 1.25, etc.
    #[must_use]
    pub fn resolution(mut self, resolution: f64) -> Self {
        self.resolution = Some(resolution);
        self
    }

    /// Set display precision - number of decimal places to show (builder pattern)
    ///
    /// The displayed value is rounded to this many decimal places.
    /// This is independent of the actual stored value.
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

    /// Set enabled state (builder pattern)
    #[must_use]
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
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
    #[must_use]
    pub fn drag_sensitivities(mut self, normal: f64, fast: f64, slow: f64) -> Self {
        self.value_per_pixel_normal = normal;
        self.value_per_pixel_fast = fast;
        self.value_per_pixel_slow = slow;
        self
    }

    /// Set normal drag sensitivity, scaling fast (5x) and slow (0.1x) proportionally
    #[must_use]
    pub fn drag_sensitivity(mut self, value_per_pixel: f64) -> Self {
        self.value_per_pixel_normal = value_per_pixel;
        self.value_per_pixel_fast = value_per_pixel * 5.0;
        self.value_per_pixel_slow = value_per_pixel * 0.1;
        self
    }

    /// Disable auto-scaling of drag sensitivity based on widget width (builder pattern)
    ///
    /// By default, when both min and max are set, the drag sensitivity is automatically
    /// calculated so that dragging across the value display covers the full range.
    /// Call this method to use the configured fixed sensitivities instead.
    #[must_use]
    pub fn manual_drag_sensitivity(mut self) -> Self {
        self.auto_scale_drag = false;
        self
    }

    /// Set step multipliers for button clicks (builder pattern)
    ///
    /// - `small`: Multiplier when Alt/Option is held (default 0.1)
    /// - `large`: Multiplier when Shift is held (default 10.0)
    ///
    /// Example: For a step of 100, with multipliers (0.01, 10.0):
    /// - Normal click: step by 100
    /// - Alt+click: step by 1 (100 * 0.01)
    /// - Shift+click: step by 1000 (100 * 10.0)
    #[must_use]
    pub fn step_multipliers(mut self, small: f64, large: f64) -> Self {
        self.step_small_multiplier = small;
        self.step_large_multiplier = large;
        self
    }

    /// Set small step multiplier for Alt/Option + click (builder pattern)
    #[must_use]
    pub fn step_small(mut self, multiplier: f64) -> Self {
        self.step_small_multiplier = multiplier;
        self
    }

    /// Set large step multiplier for Shift + click (builder pattern)
    #[must_use]
    pub fn step_large(mut self, multiplier: f64) -> Self {
        self.step_large_multiplier = multiplier;
        self
    }

    /// Get the focus handle
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    /// Check if the stepper is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Set enabled state programmatically
    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.enabled != enabled {
            self.enabled = enabled;
            cx.notify();
        }
    }

    /// Get the current value
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Get the minimum value constraint
    pub fn get_min(&self) -> Option<f64> {
        self.min
    }

    /// Get the maximum value constraint
    pub fn get_max(&self) -> Option<f64> {
        self.max
    }

    /// Get the step value for +/- buttons
    pub fn get_step(&self) -> Option<f64> {
        self.step
    }

    /// Get the value resolution
    pub fn get_resolution(&self) -> Option<f64> {
        self.resolution
    }

    /// Get the display precision (decimal places)
    pub fn get_display_precision(&self) -> Option<usize> {
        self.display_precision
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
        format_display_value(self.value, self.display_precision)
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

    fn adjust_value(&mut self, direction: f64, multiplier: f64, cx: &mut Context<Self>) {
        let step = self.step.unwrap_or(1.0) * multiplier * direction;
        let new_value = self.normalize_value(self.value + step);
        if (self.value - new_value).abs() > f64::EPSILON {
            self.value = new_value;
            cx.emit(NumberStepperEvent::Change(self.value));
            cx.notify();
        }
    }

    fn increment(&mut self, multiplier: f64, cx: &mut Context<Self>) {
        self.adjust_value(1.0, multiplier, cx);
    }

    fn decrement(&mut self, multiplier: f64, cx: &mut Context<Self>) {
        self.adjust_value(-1.0, multiplier, cx);
    }

    // ===== Edit mode methods =====

    /// Enter text editing mode
    fn enter_edit_mode(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editing = true;
        self.original_value = self.value;

        // Set the TextInput value and focus it
        let formatted = self.format_value();
        self.edit_input.update(cx, |input, cx| {
            input.set_value(&formatted, cx);
        });

        // Focus the TextInput
        self.edit_input.read(cx).focus_handle().focus(window);
        cx.notify();
    }

    /// Exit text editing mode without committing changes
    fn cancel_edit(&mut self, cx: &mut Context<Self>) {
        self.editing = false;
        self.pending_refocus = true;
        cx.notify();
    }

    /// Commit the edited text value
    fn commit_edit(&mut self, cx: &mut Context<Self>) {
        self.apply_edit_value(cx);
        self.editing = false;
        self.pending_refocus = true;
        cx.notify();
    }

    /// Apply the value from the edit input (shared logic)
    fn apply_edit_value(&mut self, cx: &mut Context<Self>) {
        // Get the content from TextInput
        let content = self.edit_input.read(cx).content().to_string();

        if let Ok(parsed) = content.trim().parse::<f64>() {
            let normalized = self.normalize_value(parsed);
            if (self.value - normalized).abs() > f64::EPSILON {
                self.value = normalized;
                cx.emit(NumberStepperEvent::Change(self.value));
            }
        }
        // If parse fails, value reverts to original (unchanged)
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

        // Calculate base value-per-pixel (auto-scale if enabled and range is defined)
        let auto_scale_range = if self.auto_scale_drag {
            self.min.zip(self.max).map(|(min, max)| max - min)
        } else {
            None
        };

        let base_value_per_pixel = if let Some(range) = auto_scale_range {
            let width = self.value_display_width.get();
            if width > 0.0 {
                range / width as f64
            } else {
                self.value_per_pixel_normal
            }
        } else {
            self.value_per_pixel_normal
        };

        // Apply modifier-based scaling
        let value_per_pixel = if modifiers.shift {
            if auto_scale_range.is_some() {
                base_value_per_pixel * 5.0 // 5x faster
            } else {
                self.value_per_pixel_fast
            }
        } else if modifiers.alt {
            if auto_scale_range.is_some() {
                base_value_per_pixel * 0.1 // 0.1x slower
            } else {
                self.value_per_pixel_slow
            }
        } else {
            base_value_per_pixel
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
        let editing = self.editing;
        let enabled = self.enabled;

        // Handle pending refocus (after Enter/Escape/Tab from TextInput)
        if self.pending_refocus {
            self.pending_refocus = false;
            self.focus_handle.focus(window);
        }

        // Check focus state after handling pending refocus
        let is_focused = self.focus_handle.is_focused(window);

        // Colors for the unified control (use disabled colors when disabled)
        let bg_color = if enabled { theme.bg_input } else { theme.disabled_bg };
        let border_color = if !enabled {
            theme.disabled_bg
        } else if is_focused || editing {
            theme.border_focus
        } else {
            theme.border_input
        };
        let separator_color = if enabled { theme.text_muted } else { theme.disabled_text };
        let text_color = if enabled { theme.text_value } else { theme.disabled_text };
        let button_text_color = if enabled { theme.text_value } else { theme.disabled_text };

        // Build the center value element (without its own border/background)
        let value_element = if editing && enabled {
            // Text edit mode - use embedded TextInput
            div()
                .id("ccf_number_value")
                .px_2()
                .py_1()
                .flex_1()
                .flex()
                .items_center()
                .overflow_hidden()
                .child(self.edit_input.clone())
        } else {
            // Normal display mode
            // Clone the width cell for the canvas closure
            let width_cell = self.value_display_width.clone();

            let mut value_div = div()
                .id("ccf_number_value")
                .relative()
                .px_2()
                .py_1()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .text_sm()
                .text_color(rgb(text_color))
                .when(enabled, |d| d.cursor(CursorStyle::ResizeLeftRight))
                .when(!enabled, |d| d.cursor_default());

            // Only add mouse/drag handlers when enabled
            if enabled {
                value_div = value_div
                    // Double-click to edit, single-click starts drag state tracking
                    .on_mouse_down(MouseButton::Left, cx.listener(|stepper, event: &MouseDownEvent, window, cx| {
                        if !stepper.enabled {
                            return;
                        }
                        stepper.focus_handle.focus(window);
                        if event.click_count == 2 {
                            // Double-click: enter edit mode
                            stepper.enter_edit_mode(window, cx);
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
                    }));
            }

            value_div
                // Canvas to measure width for auto-scaling drag sensitivity
                .child(
                    canvas(
                        move |bounds, _window, _cx| {
                            width_cell.set(bounds.size.width.into());
                            bounds
                        },
                        |_, _, _, _| {},
                    )
                    .size_full()
                    .absolute()
                )
                .child(display_value)
        };

        // Vertical separator element
        let separator = || {
            div()
                .w(px(1.0))
                .h_full()
                .bg(rgb(separator_color))
        };

        // Build decrement button
        let mut decrement_button = div()
            .id("ccf_number_decrement")
            .flex()
            .items_center()
            .justify_center()
            .px_2()
            .py_1()
            .text_color(rgb(button_text_color))
            .cursor_for_enabled(enabled)
            .when(enabled, |d| d.hover(|h| h.bg(rgb(theme.bg_hover))))
            .child("\u{2212}");  // Using proper minus sign

        if enabled {
            decrement_button = decrement_button.on_click(cx.listener(|stepper, event: &ClickEvent, window, cx| {
                if !stepper.enabled {
                    return;
                }
                stepper.focus_handle.focus(window);
                if stepper.editing {
                    // Set editing to false before anything that could trigger blur
                    stepper.editing = false;
                }
                // Shift = large step, Alt/Option = small step, Normal = 1x
                let multiplier = if event.modifiers().shift {
                    stepper.step_large_multiplier
                } else if event.modifiers().alt {
                    stepper.step_small_multiplier
                } else {
                    1.0
                };
                stepper.decrement(multiplier, cx);
            }));
        }

        // Build increment button
        let mut increment_button = div()
            .id("ccf_number_increment")
            .flex()
            .items_center()
            .justify_center()
            .px_2()
            .py_1()
            .text_color(rgb(button_text_color))
            .cursor_for_enabled(enabled)
            .when(enabled, |d| d.hover(|h| h.bg(rgb(theme.bg_hover))))
            .child("+");

        if enabled {
            increment_button = increment_button.on_click(cx.listener(|stepper, event: &ClickEvent, window, cx| {
                if !stepper.enabled {
                    return;
                }
                stepper.focus_handle.focus(window);
                if stepper.editing {
                    // Set editing to false before anything that could trigger blur
                    stepper.editing = false;
                }
                // Shift = large step, Alt/Option = small step, Normal = 1x
                let multiplier = if event.modifiers().shift {
                    stepper.step_large_multiplier
                } else if event.modifiers().alt {
                    stepper.step_small_multiplier
                } else {
                    1.0
                };
                stepper.increment(multiplier, cx);
            }));
        }

        // Unified container with all three parts
        with_focus_actions(
            div()
                .id("ccf_number_stepper")
                .track_focus(&focus_handle)
                .tab_stop(enabled),
            cx,
        )
        .on_key_down(cx.listener(|stepper, event: &KeyDownEvent, window, cx| {
                // Don't handle keys when disabled or editing (TextInput handles them)
                if !stepper.enabled || stepper.editing {
                    return;
                }
                if handle_tab_navigation(event, window) {
                    return;
                }
                let multiplier = if event.keystroke.modifiers.shift { 10.0 } else { 1.0 };
                match event.keystroke.key.as_str() {
                    "enter" => stepper.enter_edit_mode(window, cx),
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
            .child(decrement_button)
            // Left separator
            .child(separator())
            // Value display
            .child(value_element)
            // Right separator
            .child(separator())
            // Increment button
            .child(increment_button)
    }
}
