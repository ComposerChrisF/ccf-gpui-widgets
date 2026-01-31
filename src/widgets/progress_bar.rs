//! Progress bar widget
//!
//! A progress bar for showing task completion status.
//! Supports determinate (known progress) and indeterminate (unknown progress) modes.
//!
//! # Example
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::ProgressBar;
//!
//! // Determinate progress (known percentage)
//! let progress = cx.new(|_cx| {
//!     ProgressBar::new()
//!         .with_value(0.5)  // 50%
//!         .show_percentage(true)
//! });
//!
//! // Indeterminate progress (unknown duration)
//! let loading = cx.new(|_cx| {
//!     ProgressBar::new()
//!         .indeterminate()
//!         .label("Loading...")
//! });
//!
//! // Subscribe to completion
//! cx.subscribe(&progress, |this, _progress, event: &ProgressBarEvent, cx| {
//!     if let ProgressBarEvent::Complete = event {
//!         println!("Progress complete!");
//!     }
//! }).detach();
//! ```

use std::time::Duration;

use gpui::prelude::*;
use gpui::*;

use crate::theme::{get_theme_or, Theme};

/// Events emitted by ProgressBar
#[derive(Clone, Debug)]
pub enum ProgressBarEvent {
    /// Progress reached 100%
    Complete,
}

/// Progress bar widget
pub struct ProgressBar {
    /// Current value (0.0 to max, None for indeterminate)
    value: Option<f64>,
    min: f64,
    max: f64,
    custom_theme: Option<Theme>,
    show_percentage: bool,
    label: Option<SharedString>,
    /// Whether Complete event has been emitted
    completed_emitted: bool,
}

impl EventEmitter<ProgressBarEvent> for ProgressBar {}

impl ProgressBar {
    /// Create a new progress bar (determinate mode, starting at 0)
    pub fn new() -> Self {
        Self {
            value: Some(0.0),
            min: 0.0,
            max: 1.0,
            custom_theme: None,
            show_percentage: false,
            label: None,
            completed_emitted: false,
        }
    }

    /// Set initial value (builder pattern)
    pub fn with_value(mut self, value: f64) -> Self {
        self.value = Some(value.clamp(self.min, self.max));
        self
    }

    /// Set minimum value (builder pattern)
    pub fn min(mut self, min: f64) -> Self {
        self.min = min;
        if let Some(v) = self.value {
            self.value = Some(v.clamp(self.min, self.max));
        }
        self
    }

    /// Set maximum value (builder pattern)
    pub fn max(mut self, max: f64) -> Self {
        self.max = max;
        if let Some(v) = self.value {
            self.value = Some(v.clamp(self.min, self.max));
        }
        self
    }

    /// Set to indeterminate mode (builder pattern)
    pub fn indeterminate(mut self) -> Self {
        self.value = None;
        self
    }

    /// Show percentage text (builder pattern)
    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    /// Set label text (builder pattern)
    pub fn label(mut self, text: impl Into<SharedString>) -> Self {
        self.label = Some(text.into());
        self
    }

    /// Set custom theme (builder pattern)
    pub fn theme(mut self, theme: Theme) -> Self {
        self.custom_theme = Some(theme);
        self
    }

    /// Get the current value (None if indeterminate)
    pub fn value(&self) -> Option<f64> {
        self.value
    }

    /// Get the current percentage (0.0-1.0, None if indeterminate)
    pub fn percentage(&self) -> Option<f64> {
        self.value.map(|v| {
            if (self.max - self.min).abs() < f64::EPSILON {
                0.0
            } else {
                (v - self.min) / (self.max - self.min)
            }
        })
    }

    /// Check if progress is complete
    pub fn is_complete(&self) -> bool {
        self.value.is_some_and(|v| (v - self.max).abs() < f64::EPSILON)
    }

    /// Check if in indeterminate mode
    pub fn is_indeterminate(&self) -> bool {
        self.value.is_none()
    }

    /// Set value programmatically
    pub fn set_value(&mut self, value: f64, cx: &mut Context<Self>) {
        let clamped = value.clamp(self.min, self.max);
        let old_value = self.value;
        self.value = Some(clamped);

        // Emit Complete event when reaching max (only once)
        if !self.completed_emitted && (clamped - self.max).abs() < f64::EPSILON {
            // Only emit if we weren't already at max
            if old_value.is_none_or(|v| (v - self.max).abs() >= f64::EPSILON) {
                self.completed_emitted = true;
                cx.emit(ProgressBarEvent::Complete);
            }
        }

        cx.notify();
    }

    /// Set to indeterminate mode programmatically
    pub fn set_indeterminate(&mut self, cx: &mut Context<Self>) {
        self.value = None;
        self.completed_emitted = false;
        cx.notify();
    }

    /// Reset progress to 0
    pub fn reset(&mut self, cx: &mut Context<Self>) {
        self.value = Some(self.min);
        self.completed_emitted = false;
        cx.notify();
    }
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for ProgressBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = get_theme_or(cx, self.custom_theme.as_ref());
        let percentage = self.percentage();
        let show_percentage = self.show_percentage;
        let label = self.label.clone();
        let is_indeterminate = self.is_indeterminate();

        // Dimensions
        let bar_height = 8.0;

        // Calculate display percentage string
        let percentage_text = percentage.map(|p| format!("{:.0}%", p * 100.0));

        // Build the track element
        let track_element = if is_indeterminate {
            // Indeterminate: animated pulsing bar
            div()
                .relative()
                .flex_1()
                .h(px(bar_height))
                .rounded_full()
                .bg(rgb(theme.bg_input))
                .overflow_hidden()
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .bottom_0()
                        .w(relative(0.3))
                        .rounded_full()
                        .bg(rgb(theme.primary))
                        .with_animation(
                            "indeterminate_slide",
                            Animation::new(Duration::from_millis(1500))
                                .repeat(),
                            move |el, delta| {
                                // Move from -30% to 100%
                                let position = -0.3 + delta * 1.3;
                                el.left(relative(position))
                            },
                        )
                )
        } else {
            // Determinate: filled bar based on percentage
            let fill_width = percentage.unwrap_or(0.0) as f32;

            div()
                .relative()
                .flex_1()
                .h(px(bar_height))
                .rounded_full()
                .bg(rgb(theme.bg_input))
                .overflow_hidden()
                .child(
                    div()
                        .h_full()
                        .w(relative(fill_width))
                        .rounded_full()
                        .bg(rgb(theme.primary))
                )
        };

        div()
            .id("ccf_progress_bar")
            .flex()
            .flex_col()
            .gap_1()
            .w_full()
            // Label row (if present)
            .when_some(label.clone(), |d, text| {
                d.child(
                    div()
                        .flex()
                        .flex_row()
                        .justify_between()
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(theme.text_label))
                                .child(text)
                        )
                        .when(show_percentage && percentage_text.is_some(), |d| {
                            d.child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(theme.text_muted))
                                    .child(percentage_text.clone().unwrap_or_default())
                            )
                        })
                )
            })
            // Track
            .child(track_element)
            // Percentage below (if no label)
            .when(show_percentage && label.is_none() && percentage_text.is_some(), |d| {
                d.child(
                    div()
                        .text_sm()
                        .text_color(rgb(theme.text_muted))
                        .text_right()
                        .child(percentage_text.unwrap_or_default())
                )
            })
    }
}

