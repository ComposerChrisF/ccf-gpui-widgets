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
//!         .checked(true)
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
use super::focus_navigation::{FocusNext, FocusPrev};

/// Events emitted by Checkbox
#[derive(Clone, Debug)]
pub enum CheckboxEvent {
    /// Checkbox state changed
    Change(bool),
}

/// Checkbox widget
pub struct Checkbox {
    checked: bool,
    label: Option<SharedString>,
    focus_handle: FocusHandle,
    custom_theme: Option<Theme>,
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
        }
    }

    /// Set initial checked state (builder pattern)
    pub fn checked(mut self, value: bool) -> Self {
        self.checked = value;
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

        div()
            .id("ccf_checkbox")
            .track_focus(&focus_handle)
            .tab_stop(true)
            // Focus navigation (Tab / Shift+Tab)
            .on_action(cx.listener(|_this, _: &FocusNext, window, _cx| {
                window.focus_next();
            }))
            .on_action(cx.listener(|_this, _: &FocusPrev, window, _cx| {
                window.focus_prev();
            }))
            .on_key_down(cx.listener(|checkbox, event: &KeyDownEvent, window, cx| {
                match event.keystroke.key.as_str() {
                    "tab" => {
                        if event.keystroke.modifiers.shift {
                            window.focus_prev();
                        } else {
                            window.focus_next();
                        }
                    }
                    "space" | "enter" => {
                        checkbox.toggle(cx);
                    }
                    _ => {}
                }
            }))
            .flex()
            .flex_row()
            .gap_2()
            .items_center()
            .py_1()
            .px_1()
            .rounded_sm()
            .cursor_pointer()
            .border_2()
            .border_color(if is_focused { rgb(theme.border_focus) } else { rgba(0x00000000) })
            .on_mouse_down(MouseButton::Left, cx.listener(|checkbox, _event, window, cx| {
                checkbox.focus_handle.focus(window);
                checkbox.toggle(cx);
            }))
            .child(
                // Checkbox box
                div()
                    .w(px(20.))
                    .h(px(20.))
                    .border_1()
                    .border_color(rgb(theme.border_input))
                    .rounded_sm()
                    .flex()
                    .items_center()
                    .justify_center()
                    .when(checked, |d| {
                        d.bg(rgb(theme.primary))
                            .border_color(rgb(theme.primary))
                            .child(
                                div()
                                    .text_color(rgb(theme.text_primary))
                                    .text_sm()
                                    .child("✓")
                            )
                    })
                    .when(!checked, |d| {
                        d.bg(rgb(theme.bg_white))
                            .hover(|d| d.bg(rgb(theme.bg_light_hover)))
                    })
            )
            .when_some(label, |d, label_text| {
                d.child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(theme.text_label))
                        .child(label_text)
                )
            })
    }
}
