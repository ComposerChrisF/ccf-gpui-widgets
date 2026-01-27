//! Dropdown widget
//!
//! A select/dropdown widget with keyboard navigation support.
//! Use `register_keybindings()` at app startup to enable keyboard shortcuts.
//!
//! # Example
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::Dropdown;
//!
//! // Register keybindings at app startup
//! ccf_gpui_widgets::widgets::dropdown::register_keybindings(cx);
//!
//! let dropdown = cx.new(|cx| {
//!     Dropdown::new(cx)
//!         .choices(vec!["Option 1".to_string(), "Option 2".to_string()])
//!         .selected_index(0)
//! });
//!
//! // Subscribe to changes
//! cx.subscribe(&dropdown, |this, _dropdown, event: &DropdownEvent, cx| {
//!     if let DropdownEvent::Change(value) = event {
//!         println!("Selected: {}", value);
//!     }
//! }).detach();
//! ```

use gpui::prelude::*;
use gpui::*;

use crate::theme::{get_theme_or, Theme};
use super::focus_navigation::{FocusNext, FocusPrev};

// Actions for keyboard navigation
actions!(ccf_dropdown, [CloseDropdown, SelectPrevious, SelectNext, ConfirmSelection, ToggleDropdown]);

/// Register key bindings for dropdown components
///
/// Call this once at application startup:
/// ```ignore
/// ccf_gpui_widgets::widgets::dropdown::register_keybindings(cx);
/// ```
pub fn register_keybindings(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("escape", CloseDropdown, Some("CcfDropdown")),
        KeyBinding::new("up", SelectPrevious, Some("CcfDropdown")),
        KeyBinding::new("down", SelectNext, Some("CcfDropdown")),
        KeyBinding::new("enter", ConfirmSelection, Some("CcfDropdown")),
        KeyBinding::new("space", ToggleDropdown, Some("CcfDropdown")),
    ]);
}

/// Events emitted by Dropdown
#[derive(Clone, Debug)]
pub enum DropdownEvent {
    /// Selected value changed
    Change(String),
    /// Dropdown opened
    Open,
    /// Dropdown closed
    Close,
}

/// Dropdown/Select widget
pub struct Dropdown {
    choices: Vec<String>,
    selected_index: usize,
    is_open: bool,
    focus_handle: FocusHandle,
    custom_theme: Option<Theme>,
}

impl EventEmitter<DropdownEvent> for Dropdown {}

impl Focusable for Dropdown {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Dropdown {
    /// Create a new dropdown
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            choices: Vec::new(),
            selected_index: 0,
            is_open: false,
            focus_handle: cx.focus_handle().tab_stop(true),
            custom_theme: None,
        }
    }

    /// Set choices (builder pattern)
    pub fn choices(mut self, choices: Vec<String>) -> Self {
        self.choices = choices;
        self
    }

    /// Set selected index (builder pattern)
    pub fn with_selected_index(mut self, index: usize) -> Self {
        self.selected_index = index.min(self.choices.len().saturating_sub(1));
        self
    }

    /// Set selected value by string (builder pattern)
    pub fn selected_value(mut self, value: &str) -> Self {
        if let Some(index) = self.choices.iter().position(|c| c == value) {
            self.selected_index = index;
        }
        self
    }

    /// Set custom theme (builder pattern)
    pub fn theme(mut self, theme: Theme) -> Self {
        self.custom_theme = Some(theme);
        self
    }

    /// Get the currently selected value
    pub fn selected(&self) -> &str {
        self.choices.get(self.selected_index).map(|s| s.as_str()).unwrap_or("")
    }

    /// Get the currently selected index
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Set selected index programmatically
    pub fn set_selected_index(&mut self, index: usize, cx: &mut Context<Self>) {
        let index = index.min(self.choices.len().saturating_sub(1));
        if self.selected_index != index {
            self.selected_index = index;
            if let Some(choice) = self.choices.get(index) {
                cx.emit(DropdownEvent::Change(choice.clone()));
            }
            cx.notify();
        }
    }

    /// Get the focus handle
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    fn select_by_offset(&mut self, offset: isize, cx: &mut Context<Self>) {
        let new_index = (self.selected_index as isize + offset)
            .clamp(0, self.choices.len().saturating_sub(1) as isize) as usize;
        if new_index != self.selected_index {
            self.selected_index = new_index;
            if let Some(choice) = self.choices.get(self.selected_index) {
                cx.emit(DropdownEvent::Change(choice.clone()));
            }
            cx.notify();
        }
    }

    fn select_previous(&mut self, cx: &mut Context<Self>) {
        self.select_by_offset(-1, cx);
    }

    fn select_next(&mut self, cx: &mut Context<Self>) {
        self.select_by_offset(1, cx);
    }

    fn close(&mut self, cx: &mut Context<Self>) {
        if self.is_open {
            self.is_open = false;
            cx.emit(DropdownEvent::Close);
            cx.notify();
        }
    }

    fn toggle(&mut self, cx: &mut Context<Self>) {
        self.is_open = !self.is_open;
        if self.is_open {
            cx.emit(DropdownEvent::Open);
        } else {
            cx.emit(DropdownEvent::Close);
        }
        cx.notify();
    }
}

