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
    /// Whether entries have been initialized
    initialized: bool,
    /// Minimum number of entries (cannot remove below this)
    min_entries: usize,
    custom_theme: Option<Theme>,
}

impl RepeatableTextInput {
    /// Create a new repeatable text input
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            placeholder: None,
            initial_values: Vec::new(),
            entries: Vec::new(),
            initialized: false,
            min_entries: 1,
            custom_theme: None,
        }
    }

    /// Set initial values
    pub fn with_values(mut self, values: Vec<String>) -> Self {
        self.initial_values = values;
        self
    }

    /// Set placeholder text for each input
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// Set minimum number of entries (default: 1)
    ///
    /// Users cannot remove entries below this count.
    pub fn min_entries(mut self, min: usize) -> Self {
        self.min_entries = min.max(1); // At least 1
        self
    }

    /// Set a custom theme for this widget
    pub fn theme(mut self, theme: Theme) -> Self {
        self.custom_theme = Some(theme);
        self
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
        let entry = cx.new(|cx| {
            let mut state = TextInput::new(cx);
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
        }
    }

    fn add_entry(&mut self, cx: &mut Context<Self>) {
        let index = self.entries.len();
        let entry = self.create_entry(None, cx);
        self.entries.push(entry);
        cx.emit(RepeatableTextInputEvent::EntryAdded(index));
        cx.emit(RepeatableTextInputEvent::Change(self.values(cx)));
        cx.notify();
    }

    fn remove_entry(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.entries.len() > self.min_entries && index < self.entries.len() {
            self.entries.remove(index);
            cx.emit(RepeatableTextInputEvent::EntryRemoved(index));
            cx.emit(RepeatableTextInputEvent::Change(self.values(cx)));
            cx.notify();
        }
    }

    fn get_theme(&self, cx: &App) -> Theme {
        self.custom_theme.clone().unwrap_or_else(|| get_theme(cx))
    }
}

impl EventEmitter<RepeatableTextInputEvent> for RepeatableTextInput {}

impl Render for RepeatableTextInput {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        // Initialize entries on first render
        self.initialize_entries(cx);

        let theme = self.get_theme(cx);
        let entries_count = self.entries.len();
        let can_remove = entries_count > self.min_entries;

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
                    .children(self.entries.iter().enumerate().map(|(index, entry)| {
                        let entry = entry.clone();

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
                            .when(can_remove, |d| {
                                d.child(
                                    // Remove button
                                    div()
                                        .id(SharedString::from(format!("repeatable_remove_{}", index)))
                                        .px_2()
                                        .py_1()
                                        .bg(rgb(theme.delete_bg))
                                        .rounded_md()
                                        .cursor_pointer()
                                        .hover(|d| d.bg(rgb(theme.delete_bg_hover)))
                                        .on_click(cx.listener(move |this, _event, _window, cx| {
                                            this.remove_entry(index, cx);
                                        }))
                                        .child(
                                            div()
                                                .text_sm()
                                                .text_color(rgb(theme.text_label))
                                                .child("\u{2212}") // Minus sign
                                        )
                                )
                            })
                    }))
            )
            .child(
                // Add button row - use flex to align left
                div()
                    .flex()
                    .flex_row()
                    .child(
                        div()
                            .id("repeatable_add_button")
                            .px_2()
                            .py_1()
                            .bg(rgb(theme.bg_input_hover))
                            .rounded_md()
                            .cursor_pointer()
                            .hover(|d| d.bg(rgb(theme.bg_hover)))
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.add_entry(cx);
                            }))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(theme.text_label))
                                    .child("+")
                            )
                    )
            )
    }
}
