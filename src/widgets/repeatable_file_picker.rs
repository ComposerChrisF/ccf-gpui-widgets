//! Repeatable file picker widget - allows selecting multiple files with add/remove buttons
//!
//! A widget that manages a list of file path inputs, allowing users to add and remove entries.
//! Each entry supports file browsing dialogs and drag-drop.
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
use std::path::Path;
use crate::theme::{get_theme, Theme};
use crate::utils::path::{parse_path, PathInfo};
use super::text_input::{TextInput, TextInputEvent};
use super::file_picker::{FileMode, MissingDirectories};
use super::focus_navigation::{FocusNext, FocusPrev};
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

/// A single file entry in the repeatable file picker
struct FileEntry {
    value: String,
    is_editing: bool,
    edit_state: Option<Entity<TextInput>>,
}

/// Repeatable file picker widget
///
/// Manages a dynamic list of file path inputs with browse buttons and add/remove controls.
pub struct RepeatableFilePicker {
    placeholder: Option<SharedString>,
    extensions: Vec<String>,
    mode: FileMode,
    missing_directories: MissingDirectories,
    /// Initial values to populate on first render
    initial_values: Vec<String>,
    /// Each entry represents one file path
    entries: Vec<FileEntry>,
    /// Focus handles for remove buttons (one per entry)
    remove_focus_handles: Vec<FocusHandle>,
    /// Focus handle for the add button
    add_focus_handle: FocusHandle,
    /// Whether entries have been initialized
    initialized: bool,
    /// Minimum number of entries (cannot remove below this)
    min_entries: usize,
    custom_theme: Option<Theme>,
}

impl RepeatableFilePicker {
    /// Create a new repeatable file picker
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            placeholder: None,
            extensions: Vec::new(),
            mode: FileMode::Open,
            missing_directories: MissingDirectories::Error,
            initial_values: Vec::new(),
            entries: Vec::new(),
            remove_focus_handles: Vec::new(),
            add_focus_handle: cx.focus_handle().tab_stop(true),
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

    /// Set placeholder text
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// Set file extension filters (e.g., ["txt", "md"])
    pub fn extensions(mut self, exts: Vec<String>) -> Self {
        self.extensions = exts;
        self
    }

    /// Set file mode (Open or Save)
    pub fn mode(mut self, mode: FileMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set how missing directories are handled
    pub fn missing_directories(mut self, handling: MissingDirectories) -> Self {
        self.missing_directories = handling;
        self
    }

    /// Set minimum number of entries (default: 1)
    pub fn min_entries(mut self, min: usize) -> Self {
        self.min_entries = min.max(1);
        self
    }

    /// Set a custom theme for this widget
    pub fn theme(mut self, theme: Theme) -> Self {
        self.custom_theme = Some(theme);
        self
    }

    /// Get all non-empty values
    pub fn values(&self) -> Vec<String> {
        self.entries
            .iter()
            .map(|entry| entry.value.clone())
            .filter(|s| !s.is_empty())
            .collect()
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
            self.entries.push(FileEntry {
                value,
                is_editing: false,
                edit_state: None,
            });
            // Create a focus handle for the remove button
            self.remove_focus_handles.push(cx.focus_handle().tab_stop(true));
        }
    }

    fn add_entry(&mut self, cx: &mut Context<Self>) {
        let index = self.entries.len();
        self.entries.push(FileEntry {
            value: String::new(),
            is_editing: false,
            edit_state: None,
        });
        // Create a focus handle for the new remove button
        self.remove_focus_handles.push(cx.focus_handle().tab_stop(true));
        cx.emit(RepeatableFilePickerEvent::EntryAdded(index));
        cx.emit(RepeatableFilePickerEvent::Change(self.values()));
        cx.notify();
    }

    fn remove_entry(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.entries.len() > self.min_entries && index < self.entries.len() {
            self.entries.remove(index);
            self.remove_focus_handles.remove(index);
            cx.emit(RepeatableFilePickerEvent::EntryRemoved(index));
            cx.emit(RepeatableFilePickerEvent::Change(self.values()));
            cx.notify();
        }
    }

