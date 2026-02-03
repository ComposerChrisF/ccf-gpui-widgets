//! Repeatable file picker widget - allows selecting multiple files with add/remove buttons
//!
//! A widget that manages a list of file path inputs, allowing users to add and remove entries.
//! Each entry is a full FilePicker widget supporting file browsing dialogs and drag-drop.
//!
//! # Example
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::{
//!     RepeatableFilePicker, RepeatableFilePickerEvent, FileMode, MissingDirectories
//! };
//!
//! let picker = cx.new(|cx| {
//!     RepeatableFilePicker::new(cx)
//!         .with_values(vec!["/path/to/file1.txt".to_string()])
//!         .mode(FileMode::Open)
//!         .extensions(vec!["txt".to_string(), "md".to_string()])
//!         .min_entries(1)
//! });
//!
//! cx.subscribe(&picker, |this, _, event: &RepeatableFilePickerEvent, cx| {
//!     match event {
//!         RepeatableFilePickerEvent::Change(values) => {
//!             println!("Files: {:?}", values);
//!         }
//!         RepeatableFilePickerEvent::EntryAdded(index) => {
//!             println!("Added entry at index {}", index);
//!         }
//!         RepeatableFilePickerEvent::EntryRemoved(index) => {
//!             println!("Removed entry at index {}", index);
//!         }
//!     }
//! }).detach();
//! ```

use gpui::prelude::*;
use gpui::*;
use std::path::PathBuf;
use crate::theme::{get_theme, Theme};
use super::file_picker::{
    FilePicker, FilePickerEvent, FileMode, MissingDirectories,
    FilePickerValidation, ValidationDisplay,
};
use super::focus_navigation::{with_focus_actions, EnabledCursorExt};
use super::repeatable_text_input::ActivateButton as RepeatableActivateButton;

/// Events emitted by RepeatableFilePicker
#[derive(Debug, Clone)]
pub enum RepeatableFilePickerEvent {
    /// Values changed (includes all current values)
    Change(Vec<String>),
    /// A new entry was added at the given index
    EntryAdded(usize),
    /// An entry was removed from the given index
    EntryRemoved(usize),
}

/// Repeatable file picker widget
///
/// Manages a dynamic list of FilePicker widgets with add/remove controls.
pub struct RepeatableFilePicker {
    // Configuration (applied to all entries)
    placeholder: Option<SharedString>,
    extensions: Vec<String>,
    mode: FileMode,
    missing_directories: MissingDirectories,
    browse_shortcut_enabled: bool,
    validation_display: ValidationDisplay,

    /// Initial values to populate on first render
    initial_values: Vec<String>,
    /// Each entry is a FilePicker entity
    entries: Vec<Entity<FilePicker>>,
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
    /// Whether an action was just handled (to prevent double-trigger from keyboard)
    action_just_handled: bool,
}

impl RepeatableFilePicker {
    /// Create a new repeatable file picker
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            placeholder: None,
            extensions: Vec::new(),
            mode: FileMode::Open,
            missing_directories: MissingDirectories::Error,
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
            action_just_handled: false,
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

    /// Set file extension filters (e.g., ["txt", "md"])
    #[must_use]
    pub fn extensions(mut self, exts: Vec<String>) -> Self {
        self.extensions = exts;
        self
    }

    /// Set file mode (Open or Save)
    #[must_use]
    pub fn mode(mut self, mode: FileMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set how missing directories are handled
    #[must_use]
    pub fn missing_directories(mut self, handling: MissingDirectories) -> Self {
        self.missing_directories = handling;
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

    /// Get access to the underlying FilePicker entries
    pub fn entries(&self) -> &[Entity<FilePicker>] {
        &self.entries
    }

    /// Validate all entries and return their validation states
    pub fn validate_all(&self, cx: &App) -> Vec<FilePickerValidation> {
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

    /// Returns all directories that need to be created (for Save mode with MissingDirectories::Create)
    pub fn directories_to_create(&self, cx: &App) -> Vec<PathBuf> {
        self.entries
            .iter()
            .filter_map(|entry| entry.read(cx).directory_to_create())
            .collect()
    }

    /// Create a new FilePicker entry with current configuration
    fn create_entry(&self, value: Option<&str>, cx: &mut Context<Self>) -> Entity<FilePicker> {
        let placeholder = self.placeholder.clone();
        let extensions = self.extensions.clone();
        let mode = self.mode.clone();
        let missing_directories = self.missing_directories.clone();
        let browse_shortcut_enabled = self.browse_shortcut_enabled;
        let validation_display = self.validation_display.clone();
        let theme = self.custom_theme;
        let enabled = self.enabled;

        let picker = cx.new(|cx| {
            let mut p = FilePicker::new(cx)
                .mode(mode)
                .extensions(extensions)
                .missing_directories(missing_directories)
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
        cx.subscribe(&picker, |this, _picker, event: &FilePickerEvent, cx| {
            let FilePickerEvent::Change(_) = event;
            let values = this.values(cx);
            cx.emit(RepeatableFilePickerEvent::Change(values));
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
        cx.emit(RepeatableFilePickerEvent::EntryAdded(index));
        cx.emit(RepeatableFilePickerEvent::Change(self.values(cx)));
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
                    // No more "-" buttons visible, focus the first entry's picker
                    self.entries[0].update(cx, |picker, cx| picker.focus(cx));
                } else if index > 0 {
                    // Focus the previous entry's "-" button
                    self.remove_focus_handles[index - 1].focus(window);
                } else {
                    // Removed first entry, focus the new first entry's "-" button
                    self.remove_focus_handles[0].focus(window);
                }
            }

            cx.emit(RepeatableFilePickerEvent::EntryRemoved(index));
            cx.emit(RepeatableFilePickerEvent::Change(self.values(cx)));
            cx.notify();
        }
    }

    fn get_theme(&self, cx: &App) -> Theme {
        self.custom_theme.unwrap_or_else(|| get_theme(cx))
    }
}

impl EventEmitter<RepeatableFilePickerEvent> for RepeatableFilePicker {}

impl Render for RepeatableFilePicker {
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
                                .id(SharedString::from(format!("file_remove_{}", index)))
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
                            // Draw minus sign with a horizontal line
                            .child({
                                let line_color = if enabled { theme.text_label } else { theme.disabled_text };
                                div()
                                    .w(px(10.))
                                    .h(px(2.))
                                    .rounded_sm()
                                    .bg(rgb(line_color))
                            });

                        if enabled {
                            remove_button = remove_button
                                .on_action(cx.listener(move |this, _: &RepeatableActivateButton, window, cx| {
                                    this.action_just_handled = true;
                                    this.remove_entry(index, window, cx);
                                }))
                                .on_click(cx.listener(move |this, _event, window, cx| {
                                    // Skip if this click is from keyboard (action already handled)
                                    if this.action_just_handled {
                                        this.action_just_handled = false;
                                        return;
                                    }
                                    this.remove_entry(index, window, cx);
                                }));
                        }

                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                // FilePicker widget
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
                                .id("repeatable_file_add_button")
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
                            // Draw plus sign with crossed lines
                            .child({
                                let line_color = if enabled { theme.text_label } else { theme.disabled_text };
                                div()
                                    .relative()
                                    .w(px(10.))
                                    .h(px(10.))
                                    // Horizontal line
                                    .child(
                                        div()
                                            .absolute()
                                            .top(px(4.))
                                            .left(px(0.))
                                            .w(px(10.))
                                            .h(px(2.))
                                            .rounded_sm()
                                            .bg(rgb(line_color))
                                    )
                                    // Vertical line
                                    .child(
                                        div()
                                            .absolute()
                                            .top(px(0.))
                                            .left(px(4.))
                                            .w(px(2.))
                                            .h(px(10.))
                                            .rounded_sm()
                                            .bg(rgb(line_color))
                                    )
                            });

                        if enabled {
                            add_button = add_button
                                .on_action(cx.listener(|this, _: &RepeatableActivateButton, _window, cx| {
                                    this.action_just_handled = true;
                                    this.add_entry(cx);
                                }))
                                .on_click(cx.listener(|this, _event, _window, cx| {
                                    // Skip if this click is from keyboard (action already handled)
                                    if this.action_just_handled {
                                        this.action_just_handled = false;
                                        return;
                                    }
                                    this.add_entry(cx);
                                }));
                        }

                        add_button
                    })
            )
    }
}
