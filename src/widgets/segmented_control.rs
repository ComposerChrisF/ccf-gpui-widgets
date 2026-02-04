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
//! # Generic Selection
//!
//! SegmentedControl is generic over any type implementing `SelectionItem`.
//! For convenience, `SegmentOption` implements `SelectionItem` and provides
//! a simple value/label pair for string-based selections.
//!
//! ## Example: Using options() for simple selections
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
//!         .with_selected_index(0)
//! });
//!
//! // Subscribe to changes
//! cx.subscribe(&control, |this, _control, event: &SegmentedControlEvent, cx| {
//!     if let SegmentedControlEvent::Change(option) = event {
//!         println!("Selected: {} ({})", option.label, option.value);
//!     }
//! }).detach();
//! ```
//!
//! ## Example: Using custom SelectionItem types
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::{SegmentedControl, SelectionItem};
//!
//! #[derive(Clone, PartialEq)]
//! enum ZoomLevel { Fit, Hundred, TwoHundred }
//!
//! impl SelectionItem for ZoomLevel { /* ... */ }
//!
//! let control = cx.new(|cx| {
//!     SegmentedControl::new_with_items(
//!         vec![ZoomLevel::Fit, ZoomLevel::Hundred, ZoomLevel::TwoHundred],
//!         ZoomLevel::Fit,
//!         cx,
//!     )
//! });
//! ```
//!
//! # API Changes (2025-02)
//!
//! Previously used String for selection; now generic over SelectionItem.
//! - `with_selected(&str)` → `with_selected(T)` or `with_selected_index(usize)`
//! - `selected()` now returns `&T` instead of `&str`
//! - Event `Change(String)` → `Change(T)`

use gpui::prelude::*;
use gpui::*;

use crate::theme::{get_theme_or, Theme};
use super::focus_navigation::{handle_tab_navigation, with_focus_actions, EnabledCursorExt};
use super::selection::SelectionItem;

/// Events emitted by SegmentedControl
#[derive(Clone, Debug)]
pub enum SegmentedControlEvent<T: SelectionItem> {
    /// Selected value changed
    Change(T),
}

/// A single option in the segmented control
///
/// Use with `options()` builder for simple value/label pair selections.
/// Implements `SelectionItem` for use with generic SegmentedControl.
#[derive(Clone, PartialEq, Debug)]
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

impl SelectionItem for SegmentOption {
    fn label(&self) -> SharedString {
        self.label.clone().into()
    }

    fn id(&self) -> ElementId {
        let id_str = format!("segment_{}", self.value.to_lowercase().replace(' ', "_"));
        ElementId::Name(id_str.into())
    }
}

/// Segmented control widget for single-selection from horizontal options
///
/// Generic over `T: SelectionItem`. For simple string-based selections,
/// use the default `SegmentedControl<SegmentOption>` via the `options()` builder.
pub struct SegmentedControl<T: SelectionItem = SegmentOption> {
    items: Vec<T>,
    selected: T,
    focus_handle: FocusHandle,
    highlight_index: usize,
    custom_theme: Option<Theme>,
    enabled: bool,
    /// Gap between segment buttons
    button_gap: Pixels,
}

impl<T: SelectionItem> EventEmitter<SegmentedControlEvent<T>> for SegmentedControl<T> {}

