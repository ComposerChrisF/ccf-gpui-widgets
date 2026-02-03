//! Repeatable text input widget - allows multiple values with add/remove buttons
//!
//! A widget that manages a list of text inputs, allowing users to add and remove entries.
//! Useful for inputs that accept multiple values like tags, email addresses, etc.
//!
//! # Example
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::{RepeatableTextInput, RepeatableTextInputEvent};
//!
//! let input = cx.new(|cx| {
//!     RepeatableTextInput::new(cx)
//!         .with_values(vec!["value1".to_string(), "value2".to_string()])
//!         .placeholder("Enter value")
//!         .min_entries(1)
//! });
//!
//! cx.subscribe(&input, |this, _, event: &RepeatableTextInputEvent, cx| {
//!     match event {
//!         RepeatableTextInputEvent::Change(values) => {
//!             println!("Values: {:?}", values);
//!         }
//!         RepeatableTextInputEvent::EntryAdded(index) => {
//!             println!("Added entry at index {}", index);
//!         }
//!         RepeatableTextInputEvent::EntryRemoved(index) => {
//!             println!("Removed entry at index {}", index);
//!         }
//!     }
//! }).detach();
//! ```

use gpui::prelude::*;
use gpui::*;
use crate::theme::{get_theme, Theme};
use super::text_input::{TextInput, TextInputEvent};
use super::focus_navigation::{with_focus_actions, EnabledCursorExt};

// Actions for button activation
actions!(ccf_repeatable_text_input, [ActivateButton]);

/// Register key bindings for repeatable text input buttons
///
/// Call this once at application startup:
/// ```ignore
/// ccf_gpui_widgets::widgets::repeatable_text_input::register_keybindings(cx);
/// ```
pub fn register_keybindings(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("enter", ActivateButton, Some("CcfRepeatableButton")),
        KeyBinding::new("space", ActivateButton, Some("CcfRepeatableButton")),
    ]);
}

/// Events emitted by RepeatableTextInput
#[derive(Debug, Clone)]
pub enum RepeatableTextInputEvent {
    /// Values changed (includes all current values)
    Change(Vec<String>),
    /// A new entry was added at the given index
    EntryAdded(usize),
    /// An entry was removed from the given index
    EntryRemoved(usize),
}

/// Repeatable text input widget
///
/// Manages a dynamic list of text inputs with add/remove buttons.
pub struct RepeatableTextInput {
    placeholder: Option<SharedString>,
    /// Initial values to populate on first render
    initial_values: Vec<String>,
    /// Each entry is a TextInput entity
    entries: Vec<Entity<TextInput>>,
    /// Focus handles for remove buttons (one per entry)
    remove_focus_handles: Vec<FocusHandle>,
    /// Focus handle for the add button
    add_focus_handle: FocusHandle,
    /// Whether entries have been initialized
    initialized: bool,
    /// Minimum number of entries (cannot remove below this)
    min_entries: usize,
    custom_theme: Option<Theme>,
    /// Whether the widget is enabled (interactive)
    enabled: bool,
}

