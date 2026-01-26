//! File picker widget
//!
//! A file selection widget with native file dialog support, drag-and-drop,
//! and color-coded path display showing existing vs non-existing portions.
//!
//! Requires the `file-picker` feature.
//!
//! # Example
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::{FilePicker, FileMode};
//!
//! let picker = cx.new(|cx| {
//!     FilePicker::new(cx)
//!         .mode(FileMode::Save)
//!         .extensions(vec!["json".to_string(), "yaml".to_string()])
//!         .placeholder("Select output file...")
//! });
//!
//! // Subscribe to changes
//! cx.subscribe(&picker, |this, _picker, event: &FilePickerEvent, cx| {
//!     if let FilePickerEvent::Change(path) = event {
//!         println!("Selected: {}", path);
//!     }
//! }).detach();
//! ```

#[cfg(feature = "file-picker")]
use gpui::prelude::*;
#[cfg(feature = "file-picker")]
use gpui::*;

#[cfg(feature = "file-picker")]
use crate::theme::{get_theme_or, Theme};
use super::focus_navigation::{FocusNext, FocusPrev};
#[cfg(feature = "file-picker")]
use crate::utils::path::{parse_path, PathInfo};
#[cfg(feature = "file-picker")]
use crate::widgets::{TextInput, TextInputEvent};
#[cfg(feature = "file-picker")]
use std::path::Path;

/// File picker modes
#[derive(Clone, PartialEq, Default)]
pub enum FileMode {
    /// Select existing files only (default)
    #[default]
    Open,
    /// Select output file location (file may not exist)
    Save,
}

impl FileMode {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "save" => FileMode::Save,
            _ => FileMode::Open,
        }
    }
}

/// How to handle missing parent directories
#[derive(Clone, PartialEq, Default)]
pub enum MissingDirectories {
    /// Show error if parent directory is missing (default)
    #[default]
    Error,
    /// Allow missing directories (CLI handles it)
    Okay,
    /// Create missing parent directories on run
    Create,
}

impl MissingDirectories {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "okay" => MissingDirectories::Okay,
            "create" => MissingDirectories::Create,
            _ => MissingDirectories::Error,
        }
    }
}

/// Events emitted by FilePicker
#[derive(Clone, Debug)]
pub enum FilePickerEvent {
    /// File path changed
    Change(String),
}

#[cfg(feature = "file-picker")]
struct PathSegment {
    text: String,
    color: u32,
}

#[cfg(feature = "file-picker")]
struct PathDisplayInfo {
    segments: Vec<PathSegment>,
    explanation: Option<(String, u32)>,
}

/// File picker widget
#[cfg(feature = "file-picker")]
pub struct FilePicker {
    value: String,
    placeholder: Option<SharedString>,
    extensions: Vec<String>,
    mode: FileMode,
    missing_directories: MissingDirectories,
    focus_handle: FocusHandle,
    is_editing: bool,
    edit_state: Option<Entity<TextInput>>,
    custom_theme: Option<Theme>,
}

#[cfg(feature = "file-picker")]
impl EventEmitter<FilePickerEvent> for FilePicker {}

