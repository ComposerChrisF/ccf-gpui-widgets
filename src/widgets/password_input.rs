//! Password input widget with visibility toggle
//!
//! A text input that masks its content with bullet characters and provides
//! a button to toggle password visibility.
//!
//! # Example
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::{PasswordInput, PasswordInputEvent};
//!
//! let password_input = cx.new(|cx| {
//!     PasswordInput::new(cx)
//!         .placeholder("Enter password")
//! });
//!
//! // Subscribe to events
//! cx.subscribe(&password_input, |this, _, event: &PasswordInputEvent, cx| {
//!     match event {
//!         PasswordInputEvent::Change(value) => println!("Password: {}", value),
//!         PasswordInputEvent::Enter => println!("Enter pressed"),
//!         PasswordInputEvent::Blur => println!("Focus lost"),
//!     }
//! }).detach();
//! ```

use gpui::prelude::*;
use gpui::*;
use crate::theme::{get_theme, Theme};

/// Events emitted by PasswordInput
#[derive(Debug, Clone)]
pub enum PasswordInputEvent {
    /// Password value changed
    Change(String),
    /// Enter key was pressed
    Enter,
    /// Input lost focus (including Escape key)
    Blur,
}

/// Password input widget with visibility toggle
///
/// The input masks characters by default and provides a button to show/hide
/// the password. Implements `Focusable` for keyboard navigation.
pub struct PasswordInput {
    value: String,
    placeholder: Option<SharedString>,
    focus_handle: FocusHandle,
    show_password: bool,
    custom_theme: Option<Theme>,
}

impl PasswordInput {
    /// Create a new password input
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            value: String::new(),
            placeholder: None,
            focus_handle: cx.focus_handle(),
            show_password: false,
            custom_theme: None,
        }
    }

    /// Set an initial value
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    /// Set placeholder text shown when empty
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// Set a custom theme for this widget
    pub fn theme(mut self, theme: Theme) -> Self {
        self.custom_theme = Some(theme);
        self
    }

    /// Get the current password value
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Set the password value programmatically
    pub fn set_value(&mut self, value: &str, cx: &mut Context<Self>) {
        self.value = value.to_string();
        cx.emit(PasswordInputEvent::Change(self.value.clone()));
        cx.notify();
    }

    /// Get the focus handle for this input
    pub fn focus_handle(&self, _cx: &Context<Self>) -> FocusHandle {
        self.focus_handle.clone()
    }

    fn toggle_visibility(&mut self, cx: &mut Context<Self>) {
        self.show_password = !self.show_password;
        cx.notify();
    }

    fn get_theme(&self, cx: &App) -> Theme {
        self.custom_theme.clone().unwrap_or_else(|| get_theme(cx))
    }
}

impl EventEmitter<PasswordInputEvent> for PasswordInput {}

impl Focusable for PasswordInput {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for PasswordInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = self.get_theme(cx);
        let focus_handle = self.focus_handle.clone();
        let is_focused = self.focus_handle.is_focused(window);

        let display_text = if self.value.is_empty() {
            self.placeholder.clone().map(|s| s.to_string()).unwrap_or_default()
        } else if self.show_password {
            self.value.clone()
        } else {
            "\u{2022}".repeat(self.value.len()) // Bullet character
        };

        let text_is_placeholder = self.value.is_empty();

        div()
            .flex()
            .flex_row()
            .gap_2()
            .items_center()
            .child(
                // Password input field
                div()
                    .id("password_input_field")
                    .flex()
                    .flex_1()
                    .px_3()
                    .py_2()
                    .bg(rgb(theme.bg_input))
                    .rounded_md()
                    .border_1()
                    .border_color(if is_focused {
                        rgb(theme.border_focus)
                    } else {
                        rgb(theme.border_default)
                    })
                    .track_focus(&focus_handle)
                    .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                        let key = event.keystroke.key.as_str();

                        if key == "enter" {
                            cx.emit(PasswordInputEvent::Enter);
                            return;
                        }

                        if key == "escape" {
                            cx.emit(PasswordInputEvent::Blur);
                            return;
                        }

                        if key == "backspace" {
                            this.value.pop();
                            cx.emit(PasswordInputEvent::Change(this.value.clone()));
                            cx.notify();
                            return;
                        }

                        if let Some(key_char) = event.keystroke.key_char.as_ref() {
                            this.value.push_str(key_char);
                            cx.emit(PasswordInputEvent::Change(this.value.clone()));
                            cx.notify();
                        }
                    }))
                    .child(
                        div()
                            .text_sm()
                            .when(text_is_placeholder, |d| d.text_color(rgb(theme.text_placeholder)))
                            .when(!text_is_placeholder, |d| d.text_color(rgb(theme.text_value)))
                            .child(display_text)
                    )
            )
            .child(
                // Toggle visibility button
                div()
                    .id("password_toggle_button")
                    .px_2()
                    .py_2()
                    .bg(rgb(theme.bg_input_hover))
                    .rounded_md()
                    .cursor_pointer()
                    .hover(|d| d.bg(rgb(theme.bg_hover)))
                    .on_click(cx.listener(|this, _event, _window, cx| {
                        this.toggle_visibility(cx);
                    }))
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(theme.text_label))
                            .child(if self.show_password { "\u{1F441}" } else { "\u{1F441}\u{200D}\u{1F5E8}" })
                    )
            )
    }
}