impl Render for Dropdown {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = get_theme_or(cx, self.custom_theme.as_ref());
        let is_focused = self.focus_handle.is_focused(window);

        // Close dropdown if it's open but we lost focus
        if self.is_open && !is_focused {
            self.is_open = false;
        }

        let selected = self.choices
            .get(self.selected_index)
            .cloned()
            .unwrap_or_default();
        let is_open = self.is_open;
        let focus_handle = self.focus_handle.clone();

        let bg_white = theme.bg_white;
        let bg_light_hover = theme.bg_light_hover;
        let border_focus = theme.border_focus;
        let border_input = theme.border_input;
        let text_black = theme.text_black;
        let text_icon = theme.text_icon;
        let text_primary = theme.text_primary;
        let primary = theme.primary;

        div()
            .id("ccf_dropdown")
            .relative()
            .key_context("CcfDropdown")
            .track_focus(&focus_handle)
            .tab_stop(true)
            // Focus navigation (Tab / Shift+Tab)
            .on_action(cx.listener(|_this, _: &FocusNext, window, _cx| {
                window.focus_next();
            }))
            .on_action(cx.listener(|_this, _: &FocusPrev, window, _cx| {
                window.focus_prev();
            }))
            .on_action(cx.listener(|dropdown, _: &CloseDropdown, _window, cx| {
                dropdown.close(cx);
            }))
            .on_action(cx.listener(|dropdown, _: &SelectPrevious, _window, cx| {
                dropdown.select_previous(cx);
            }))
            .on_action(cx.listener(|dropdown, _: &SelectNext, _window, cx| {
                dropdown.select_next(cx);
            }))
            .on_action(cx.listener(|dropdown, _: &ConfirmSelection, _window, cx| {
                dropdown.close(cx);
            }))
            .on_action(cx.listener(|dropdown, _: &ToggleDropdown, window, cx| {
                dropdown.toggle(cx);
                dropdown.focus_handle.focus(window);
            }))
            .on_key_down(cx.listener(|_dropdown, event: &KeyDownEvent, window, _cx| {
                if event.keystroke.key == "tab" {
                    if event.keystroke.modifiers.shift {
                        window.focus_prev();
                    } else {
                        window.focus_next();
                    }
                }
            }))
            .child(
                // Dropdown button
                div()
                    .id("ccf_dropdown_button")
                    .flex()
                    .flex_row()
                    .justify_between()
                    .items_center()
                    .w_full()
                    .h(px(32.))
                    .px_3()
                    .border_3()
                    .border_color(if is_focused { rgb(border_focus) } else { rgb(bg_white) })
                    .rounded_md()
                    .cursor_pointer()
                    .bg(rgb(bg_white))
                    .text_sm()
                    .text_color(rgb(text_black))
                    .hover(|d| d.bg(rgb(bg_light_hover)))
                    .child(selected.clone())
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(text_icon))
                            .child("▼")
                    )
                    .on_click(cx.listener(move |dropdown, _event, window, cx| {
                        dropdown.toggle(cx);
                        dropdown.focus_handle.focus(window);
                    }))
            )
            .when(is_open, |parent| {
                let selected_index = self.selected_index;
                let choices_list: Vec<_> = self.choices.iter().enumerate().map(|(i, choice)| {
                    let is_selected = i == selected_index;
                    let choice_clone = choice.clone();

                    div()
                        .id(("ccf_dropdown_choice", i))
                        .px_3()
                        .py_2()
                        .cursor_pointer()
                        .text_sm()
                        .when(is_selected, |d| {
                            d.bg(rgb(primary)).text_color(rgb(text_primary))
                        })
                        .when(!is_selected, |d| {
                            d.text_color(rgb(text_black))
                                .hover(|d| d.bg(rgb(bg_light_hover)))
                        })
                        .child(choice.clone())
                        // Use on_mouse_down to handle selection immediately and prevent click-through
                        .on_mouse_down(MouseButton::Left, cx.listener(move |dropdown, _event, _window, cx| {
                            dropdown.selected_index = i;
                            dropdown.is_open = false;
                            cx.emit(DropdownEvent::Change(choice_clone.clone()));
                            cx.emit(DropdownEvent::Close);
                            cx.notify();
                        }))
                }).collect();

                parent.child(
                    deferred(
                        anchored()
                            .anchor(Corner::TopLeft)
                            .child(
                                div()
                                    .id("ccf_dropdown_menu")
                                    .occlude()  // Block all mouse events from reaching elements below
                                    .absolute()
                                    .top(px(2.))
                                    .left_0()
                                    .w_full()
                                    .min_w(px(200.))
                                    .border_1()
                                    .border_color(rgb(border_input))
                                    .rounded_md()
                                    .bg(rgb(bg_white))
                                    .max_h(px(200.))
                                    .overflow_y_scroll()
                                    .shadow_lg()
                                    .children(choices_list)
                                    .on_mouse_down_out(cx.listener(|dropdown, _event, _window, cx| {
                                        dropdown.is_open = false;
                                        cx.emit(DropdownEvent::Close);
                                        cx.notify();
                                    }))
                            )
                    )
                )
            })
    }
}
