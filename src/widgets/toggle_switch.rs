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
//!         .with_on(true)
//!         .label("Enable notifications")
//! });
//!
//! // Subscribe to changes
//! cx.subscribe(&toggle, |this, _toggle, event: &ToggleSwitchEvent, cx| {
//!     if let ToggleSwitchEvent::Toggle(is_on) = event {
//!         println!("Toggle is now: {}", is_on);
//!     }
//! }).detach();
//! ```

use gpui::prelude::*;
use gpui::*;

use crate::theme::{get_theme_or, Theme};
use super::focus_navigation::{handle_tab_navigation, with_focus_actions, EnabledCursorExt};

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
    /// Toggle state changed.
    /// The boolean indicates the new on/off state: `true` = on, `false` = off.
    Toggle(bool),
}

/// Toggle switch widget
pub struct ToggleSwitch {
    /// Whether the toggle is in the "on" state
    on: bool,
    label: Option<SharedString>,
    label_position: LabelPosition,
    focus_handle: FocusHandle,
    custom_theme: Option<Theme>,
    /// Whether the widget is enabled (interactive)
    enabled: bool,
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
            on: false,
            label: None,
            label_position: LabelPosition::default(),
            focus_handle: cx.focus_handle().tab_stop(true),
            custom_theme: None,
            enabled: true,
        }
    }

    /// Set initial on/off state (builder pattern)
    #[must_use]
    pub fn with_on(mut self, value: bool) -> Self {
        self.on = value;
        self
    }

    /// Set label text (builder pattern)
    #[must_use]
    pub fn label(mut self, text: impl Into<SharedString>) -> Self {
        self.label = Some(text.into());
        self
    }

    /// Set label position (builder pattern)
    #[must_use]
    pub fn label_position(mut self, position: LabelPosition) -> Self {
        self.label_position = position;
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

    /// Get current on/off state
    pub fn is_on(&self) -> bool {
        self.on
    }

    /// Set on/off state programmatically
    pub fn set_on(&mut self, on: bool, cx: &mut Context<Self>) {
        if self.on != on {
            self.on = on;
            cx.emit(ToggleSwitchEvent::Toggle(on));
            cx.notify();
        }
    }

    /// Get the focus handle
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    /// Check if the toggle switch is enabled
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

    fn toggle(&mut self, cx: &mut Context<Self>) {
        self.on = !self.on;
        cx.emit(ToggleSwitchEvent::Toggle(self.on));
        cx.notify();
    }
}

impl Render for ToggleSwitch {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = get_theme_or(cx, self.custom_theme.as_ref());
        let is_on = self.on;
        let label = self.label.clone();
        let label_position = self.label_position;
        let focus_handle = self.focus_handle.clone();
        let is_focused = self.focus_handle.is_focused(window);
        let enabled = self.enabled;

        // Toggle dimensions
        let track_width = 44.0;
        let track_height = 24.0;
        let thumb_size = 18.0;
        let thumb_padding = 3.0;

        // Calculate thumb position (left edge when off, right edge when on)
        let thumb_left = if is_on {
            track_width - thumb_size - thumb_padding
        } else {
            thumb_padding
        };

        // Helper to create label element
        let make_label = |text: SharedString| {
            div()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .when(enabled, |d| d.text_color(rgb(theme.text_label)))
                .when(!enabled, |d| d.text_color(rgb(theme.disabled_text)))
                .child(text)
        };

        // Helper to create toggle element
        let make_toggle = || {
            let (track_bg, thumb_bg) = if enabled {
                let track = if is_on { theme.primary } else { theme.bg_input };
                (track, theme.bg_white)
            } else {
                let track = if is_on { theme.disabled_text } else { theme.disabled_bg };
                (track, theme.disabled_bg)
            };

            div()
                .w(px(track_width))
                .h(px(track_height))
                .rounded(px(track_height / 2.0)) // Pill shape
                .relative()
                .bg(rgb(track_bg))
                .cursor_for_enabled(enabled)
                .child(
                    // Thumb
                    div()
                        .absolute()
                        .top(px(thumb_padding))
                        .left(px(thumb_left))
                        .w(px(thumb_size))
                        .h(px(thumb_size))
                        .rounded_full()
                        .bg(rgb(thumb_bg))
                        .when(enabled, |d| d.shadow_sm())
                )
        };

        let mut container = with_focus_actions(
            div()
                .id("ccf_toggle_switch")
                .track_focus(&focus_handle)
                .tab_stop(enabled),
            cx,
        )
        .on_key_down(cx.listener(move |toggle, event: &KeyDownEvent, window, cx| {
            if !toggle.enabled {
                return;
            }
            if handle_tab_navigation(event, window) {
                return;
            }
            if matches!(event.keystroke.key.as_str(), "space" | "enter") {
                toggle.toggle(cx);
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
        .border_color(if is_focused && enabled { rgb(theme.border_focus) } else { rgba(0x00000000) });

        if enabled {
            container = container.on_mouse_down(MouseButton::Left, cx.listener(|toggle, _event, window, cx| {
                toggle.focus_handle.focus(window);
                toggle.toggle(cx);
            }));
        }

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
