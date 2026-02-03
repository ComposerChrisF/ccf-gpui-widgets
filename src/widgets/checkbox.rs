//! Checkbox widget
//!
//! A simple checkbox with optional label. Supports keyboard interaction
//! (Space/Enter to toggle) and mouse clicks.
//!
//! # Example
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::Checkbox;
//!
//! let checkbox = cx.new(|cx| {
//!     Checkbox::new(cx)
//!         .with_checked(true)
//!         .label("Enable feature")
//! });
//!
//! // Subscribe to changes
//! cx.subscribe(&checkbox, |this, _checkbox, event: &CheckboxEvent, cx| {
//!     if let CheckboxEvent::Change(checked) = event {
//!         println!("Checkbox is now: {}", checked);
//!     }
//! }).detach();
//! ```

use gpui::prelude::*;
use gpui::*;

use crate::theme::{get_theme_or, Theme};
use super::focus_navigation::{handle_tab_navigation, with_focus_actions, EnabledCursorExt};

/// Events emitted by Checkbox
#[derive(Clone, Debug)]
pub enum CheckboxEvent {
    /// Checkbox state changed.
    /// The boolean indicates the new checked state: `true` = checked, `false` = unchecked.
    Change(bool),
}

/// Checkbox widget
pub struct Checkbox {
    checked: bool,
    label: Option<SharedString>,
    focus_handle: FocusHandle,
    custom_theme: Option<Theme>,
    /// Whether the widget is enabled (interactive)
    enabled: bool,
}

impl EventEmitter<CheckboxEvent> for Checkbox {}

impl Focusable for Checkbox {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Checkbox {
    /// Create a new checkbox
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            checked: false,
            label: None,
            focus_handle: cx.focus_handle().tab_stop(true),
            custom_theme: None,
            enabled: true,
        }
    }

    /// Set initial checked state (builder pattern)
    #[must_use]
    pub fn with_checked(mut self, value: bool) -> Self {
        self.checked = value;
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

    /// Set enabled state (builder pattern)
    #[must_use]
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Get current checked state
    pub fn is_checked(&self) -> bool {
        self.checked
    }

    /// Set checked state programmatically
    pub fn set_checked(&mut self, checked: bool, cx: &mut Context<Self>) {
        if self.checked != checked {
            self.checked = checked;
            cx.emit(CheckboxEvent::Change(checked));
            cx.notify();
        }
    }

    /// Get the focus handle
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    /// Check if the checkbox is enabled
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

    /// Set label text programmatically
    pub fn set_label(&mut self, label: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.label = Some(label.into());
        cx.notify();
    }

    /// Clear the label
    pub fn clear_label(&mut self, cx: &mut Context<Self>) {
        self.label = None;
        cx.notify();
    }

    fn toggle(&mut self, cx: &mut Context<Self>) {
        self.checked = !self.checked;
        cx.emit(CheckboxEvent::Change(self.checked));
        cx.notify();
    }
}

impl Render for Checkbox {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = get_theme_or(cx, self.custom_theme.as_ref());
        let checked = self.checked;
        let label = self.label.clone();
        let focus_handle = self.focus_handle.clone();
        let is_focused = self.focus_handle.is_focused(window);
        let enabled = self.enabled;

        with_focus_actions(
            div()
                .id("ccf_checkbox")
                .track_focus(&focus_handle)
                .tab_stop(enabled),
            cx,
        )
        .on_key_down(cx.listener(move |checkbox, event: &KeyDownEvent, window, cx| {
            if !checkbox.enabled {
                return;
            }
            if handle_tab_navigation(event, window) {
                return;
            }
            if matches!(event.keystroke.key.as_str(), "space" | "enter") {
                checkbox.toggle(cx);
            }
        }))
        .flex()
        .flex_row()
        .gap_2()
        .items_center()
        .py_1()
        .px_1()
        .rounded_sm()
        .cursor_for_enabled(enabled)
            .border_2()
            .border_color(if is_focused && enabled { rgb(theme.border_focus) } else { rgba(0x00000000) })
            .when(enabled, |d| {
                d.on_mouse_down(MouseButton::Left, cx.listener(|checkbox, _event, window, cx| {
                    checkbox.focus_handle.focus(window);
                    checkbox.toggle(cx);
                }))
            })
            .child(
                // Checkbox box
                div()
                    .w(px(20.))
                    .h(px(20.))
                    .border_1()
                    .rounded_sm()
                    .flex()
                    .items_center()
                    .justify_center()
                    .when(!enabled, |d| {
                        // Disabled styling
                        d.bg(rgb(theme.disabled_bg))
                            .border_color(rgb(theme.disabled_bg))
                            .when(checked, |d| {
                                d.child(
                                    div()
                                        .text_color(rgb(theme.disabled_text))
                                        .text_sm()
                                        .child("✓")
                                )
                            })
                    })
                    .when(enabled && checked, |d| {
                        d.bg(rgb(theme.primary))
                            .border_color(rgb(theme.primary))
                            .child(
                                div()
                                    .text_color(rgb(theme.text_black))
                                    .text_sm()
                                    .child("✓")
                            )
                    })
                    .when(enabled && !checked, |d| {
                        d.bg(rgb(theme.bg_input))
                            .border_color(rgb(theme.border_input))
                            .hover(|d| d.bg(rgb(theme.bg_input_hover)))
                    })
            )
            .when_some(label, |d, label_text| {
                d.child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .when(enabled, |d| d.text_color(rgb(theme.text_label)))
                        .when(!enabled, |d| d.text_color(rgb(theme.disabled_text)))
                        .child(label_text)
                )
            })
    }
}
