//! Toggle switch widget
//!
//! A toggle switch (on/off) control similar to iOS-style switches.
//! Supports keyboard interaction (Space/Enter to toggle) and mouse clicks.
//!
//! # Example
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::ToggleSwitch;
//!
//! let toggle = cx.new(|cx| {
//!     ToggleSwitch::new(cx)
//!         .with_enabled(true)
//!         .label("Enable notifications")
//! });
//!
//! // Subscribe to changes
//! cx.subscribe(&toggle, |this, _toggle, event: &ToggleSwitchEvent, cx| {
//!     if let ToggleSwitchEvent::Toggle(enabled) = event {
//!         println!("Toggle is now: {}", enabled);
//!     }
//! }).detach();
//! ```

use gpui::prelude::*;
use gpui::*;

use crate::theme::{get_theme_or, Theme};
use super::focus_navigation::{FocusNext, FocusPrev};

/// Position of the label relative to the toggle switch
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LabelPosition {
    /// Label appears to the left of the toggle
    Left,
    /// Label appears to the right of the toggle (default)
    #[default]
    Right,
}

/// Events emitted by ToggleSwitch
#[derive(Clone, Debug)]
pub enum ToggleSwitchEvent {
    /// Toggle state changed
    Toggle(bool),
}

/// Toggle switch widget
pub struct ToggleSwitch {
    enabled: bool,
    label: Option<SharedString>,
    label_position: LabelPosition,
    focus_handle: FocusHandle,
    custom_theme: Option<Theme>,
}

impl EventEmitter<ToggleSwitchEvent> for ToggleSwitch {}

impl Focusable for ToggleSwitch {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl ToggleSwitch {
    /// Create a new toggle switch
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            enabled: false,
            label: None,
            label_position: LabelPosition::default(),
            focus_handle: cx.focus_handle().tab_stop(true),
            custom_theme: None,
        }
    }

    /// Set initial enabled state (builder pattern)
    pub fn with_enabled(mut self, value: bool) -> Self {
        self.enabled = value;
        self
    }

    /// Set label text (builder pattern)
    pub fn label(mut self, text: impl Into<SharedString>) -> Self {
        self.label = Some(text.into());
        self
    }

    /// Set label position (builder pattern)
    pub fn label_position(mut self, position: LabelPosition) -> Self {
        self.label_position = position;
        self
    }

    /// Set custom theme (builder pattern)
    pub fn theme(mut self, theme: Theme) -> Self {
        self.custom_theme = Some(theme);
        self
    }

    /// Get current enabled state
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Set enabled state programmatically
    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.enabled != enabled {
            self.enabled = enabled;
            cx.emit(ToggleSwitchEvent::Toggle(enabled));
            cx.notify();
        }
    }

    /// Get the focus handle
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    fn toggle(&mut self, cx: &mut Context<Self>) {
        self.enabled = !self.enabled;
        cx.emit(ToggleSwitchEvent::Toggle(self.enabled));
        cx.notify();
    }
}

impl Render for ToggleSwitch {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = get_theme_or(cx, self.custom_theme.as_ref());
        let enabled = self.enabled;
        let label = self.label.clone();
        let label_position = self.label_position;
        let focus_handle = self.focus_handle.clone();
        let is_focused = self.focus_handle.is_focused(window);

        // Toggle dimensions
        let track_width = 44.0;
        let track_height = 24.0;
        let thumb_size = 18.0;
        let thumb_padding = 3.0;

        // Calculate thumb position (left edge when off, right edge when on)
        let thumb_left = if enabled {
            track_width - thumb_size - thumb_padding
        } else {
            thumb_padding
        };

        // Helper to create label element
        let make_label = |text: SharedString| {
            div()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(rgb(theme.text_label))
                .child(text)
        };

        // Helper to create toggle element
        let make_toggle = || {
            div()
                .w(px(track_width))
                .h(px(track_height))
                .rounded(px(track_height / 2.0)) // Pill shape
                .relative()
                .cursor_pointer()
                .when(enabled, |d| d.bg(rgb(theme.primary)))
                .when(!enabled, |d| d.bg(rgb(theme.bg_input)))
                .child(
                    // Thumb
                    div()
                        .absolute()
                        .top(px(thumb_padding))
                        .left(px(thumb_left))
                        .w(px(thumb_size))
                        .h(px(thumb_size))
                        .rounded_full()
                        .bg(rgb(theme.bg_white))
                        .shadow_sm()
                )
        };

        let mut container = div()
            .id("ccf_toggle_switch")
            .track_focus(&focus_handle)
            .tab_stop(true)
            // Focus navigation (Tab / Shift+Tab)
            .on_action(cx.listener(|_this, _: &FocusNext, window, _cx| {
                window.focus_next();
            }))
            .on_action(cx.listener(|_this, _: &FocusPrev, window, _cx| {
                window.focus_prev();
            }))
            .on_key_down(cx.listener(|toggle, event: &KeyDownEvent, window, cx| {
                match event.keystroke.key.as_str() {
                    "tab" => {
                        if event.keystroke.modifiers.shift {
                            window.focus_prev();
                        } else {
                            window.focus_next();
                        }
                    }
                    "space" | "enter" => {
                        toggle.toggle(cx);
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
            .on_mouse_down(MouseButton::Left, cx.listener(|toggle, _event, window, cx| {
                toggle.focus_handle.focus(window);
                toggle.toggle(cx);
            }));

        // Arrange label and toggle based on position
        match (label_position, label) {
            (LabelPosition::Left, Some(text)) => {
                container = container.child(make_label(text)).child(make_toggle());
            }
            (LabelPosition::Right, Some(text)) => {
                container = container.child(make_toggle()).child(make_label(text));
            }
            (_, None) => {
                container = container.child(make_toggle());
            }
        }

        container
    }
}
