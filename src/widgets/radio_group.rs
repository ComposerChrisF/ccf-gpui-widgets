//! Radio group widget
//!
//! A group of radio buttons for single-selection from multiple choices.
//! Supports keyboard navigation (Up/Down arrows, Space to select).
//!
//! # Example
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::RadioGroup;
//!
//! let radio = cx.new(|cx| {
//!     RadioGroup::new(cx)
//!         .choices(vec!["Small".to_string(), "Medium".to_string(), "Large".to_string()])
//!         .with_selected_value("Medium")
//! });
//!
//! // Subscribe to changes
//! cx.subscribe(&radio, |this, _radio, event: &RadioGroupEvent, cx| {
//!     if let RadioGroupEvent::Change(value) = event {
//!         println!("Selected: {}", value);
//!     }
//! }).detach();
//! ```

use gpui::prelude::*;
use gpui::*;

use crate::theme::{get_theme_or, Theme};
use super::focus_navigation::{FocusNext, FocusPrev, handle_tab_navigation};

/// Events emitted by RadioGroup
#[derive(Clone, Debug)]
pub enum RadioGroupEvent {
    /// Selected value changed
    Change(String),
}

/// Radio group widget for single-selection
pub struct RadioGroup {
    choices: Vec<String>,
    selected: String,
    focus_handle: FocusHandle,
    highlight_index: usize,
    custom_theme: Option<Theme>,
    /// Whether the widget is enabled (interactive)
    enabled: bool,
}

impl EventEmitter<RadioGroupEvent> for RadioGroup {}

impl Focusable for RadioGroup {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl RadioGroup {
    /// Create a new radio group
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            choices: Vec::new(),
            selected: String::new(),
            focus_handle: cx.focus_handle().tab_stop(true),
            highlight_index: 0,
            custom_theme: None,
            enabled: true,
        }
    }

    /// Set choices (builder pattern)
    #[must_use]
    pub fn choices(mut self, choices: Vec<String>) -> Self {
        self.choices = choices;
        if !self.choices.is_empty() && self.selected.is_empty() {
            self.selected = self.choices[0].clone();
        }
        self
    }

    /// Set selected value (builder pattern)
    #[must_use]
    pub fn with_selected_value(mut self, value: &str) -> Self {
        if let Some(index) = self.choices.iter().position(|c| c == value) {
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
        if let Some(index) = self.choices.iter().position(|c| c == value) {
            if self.selected != value {
                self.selected = value.to_string();
                self.highlight_index = index;
                cx.emit(RadioGroupEvent::Change(value.to_string()));
                cx.notify();
            }
        }
    }

    /// Get the focus handle
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    /// Check if the radio group is enabled
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
        if let Some(choice) = self.choices.get(self.highlight_index) {
            if self.selected != *choice {
                self.selected = choice.clone();
                cx.emit(RadioGroupEvent::Change(self.selected.clone()));
            }
        }
    }
}

impl Render for RadioGroup {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = get_theme_or(cx, self.custom_theme.as_ref());
        let focus_handle = self.focus_handle.clone();
        let is_focused = self.focus_handle.is_focused(window);
        let highlight_index = self.highlight_index;
        let num_choices = self.choices.len();
        let enabled = self.enabled;

        div()
            .id("ccf_radio_group")
            .track_focus(&focus_handle)
            .tab_stop(enabled)
            // Focus navigation (Tab / Shift+Tab)
            .on_action(cx.listener(|_this, _: &FocusNext, window, _cx| {
                window.focus_next();
            }))
            .on_action(cx.listener(|_this, _: &FocusPrev, window, _cx| {
                window.focus_prev();
            }))
            .on_key_down(cx.listener(move |radio_group, event: &KeyDownEvent, window, cx| {
                if !radio_group.enabled {
                    return;
                }
                if handle_tab_navigation(event, window) {
                    return;
                }
                match event.keystroke.key.as_str() {
                    "up" => {
                        if radio_group.highlight_index > 0 {
                            radio_group.highlight_index -= 1;
                        } else if num_choices > 0 {
                            radio_group.highlight_index = num_choices - 1;
                        }
                        radio_group.select_by_index(cx);
                        cx.notify();
                    }
                    "down" => {
                        if radio_group.highlight_index < num_choices.saturating_sub(1) {
                            radio_group.highlight_index += 1;
                        } else {
                            radio_group.highlight_index = 0;
                        }
                        radio_group.select_by_index(cx);
                        cx.notify();
                    }
                    "space" => {
                        radio_group.select_by_index(cx);
                        cx.notify();
                    }
                    _ => {}
                }
            }))
            .flex()
            .flex_col()
            .gap_1()
            .p_2()
            .when(enabled, |d| d.bg(rgb(theme.bg_input)))
            .when(!enabled, |d| d.bg(rgb(theme.disabled_bg)))
            .border_1()
            .when(enabled, |d| {
                d.border_color(if is_focused { rgb(theme.border_focus) } else { rgb(theme.border_input) })
            })
            .when(!enabled, |d| d.border_color(rgb(theme.disabled_bg)))
            .rounded_md()
            .children(self.choices.iter().enumerate().map(|(idx, choice)| {
                let choice_clone = choice.clone();
                let is_selected = self.selected == *choice;
                let is_highlighted = is_focused && idx == highlight_index && enabled;

                div()
                    .id(("ccf_radio_choice", idx))
                    .flex()
                    .flex_row()
                    .gap_2()
                    .items_center()
                    .py_1()
                    .px_1()
                    .when(enabled, |d| d.cursor_pointer())
                    .when(!enabled, |d| d.cursor_default())
                    .rounded_sm()
                    .when(is_highlighted, |d| d.bg(rgb(theme.bg_input_hover)))
                    .when(!is_highlighted && enabled, |d| d.hover(|d| d.bg(rgb(theme.bg_input_hover))))
                    .when(enabled, |d| {
                        d.on_click(cx.listener(move |radio_group, _event, window, cx| {
                            radio_group.focus_handle.focus(window);
                            radio_group.selected = choice_clone.clone();
                            radio_group.highlight_index = idx;
                            cx.emit(RadioGroupEvent::Change(choice_clone.clone()));
                            cx.notify();
                        }))
                    })
                    .child({
                        // Radio button (circle)
                        let border_color = if enabled { theme.border_checkbox } else { theme.disabled_text };
                        let inner_color = if enabled { theme.accent } else { theme.disabled_text };

                        div()
                            .w(px(16.))
                            .h(px(16.))
                            .border_1()
                            .border_color(rgb(border_color))
                            .rounded(px(8.))
                            .when(is_selected, |d| {
                                d.child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .size_full()
                                        .child(
                                            div()
                                                .w(px(8.))
                                                .h(px(8.))
                                                .bg(rgb(inner_color))
                                                .rounded(px(4.))
                                        )
                                )
                            })
                    })
                    .child(
                        div()
                            .text_sm()
                            .when(enabled, |d| d.text_color(rgb(theme.text_value)))
                            .when(!enabled, |d| d.text_color(rgb(theme.disabled_text)))
                            .child(choice.clone())
                    )
            }))
    }
}
