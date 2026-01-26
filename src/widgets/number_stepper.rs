//! Number stepper widget
//!
//! A numeric input with increment/decrement buttons. Supports min/max constraints,
//! step size, and precision formatting.
//!
//! # Example
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::NumberStepper;
//!
//! let stepper = cx.new(|cx| {
//!     NumberStepper::new(cx)
//!         .value(50.0)
//!         .min(0.0)
//!         .max(100.0)
//!         .step(5.0)
//! });
//!
//! // Subscribe to changes
//! cx.subscribe(&stepper, |this, _stepper, event: &NumberStepperEvent, cx| {
//!     if let NumberStepperEvent::Change(value) = event {
//!         println!("Value: {}", value);
//!     }
//! }).detach();
//! ```

use gpui::prelude::*;
use gpui::*;

use crate::theme::{get_theme_or, Theme};
use super::focus_navigation::{FocusNext, FocusPrev};

/// Events emitted by NumberStepper
#[derive(Clone, Debug)]
pub enum NumberStepperEvent {
    /// Value changed
    Change(f64),
}

/// Number stepper widget with +/- buttons
pub struct NumberStepper {
    value: f64,
    min: Option<f64>,
    max: Option<f64>,
    step: Option<f64>,
    precision: Option<usize>,
    focus_handle: FocusHandle,
    custom_theme: Option<Theme>,
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
            precision: None,
            focus_handle: cx.focus_handle().tab_stop(true),
            custom_theme: None,
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

    /// Set step value (builder pattern)
    pub fn step(mut self, step: f64) -> Self {
        self.step = Some(step);
        self
    }

    /// Set precision for display (builder pattern)
    pub fn precision(mut self, precision: usize) -> Self {
        self.precision = Some(precision);
        self
    }

    /// Set custom theme (builder pattern)
    pub fn theme(mut self, theme: Theme) -> Self {
        self.custom_theme = Some(theme);
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
        let clamped = self.clamp_value(value);
        if (self.value - clamped).abs() > f64::EPSILON {
            self.value = clamped;
            cx.emit(NumberStepperEvent::Change(self.value));
            cx.notify();
        }
    }

    fn format_value(&self) -> String {
        match self.precision {
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

    fn clamp_value(&self, value: f64) -> f64 {
        let min = self.min.unwrap_or(f64::NEG_INFINITY);
        let max = self.max.unwrap_or(f64::INFINITY);
        value.clamp(min, max)
    }

    fn increment(&mut self, cx: &mut Context<Self>) {
        let step = self.step.unwrap_or(1.0);
        let new_value = self.clamp_value(self.value + step);
        if (self.value - new_value).abs() > f64::EPSILON {
            self.value = new_value;
            cx.emit(NumberStepperEvent::Change(self.value));
            cx.notify();
        }
    }

    fn decrement(&mut self, cx: &mut Context<Self>) {
        let step = self.step.unwrap_or(1.0);
        let new_value = self.clamp_value(self.value - step);
        if (self.value - new_value).abs() > f64::EPSILON {
            self.value = new_value;
            cx.emit(NumberStepperEvent::Change(self.value));
            cx.notify();
        }
    }
}

impl Render for NumberStepper {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = get_theme_or(cx, self.custom_theme.as_ref());
        let display_value = self.format_value();
        let focus_handle = self.focus_handle.clone();
        let is_focused = self.focus_handle.is_focused(window);

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
                match event.keystroke.key.as_str() {
                    "tab" => {
                        if event.keystroke.modifiers.shift {
                            window.focus_prev();
                        } else {
                            window.focus_next();
                        }
                    }
                    "up" => stepper.increment(cx),
                    "down" => stepper.decrement(cx),
                    _ => {}
                }
            }))
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .child(
                // Decrement button
                div()
                    .id("ccf_number_decrement")
                    .px_2()
                    .py_1()
                    .bg(rgb(theme.bg_input_hover))
                    .rounded_md()
                    .cursor_pointer()
                    .hover(|d| d.bg(rgb(theme.bg_hover)))
                    .on_click(cx.listener(|stepper, _event, window, cx| {
                        stepper.focus_handle.focus(window);
                        stepper.decrement(cx);
                    }))
                    .child("-")
            )
            .child(
                // Display value
                div()
                    .id("ccf_number_value")
                    .px_3()
                    .py_2()
                    .bg(rgb(theme.bg_input))
                    .border_1()
                    .border_color(if is_focused { rgb(theme.border_focus) } else { rgb(theme.border_input) })
                    .rounded_md()
                    .w(px(100.0))
                    .text_sm()
                    .text_color(rgb(theme.text_value))
                    .cursor_pointer()
                    .on_click(cx.listener(|stepper, _event, window, cx| {
                        stepper.focus_handle.focus(window);
                        cx.notify();
                    }))
                    .child(display_value)
            )
            .child(
                // Increment button
                div()
                    .id("ccf_number_increment")
                    .px_2()
                    .py_1()
                    .bg(rgb(theme.bg_input_hover))
                    .rounded_md()
                    .cursor_pointer()
                    .hover(|d| d.bg(rgb(theme.bg_hover)))
                    .on_click(cx.listener(|stepper, _event, window, cx| {
                        stepper.focus_handle.focus(window);
                        stepper.increment(cx);
                    }))
                    .child("+")
            )
    }
}
