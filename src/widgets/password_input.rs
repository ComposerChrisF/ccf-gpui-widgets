//! Password input widget with visibility toggle
//!
//! A text input that masks its content with bullet characters and provides
//! a button to toggle password visibility. Uses TextInput internally for
//! full editing support (cursor movement, selection, clipboard, etc.)
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
use super::text_input::{TextInput, TextInputEvent};

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
/// the password. Uses TextInput internally for full editing support including
/// cursor movement, selection, and clipboard operations.
pub struct PasswordInput {
    text_input: Entity<TextInput>,
    toggle_focus_handle: FocusHandle,
    show_password: bool,
    custom_theme: Option<Theme>,
    // Store placeholder for deferred application
    pending_placeholder: Option<SharedString>,
    pending_value: Option<String>,
}

impl PasswordInput {
    /// Create a new password input
    pub fn new(cx: &mut Context<Self>) -> Self {
        let text_input = cx.new(|cx| TextInput::new(cx).masked(true).borderless(true));

        // Subscribe to TextInput events and re-emit as PasswordInput events
        cx.subscribe(&text_input, |_this, input, event: &TextInputEvent, cx| {
            match event {
                TextInputEvent::Change => {
                    let value = input.read(cx).content().to_string();
                    cx.emit(PasswordInputEvent::Change(value));
                }
                TextInputEvent::Enter => {
                    cx.emit(PasswordInputEvent::Enter);
                }
                TextInputEvent::Escape | TextInputEvent::Blur => {
                    cx.emit(PasswordInputEvent::Blur);
                }
                TextInputEvent::Focus => {}
            }
        })
        .detach();

        Self {
            text_input,
            toggle_focus_handle: cx.focus_handle().tab_stop(true),
            show_password: false,
            custom_theme: None,
            pending_placeholder: None,
            pending_value: None,
        }
    }

    /// Set an initial value (builder pattern)
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.pending_value = Some(value.into());
        self
    }

    /// Set placeholder text shown when empty (builder pattern)
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.pending_placeholder = Some(text.into());
        self
    }

    /// Set a custom theme for this widget (builder pattern)
    pub fn theme(mut self, theme: Theme) -> Self {
        self.custom_theme = Some(theme);
        self
    }

    /// Apply any pending builder values (called on first render)
    fn apply_pending(&mut self, cx: &mut Context<Self>) {
        if let Some(placeholder) = self.pending_placeholder.take() {
            self.text_input.update(cx, |input, cx| {
                input.set_placeholder(placeholder, cx);
            });
        }
        if let Some(value) = self.pending_value.take() {
            self.text_input.update(cx, |input, cx| {
                input.set_value(&value, cx);
            });
        }
    }

    /// Get the current password value
    pub fn value<'a>(&'a self, cx: &'a App) -> &'a str {
        self.text_input.read(cx).content()
    }

    /// Set the password value programmatically
    pub fn set_value(&mut self, value: &str, cx: &mut Context<Self>) {
        self.text_input.update(cx, |input, cx| {
            input.set_value(value, cx);
        });
    }

    /// Get the focus handle for this input (delegates to TextInput)
    pub fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.text_input.read(cx).focus_handle(cx)
    }

    fn toggle_visibility(&mut self, cx: &mut Context<Self>) {
        self.show_password = !self.show_password;
        self.text_input.update(cx, |input, cx| {
            input.set_masked(!self.show_password, cx);
        });
        cx.notify();
    }

    fn get_theme(&self, cx: &App) -> Theme {
        self.custom_theme.clone().unwrap_or_else(|| get_theme(cx))
    }
}

impl EventEmitter<PasswordInputEvent> for PasswordInput {}

impl Focusable for PasswordInput {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.text_input.read(cx).focus_handle(cx)
    }
}

impl Render for PasswordInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        // Apply any pending builder values on first render
        self.apply_pending(cx);

        let theme = self.get_theme(cx);
        let toggle_focus_handle = self.toggle_focus_handle.clone();
        let toggle_is_focused = self.toggle_focus_handle.is_focused(window);
        let text_input_is_focused = self.text_input.read(cx).focus_handle(cx).is_focused(window);

        // Colors matching the unified control style (like NumberStepper)
        let bg_color = theme.bg_input;
        let border_color = if text_input_is_focused || toggle_is_focused {
            theme.border_focus
        } else {
            theme.border_input
        };
        let separator_color = theme.text_muted;
        let button_text_color = theme.text_muted;

        // Eye icons: simple line-art style
        // ◎ (bullseye) for "click to show" when hidden
        // ⊖ (circled minus) for "click to hide" when visible
        let eye_icon = if self.show_password { "⊖" } else { "◎" };

        // Vertical separator element
        let separator = div()
            .w(px(1.0))
            .h_full()
            .bg(rgb(separator_color));

        // Unified container - text input and toggle button share border/background
        div()
            .id("ccf_password_input")
            .flex()
            .flex_row()
            .items_center()
            .h(px(28.0))  // Match TextInput height
            .bg(rgb(bg_color))
            .border_1()
            .border_color(rgb(border_color))
            .rounded_md()
            .overflow_hidden()
            .child(
                // Text input takes most of the space (borderless, embedded in container)
                div()
                    .flex_1()
                    .h_full()
                    .child(self.text_input.clone())
            )
            .child(separator)
            .child(
                // Toggle visibility button - fixed width to prevent size change between icons
                div()
                    .id("password_toggle_button")
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(28.0))  // Fixed width to match height (square button)
                    .h_full()
                    .cursor_pointer()
                    .text_color(rgb(button_text_color))
                    .when(toggle_is_focused, |d| d.bg(rgb(theme.bg_hover)))
                    .hover(|d| d.bg(rgb(theme.bg_hover)))
                    .track_focus(&toggle_focus_handle)
                    .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                        let key = event.keystroke.key.as_str();
                        match key {
                            "enter" | "space" => {
                                this.toggle_visibility(cx);
                            }
                            "tab" => {
                                if event.keystroke.modifiers.shift {
                                    window.focus_prev();
                                } else {
                                    window.focus_next();
                                }
                            }
                            _ => {}
                        }
                    }))
                    .on_click(cx.listener(|this, _event, _window, cx| {
                        this.toggle_visibility(cx);
                    }))
                    .child(
                        div()
                            .text_sm()
                            .child(eye_icon)
                    )
            )
    }
}