#[cfg(feature = "file-picker")]
impl Focusable for FilePicker {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

#[cfg(feature = "file-picker")]
impl FilePicker {
    /// Create a new file picker
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            value: String::new(),
            placeholder: None,
            extensions: Vec::new(),
            mode: FileMode::Open,
            missing_directories: MissingDirectories::Error,
            focus_handle: cx.focus_handle().tab_stop(true),
            is_editing: false,
            edit_state: None,
            custom_theme: None,
        }
    }

    /// Set initial value (builder pattern)
    pub fn with_value(mut self, path: impl Into<String>) -> Self {
        self.value = path.into();
        self
    }

    /// Set placeholder text (builder pattern)
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// Set file extensions filter (builder pattern)
    pub fn extensions(mut self, extensions: Vec<String>) -> Self {
        self.extensions = extensions;
        self
    }

    /// Set file mode (builder pattern)
    pub fn mode(mut self, mode: FileMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set missing directories handling (builder pattern)
    pub fn missing_directories(mut self, handling: MissingDirectories) -> Self {
        self.missing_directories = handling;
        self
    }

    /// Set custom theme (builder pattern)
    pub fn theme(mut self, theme: Theme) -> Self {
        self.custom_theme = Some(theme);
        self
    }

    /// Get the current file path
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Set value programmatically
    pub fn set_value(&mut self, path: &str, cx: &mut Context<Self>) {
        if self.value != path {
            self.value = path.to_string();
            cx.emit(FilePickerEvent::Change(self.value.clone()));
            cx.notify();
        }
    }

    /// Get the focus handle
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    /// Returns true if this file picker needs directory creation
    pub fn needs_directory_creation(&self) -> bool {
        self.mode == FileMode::Save && self.missing_directories == MissingDirectories::Create
    }

    /// Returns the parent directory path if it needs to be created
    pub fn directory_to_create(&self) -> Option<std::path::PathBuf> {
        if !self.needs_directory_creation() || self.value.is_empty() {
            return None;
        }

        let path = Path::new(&self.value);
        let parent = path.parent()?;

        if !parent.as_os_str().is_empty() && !parent.exists() {
            Some(parent.to_path_buf())
        } else {
            None
        }
    }

    fn compute_path_display(&self, path_info: &PathInfo, theme: &Theme) -> PathDisplayInfo {
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

        match &self.mode {
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
                            let (color, msg) = match &self.missing_directories {
                                MissingDirectories::Create => (theme.success, "path will be created"),
                                MissingDirectories::Okay => (theme.success, "path will be created by CLI"),
                                MissingDirectories::Error => (theme.error, "path does not exist"),
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

    fn start_editing(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.is_editing = true;

        let value = self.value.clone();
        let edit_state = cx.new(|cx| {
            TextInput::new(cx)
                .value(value)
                .select_on_focus(true)
        });

        // Subscribe to text input events
        cx.subscribe(&edit_state, |this, edit_state, event: &TextInputEvent, cx| {
            match event {
                TextInputEvent::Enter | TextInputEvent::Blur => {
                    this.is_editing = false;
                    let text = edit_state.read(cx).content().to_string();
                    let path_info = parse_path(&text);
                    let new_value = path_info.full_path_string();
                    if this.value != new_value {
                        this.value = new_value;
                        cx.emit(FilePickerEvent::Change(this.value.clone()));
                    }
                    cx.notify();
                }
                _ => {}
            }
        }).detach();

        self.edit_state = Some(edit_state.clone());

        // Focus the input
        edit_state.read(cx).focus_handle(cx).focus(window);
    }

    fn open_file_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let extensions = self.extensions.clone();
        let entity = cx.entity().clone();
        let is_save_mode = self.mode == FileMode::Save;

        let initial_dir = if !self.value.is_empty() {
            let path = Path::new(&self.value);
            let parent = if path.is_dir() {
                Some(path.to_path_buf())
            } else {
                path.parent().map(|p| p.to_path_buf())
            };
            parent.filter(|p| p.exists())
        } else {
            None
        }.or_else(|| std::env::current_dir().ok());

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
                let _ = async_cx.update_entity(&entity, |this: &mut FilePicker, cx| {
                    if this.value != path_str {
                        this.value = path_str;
                        cx.emit(FilePickerEvent::Change(this.value.clone()));
                    }
                    cx.notify();
                });
            }
        }).detach();
    }
}

#[cfg(feature = "file-picker")]
impl Render for FilePicker {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = get_theme_or(cx, self.custom_theme.as_ref());
        let focus_handle = self.focus_handle.clone();

        // Handle focus lost during editing
        if self.is_editing {
            if let Some(edit_state) = &self.edit_state {
                if !edit_state.read(cx).focus_handle(cx).is_focused(window) {
                    self.is_editing = false;
                    let text = edit_state.read(cx).content().to_string();
                    let path_info = parse_path(&text);
                    self.value = path_info.full_path_string();
                }
            }
        }

        let path_info = if self.value.is_empty() {
            PathInfo::empty()
        } else {
            parse_path(&self.value)
        };

