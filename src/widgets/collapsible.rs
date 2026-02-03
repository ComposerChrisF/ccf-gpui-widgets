//! Collapsible section widget
//!
//! A section header that can be collapsed/expanded. Use with child content
//! that you conditionally render based on the collapsed state.
//!
//! Can also be used as a static (non-collapsible) section header by calling
//! `.collapsible(false)`.
//!
//! # Example - Collapsible Section
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::Collapsible;
//!
//! let section = cx.new(|cx| {
//!     Collapsible::new("Advanced Options", cx)
//!         .with_collapsed(true)
//! });
//!
//! // In your parent render, wrap header and content in a container
//! // to get clean borders without visual seams:
//! div()
//!     .overflow_hidden()
//!     .rounded_md()
//!     .border_1()
//!     .border_color(rgb(theme.border_default))
//!     .child(section.clone())
//!     .when(!section.read(cx).is_collapsed(), |d| {
//!         d.child(
//!             div()
//!                 .p_3()
//!                 .bg(rgb(theme.bg_input))
//!                 .child(/* your content here */)
//!         )
//!     })
//! ```
//!
//! # Example - Static Section Header
//!
//! ```ignore
//! let header = cx.new(|cx| {
//!     Collapsible::new("Settings", cx)
//!         .collapsible(false)  // No chevron, not interactive
//! });
//!
//! // Content is always visible
//! div()
//!     .overflow_hidden()
//!     .rounded_md()
//!     .border_1()
//!     .border_color(rgb(theme.border_default))
//!     .child(header.clone())
//!     .child(
//!         div()
//!             .p_3()
//!             .bg(rgb(theme.bg_input))
//!             .child(/* your content */)
//!     )
//! ```

use gpui::prelude::*;
use gpui::*;

use crate::theme::{get_theme_or, Theme};
use super::focus_navigation::{handle_tab_navigation, with_focus_actions, EnabledCursorExt};

/// Events emitted by Collapsible
#[derive(Clone, Debug)]
pub enum CollapsibleEvent {
    /// Collapsed state changed.
    /// The boolean indicates the new collapsed state: `true` = collapsed, `false` = expanded.
    Change(bool),
}

/// Collapsible section widget
pub struct Collapsible {
    title: SharedString,
    collapsed: bool,
    focus_handle: FocusHandle,
    custom_theme: Option<Theme>,
    /// Whether the widget is enabled (interactive)
    enabled: bool,
    /// Whether collapsing is allowed (when false, acts as static section header)
    collapsible: bool,
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
            enabled: true,
            collapsible: true,
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

    /// Set enabled state (builder pattern)
    #[must_use]
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set whether collapsing is allowed (builder pattern)
    ///
    /// When `false`, the widget acts as a static section header:
    /// - No chevron icon
    /// - No click/keyboard interaction
    /// - Not focusable
    ///
    /// Default is `true`.
    #[must_use]
    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.collapsible = collapsible;
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
            cx.emit(CollapsibleEvent::Change(collapsed));
            cx.notify();
        }
    }

    /// Toggle collapsed state
    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        self.collapsed = !self.collapsed;
        cx.emit(CollapsibleEvent::Change(self.collapsed));
        cx.notify();
    }

    /// Check if the collapsible is enabled
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

    /// Check if collapsing is allowed
    pub fn is_collapsible(&self) -> bool {
        self.collapsible
    }

    /// Set whether collapsing is allowed programmatically
    pub fn set_collapsible(&mut self, collapsible: bool, cx: &mut Context<Self>) {
        if self.collapsible != collapsible {
            self.collapsible = collapsible;
            cx.notify();
        }
    }
}

impl Render for Collapsible {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = get_theme_or(cx, self.custom_theme.as_ref());
        let collapsed = self.collapsed;
        let title = self.title.clone();
        let collapsible = self.collapsible;
        let enabled = self.enabled;
        // Only show interactive state when both collapsible and enabled
        let interactive = collapsible && enabled;

        // For non-collapsible mode, return a simple static header (always has content below)
        if !collapsible {
            return div()
                .id("ccf_collapsible_header")
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .py(px(6.))
                .px_2()
                .bg(rgb(theme.bg_section_header))
                .rounded_t_md()
                .border_2()
                .border_color(rgba(0x00000000))
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(theme.text_section_header))
                        .child(title)
                );
        }

        // Collapsible mode - full interactive header
        let chevron = if collapsed { "▶" } else { "▼" };
        let focus_handle = self.focus_handle.clone();
        let is_focused = self.focus_handle.is_focused(window);

        with_focus_actions(
            div()
                .id("ccf_collapsible_header")
                .track_focus(&focus_handle)
                .tab_stop(enabled),
            cx,
        )
        .on_key_down(cx.listener(move |this, event: &KeyDownEvent, window, cx| {
            if !this.enabled {
                return;
            }
            if handle_tab_navigation(event, window) {
                return;
            }
            // Arrow keys for expand/collapse, space/enter to toggle
            match event.keystroke.key.as_str() {
                "down" => this.set_collapsed(false, cx),
                "up" => this.set_collapsed(true, cx),
                "space" | "enter" => this.toggle(cx),
                _ => {}
            }
        }))
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .py(px(6.))
        .px_2()
        .when(enabled, |d| d.bg(rgb(theme.bg_section_header)))
        .when(!enabled, |d| d.bg(rgb(theme.disabled_bg)))
        // Rounded corners: all corners when collapsed, only top when expanded
        .when(collapsed, |d| d.rounded_md())
        .when(!collapsed, |d| d.rounded_t_md())
        .cursor_for_enabled(interactive)
        .border_2()
        .border_color(if is_focused && enabled { rgb(theme.border_focus) } else { rgba(0x00000000) })
        .when(interactive, |d| {
            d.hover(|d| d.bg(rgb(theme.bg_section_header_hover)))
                .on_mouse_down(MouseButton::Left, cx.listener(|this, _event, window, cx| {
                    this.focus_handle.focus(window);
                    this.toggle(cx);
                }))
        })
        .child(
            // Chevron icon
            div()
                .text_sm()
                .when(enabled, |d| d.text_color(rgb(theme.text_dimmed)))
                .when(!enabled, |d| d.text_color(rgb(theme.disabled_text)))
                .w(px(16.))
                .child(chevron)
        )
        .child(
            // Section title
            div()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .when(enabled, |d| d.text_color(rgb(theme.text_section_header)))
                .when(!enabled, |d| d.text_color(rgb(theme.disabled_text)))
                .child(title)
        )
    }
}
