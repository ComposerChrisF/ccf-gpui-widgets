//! Color swatch widget
//!
//! A color preview with hex input. Displays a colored square alongside
//! a text input for the hex color value.
//!
//! # Example
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::ColorSwatch;
//!
//! let swatch = cx.new(|cx| {
//!     ColorSwatch::new(cx)
//!         .value("#3b82f6")
//! });
//!
//! // Subscribe to changes
//! cx.subscribe(&swatch, |this, _swatch, event: &ColorSwatchEvent, cx| {
//!     if let ColorSwatchEvent::Change(hex) = event {
//!         println!("Color: {}", hex);
//!     }
//! }).detach();
//! ```

use gpui::prelude::*;
use gpui::*;

use crate::theme::{get_theme_or, Theme};
use super::focus_navigation::{FocusNext, FocusPrev};

/// Events emitted by ColorSwatch
#[derive(Clone, Debug)]
pub enum ColorSwatchEvent {
    /// Color value changed
    Change(String),
}

/// Color swatch widget with hex input
pub struct ColorSwatch {
    value: String,
    placeholder: String,
    focus_handle: FocusHandle,
    custom_theme: Option<Theme>,
}

impl EventEmitter<ColorSwatchEvent> for ColorSwatch {}

impl Focusable for ColorSwatch {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl ColorSwatch {
    /// Create a new color swatch
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            value: "#000000".to_string(),
            placeholder: "#000000".to_string(),
            focus_handle: cx.focus_handle().tab_stop(true),
            custom_theme: None,
        }
    }

    /// Set initial value (builder pattern)
    /// Accepts hex colors with or without # prefix
    pub fn with_value(mut self, color: impl Into<String>) -> Self {
        self.value = Self::validate_color(color.into());
        self
    }

    /// Set placeholder text (builder pattern)
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// Set custom theme (builder pattern)
    pub fn theme(mut self, theme: Theme) -> Self {
        self.custom_theme = Some(theme);
        self
    }

    /// Get the current hex value
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Set value programmatically
    pub fn set_value(&mut self, color: &str, cx: &mut Context<Self>) {
        let validated = Self::validate_color(color.to_string());
        if self.value != validated {
            self.value = validated;
            cx.emit(ColorSwatchEvent::Change(self.value.clone()));
            cx.notify();
        }
    }

    /// Get the focus handle
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    fn validate_color(color: String) -> String {
        let trimmed = color.trim();
        if trimmed.starts_with('#') && (trimmed.len() == 7 || trimmed.len() == 4) {
            trimmed.to_string()
        } else if trimmed.len() == 6 || trimmed.len() == 3 {
            format!("#{}", trimmed)
        } else {
            "#000000".to_string()
        }
    }

    fn parse_color(&self) -> Rgba {
        let hex = self.value.trim_start_matches('#');

        let expanded_hex = if hex.len() == 3 {
            hex.chars()
                .flat_map(|c| std::iter::repeat_n(c, 2))
                .collect::<String>()
        } else {
            hex.to_string()
        };

        if expanded_hex.len() == 6 {
            if let Ok(rgb_val) = u32::from_str_radix(&expanded_hex, 16) {
                return rgba((rgb_val << 8) | 0xff);
            }
        }

        rgba(0x000000ff)
    }
}

impl Render for ColorSwatch {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = get_theme_or(cx, self.custom_theme.as_ref());
        let focus_handle = self.focus_handle.clone();
        let is_focused = self.focus_handle.is_focused(window);
        let placeholder = self.placeholder.clone();
        let color = self.parse_color();
        let value = self.value.clone();

        div()
            .id("ccf_color_swatch")
            .flex()
            .flex_row()
            .gap_2()
            .items_center()
            .child(
                // Color preview box
                div()
                    .w(px(40.))
                    .h(px(32.))
                    .bg(color)
                    .border_1()
                    .border_color(rgb(theme.border_checkbox))
                    .rounded_md()
            )
            .child(
                // Hex color input
                div()
                    .id("ccf_color_hex_input")
                    .flex()
                    .flex_1()
                    .px_3()
                    .py_2()
                    .bg(rgb(theme.bg_input))
                    .border_1()
                    .border_color(if is_focused { rgb(theme.border_focus) } else { rgb(theme.border_input) })
                    .rounded_md()
                    .track_focus(&focus_handle)
                    .tab_stop(true)
                    // Focus navigation (Tab / Shift+Tab)
                    .on_action(cx.listener(|_this, _: &FocusNext, window, _cx| {
                        window.focus_next();
                    }))
                    .on_action(cx.listener(|_this, _: &FocusPrev, window, _cx| {
                        window.focus_prev();
                    }))
                    .on_key_down(cx.listener(|swatch, event: &KeyDownEvent, window, cx| {
                        match event.keystroke.key.as_str() {
                            "tab" => {
                                if event.keystroke.modifiers.shift {
                                    window.focus_prev();
                                } else {
                                    window.focus_next();
                                }
                                return;
                            }
                            "backspace" => {
                                swatch.value.pop();
                                cx.emit(ColorSwatchEvent::Change(swatch.value.clone()));
                            }
                            _ => {
                                if let Some(key_char) = event.keystroke.key_char.as_ref() {
                                    if key_char.chars().all(|c| c.is_ascii_hexdigit() || c == '#') {
                                        swatch.value.push_str(key_char);
                                        cx.emit(ColorSwatchEvent::Change(swatch.value.clone()));
                                    }
                                }
                            }
                        }
                        cx.notify();
                    }))
                    .child(
                        div()
                            .text_sm()
                            .when(value.is_empty(), |d| d.text_color(rgb(theme.text_dimmed)).child(placeholder))
                            .when(!value.is_empty(), |d| d.text_color(rgb(theme.text_value)).child(value))
                    )
            )
    }
}
