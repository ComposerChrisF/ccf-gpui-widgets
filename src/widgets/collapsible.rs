//! Collapsible section widget
//!
//! A section header that can be collapsed/expanded. Use with child content
//! that you conditionally render based on the collapsed state.
//!
//! # Example
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::Collapsible;
//!
//! let section = cx.new(|_cx| {
//!     Collapsible::new("Advanced Options")
//!         .collapsed(true)
//! });
//!
//! // In your parent render:
//! div()
//!     .child(section.clone())
//!     .when(!section.read(cx).is_collapsed(), |d| {
//!         d.child(/* your content here */)
//!     })
//! ```

use gpui::prelude::*;
use gpui::*;

use crate::theme::{get_theme_or, Theme};

/// Events emitted by Collapsible
#[derive(Clone, Debug)]
pub enum CollapsibleEvent {
    /// Collapsed state changed
    Toggle(bool),
}

/// Collapsible section widget
pub struct Collapsible {
    title: SharedString,
    collapsed: bool,
    custom_theme: Option<Theme>,
}

impl EventEmitter<CollapsibleEvent> for Collapsible {}

impl Collapsible {
    /// Create a new collapsible section
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            collapsed: false,
            custom_theme: None,
        }
    }

    /// Set initial collapsed state (builder pattern)
    pub fn with_collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    #[deprecated(since = "0.2.0", note = "Use `with_collapsed()` instead")]
    /// Set initial collapsed state (builder pattern) - deprecated alias
    pub fn collapsed(self, collapsed: bool) -> Self {
        self.with_collapsed(collapsed)
    }

    /// Set custom theme (builder pattern)
    pub fn theme(mut self, theme: Theme) -> Self {
        self.custom_theme = Some(theme);
        self
    }

    /// Check if currently collapsed
    pub fn is_collapsed(&self) -> bool {
        self.collapsed
    }

    /// Set collapsed state programmatically
    pub fn set_collapsed(&mut self, collapsed: bool, cx: &mut Context<Self>) {
        if self.collapsed != collapsed {
            self.collapsed = collapsed;
            cx.emit(CollapsibleEvent::Toggle(collapsed));
            cx.notify();
        }
    }

    /// Toggle collapsed state
    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        self.collapsed = !self.collapsed;
        cx.emit(CollapsibleEvent::Toggle(self.collapsed));
        cx.notify();
    }
}

impl Render for Collapsible {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = get_theme_or(cx, self.custom_theme.as_ref());
        let collapsed = self.collapsed;
        let chevron = if collapsed { "▶" } else { "▼" };
        let title = self.title.clone();

        div()
            .flex()
            .flex_col()
            .w_full()
            .child(
                // Header row - clickable to toggle
                div()
                    .id("ccf_collapsible_header")
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .py_2()
                    .px_2()
                    .bg(rgb(theme.bg_section_header))
                    .rounded_md()
                    .cursor_pointer()
                    .hover(|d| d.bg(rgb(theme.bg_section_header_hover)))
                    .on_click(cx.listener(|this, _event, _window, cx| {
                        this.toggle(cx);
                    }))
                    .child(
                        // Chevron icon
                        div()
                            .text_sm()
                            .text_color(rgb(theme.text_dimmed))
                            .w(px(16.))
                            .child(chevron)
                    )
                    .child(
                        // Section title
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(theme.text_section_header))
                            .child(title)
                    )
            )
    }
}