impl RepeatableTextInput {
    /// Create a new repeatable text input
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            placeholder: None,
            initial_values: Vec::new(),
            entries: Vec::new(),
            remove_focus_handles: Vec::new(),
            add_focus_handle: cx.focus_handle().tab_stop(true),
            initialized: false,
            min_entries: 1,
            custom_theme: None,
            enabled: true,
        }
    }

    /// Set initial values
    #[must_use]
    pub fn with_values(mut self, values: Vec<String>) -> Self {
        self.initial_values = values;
        self
    }

    /// Set placeholder text for each input
    #[must_use]
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// Set minimum number of entries (default: 1)
    ///
    /// Users cannot remove entries below this count.
    #[must_use]
    pub fn min_entries(mut self, min: usize) -> Self {
        self.min_entries = min.max(1); // At least 1
        self
    }

    /// Set a custom theme for this widget
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

    /// Check if the widget is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Set enabled state programmatically
    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.enabled != enabled {
            self.enabled = enabled;
            // Update child entries
            for entry in &self.entries {
                entry.update(cx, |e, cx| e.set_enabled(enabled, cx));
            }
            cx.notify();
        }
    }

    /// Get all non-empty values
    pub fn values(&self, cx: &App) -> Vec<String> {
        self.entries
            .iter()
            .map(|entry| entry.read(cx).content().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Create a new text input entry
    fn create_entry(&self, value: Option<&str>, cx: &mut Context<Self>) -> Entity<TextInput> {
        let placeholder = self.placeholder.clone();
        let enabled = self.enabled;
        let entry = cx.new(|cx| {
            let mut state = TextInput::new(cx).with_enabled(enabled);
            if let Some(ph) = placeholder {
                state = state.placeholder(ph);
            }
            state
        });

        if let Some(v) = value.filter(|s| !s.is_empty()) {
            entry.update(cx, |state, cx| {
                state.set_value(v, cx);
            });
        }

        // Subscribe to changes to emit our own change events
        cx.subscribe(&entry, |this, _input, event: &TextInputEvent, cx| {
            if matches!(event, TextInputEvent::Change) {
                // Re-read all values and emit change
                let values = this.values(cx);
                cx.emit(RepeatableTextInputEvent::Change(values));
            }
        }).detach();

        entry
    }

    /// Initialize entries from initial values (called during first render)
    fn initialize_entries(&mut self, cx: &mut Context<Self>) {
        if self.initialized {
            return;
        }
        self.initialized = true;

        let values = std::mem::take(&mut self.initial_values);
        // Always have at least min_entries entries
        let values = if values.is_empty() {
            vec![String::new(); self.min_entries]
        } else if values.len() < self.min_entries {
            let mut v = values;
            v.resize(self.min_entries, String::new());
            v
        } else {
            values
        };

        for value in values {
            let entry = self.create_entry(Some(&value), cx);
            self.entries.push(entry);
            // Create a focus handle for the remove button
            self.remove_focus_handles.push(cx.focus_handle().tab_stop(true));
        }
    }

    fn add_entry(&mut self, cx: &mut Context<Self>) {
        let index = self.entries.len();
        let entry = self.create_entry(None, cx);
        self.entries.push(entry);
        // Create a focus handle for the new remove button
        self.remove_focus_handles.push(cx.focus_handle().tab_stop(true));
        cx.emit(RepeatableTextInputEvent::EntryAdded(index));
        cx.emit(RepeatableTextInputEvent::Change(self.values(cx)));
        cx.notify();
    }

    fn remove_entry(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.entries.len() > self.min_entries && index < self.entries.len() {
            self.entries.remove(index);
            self.remove_focus_handles.remove(index);
            cx.emit(RepeatableTextInputEvent::EntryRemoved(index));
            cx.emit(RepeatableTextInputEvent::Change(self.values(cx)));
            cx.notify();
        }
    }

    fn get_theme(&self, cx: &App) -> Theme {
        self.custom_theme.unwrap_or_else(|| get_theme(cx))
    }
}

impl EventEmitter<RepeatableTextInputEvent> for RepeatableTextInput {}

impl Render for RepeatableTextInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        // Initialize entries on first render
        self.initialize_entries(cx);

        let theme = self.get_theme(cx);
        let entries_count = self.entries.len();
        let can_remove = entries_count > self.min_entries;
        let add_focused = self.add_focus_handle.is_focused(window);
        let enabled = self.enabled;

        // Collect entries with their remove button focus handles
        let entry_data: Vec<_> = self.entries.iter()
            .zip(self.remove_focus_handles.iter())
            .enumerate()
            .map(|(index, (entry, focus_handle))| {
                let is_focused = focus_handle.is_focused(window);
                (index, entry.clone(), focus_handle.clone(), is_focused)
            })
            .collect();

        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                // Entries list
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .children(entry_data.into_iter().map(|(index, entry, focus_handle, is_focused)| {
                        let mut remove_button = with_focus_actions(
                            div()
                                .id(SharedString::from(format!("repeatable_remove_{}", index)))
                                .key_context("CcfRepeatableButton")
                                .track_focus(&focus_handle)
                                .tab_stop(enabled),
                            cx,
                        )
                        .flex()
                        .items_center()
                        .justify_center()
                        .h(px(28.))
                        .w(px(28.))
                        .rounded_md()
                        .border_2()
                        .cursor_for_enabled(enabled)
                        .when(enabled, |d| {
                            d.bg(rgb(theme.delete_bg))
                                .hover(|d| d.bg(rgb(theme.delete_bg_hover)))
                                .border_color(if is_focused { rgb(theme.border_focus) } else { rgba(0x00000000) })
                        })
                        .when(!enabled, |d| {
                            d.bg(rgb(theme.disabled_bg))
                                .border_color(rgba(0x00000000))
                        })
                            .child(
                                div()
                                    .text_sm()
                                    .when(enabled, |d| d.text_color(rgb(theme.text_label)))
                                    .when(!enabled, |d| d.text_color(rgb(theme.disabled_text)))
                                    .child("\u{2212}") // Minus sign
                            );

                        if enabled {
                            remove_button = remove_button
                                .on_action(cx.listener(move |this, _: &ActivateButton, _window, cx| {
                                    this.remove_entry(index, cx);
                                }))
                                .on_click(cx.listener(move |this, _event, _window, cx| {
                                    this.remove_entry(index, cx);
                                }));
                        }

                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                // Input field
                                div()
                                    .flex_1()
                                    .child(entry)
                            )
                            .when(can_remove, |d| d.child(remove_button))
                    }))
            )
            .child(
                // Add button row - use flex to align left
                div()
                    .flex()
                    .flex_row()
                    .child({
                        // Add button - height matches text input (28px)
                        let mut add_button = with_focus_actions(
                            div()
                                .id("repeatable_add_button")
                                .key_context("CcfRepeatableButton")
                                .track_focus(&self.add_focus_handle)
                                .tab_stop(enabled),
                            cx,
                        )
                        .flex()
                        .items_center()
                        .justify_center()
                        .h(px(28.))
                        .w(px(28.))
                        .rounded_md()
                        .border_2()
                        .cursor_for_enabled(enabled)
                        .when(enabled, |d| {
                            d.bg(rgb(theme.bg_input_hover))
                                .hover(|d| d.bg(rgb(theme.bg_hover)))
                                .border_color(if add_focused { rgb(theme.border_focus) } else { rgba(0x00000000) })
                        })
                        .when(!enabled, |d| {
                            d.bg(rgb(theme.disabled_bg))
                                .border_color(rgba(0x00000000))
                        })
                            .child(
                                div()
                                    .text_sm()
                                    .when(enabled, |d| d.text_color(rgb(theme.text_label)))
                                    .when(!enabled, |d| d.text_color(rgb(theme.disabled_text)))
                                    .child("+")
                            );

                        if enabled {
                            add_button = add_button
                                .on_action(cx.listener(|this, _: &ActivateButton, _window, cx| {
                                    this.add_entry(cx);
                                }))
                                .on_click(cx.listener(|this, _event, _window, cx| {
                                    this.add_entry(cx);
                                }));
                        }

                        add_button
                    })
            )
    }
}