        let path_display = self.compute_path_display(&path_info, &theme);

        let basename = if !self.value.is_empty() {
            Path::new(&self.value)
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
        } else {
            None
        };

        let placeholder = self.placeholder.clone()
            .unwrap_or_else(|| SharedString::from("Click to enter path, or drag & drop"));

        div()
            .id("ccf_file_picker")
            .flex()
            .flex_row()
            .gap_2()
            .items_start()
            .child(
                // Path display area
                div()
                    .id("ccf_file_picker_field")
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
                    .track_focus(&focus_handle)
                    .tab_stop(true)
                    // Focus navigation (Tab / Shift+Tab)
                    .on_action(cx.listener(|_this, _: &FocusNext, window, _cx| {
                        window.focus_next();
                    }))
                    .on_action(cx.listener(|_this, _: &FocusPrev, window, _cx| {
                        window.focus_prev();
                    }))
                    .on_key_down(cx.listener(|_picker, event: &KeyDownEvent, window, _cx| {
                        if event.keystroke.key == "tab" {
                            if event.keystroke.modifiers.shift {
                                window.focus_prev();
                            } else {
                                window.focus_next();
                            }
                        }
                    }))
                    .drag_over::<ExternalPaths>({
                        let bg_hover = theme.bg_input_hover;
                        let border = theme.border_focus;
                        move |d, _, _, _| {
                            d.bg(rgb(bg_hover))
                                .border_color(rgb(border))
                        }
                    })
                    .on_drop(cx.listener(|picker, paths: &ExternalPaths, _window, cx| {
                        if let Some(path) = paths.paths().first() {
                            if path.is_file() {
                                let path_str = path.to_string_lossy().to_string();
                                if picker.value != path_str {
                                    picker.value = path_str;
                                    cx.emit(FilePickerEvent::Change(picker.value.clone()));
                                }
                                cx.notify();
                            }
                        }
                    }))
                    // Empty state
                    .when(!self.is_editing && self.value.is_empty(), |d| {
                        d.on_click(cx.listener(|picker, _event, window, cx| {
                            picker.start_editing(window, cx);
                            cx.notify();
                        }))
                        .cursor_pointer()
                        .hover(|d| d.bg(rgb(theme.bg_input_hover)))
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
                                .child(placeholder.clone())
                        )
                    })
                    // Display mode
                    .when(!self.is_editing && !self.value.is_empty(), |d| {
                        d.on_click(cx.listener(|picker, _event, window, cx| {
                            picker.start_editing(window, cx);
                            cx.notify();
                        }))
                        .cursor_pointer()
                        .hover(|d| d.bg(rgb(theme.bg_input_hover)))
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
                    .when(self.is_editing && self.edit_state.is_some(), |d| {
                        let Some(edit_state) = self.edit_state.as_ref() else { return d };
                        let edit_text = edit_state.read(cx).content().to_string();
                        let edit_path_info = if edit_text.is_empty() {
                            PathInfo::empty()
                        } else {
                            parse_path(&edit_text)
                        };
                        let edit_display = self.compute_path_display(&edit_path_info, &theme);

                        d.child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(edit_state.clone())
                                .child(
                                    div()
                                        .text_xs()
                                        .flex()
                                        .flex_row()
                                        .flex_wrap()
                                        .overflow_x_hidden()
                                        .when(!edit_path_info.full_path.as_os_str().is_empty(), |d| {
                                            d.children(
                                                edit_display.segments.iter().map(|segment| {
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
                                .when_some(edit_display.explanation.clone(), |d, (msg, color)| {
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
                    .id("ccf_file_browse_button")
                    .px_3()
                    .py_2()
                    .bg(rgb(theme.bg_input_hover))
                    .rounded_md()
                    .cursor_pointer()
                    .hover(|d| d.bg(rgb(theme.bg_hover)))
                    .on_click(cx.listener(|picker, _event, window, cx| {
                        picker.open_file_dialog(window, cx);
                    }))
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(theme.text_label))
                            .child("Browse...")
                    )
            )
    }
}
