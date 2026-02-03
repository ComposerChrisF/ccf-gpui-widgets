//! Segmented control widget
//!
//! A horizontal row of mutually exclusive button-style options.
//! Similar to iOS segmented controls or button groups in other UI frameworks.
//!
//! # Features
//!
//! - Horizontal layout of button-style segments
//! - Single selection (like radio buttons but compact horizontal style)
//! - Keyboard navigation (Left/Right arrows, Space/Enter to select)
//! - Themeable with focus ring support
//!
//! # Example
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::SegmentedControl;
//!
//! let control = cx.new(|cx| {
//!     SegmentedControl::new(cx)
//!         .options(vec![
//!             ("fit", "Fit to Window"),
//!             ("100", "100%"),
//!             ("200", "200%"),
//!         ])
//!         .with_selected("fit")
//! });
//!
//! // Subscribe to changes
//! cx.subscribe(&control, |this, _control, event: &SegmentedControlEvent, cx| {
//!     if let SegmentedControlEvent::Change(value) = event {
//!         println!("Selected: {}", value);
//!     }
//! }).detach();
//! ```

use gpui::prelude::*;
use gpui::*;

use crate::theme::{get_theme_or, Theme};
use super::focus_navigation::{handle_tab_navigation, with_focus_actions, EnabledCursorExt};

/// Events emitted by SegmentedControl
#[derive(Clone, Debug)]
pub enum SegmentedControlEvent {
    /// Selected value changed
    Change(String),
}

/// A single option in the segmented control
#[derive(Clone)]
pub struct SegmentOption {
    /// The value (used for identification and events)
    pub value: String,
    /// The display label
    pub label: String,
}

impl SegmentOption {
    /// Create a new option
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

/// Segmented control widget for single-selection from horizontal options
pub struct SegmentedControl {
    options: Vec<SegmentOption>,
    selected: String,
    focus_handle: FocusHandle,
    highlight_index: usize,
    custom_theme: Option<Theme>,
    enabled: bool,
}

impl EventEmitter<SegmentedControlEvent> for SegmentedControl {}

impl Focusable for SegmentedControl {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl SegmentedControl {
    /// Create a new segmented control
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            options: Vec::new(),
            selected: String::new(),
            focus_handle: cx.focus_handle().tab_stop(true),
            highlight_index: 0,
            custom_theme: None,
            enabled: true,
        }
    }

    /// Set options from a slice of (value, label) tuples (builder pattern)
    #[must_use]
    pub fn options(mut self, options: Vec<(&str, &str)>) -> Self {
        self.options = options
            .into_iter()
            .map(|(v, l)| SegmentOption::new(v, l))
            .collect();
        if !self.options.is_empty() && self.selected.is_empty() {
            self.selected = self.options[0].value.clone();
        }
        self
    }

    /// Set options from SegmentOption structs (builder pattern)
    #[must_use]
    pub fn with_options(mut self, options: Vec<SegmentOption>) -> Self {
        self.options = options;
        if !self.options.is_empty() && self.selected.is_empty() {
            self.selected = self.options[0].value.clone();
        }
        self
    }

    /// Set selected value (builder pattern)
    #[must_use]
    pub fn with_selected(mut self, value: &str) -> Self {
        if let Some(index) = self.options.iter().position(|o| o.value == value) {
            self.selected = value.to_string();
            self.highlight_index = index;
        }
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

    /// Get the currently selected value
    pub fn selected(&self) -> &str {
        &self.selected
    }

    /// Set selected value programmatically
    pub fn set_selected(&mut self, value: &str, cx: &mut Context<Self>) {
        if let Some(index) = self.options.iter().position(|o| o.value == value) {
            if self.selected != value {
                self.selected = value.to_string();
                self.highlight_index = index;
                cx.emit(SegmentedControlEvent::Change(value.to_string()));
                cx.notify();
            }
        }
    }

    /// Get the focus handle
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    /// Check if enabled
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

    fn select_by_index(&mut self, cx: &mut Context<Self>) {
        if let Some(option) = self.options.get(self.highlight_index) {
            if self.selected != option.value {
                self.selected = option.value.clone();
                cx.emit(SegmentedControlEvent::Change(self.selected.clone()));
            }
        }
    }
}

impl Render for SegmentedControl {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = get_theme_or(cx, self.custom_theme.as_ref());
        let focus_handle = self.focus_handle.clone();
        let is_focused = self.focus_handle.is_focused(window);
        let highlight_index = self.highlight_index;
        let num_options = self.options.len();
        let enabled = self.enabled;

        with_focus_actions(
            div()
                .id("ccf_segmented_control")
                .track_focus(&focus_handle)
                .tab_stop(enabled),
            cx,
        )
        .on_key_down(cx.listener(move |control, event: &KeyDownEvent, window, cx| {
            if !control.enabled {
                return;
            }
            if handle_tab_navigation(event, window) {
                return;
            }
            match event.keystroke.key.as_str() {
                "left" => {
                    if control.highlight_index > 0 {
                        control.highlight_index -= 1;
                    } else if num_options > 0 {
                        control.highlight_index = num_options - 1;
                    }
                    control.select_by_index(cx);
                    cx.notify();
                }
                "right" => {
                    if control.highlight_index < num_options.saturating_sub(1) {
                        control.highlight_index += 1;
                    } else {
                        control.highlight_index = 0;
                    }
                    control.select_by_index(cx);
                    cx.notify();
                }
                "space" | "enter" => {
                    control.select_by_index(cx);
                    cx.notify();
                }
                _ => {}
            }
        }))
        .flex()
        .flex_row()
        .gap_1()
        .children(self.options.iter().enumerate().map(|(idx, option)| {
            let value = option.value.clone();
            let is_selected = self.selected == option.value;
            let is_highlighted = is_focused && idx == highlight_index && enabled;

            let mut segment = div()
                .id(("ccf_segment", idx))
                .px_3()
                .py_1()
                .rounded(px(4.0))
                .border_1()
                .text_sm()
                .cursor_for_enabled(enabled);

            // Apply styling based on state
            // Create semi-transparent version of focus color for selected background
            let selected_bg = (theme.border_focus << 8) | 0x22;

            segment = if !enabled {
                segment
                    .border_color(rgb(theme.disabled_bg))
                    .text_color(rgb(theme.disabled_text))
                    .bg(rgb(theme.disabled_bg))
            } else if is_selected {
                segment
                    .border_color(rgb(theme.border_focus))
                    .bg(rgba(selected_bg))
                    .text_color(rgb(theme.text_primary))
            } else if is_highlighted {
                segment
                    .border_color(rgb(theme.border_input))
                    .bg(rgb(theme.bg_hover))
                    .text_color(rgb(theme.text_primary))
            } else {
                segment
                    .border_color(rgb(theme.border_default))
                    .text_color(rgb(theme.text_value))
            };

            if enabled {
                segment = segment
                    .hover(|s| s.bg(rgb(theme.bg_hover)))
                    .on_mouse_down(MouseButton::Left, cx.listener(move |control, _event: &MouseDownEvent, window, cx| {
                        control.focus_handle.focus(window);
                        if let Some(index) = control.options.iter().position(|o| o.value == value) {
                            control.highlight_index = index;
                            control.select_by_index(cx);
                        }
                        cx.notify();
                    }));
            }

            segment.child(option.label.clone())
        }))
    }
}
