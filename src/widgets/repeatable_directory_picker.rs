//! Repeatable directory picker widget - allows selecting multiple directories with add/remove buttons
//!
//! A widget that manages a list of directory path inputs, allowing users to add and remove entries.
//! Each entry is a full DirectoryPicker widget supporting folder browsing dialogs and drag-drop.
//!
//! Requires the `file-picker` feature.
//!
//! # Example
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::{
//!     RepeatableDirectoryPicker, RepeatableDirectoryPickerEvent
//! };
//!
//! let picker = cx.new(|cx| {
//!     RepeatableDirectoryPicker::new(cx)
//!         .with_values(vec!["/path/to/dir1".to_string()])
//!         .placeholder("Select directory...")
//!         .min_entries(1)
//! });
//!
//! cx.subscribe(&picker, |this, _, event: &RepeatableDirectoryPickerEvent, cx| {
//!     match event {
//!         RepeatableDirectoryPickerEvent::Change(values) => {
//!             println!("Directories: {:?}", values);
//!         }
//!         RepeatableDirectoryPickerEvent::EntryAdded(index) => {
//!             println!("Added entry at index {}", index);
//!         }
//!         RepeatableDirectoryPickerEvent::EntryRemoved(index) => {
//!             println!("Removed entry at index {}", index);
//!         }
//!     }
//! }).detach();
//! ```

use gpui::prelude::*;
use gpui::*;
use crate::theme::{get_theme, Theme};
use super::directory_picker::{
    DirectoryPicker, DirectoryPickerEvent,
    DirectoryPickerValidation, ValidationDisplay,
};
use super::focus_navigation::{with_focus_actions, EnabledCursorExt};
use super::repeatable_text_input::ActivateButton as RepeatableActivateButton;

/// Events emitted by RepeatableDirectoryPicker
#[derive(Debug, Clone)]
pub enum RepeatableDirectoryPickerEvent {
    /// Values changed (includes all current values)
    Change(Vec<String>),
    /// A new entry was added at the given index
    EntryAdded(usize),
    /// An entry was removed from the given index
    EntryRemoved(usize),
}

/// Repeatable directory picker widget
///
/// Manages a dynamic list of DirectoryPicker widgets with add/remove controls.
pub struct RepeatableDirectoryPicker {
    // Configuration (applied to all entries)
    placeholder: Option<SharedString>,
    browse_shortcut_enabled: bool,
    validation_display: ValidationDisplay,

    /// Initial values to populate on first render
    initial_values: Vec<String>,
    /// Each entry is a DirectoryPicker entity
    entries: Vec<Entity<DirectoryPicker>>,
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

impl RepeatableDirectoryPicker {
    /// Create a new repeatable directory picker
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            placeholder: None,
            browse_shortcut_enabled: true,
            validation_display: ValidationDisplay::default(),
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

    /// Set placeholder text
    #[must_use]
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// Enable or disable Cmd+O / Ctrl+O browse shortcut (builder pattern)
    ///
    /// The shortcut is enabled by default. Disable it if your application
    /// uses Cmd+O for another purpose (e.g., opening files at the app level).
    #[must_use]
    pub fn browse_shortcut(mut self, enabled: bool) -> Self {
        self.browse_shortcut_enabled = enabled;
        self
    }

    /// Set how validation feedback is displayed (builder pattern)
    ///
    /// Controls whether path coloring and/or explanation messages are shown.
    /// Default is `ValidationDisplay::Full` (show both).
    #[must_use]
    pub fn validation_display(mut self, display: ValidationDisplay) -> Self {
        self.validation_display = display;
        self
    }

    /// Set minimum number of entries (default: 1)
    #[must_use]
    pub fn min_entries(mut self, min: usize) -> Self {
        self.min_entries = min.max(1);
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
            .map(|entry| entry.read(cx).value().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Get access to the underlying DirectoryPicker entries
    pub fn entries(&self) -> &[Entity<DirectoryPicker>] {
        &self.entries
    }

    /// Validate all entries and return their validation states
    pub fn validate_all(&self, cx: &App) -> Vec<DirectoryPickerValidation> {
        self.entries
            .iter()
            .map(|entry| entry.read(cx).validate())
            .collect()
    }

    /// Returns true if all non-empty entries are valid
    pub fn is_all_valid(&self, cx: &App) -> bool {
        self.entries
            .iter()
            .all(|entry| {
                let picker = entry.read(cx);
                picker.value().is_empty() || picker.is_valid()
            })
    }

    /// Create a new DirectoryPicker entry with current configuration
    fn create_entry(&self, value: Option<&str>, cx: &mut Context<Self>) -> Entity<DirectoryPicker> {
        let placeholder = self.placeholder.clone();
        let browse_shortcut_enabled = self.browse_shortcut_enabled;
        let validation_display = self.validation_display.clone();
        let theme = self.custom_theme;
        let enabled = self.enabled;

        let picker = cx.new(|cx| {
            let mut p = DirectoryPicker::new(cx)
                .browse_shortcut(browse_shortcut_enabled)
                .validation_display(validation_display)
                .with_enabled(enabled);

            if let Some(ph) = placeholder {
                p = p.placeholder(ph);
            }
            if let Some(th) = theme {
                p = p.theme(th);
            }
            p
        });

        if let Some(v) = value.filter(|s| !s.is_empty()) {
            picker.update(cx, |p, cx| {
                p.set_value(v, cx);
            });
        }

        // Subscribe to changes to emit our own change events
        cx.subscribe(&picker, |this, _picker, event: &DirectoryPickerEvent, cx| {
            let DirectoryPickerEvent::Change(_) = event;
            let values = this.values(cx);
            cx.emit(RepeatableDirectoryPickerEvent::Change(values));
        }).detach();

        picker
    }

    /// Initialize entries from initial values (called during first render)
    fn initialize_entries(&mut self, cx: &mut Context<Self>) {
        if self.initialized {
            return;
        }
        self.initialized = true;

        let values = std::mem::take(&mut self.initial_values);
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
            self.remove_focus_handles.push(cx.focus_handle().tab_stop(true));
        }
    }

    fn add_entry(&mut self, cx: &mut Context<Self>) {
        let index = self.entries.len();
        let entry = self.create_entry(None, cx);
        self.entries.push(entry);
        self.remove_focus_handles.push(cx.focus_handle().tab_stop(true));
        cx.emit(RepeatableDirectoryPickerEvent::EntryAdded(index));
        cx.emit(RepeatableDirectoryPickerEvent::Change(self.values(cx)));
        cx.notify();
    }

    fn remove_entry(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.entries.len() > self.min_entries && index < self.entries.len() {
            self.entries.remove(index);
            self.remove_focus_handles.remove(index);
            cx.emit(RepeatableDirectoryPickerEvent::EntryRemoved(index));
            cx.emit(RepeatableDirectoryPickerEvent::Change(self.values(cx)));
            cx.notify();
        }
    }

    fn get_theme(&self, cx: &App) -> Theme {
        self.custom_theme.unwrap_or_else(|| get_theme(cx))
    }
}

impl EventEmitter<RepeatableDirectoryPickerEvent> for RepeatableDirectoryPicker {}

impl Render for RepeatableDirectoryPicker {
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
                                .id(SharedString::from(format!("dir_remove_{}", index)))
                                .key_context("CcfRepeatableButton")
                                .track_focus(&focus_handle)
                                .tab_stop(enabled),
                            cx,
                        )
                        .flex()
                        .items_center()
                        .justify_center()
                        .h(px(52.)) // Match DirectoryPicker height
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
                                .on_action(cx.listener(move |this, _: &RepeatableActivateButton, _window, cx| {
                                    this.remove_entry(index, cx);
                                }))
                                .on_click(cx.listener(move |this, _event, _window, cx| {
                                    this.remove_entry(index, cx);
                                }));
                        }

                        div()
                            .flex()
                            .flex_row()
                            .items_start()
                            .gap_2()
                            .child(
                                // DirectoryPicker widget
                                div()
                                    .flex_1()
                                    .child(entry)
                            )
                            .when(can_remove, |d| d.child(remove_button))
                    }))
            )
            .child(
                // Add button row
                div()
                    .flex()
                    .flex_row()
                    .child({
                        let mut add_button = with_focus_actions(
                            div()
                                .id("repeatable_dir_add_button")
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
                                .on_action(cx.listener(|this, _: &RepeatableActivateButton, _window, cx| {
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