    fn start_editing(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if index >= self.entries.len() {
            return;
        }

        let entry = &mut self.entries[index];
        entry.is_editing = true;

        let current_value = entry.value.clone();
        let placeholder = self.placeholder.clone();
        let edit_state = cx.new(|cx| {
            let mut state = TextInput::new(cx);
            if let Some(ph) = placeholder {
                state = state.placeholder(ph);
            }
            state.select_on_focus = true;
            state
        });

        edit_state.update(cx, |state, cx| {
            state.set_value(&current_value, cx);
        });

        // Focus the input
        edit_state.read(cx).focus_handle(cx).focus(window);

        // Subscribe to input events for Enter/Blur
        let entry_index = index;
        cx.subscribe(&edit_state, move |this, state, event: &TextInputEvent, cx| {
            match event {
                TextInputEvent::Enter | TextInputEvent::Blur => {
                    if entry_index < this.entries.len() {
                        let text = state.read(cx).content().to_string();
                        let path_info = parse_path(&text);
                        this.entries[entry_index].value = path_info.full_path_string();
                        this.entries[entry_index].is_editing = false;
                        this.entries[entry_index].edit_state = None;
                        cx.emit(RepeatableFilePickerEvent::Change(this.values()));
                    }
                    cx.notify();
                }
                _ => {}
            }
        }).detach();

        entry.edit_state = Some(edit_state);
        cx.notify();
    }

    fn open_file_dialog(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if index >= self.entries.len() {
            return;
        }

        let extensions = self.extensions.clone();
        let entity = cx.entity().clone();
        let is_save_mode = self.mode == FileMode::Save;
        let current_value = self.entries[index].value.clone();

        // Determine initial directory for the dialog
        let initial_dir = if !current_value.is_empty() {
            let path = Path::new(&current_value);
            let parent = if path.is_dir() {
                Some(path.to_path_buf())
            } else {
                path.parent().map(|p| p.to_path_buf())
            };
            parent.filter(|p| p.exists())
        } else {
            None
        }.or_else(|| std::env::current_dir().ok());

        let entry_index = index;
        window.spawn(cx, async move |async_cx| {
            let result = async_cx.background_executor().spawn(async move {
                let mut dialog = rfd::AsyncFileDialog::new();

                if let Some(dir) = initial_dir {
                    dialog = dialog.set_directory(&dir);
                }

                if !extensions.is_empty() {
                    let ext_refs: Vec<&str> = extensions.iter().map(|s| s.as_str()).collect();
                    dialog = dialog.add_filter("Files", &ext_refs);
                }

                if is_save_mode {
                    dialog.save_file().await.map(|f| f.path().to_path_buf())
                } else {
                    dialog.pick_file().await.map(|f| f.path().to_path_buf())
                }
            }).await;

            if let Some(path) = result {
                let path_str = path.to_string_lossy().to_string();
                let _ = async_cx.update_entity(&entity, move |this: &mut RepeatableFilePicker, cx| {
                    if entry_index < this.entries.len() {
                        this.entries[entry_index].value = path_str;
                        cx.emit(RepeatableFilePickerEvent::Change(this.values()));
                        cx.notify();
                    }
                });
            }
        }).detach();
    }

    fn get_theme(&self, cx: &App) -> Theme {
        self.custom_theme.clone().unwrap_or_else(|| get_theme(cx))
    }

