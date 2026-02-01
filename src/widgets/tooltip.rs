//! Tooltip widget
//!
//! A lightweight tooltip for displaying text messages.
//! Uses GPUI's native `.tooltip()` method for positioning.
//!
//! # Example
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::Tooltip;
//!
//! div()
//!     .id("my_element")
//!     .child("Hover me")
//!     .tooltip(|_window, cx| {
//!         cx.new(|_cx| Tooltip::new("This is a tooltip"))
//!     })
//! ```

use gpui::prelude::*;
use gpui::*;

use crate::theme::{get_theme_or, Theme};

/// A simple tooltip view for displaying text messages
pub struct Tooltip {
    text: SharedString,
    custom_theme: Option<Theme>,
}

impl Tooltip {
    /// Create a new tooltip with the given text
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            custom_theme: None,
        }
    }

    /// Set custom theme (builder pattern)
    #[must_use]
    pub fn theme(mut self, theme: Theme) -> Self {
        self.custom_theme = Some(theme);
        self
    }
}

impl Render for Tooltip {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = get_theme_or(cx, self.custom_theme.as_ref());

        div()
            .px_2()
            .py_1()
            .bg(rgb(theme.tooltip_bg))
            .border_1()
            .border_color(rgb(theme.tooltip_border))
            .rounded_md()
            .shadow_md()
            .text_sm()
            .text_color(rgb(theme.tooltip_text))
            .child(self.text.clone())
    }
}
