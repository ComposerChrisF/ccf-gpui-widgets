//! Radio group widget
//!
//! A group of radio buttons for single-selection from multiple choices.
//! Supports keyboard navigation (Up/Down arrows, Space to select).
//!
//! # Generic Selection
//!
//! RadioGroup is generic over any type implementing `SelectionItem`. For simple string
//! selections, use the `choices()` builder which creates `StringItem` instances internally.
//!
//! ## Example: Using choices() for strings (backward compatible)
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::RadioGroup;
//!
//! let radio = cx.new(|cx| {
//!     RadioGroup::new(cx)
//!         .choices(vec!["Small".to_string(), "Medium".to_string(), "Large".to_string()])
//!         .with_selected_index(1)
//! });
//!
//! // Subscribe to changes
//! cx.subscribe(&radio, |this, _radio, event: &RadioGroupEvent, cx| {
//!     if let RadioGroupEvent::Change(item) = event {
//!         println!("Selected: {}", item.value());
//!     }
//! }).detach();
//! ```
//!
//! ## Example: Using custom SelectionItem types
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::{RadioGroup, SelectionItem};
//! use gpui::*;
//!
//! #[derive(Clone, PartialEq)]
//! enum Size { Small, Medium, Large }
//!
//! impl SelectionItem for Size {
//!     fn label(&self) -> SharedString { /* ... */ }
//!     fn id(&self) -> ElementId { /* ... */ }
//! }
//!
//! let radio = cx.new(|cx| {
//!     RadioGroup::new_with_items(
//!         vec![Size::Small, Size::Medium, Size::Large],
//!         Size::Medium,
//!         cx,
//!     )
//! });
//! ```
//!
//! # API Changes (2025-02)
//!
//! Previously used String directly; now generic over SelectionItem.
//! - `with_selected_value(&str)` → `with_selected(T)` or `with_selected_index(usize)`
//! - `selected()` now returns `&T` instead of `&str`
//! - Event `Change(String)` → `Change(T)`
//! - Use `choices()` for backward-compatible string selection

use gpui::prelude::*;
use gpui::*;

use crate::theme::{get_theme_or, Theme};
use super::focus_navigation::{handle_tab_navigation, with_focus_actions, EnabledCursorExt};
use super::selection::{SelectionItem, StringItem};

/// Events emitted by RadioGroup
#[derive(Clone, Debug)]
pub enum RadioGroupEvent<T: SelectionItem> {
    /// Selected value changed
    Change(T),
}

/// Radio group widget for single-selection
///
/// Generic over `T: SelectionItem`. For simple string selections, use the default
/// `RadioGroup<StringItem>` via the `choices()` builder.
pub struct RadioGroup<T: SelectionItem = StringItem> {
    items: Vec<T>,
    selected: T,
    focus_handle: FocusHandle,
    highlight_index: usize,
    custom_theme: Option<Theme>,
    /// Whether the widget is enabled (interactive)
    enabled: bool,
}

impl<T: SelectionItem> EventEmitter<RadioGroupEvent<T>> for RadioGroup<T> {}

impl<T: SelectionItem> Focusable for RadioGroup<T> {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl RadioGroup<StringItem> {
    /// Create a new radio group for string selections
    ///
    /// Use `choices()` to set the available options.
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            items: Vec::new(),
            selected: StringItem::new(""),
            focus_handle: cx.focus_handle().tab_stop(true),
            highlight_index: 0,
            custom_theme: None,
            enabled: true,
        }
    }

    /// Set choices from strings (builder pattern)
    ///
    /// This is the backward-compatible API for string-based selections.
    #[must_use]
    pub fn choices(mut self, choices: Vec<String>) -> Self {
        self.items = choices.into_iter().map(StringItem::new).collect();
        if !self.items.is_empty() && self.selected.value().is_empty() {
            self.selected = self.items[0].clone();
        }
        self
    }

    /// Set selected value by string (builder pattern)
    ///
    /// For use with `choices()` API.
    #[must_use]
    pub fn with_selected_value(mut self, value: &str) -> Self {
        if let Some(index) = self.items.iter().position(|c| c.value() == value) {
            self.selected = self.items[index].clone();
            self.highlight_index = index;
        }
        self
    }

    /// Set selected by string value programmatically
    ///
    /// For use with `choices()` API. Emits Change event if value changes.
    pub fn set_selected_value(&mut self, value: &str, cx: &mut Context<Self>) {
        if let Some(index) = self.items.iter().position(|c| c.value() == value) {
            if self.selected.value() != value {
                self.selected = self.items[index].clone();
                self.highlight_index = index;
                cx.emit(RadioGroupEvent::Change(self.selected.clone()));
                cx.notify();
            }
        }
    }

    /// Get the selected value as a string
    ///
    /// Convenience method for string-based radio groups.
    pub fn selected_value(&self) -> &str {
        self.selected.value()
    }
}

