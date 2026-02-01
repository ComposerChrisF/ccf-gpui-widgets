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
//! let section = cx.new(|cx| {
//!     Collapsible::new("Advanced Options", cx)
//!         .with_collapsed(true)
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
use super::focus_navigation::{FocusNext, FocusPrev};

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
    focus_handle: FocusHandle,
    custom_theme: Option<Theme>,
}

impl EventEmitter<CollapsibleEvent> for Collapsible {}

impl Focusable for Collapsible {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Collapsible {
    /// Create a new collapsible section
    pub fn new(title: impl Into<SharedString>, cx: &mut Context<Self>) -> Self {
        Self {
            title: title.into(),
            collapsed: false,
            focus_handle: cx.focus_handle().tab_stop(true),
            custom_theme: None,
        }
    }

    /// Set initial collapsed state (builder pattern)
    #[must_use]
    pub fn with_collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    /// Set custom theme (builder pattern)
    #[must_use]
    pub fn theme(mut self, theme: Theme) -> Self {
        self.custom_theme = Some(theme);
        self
    }

    /// Get the focus handle
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = get_theme_or(cx, self.custom_theme.as_ref());
        let collapsed = self.collapsed;
        let chevron = if collapsed { "▶" } else { "▼" };
        let title = self.title.clone();
        let focus_handle = self.focus_handle.clone();
        let is_focused = self.focus_handle.is_focused(window);

        div()
            .flex()
            .flex_col()
            .w_full()
            .child(
                // Header row - clickable to toggle
                div()
                    .id("ccf_collapsible_header")
                    .track_focus(&focus_handle)
                    .tab_stop(true)
                    // Focus navigation (Tab / Shift+Tab)
                    .on_action(cx.listener(|_this, _: &FocusNext, window, _cx| {
                        window.focus_next();
                    }))
                    .on_action(cx.listener(|_this, _: &FocusPrev, window, _cx| {
                        window.focus_prev();
                    }))
                    .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                        // Handle tab navigation and arrow keys for expand/collapse
                        // Space/enter are handled by on_click via synthetic click events
                        match event.keystroke.key.as_str() {
                            "tab" => {
                                if event.keystroke.modifiers.shift {
                                    window.focus_prev();
                                } else {
                                    window.focus_next();
                                }
                            }
                            "down" => {
                                // Down arrow expands
                                this.set_collapsed(false, cx);
                            }
                            "up" => {
                                // Up arrow collapses
                                this.set_collapsed(true, cx);
                            }
                            _ => {}
                        }
                    }))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .py_2()
                    .px_2()
                    .bg(rgb(theme.bg_section_header))
                    .rounded_md()
                    .cursor_pointer()
                    .border_2()
                    .border_color(if is_focused { rgb(theme.border_focus) } else { rgba(0x00000000) })
                    .hover(|d| d.bg(rgb(theme.bg_section_header_hover)))
                    .on_click(cx.listener(|this, _event, window, cx| {
                        this.focus_handle.focus(window);
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
