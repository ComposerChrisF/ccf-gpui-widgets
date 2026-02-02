//! Checkbox group widget
//!
//! A group of checkboxes for multi-selection from multiple choices.
//! Supports keyboard navigation (Up/Down arrows, Space to toggle).
//!
//! # Example
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::CheckboxGroup;
//!
//! let group = cx.new(|cx| {
//!     CheckboxGroup::new(cx)
//!         .choices(vec!["Red".to_string(), "Green".to_string(), "Blue".to_string()])
//!         .selected(vec!["Red".to_string(), "Blue".to_string()])
//! });
//!
//! // Subscribe to changes
//! cx.subscribe(&group, |this, _group, event: &CheckboxGroupEvent, cx| {
//!     if let CheckboxGroupEvent::Change(selected) = event {
//!         println!("Selected: {:?}", selected);
//!     }
//! }).detach();
//! ```

use std::collections::HashSet;

use gpui::prelude::*;
use gpui::*;

use crate::theme::{get_theme_or, Theme};
use super::focus_navigation::{FocusNext, FocusPrev, handle_tab_navigation};

/// Events emitted by CheckboxGroup
#[derive(Clone, Debug)]
pub enum CheckboxGroupEvent {
    /// Selection changed (contains all currently selected values)
    Change(Vec<String>),
}

/// Checkbox group widget for multi-selection
pub struct CheckboxGroup {
    choices: Vec<String>,
    selected: HashSet<String>,
    focus_handle: FocusHandle,
    highlight_index: usize,
    custom_theme: Option<Theme>,
    /// Whether the widget is enabled (interactive)
    enabled: bool,
}

impl EventEmitter<CheckboxGroupEvent> for CheckboxGroup {}

impl Focusable for CheckboxGroup {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl CheckboxGroup {
    /// Create a new checkbox group
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            choices: Vec::new(),
            selected: HashSet::new(),
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
        self
    }

    /// Set initially selected values (builder pattern)
    #[must_use]
    pub fn with_selected(mut self, selected: Vec<String>) -> Self {
        self.selected = selected.into_iter().collect();
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

    /// Get the currently selected values (sorted)
    pub fn get_selected(&self) -> Vec<String> {
        let mut result: Vec<String> = self.selected.iter().cloned().collect();
        result.sort();
        result
    }

    /// Check if a specific value is selected
    pub fn is_selected(&self, value: &str) -> bool {
        self.selected.contains(value)
    }

    /// Set selected values programmatically
    pub fn set_selected(&mut self, selected: Vec<String>, cx: &mut Context<Self>) {
        self.selected = selected.into_iter().collect();
        cx.emit(CheckboxGroupEvent::Change(self.get_selected()));
        cx.notify();
    }

    /// Get the focus handle
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    /// Check if the checkbox group is enabled
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

    fn toggle_choice(&mut self, choice: String, cx: &mut Context<Self>) {
        if self.selected.contains(&choice) {
            self.selected.remove(&choice);
        } else {
            self.selected.insert(choice);
        }
        cx.emit(CheckboxGroupEvent::Change(self.get_selected()));
    }
}

impl Render for CheckboxGroup {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = get_theme_or(cx, self.custom_theme.as_ref());
        let focus_handle = self.focus_handle.clone();
        let is_focused = self.focus_handle.is_focused(window);
        let highlight_index = self.highlight_index;
        let num_choices = self.choices.len();
        let enabled = self.enabled;

        div()
            .id("ccf_checkbox_group")
            .track_focus(&focus_handle)
            .tab_stop(enabled)
            // Focus navigation (Tab / Shift+Tab)
            .on_action(cx.listener(|_this, _: &FocusNext, window, _cx| {
                window.focus_next();
            }))
            .on_action(cx.listener(|_this, _: &FocusPrev, window, _cx| {
                window.focus_prev();
            }))
            .on_key_down(cx.listener(move |group, event: &KeyDownEvent, window, cx| {
                if !group.enabled {
                    return;
                }
                if handle_tab_navigation(event, window) {
                    return;
                }
                match event.keystroke.key.as_str() {
                    "up" => {
                        if group.highlight_index > 0 {
                            group.highlight_index -= 1;
                        } else if num_choices > 0 {
                            group.highlight_index = num_choices - 1;
                        }
                        cx.notify();
                    }
                    "down" => {
                        if group.highlight_index < num_choices.saturating_sub(1) {
                            group.highlight_index += 1;
                        } else {
                            group.highlight_index = 0;
                        }
                        cx.notify();
                    }
                    "space" => {
                        if let Some(choice) = group.choices.get(group.highlight_index).cloned() {
                            group.toggle_choice(choice, cx);
                        }
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
                let is_selected = self.selected.contains(choice);
                let is_highlighted = is_focused && idx == highlight_index && enabled;

                div()
                    .id(("ccf_checkbox_group_choice", idx))
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
                        d.on_click(cx.listener(move |group, _event, window, cx| {
                            group.focus_handle.focus(window);
                            group.highlight_index = idx;
                            group.toggle_choice(choice_clone.clone(), cx);
                            cx.notify();
                        }))
                    })
                    .child({
                        // Checkbox
                        let (bg_color, border_color, check_color) = if enabled {
                            if is_selected {
                                (theme.accent, theme.border_checkbox, theme.bg_white)
                            } else {
                                (theme.bg_input, theme.border_checkbox, 0)
                            }
                        } else if is_selected {
                            (theme.disabled_text, theme.disabled_text, theme.disabled_bg)
                        } else {
                            (theme.disabled_bg, theme.disabled_text, 0)
                        };

                        div()
                            .w(px(16.))
                            .h(px(16.))
                            .border_1()
                            .border_color(rgb(border_color))
                            .bg(rgb(bg_color))
                            .rounded(px(3.))
                            .when(is_selected, |d| {
                                d.child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .size_full()
                                        .text_color(rgb(check_color))
                                        .text_xs()
                                        .child("✓")
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