impl<T: SelectionItem> RadioGroup<T> {
    /// Create a new radio group with items and initial selection
    pub fn new_with_items(items: Vec<T>, selected: T, cx: &mut Context<Self>) -> Self {
        let highlight_index = items.iter().position(|i| *i == selected).unwrap_or(0);
        Self {
            items,
            selected,
            focus_handle: cx.focus_handle().tab_stop(true),
            highlight_index,
            custom_theme: None,
            enabled: true,
        }
    }

    /// Set items (builder pattern)
    #[must_use]
    pub fn with_items(mut self, items: Vec<T>) -> Self {
        self.items = items;
        if !self.items.is_empty() {
            self.selected = self.items[0].clone();
            self.highlight_index = 0;
        }
        self
    }

    /// Set selected item (builder pattern)
    #[must_use]
    pub fn with_selected(mut self, item: T) -> Self {
        if let Some(index) = self.items.iter().position(|i| *i == item) {
            self.selected = item;
            self.highlight_index = index;
        }
        self
    }

    /// Set selected by index (builder pattern)
    #[must_use]
    pub fn with_selected_index(mut self, index: usize) -> Self {
        if let Some(item) = self.items.get(index) {
            self.selected = item.clone();
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

    /// Get the currently selected item
    pub fn selected(&self) -> &T {
        &self.selected
    }

    /// Get the currently selected index
    pub fn selected_index(&self) -> usize {
        self.items.iter().position(|i| *i == self.selected).unwrap_or(0)
    }

    /// Set selected item programmatically
    ///
    /// Emits Change event if selection changes.
    pub fn set_selected(&mut self, item: T, cx: &mut Context<Self>) {
        if let Some(index) = self.items.iter().position(|i| *i == item) {
            if self.selected != item {
                self.selected = item;
                self.highlight_index = index;
                cx.emit(RadioGroupEvent::Change(self.selected.clone()));
                cx.notify();
            }
        }
    }

    /// Set selected by index programmatically
    ///
    /// Emits Change event if selection changes.
    pub fn set_selected_index(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(item) = self.items.get(index).cloned() {
            if self.selected != item {
                self.selected = item;
                self.highlight_index = index;
                cx.emit(RadioGroupEvent::Change(self.selected.clone()));
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
        if let Some(item) = self.items.get(self.highlight_index) {
            if self.selected != *item {
                self.selected = item.clone();
                cx.emit(RadioGroupEvent::Change(self.selected.clone()));
            }
        }
    }
}

impl<T: SelectionItem> Render for RadioGroup<T> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = get_theme_or(cx, self.custom_theme.as_ref());
        let focus_handle = self.focus_handle.clone();
        let is_focused = self.focus_handle.is_focused(window);
        let highlight_index = self.highlight_index;
        let num_items = self.items.len();
        let enabled = self.enabled;

        with_focus_actions(
            div()
                .id("ccf_radio_group")
                .track_focus(&focus_handle)
                .tab_stop(enabled),
            cx,
        )
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
                        } else if num_items > 0 {
                            radio_group.highlight_index = num_items - 1;
                        }
                        radio_group.select_by_index(cx);
                        cx.notify();
                    }
                    "down" => {
                        if radio_group.highlight_index < num_items.saturating_sub(1) {
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
            .children(self.items.iter().enumerate().map(|(idx, item)| {
                let item_clone = item.clone();
                let is_selected = self.selected == *item;
                let is_highlighted = is_focused && idx == highlight_index && enabled;

                div()
                    .id(item.id())
                    .flex()
                    .flex_row()
                    .gap_2()
                    .items_center()
                    .py_1()
                    .px_1()
                    .cursor_for_enabled(enabled)
                    .rounded_sm()
                    .when(is_highlighted, |d| d.bg(rgb(theme.bg_input_hover)))
                    .when(!is_highlighted && enabled, |d| d.hover(|d| d.bg(rgb(theme.bg_input_hover))))
                    .when(enabled, |d| {
                        d.on_click(cx.listener(move |radio_group, _event, window, cx| {
                            radio_group.focus_handle.focus(window);
                            radio_group.selected = item_clone.clone();
                            radio_group.highlight_index = idx;
                            cx.emit(RadioGroupEvent::Change(item_clone.clone()));
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
                            .child(item.label())
                    )
            }))
    }
}