    fn render_entry(&mut self, index: usize, remove_focus_handle: FocusHandle, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.get_theme(cx);
        let entries_count = self.entries.len();
        let entry = &mut self.entries[index];
        let can_remove = entries_count > self.min_entries;
        let remove_is_focused = remove_focus_handle.is_focused(window);

        // Check if we're editing but the input lost focus
        if entry.is_editing {
            if let Some(edit_state) = &entry.edit_state {
                if !edit_state.read(cx).focus_handle(cx).is_focused(window) {
                    let text = edit_state.read(cx).content().to_string();
                    let path_info = parse_path(&text);
                    entry.value = path_info.full_path_string();
                    entry.is_editing = false;
                    entry.edit_state = None;
                }
            }
        }

        let path_info = parse_path(&entry.value);
        let path_display = compute_file_path_display(&path_info, &self.mode, &self.missing_directories, &theme);

        let basename = if !entry.value.is_empty() {
            Path::new(&entry.value)
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
        } else {
            None
        };

        let is_editing = entry.is_editing;
        let edit_state_clone = entry.edit_state.clone();
        let value_empty = entry.value.is_empty();
        let placeholder = self.placeholder.clone();

        // Get edit display info if editing
        let edit_display = if is_editing {
            edit_state_clone.as_ref().map(|state| {
                let text = state.read(cx).content().to_string();
                let edit_path_info = parse_path(&text);
                let display = compute_file_path_display(&edit_path_info, &self.mode, &self.missing_directories, &theme);
                (edit_path_info, display)
            })
        } else {
            None
        };

        let entry_id = SharedString::from(format!("file_entry_{}", index));

        div()
            .flex()
            .flex_row()
            .items_start()
            .gap_2()
            .child(
                // Path display area
                div()
                    .id(entry_id)
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .px_3()
                    .py_2()
                    .bg(rgb(theme.bg_input))
                    .rounded_md()
                    .border_1()
                    .border_color(rgb(theme.border_default))
                    .drag_over::<ExternalPaths>({
                        let hover_bg = theme.bg_input_hover;
                        let focus_border = theme.border_focus;
                        move |d, _, _, _| {
                            d.bg(rgb(hover_bg))
                                .border_color(rgb(focus_border))
                        }
                    })
                    .on_drop(cx.listener(move |this, paths: &ExternalPaths, _window, cx| {
                        if let Some(path) = paths.paths().first() {
                            if path.is_file() && index < this.entries.len() {
                                this.entries[index].value = path.to_string_lossy().to_string();
                                cx.emit(RepeatableFilePickerEvent::Change(this.values()));
                                cx.notify();
                            }
                        }
                    }))
                    // Empty state
                    .when(!is_editing && value_empty, |d| {
                        d.on_click(cx.listener(move |this, _event, window, cx| {
                            this.start_editing(index, window, cx);
                        }))
                        .cursor_pointer()
                        .hover(|d| d.bg(rgb(theme.bg_path_hover)))
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgb(theme.text_dimmed))
                                .child("No file selected")
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(theme.text_dimmed))
                                .line_height(relative(1.4))
                                .child(placeholder.clone().map(|s| s.to_string()).unwrap_or_else(|| "Click to enter path, or drag & drop".to_string()))
                        )
                    })
                    // Display mode
                    .when(!is_editing && !value_empty, |d| {
                        d.on_click(cx.listener(move |this, _event, window, cx| {
                            this.start_editing(index, window, cx);
                        }))
                        .cursor_pointer()
                        .hover(|d| d.bg(rgb(theme.bg_path_hover)))
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgb(theme.text_label))
                                .child(basename.clone().unwrap_or_default())
                        )
                        .child(
                            div()
                                .text_xs()
                                .flex()
                                .flex_row()
                                .flex_wrap()
                                .overflow_x_hidden()
                                .children(
                                    path_display.segments.iter().map(|segment| {
                                        div()
                                            .text_color(rgb(segment.color))
                                            .line_height(relative(1.4))
                                            .child(segment.text.clone())
                                    })
                                )
                        )
                        .when_some(path_display.explanation.clone(), |d, (msg, color)| {
                            d.child(
                                div()
                                    .text_xs()
                                    .italic()
                                    .text_color(rgb(color))
                                    .mt_1()
                                    .child(msg)
                            )
                        })
                    })
                    // Edit mode
                    .when_some(edit_display, |d, (edit_path_info, edit_disp)| {
                        d.child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .when_some(edit_state_clone.clone(), |d, state| {
                                    d.child(state)
                                })
                                .child(
                                    div()
                                        .text_xs()
                                        .flex()
                                        .flex_row()
                                        .flex_wrap()
                                        .overflow_x_hidden()
                                        .when(!edit_path_info.full_path.as_os_str().is_empty(), |d| {
                                            d.children(
                                                edit_disp.segments.iter().map(|segment| {
                                                    div()
                                                        .text_color(rgb(segment.color))
                                                        .line_height(relative(1.4))
                                                        .child(segment.text.clone())
                                                })
                                            )
                                        })
                                        .when(edit_path_info.full_path.as_os_str().is_empty(), |d| {
                                            d.child(
                                                div()
                                                    .text_color(rgb(theme.text_dimmed))
                                                    .child("(empty path)")
                                            )
                                        })
                                )
                                .when_some(edit_disp.explanation.clone(), |d, (msg, color)| {
                                    d.child(
                                        div()
                                            .text_xs()
                                            .italic()
                                            .text_color(rgb(color))
                                            .child(msg)
                                    )
                                })
                        )
                    })
            )
            .child(
                // Browse button
                div()
                    .id(SharedString::from(format!("file_browse_{}", index)))
                    .px_3()
                    .py_2()
                    .bg(rgb(theme.bg_input_hover))
                    .rounded_md()
                    .cursor_pointer()
                    .hover(|d| d.bg(rgb(theme.bg_hover)))
                    .on_click(cx.listener(move |this, _event, window, cx| {
                        this.open_file_dialog(index, window, cx);
                    }))
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(theme.text_label))
                            .child("Browse...")
                    )
            )
            .when(can_remove, |d| {
                d.child(
                    // Remove button
                    div()
                        .id(SharedString::from(format!("file_remove_{}", index)))
                        .key_context("CcfRepeatableButton")
                        .track_focus(&remove_focus_handle)
                        .px_2()
                        .py_2()
                        .bg(rgb(theme.delete_bg))
                        .rounded_md()
                        .cursor_pointer()
                        .hover(|d| d.bg(rgb(theme.delete_bg_hover)))
                        .border_2()
                        .border_color(if remove_is_focused { rgb(theme.border_focus) } else { rgba(0x00000000) })
                        .on_action(cx.listener(|_this, _: &FocusNext, window, _cx| {
                            window.focus_next();
                        }))
                        .on_action(cx.listener(|_this, _: &FocusPrev, window, _cx| {
                            window.focus_prev();
                        }))
                        .on_action(cx.listener(move |this, _: &RepeatableActivateButton, _window, cx| {
                            this.remove_entry(index, cx);
                        }))
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
    }
}

impl EventEmitter<RepeatableFilePickerEvent> for RepeatableFilePicker {}

impl Render for RepeatableFilePicker {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        // Initialize entries on first render
        self.initialize_entries(cx);

        let theme = self.get_theme(cx);
        let entries_len = self.entries.len();
        let add_focused = self.add_focus_handle.is_focused(window);

        // Collect focus handles for entries
        let focus_handles: Vec<_> = self.remove_focus_handles.clone();

        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .children((0..entries_len).map(|index| {
                        let focus_handle = focus_handles.get(index).cloned().unwrap_or_else(|| cx.focus_handle());
                        self.render_entry(index, focus_handle, window, cx)
                    }))
            )
            .child(
                // Add button row - use flex to align left
                div()
                    .flex()
                    .flex_row()
                    .child(
                        div()
                            .id("repeatable_file_add_button")
                            .key_context("CcfRepeatableButton")
                            .track_focus(&self.add_focus_handle)
                            .px_2()
                            .py_1()
                            .bg(rgb(theme.bg_input_hover))
                            .rounded_md()
                            .cursor_pointer()
                            .hover(|d| d.bg(rgb(theme.bg_hover)))
                            .border_2()
                            .border_color(if add_focused { rgb(theme.border_focus) } else { rgba(0x00000000) })
                            .on_action(cx.listener(|_this, _: &FocusNext, window, _cx| {
                                window.focus_next();
                            }))
                            .on_action(cx.listener(|_this, _: &FocusPrev, window, _cx| {
                                window.focus_prev();
                            }))
                            .on_action(cx.listener(|this, _: &RepeatableActivateButton, _window, cx| {
                                this.add_entry(cx);
                            }))
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

// Path display helper structures and functions

/// A segment of the path display with its color
struct PathSegment {
    text: String,
    color: u32,
}

/// Result of computing path display info
struct PathDisplayInfo {
    segments: Vec<PathSegment>,
    explanation: Option<(String, u32)>,
}

/// Compute the colored segments and explanation for displaying a file path.
fn compute_file_path_display(
    path_info: &PathInfo,
    mode: &FileMode,
    missing_directories: &MissingDirectories,
    theme: &Theme,
) -> PathDisplayInfo {
    let mut segments = Vec::new();
    let mut explanation: Option<(String, u32)> = None;

    if path_info.full_path.as_os_str().is_empty() {
        return PathDisplayInfo { segments, explanation };
    }

    let full_path = &path_info.full_path;
    let file_exists = full_path.exists() && full_path.is_file();
    let is_directory = full_path.exists() && full_path.is_dir();

    // Special case: path points to a directory instead of a file
    if is_directory {
        if let Some(parent) = full_path.parent() {
            let parent_str = parent.to_string_lossy().to_string();
            if !parent_str.is_empty() {
                segments.push(PathSegment {
                    text: parent_str,
                    color: theme.text_muted,
                });
            }
        }
        if let Some(dirname) = full_path.file_name() {
            segments.push(PathSegment {
                text: format!("/{}", dirname.to_string_lossy()),
                color: theme.warning,
            });
        }
        return PathDisplayInfo {
            segments,
            explanation: Some(("file expected, but path is a directory".to_string(), theme.warning)),
        };
    }

    match mode {
        FileMode::Open => {
            if path_info.fully_exists() {
                segments.push(PathSegment {
                    text: path_info.existing_canonical.to_string_lossy().to_string(),
                    color: theme.text_muted,
                });
            } else {
                let existing = path_info.existing_canonical.to_string_lossy().to_string();
                if !existing.is_empty() {
                    segments.push(PathSegment {
                        text: existing,
                        color: theme.text_muted,
                    });
                }
                let non_existing = path_info.non_existing_suffix.to_string_lossy().to_string();
                if !non_existing.is_empty() {
                    segments.push(PathSegment {
                        text: format!("/{}", non_existing),
                        color: theme.error,
                    });
                    explanation = Some(("path does not exist".to_string(), theme.error));
                }
            }
        }
        FileMode::Save => {
            if file_exists {
                if let Some(parent) = full_path.parent() {
                    let parent_str = parent.to_string_lossy().to_string();
                    if !parent_str.is_empty() {
                        segments.push(PathSegment {
                            text: parent_str,
                            color: theme.text_muted,
                        });
                    }
                }
                if let Some(filename) = full_path.file_name() {
                    segments.push(PathSegment {
                        text: format!("/{}", filename.to_string_lossy()),
                        color: theme.warning,
                    });
                }
                explanation = Some(("file exists and will be overwritten".to_string(), theme.warning));
            } else if path_info.fully_exists() {
                if let Some(parent) = full_path.parent() {
                    let parent_str = parent.to_string_lossy().to_string();
                    if !parent_str.is_empty() && parent.exists() {
                        segments.push(PathSegment {
                            text: parent_str,
                            color: theme.text_muted,
                        });
                        if let Some(filename) = full_path.file_name() {
                            segments.push(PathSegment {
                                text: format!("/{}", filename.to_string_lossy()),
                                color: theme.success,
                            });
                        }
                        explanation = Some(("file will be created".to_string(), theme.success));
                    } else {
                        segments.push(PathSegment {
                            text: path_info.existing_canonical.to_string_lossy().to_string(),
                            color: theme.text_muted,
                        });
                    }
                }
            } else {
                let parent_exists = full_path.parent().map(|p| p.exists()).unwrap_or(false);

                if parent_exists {
                    if let Some(parent) = full_path.parent() {
                        segments.push(PathSegment {
                            text: parent.to_string_lossy().to_string(),
                            color: theme.text_muted,
                        });
                    }
                    if let Some(filename) = full_path.file_name() {
                        segments.push(PathSegment {
                            text: format!("/{}", filename.to_string_lossy()),
                            color: theme.success,
                        });
                    }
                    explanation = Some(("file will be created".to_string(), theme.success));
                } else {
                    let existing = path_info.existing_canonical.to_string_lossy().to_string();
                    if !existing.is_empty() {
                        segments.push(PathSegment {
                            text: existing,
                            color: theme.text_muted,
                        });
                    }

                    let non_existing = path_info.non_existing_suffix.to_string_lossy().to_string();
                    if !non_existing.is_empty() {
                        let (color, msg) = match missing_directories {
                            MissingDirectories::Create => {
                                (theme.success, "path will be created")
                            }
                            MissingDirectories::Okay => {
                                (theme.success, "path will be created by CLI")
                            }
                            MissingDirectories::Error => {
                                (theme.error, "path does not exist")
                            }
                        };
                        segments.push(PathSegment {
                            text: format!("/{}", non_existing),
                            color,
                        });
                        explanation = Some((msg.to_string(), color));
                    }
                }
            }
        }
    }

    PathDisplayInfo { segments, explanation }
}
