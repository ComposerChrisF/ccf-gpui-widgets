//! Spinner widget
//!
//! A loading spinner for indicating ongoing operations.
//! Purely visual, not focusable.
//!
//! # Example
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::{Spinner, SpinnerSize};
//!
//! // Small inline spinner
//! let small = cx.new(|_cx| {
//!     Spinner::new()
//!         .size(SpinnerSize::Small)
//! });
//!
//! // Medium spinner with label
//! let loading = cx.new(|_cx| {
//!     Spinner::new()
//!         .size(SpinnerSize::Medium)
//!         .label("Loading...")
//! });
//!
//! // Large centered spinner
//! let large = cx.new(|_cx| {
//!     Spinner::new()
//!         .size(SpinnerSize::Large)
//! });
//! ```

use std::f32::consts::PI;
use std::time::Duration;

use gpui::prelude::*;
use gpui::*;

use crate::theme::{get_theme_or, Theme};

/// Spinner size presets
#[derive(Clone, Copy, Debug, Default)]
pub enum SpinnerSize {
    /// Small (16px)
    Small,
    /// Medium (24px, default)
    #[default]
    Medium,
    /// Large (32px)
    Large,
    /// Custom size in pixels
    Custom(f32),
}

impl SpinnerSize {
    /// Get the size in pixels
    pub fn pixels(&self) -> f32 {
        match self {
            SpinnerSize::Small => 16.0,
            SpinnerSize::Medium => 24.0,
            SpinnerSize::Large => 32.0,
            SpinnerSize::Custom(px) => *px,
        }
    }
}

/// Spinner widget
pub struct Spinner {
    size: SpinnerSize,
    custom_theme: Option<Theme>,
    label: Option<SharedString>,
}

impl Spinner {
    /// Create a new spinner
    pub fn new() -> Self {
        Self {
            size: SpinnerSize::default(),
            custom_theme: None,
            label: None,
        }
    }

    /// Set spinner size (builder pattern)
    #[must_use]
    pub fn size(mut self, size: SpinnerSize) -> Self {
        self.size = size;
        self
    }

    /// Set label text (builder pattern)
    #[must_use]
    pub fn label(mut self, text: impl Into<SharedString>) -> Self {
        self.label = Some(text.into());
        self
    }

    /// Set custom theme (builder pattern)
    #[must_use]
    pub fn theme(mut self, theme: Theme) -> Self {
        self.custom_theme = Some(theme);
        self
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for Spinner {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = get_theme_or(cx, self.custom_theme.as_ref());
        let size = self.size.pixels();
        let label = self.label.clone();

        // Number of dots in the spinner
        let dot_count = 8;
        let dot_size = size * 0.15;
        let radius = (size - dot_size) / 2.0;

        div()
            .id("ccf_spinner")
            .flex()
            .flex_row()
            .gap_2()
            .items_center()
            // Spinner container
            .child(
                div()
                    .relative()
                    .w(px(size))
                    .h(px(size))
                    .children((0..dot_count).map(|i| {
                        // Calculate position for each dot
                        let angle = (i as f32 / dot_count as f32) * 2.0 * PI;
                        let x = radius * angle.cos() + (size - dot_size) / 2.0;
                        let y = radius * angle.sin() + (size - dot_size) / 2.0;

                        // Base opacity for static appearance
                        let base_opacity = 0.2 + (i as f32 / dot_count as f32) * 0.8;
                        let dot_index = i;

                        div()
                            .absolute()
                            .left(px(x))
                            .top(px(y))
                            .w(px(dot_size))
                            .h(px(dot_size))
                            .rounded_full()
                            .bg(rgb(theme.primary))
                            .with_animation(
                                ElementId::Name(format!("spinner_dot_{}", i).into()),
                                Animation::new(Duration::from_millis(1000))
                                    .repeat(),
                                move |el, delta| {
                                    // Create a "chasing" effect by offsetting each dot's animation phase
                                    let phase = (delta + (dot_index as f32 / dot_count as f32)) % 1.0;
                                    // Opacity varies: high when "active", low otherwise
                                    let opacity = if phase < 0.125 {
                                        1.0
                                    } else {
                                        base_opacity * (1.0 - phase * 0.5)
                                    };
                                    el.opacity(opacity)
                                },
                            )
                    }))
            )
            // Optional label
            .when_some(label, |d, text| {
                d.child(
                    div()
                        .text_sm()
                        .text_color(rgb(theme.text_muted))
                        .child(text)
                )
            })
    }
}
