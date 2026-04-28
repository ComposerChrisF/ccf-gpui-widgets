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

use super::focus_navigation::{repeatable_add_button, repeatable_remove_button};
use super::text_input::{TextInput, TextInputEvent};
use crate::theme::{get_theme, Theme};
use gpui::prelude::*;
use gpui::*;

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
    /// Prevents double-trigger when Space/Enter activates both on_action and on_click.
    /// See focus_navigation.rs module comment for details. DO NOT REMOVE.
    action_just_handled: bool,
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
            action_just_handled: false,
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
        })
        .detach();

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
            self.remove_focus_handles
                .push(cx.focus_handle().tab_stop(true));
        }
    }

    fn add_entry(&mut self, cx: &mut Context<Self>) {
        let index = self.entries.len();
        let entry = self.create_entry(None, cx);
        self.entries.push(entry);
        // Create a focus handle for the new remove button
        self.remove_focus_handles
            .push(cx.focus_handle().tab_stop(true));
        cx.emit(RepeatableTextInputEvent::EntryAdded(index));
        cx.emit(RepeatableTextInputEvent::Change(self.values(cx)));
        cx.notify();
    }

    fn remove_entry(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if self.entries.len() > self.min_entries && index < self.entries.len() {
            // Check if the remove button being pressed has focus
            let had_focus = self.remove_focus_handles[index].is_focused(window);

            self.entries.remove(index);
            self.remove_focus_handles.remove(index);

            // Move focus if the removed button had focus
            if had_focus {
                if self.entries.len() <= self.min_entries {
                    // No more "-" buttons visible, focus the first entry's input
                    self.entries[0].read(cx).focus_handle().focus(window);
                } else if index > 0 {
                    // Focus the previous entry's "-" button
                    self.remove_focus_handles[index - 1].focus(window);
                } else {
                    // Removed first entry, focus the new first entry's "-" button
                    self.remove_focus_handles[0].focus(window);
                }
            }

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
        let entry_data: Vec<_> = self
            .entries
            .iter()
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
                    .children(entry_data.into_iter().map(
                        |(index, entry, focus_handle, is_focused)| {
                            let remove_button = repeatable_remove_button(
                                format!("repeatable_remove_{}", index),
                                &focus_handle,
                                &theme,
                                enabled,
                                is_focused,
                                // on_action: set flag, then perform action
                                move |this: &mut Self, window, cx| {
                                    this.action_just_handled = true;
                                    this.remove_entry(index, window, cx);
                                },
                                // on_click: skip if action just handled, otherwise perform action
                                move |this: &mut Self, window, cx| {
                                    if this.action_just_handled {
                                        this.action_just_handled = false;
                                        return;
                                    }
                                    this.remove_entry(index, window, cx);
                                },
                                cx,
                            );

                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_2()
                                .child(
                                    // Input field
                                    div().flex_1().child(entry),
                                )
                                .when(can_remove, |d| d.child(remove_button))
                        },
                    )),
            )
            .child(
                // Add button row - use flex to align left
                div().flex().flex_row().child(repeatable_add_button(
                    "repeatable_add_button",
                    &self.add_focus_handle,
                    &theme,
                    enabled,
                    add_focused,
                    // on_action: set flag, then perform action
                    |this: &mut Self, _window, cx| {
                        this.action_just_handled = true;
                        this.add_entry(cx);
                    },
                    // on_click: skip if action just handled, otherwise perform action
                    |this: &mut Self, _window, cx| {
                        if this.action_just_handled {
                            this.action_just_handled = false;
                            return;
                        }
                        this.add_entry(cx);
                    },
                    cx,
                )),
            )
    }
}