impl<T: SelectionItem> Focusable for SegmentedControl<T> {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl SegmentedControl<SegmentOption> {
    /// Create a new segmented control for SegmentOption selections
    ///
    /// Use `options()` to set the available options.
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            items: Vec::new(),
            selected: SegmentOption::new("", ""),
            focus_handle: cx.focus_handle().tab_stop(true),
            highlight_index: 0,
            custom_theme: None,
            enabled: true,
            button_gap: px(8.0),
        }
    }

    /// Set options from a slice of (value, label) tuples (builder pattern)
    #[must_use]
    pub fn options(mut self, options: Vec<(&str, &str)>) -> Self {
        self.items = options
            .into_iter()
            .map(|(v, l)| SegmentOption::new(v, l))
            .collect();
        if !self.items.is_empty() && self.selected.value.is_empty() {
            self.selected = self.items[0].clone();
        }
        self
    }

    /// Set options from SegmentOption structs (builder pattern)
    #[must_use]
    pub fn with_options(mut self, options: Vec<SegmentOption>) -> Self {
        self.items = options;
        if !self.items.is_empty() && self.selected.value.is_empty() {
            self.selected = self.items[0].clone();
        }
        self
    }

    /// Set selected value by string (builder pattern)
    ///
    /// For use with `options()` API. Matches by value field.
    #[must_use]
    pub fn with_selected_value(mut self, value: &str) -> Self {
        if let Some(index) = self.items.iter().position(|o| o.value == value) {
            self.selected = self.items[index].clone();
            self.highlight_index = index;
        }
        self
    }

    /// Set selected by string value programmatically
    ///
    /// For use with `options()` API. Emits Change event if value changes.
    pub fn set_selected_value(&mut self, value: &str, cx: &mut Context<Self>) {
        if let Some(index) = self.items.iter().position(|o| o.value == value) {
            if self.selected.value != value {
                self.selected = self.items[index].clone();
                self.highlight_index = index;
                cx.emit(SegmentedControlEvent::Change(self.selected.clone()));
                cx.notify();
            }
        }
    }

    /// Get the selected value as a string
    ///
    /// Convenience method for SegmentOption-based controls.
    pub fn selected_value(&self) -> &str {
        &self.selected.value
    }
}

impl<T: SelectionItem> SegmentedControl<T> {
    /// Create a new segmented control with items and initial selection
    pub fn new_with_items(items: Vec<T>, selected: T, cx: &mut Context<Self>) -> Self {
        let highlight_index = items.iter().position(|i| *i == selected).unwrap_or(0);
        Self {
            items,
            selected,
            focus_handle: cx.focus_handle().tab_stop(true),
            highlight_index,
            custom_theme: None,
            enabled: true,
            button_gap: px(8.0),
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

    /// Set gap between segment buttons (builder pattern)
    ///
    /// Default is 8px.
    #[must_use]
    pub fn with_button_gap(mut self, gap: impl Into<Pixels>) -> Self {
        self.button_gap = gap.into();
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
                cx.emit(SegmentedControlEvent::Change(self.selected.clone()));
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
                cx.emit(SegmentedControlEvent::Change(self.selected.clone()));
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
        if let Some(item) = self.items.get(self.highlight_index) {
            if self.selected != *item {
                self.selected = item.clone();
                cx.emit(SegmentedControlEvent::Change(self.selected.clone()));
            }
        }
    }
}

impl<T: SelectionItem> Render for SegmentedControl<T> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = get_theme_or(cx, self.custom_theme.as_ref());
        let focus_handle = self.focus_handle.clone();
        let is_focused = self.focus_handle.is_focused(window);
        let highlight_index = self.highlight_index;
        let num_items = self.items.len();
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
                    } else if num_items > 0 {
                        control.highlight_index = num_items - 1;
                    }
                    control.select_by_index(cx);
                    cx.notify();
                    cx.stop_propagation();
                }
                "right" => {
                    if control.highlight_index < num_items.saturating_sub(1) {
                        control.highlight_index += 1;
                    } else {
                        control.highlight_index = 0;
                    }
                    control.select_by_index(cx);
                    cx.notify();
                    cx.stop_propagation();
                }
                "space" | "enter" => {
                    control.select_by_index(cx);
                    cx.notify();
                    cx.stop_propagation();
                }
                _ => {}
            }
        }))
        .flex()
        .flex_row()
        .gap(self.button_gap)
        .children(self.items.iter().enumerate().map(|(idx, item)| {
            let item_clone = item.clone();
            let is_selected = self.selected == *item;
            let is_highlighted = is_focused && idx == highlight_index && enabled;

            let mut segment = div()
                .id(item.id())
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
                        if let Some(index) = control.items.iter().position(|i| *i == item_clone) {
                            control.highlight_index = index;
                            control.select_by_index(cx);
                        }
                        cx.notify();
                    }));
            }

            // Inner focus ring around text (border always present to prevent layout shift)
            segment.child(
                div()
                    .px_1()
                    .border_1()
                    .rounded_sm()
                    .when(is_highlighted, |d| d.border_color(rgb(theme.border_focus)))
                    .when(!is_highlighted, |d| d.border_color(rgba(0x00000000)))
                    .child(item.label())
            )
        }))
    }
}
