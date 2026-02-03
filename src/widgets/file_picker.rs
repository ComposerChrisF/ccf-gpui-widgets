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
use super::focus_navigation::{FocusNext, FocusPrev, EnabledCursorExt};
#[cfg(feature = "file-picker")]
use crate::utils::path::{parse_path, PathInfo};
#[cfg(feature = "file-picker")]
use crate::widgets::{TextInput, TextInputEvent, Tooltip};
#[cfg(feature = "file-picker")]
use std::path::Path;
#[cfg(feature = "file-picker")]
use super::path_display::PathDisplayInfo;

use std::str::FromStr;

// Actions for keyboard handling
#[cfg(feature = "file-picker")]
actions!(
    ccf_file_picker,
    [
        BrowseFile,
        ActivateButton,
    ]
);

/// Register key bindings for file picker
///
/// Call this once at application startup:
/// ```ignore
/// ccf_gpui_widgets::widgets::file_picker::register_keybindings(cx);
/// ```
#[cfg(feature = "file-picker")]
pub fn register_keybindings(cx: &mut App) {
    // Browse shortcut (Cmd+O / Ctrl+O)
    #[cfg(target_os = "macos")]
    cx.bind_keys([
        KeyBinding::new("cmd-o", BrowseFile, Some("CcfFilePicker")),
    ]);

    #[cfg(not(target_os = "macos"))]
    cx.bind_keys([
        KeyBinding::new("ctrl-o", BrowseFile, Some("CcfFilePicker")),
    ]);

    // Button activation (Enter/Space when button is focused)
    cx.bind_keys([
        KeyBinding::new("enter", ActivateButton, Some("CcfFilePickerButton")),
        KeyBinding::new("space", ActivateButton, Some("CcfFilePickerButton")),
    ]);
}

/// File picker modes
#[derive(Clone, PartialEq, Default)]
pub enum FileMode {
    /// Select existing files only (default)
    #[default]
    Open,
    /// Select output file location (file may not exist)
    Save,
}

impl FromStr for FileMode {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "save" => FileMode::Save,
            _ => FileMode::Open,
        })
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

impl FromStr for MissingDirectories {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "okay" => MissingDirectories::Okay,
            "create" => MissingDirectories::Create,
            _ => MissingDirectories::Error,
        })
    }
}

/// Events emitted by FilePicker
#[derive(Clone, Debug)]
pub enum FilePickerEvent {
    /// File path changed
    Change(String),
}

/// Validation state for FilePicker
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilePickerValidation {
    /// No path entered
    Empty,
    /// Path exists and is a file (Open mode) or will be created (Save mode with existing parent)
    Valid,
    /// Open mode: path does not exist
    PathDoesNotExist,
    /// Path points to a directory instead of a file
    IsDirectory,
    /// Save mode: file exists and will be overwritten
    WillOverwrite,
    /// Save mode: parent directory does not exist (with MissingDirectories::Error)
    ParentDoesNotExist,
    /// Save mode: directories will be created (with MissingDirectories::Create or Okay)
    WillCreatePath,
}

// Re-export ValidationDisplay from shared module
pub use super::path_display::ValidationDisplay;

/// Validate a file path for the given mode and missing directories policy
///
/// This is a standalone function that can be used without a FilePicker instance,
/// useful for validation logic and testing.
pub fn validate_file_path(
    path: &str,
    mode: &FileMode,
    missing_directories: &MissingDirectories,
) -> FilePickerValidation {
    use std::path::Path;

    if path.is_empty() {
        return FilePickerValidation::Empty;
    }

    let path = Path::new(path);

    // Check if path points to a directory (invalid for file picker)
    if path.is_dir() {
        return FilePickerValidation::IsDirectory;
    }

    match mode {
        FileMode::Open => {
            if path.is_file() {
                FilePickerValidation::Valid
            } else {
                FilePickerValidation::PathDoesNotExist
            }
        }
        FileMode::Save => {
            if path.is_file() {
                FilePickerValidation::WillOverwrite
            } else {
                // Check if parent directory exists
                let parent_exists = path.parent().is_some_and(|p| p.exists() || p.as_os_str().is_empty());

                if parent_exists {
                    FilePickerValidation::Valid
                } else {
                    match missing_directories {
                        MissingDirectories::Error => FilePickerValidation::ParentDoesNotExist,
                        MissingDirectories::Create | MissingDirectories::Okay => {
                            FilePickerValidation::WillCreatePath
                        }
                    }
                }
            }
        }
    }
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
    edit_button_focus_handle: FocusHandle,
    browse_button_focus_handle: FocusHandle,
    is_editing: bool,
    edit_state: Option<Entity<TextInput>>,
    custom_theme: Option<Theme>,
    /// Whether to refocus self on next render (after ESC from TextInput)
    pending_refocus: bool,
    /// Whether Cmd+O / Ctrl+O shortcut is enabled
    browse_shortcut_enabled: bool,
    /// How validation feedback is displayed
    validation_display: ValidationDisplay,
    /// Whether the widget is enabled for interaction
    enabled: bool,
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
            focus_handle: cx.focus_handle(),
            edit_button_focus_handle: cx.focus_handle().tab_stop(true),
            browse_button_focus_handle: cx.focus_handle().tab_stop(true),
            is_editing: false,
            edit_state: None,
            custom_theme: None,
            pending_refocus: false,
            browse_shortcut_enabled: true,
            validation_display: ValidationDisplay::default(),
            enabled: true,
        }
    }

    /// Set initial value (builder pattern)
    #[must_use]
    pub fn with_value(mut self, path: impl Into<String>) -> Self {
        self.value = path.into();
        self
    }

    /// Set placeholder text (builder pattern)
    #[must_use]
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// Set file extensions filter (builder pattern)
    #[must_use]
    pub fn extensions(mut self, extensions: Vec<String>) -> Self {
        self.extensions = extensions;
        self
    }

    /// Set file mode (builder pattern)
    #[must_use]
    pub fn mode(mut self, mode: FileMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set missing directories handling (builder pattern)
    #[must_use]
    pub fn missing_directories(mut self, handling: MissingDirectories) -> Self {
        self.missing_directories = handling;
        self
    }

    /// Set custom theme (builder pattern)
    #[must_use]
    pub fn theme(mut self, theme: Theme) -> Self {
        self.custom_theme = Some(theme);
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

    /// Set whether the widget is enabled (builder pattern)
    ///
    /// When disabled, the widget cannot be edited or browsed.
    /// Default is `true`.
    #[must_use]
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Returns whether the widget is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Set whether the widget is enabled
    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.enabled != enabled {
            self.enabled = enabled;
            cx.notify();
        }
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

    /// Validate the current path and return the validation state
    ///
    /// Use this to check if the path is valid before taking action.
    pub fn validate(&self) -> FilePickerValidation {
        validate_file_path(&self.value, &self.mode, &self.missing_directories)
    }

    /// Returns true if the current path is valid for the configured mode
    ///
    /// For Open mode: path must exist and be a file.
    /// For Save mode: path must not be a directory, and either:
    ///   - Parent exists (file will be created or overwritten), or
    ///   - MissingDirectories is Create/Okay (path will be created)
    pub fn is_valid(&self) -> bool {
        matches!(
            self.validate(),
            FilePickerValidation::Valid
                | FilePickerValidation::WillOverwrite
                | FilePickerValidation::WillCreatePath
        )
    }

    fn compute_path_display(&self, path_info: &PathInfo, theme: &Theme, validation_display: &ValidationDisplay) -> PathDisplayInfo {
        let mut info = PathDisplayInfo::new();

        if path_info.full_path.as_os_str().is_empty() {
            return info;
        }

        let full_path = &path_info.full_path;

        // Check if colors and/or messages should be shown
        let show_colors = matches!(validation_display, ValidationDisplay::Full | ValidationDisplay::ColorsOnly);
        let show_message = matches!(validation_display, ValidationDisplay::Full | ValidationDisplay::MessageOnly);

        // Helper to get the appropriate color (respects show_colors setting)
        let color_or_muted = |color: u32| -> u32 {
            if show_colors { color } else { theme.text_muted }
        };

        // Special case: path points to a directory instead of a file
        if full_path.is_dir() {
            if let Some(parent) = full_path.parent() {
                info.add_segment(&parent.to_string_lossy(), theme.text_muted);
            }
            if let Some(dirname) = full_path.file_name() {
                info.add_path_prefix(&dirname.to_string_lossy(), color_or_muted(theme.warning));
            }
            if show_message {
                info.set_explanation("file expected, but path is a directory", theme.warning);
            }
            return info;
        }

        let file_exists = full_path.is_file();

        match &self.mode {
            FileMode::Open => {
                if path_info.fully_exists() {
                    info.add_segment(&path_info.existing_canonical.to_string_lossy(), theme.text_muted);
                } else {
                    info.add_segment(&path_info.existing_canonical.to_string_lossy(), theme.text_muted);
                    let non_existing = path_info.non_existing_suffix.to_string_lossy();
                    if !non_existing.is_empty() {
                        info.add_path_prefix(&non_existing, color_or_muted(theme.error));
                        if show_message {
                            info.set_explanation("path does not exist", theme.error);
                        }
                    }
                }
            }
            FileMode::Save => {
                if file_exists {
                    if let Some(parent) = full_path.parent() {
                        info.add_segment(&parent.to_string_lossy(), theme.text_muted);
                    }
                    if let Some(filename) = full_path.file_name() {
                        info.add_path_prefix(&filename.to_string_lossy(), color_or_muted(theme.warning));
                    }
                    if show_message {
                        info.set_explanation("file exists and will be overwritten", theme.warning);
                    }
                } else {
                    let parent_exists = full_path.parent().is_some_and(|p| p.exists());

                    if parent_exists {
                        if let Some(parent) = full_path.parent() {
                            info.add_segment(&parent.to_string_lossy(), theme.text_muted);
                        }
                        if let Some(filename) = full_path.file_name() {
                            info.add_path_prefix(&filename.to_string_lossy(), color_or_muted(theme.success));
                        }
                        if show_message {
                            info.set_explanation("file will be created", theme.success);
                        }
                    } else {
                        info.add_segment(&path_info.existing_canonical.to_string_lossy(), theme.text_muted);
                        let non_existing = path_info.non_existing_suffix.to_string_lossy();
                        if !non_existing.is_empty() {
                            let (color, msg) = match &self.missing_directories {
                                MissingDirectories::Create => (theme.success, "path will be created"),
                                MissingDirectories::Okay => (theme.success, "path will be created by CLI"),
                                MissingDirectories::Error => (theme.error, "path does not exist"),
                            };
                            info.add_path_prefix(&non_existing, color_or_muted(color));
                            if show_message {
                                info.set_explanation(msg, color);
                            }
                        }
                    }
                }
            }
        }

        info
    }

    fn start_editing(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.enabled {
            return;
        }
        self.is_editing = true;

        let value = self.value.clone();
        let edit_state = cx.new(|cx| {
            TextInput::new(cx)
                .with_value(value)
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
                TextInputEvent::Escape => {
                    // Cancel editing and refocus the picker
                    this.is_editing = false;
                    this.pending_refocus = true;
                    cx.notify();
                }
                _ => {}
            }
        }).detach();

        self.edit_state = Some(edit_state.clone());

        // Focus the input
        edit_state.read(cx).focus_handle().focus(window);
    }

    fn open_file_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.enabled {
            return;
        }
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
        // Handle pending refocus (after ESC from TextInput)
        if self.pending_refocus {
            self.pending_refocus = false;
            self.edit_button_focus_handle.focus(window);
        }

        // Handle focus lost during editing
        if self.is_editing {
            if let Some(edit_state) = &self.edit_state {
                if !edit_state.read(cx).focus_handle().is_focused(window) {
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

        let path_display = self.compute_path_display(&path_info, &theme, &self.validation_display);

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

        let browse_shortcut_enabled = self.browse_shortcut_enabled;
        let enabled = self.enabled;

        let is_save_mode = self.mode == FileMode::Save;
        let edit_button_focus_handle = self.edit_button_focus_handle.clone();
        let edit_button_is_focused = edit_button_focus_handle.is_focused(window);
        let browse_button_focus_handle = self.browse_button_focus_handle.clone();
        let browse_button_is_focused = browse_button_focus_handle.is_focused(window);

        div()
            .id("ccf_file_picker")
            .key_context("CcfFilePicker")
            .flex()
            .flex_row()
            .w_full()
            .when(enabled, |d| d.bg(rgb(theme.bg_input)))
            .when(!enabled, |d| d.bg(rgb(theme.disabled_bg)))
            .rounded_md()
            .border_1()
            .border_color(rgb(theme.border_default))
            // Handle Cmd+O / Ctrl+O to open file dialog (when enabled)
            .when(browse_shortcut_enabled && enabled, |d| {
                d.on_action(cx.listener(|picker, _: &BrowseFile, window, cx| {
                    picker.open_file_dialog(window, cx);
                }))
            })
            .when(enabled, |d| {
                d.drag_over::<ExternalPaths>({
                    let bg_hover = theme.bg_input_hover;
                    let border = theme.border_focus;
                    move |d, _, _, _| {
                        d.bg(rgb(bg_hover))
                            .border_color(rgb(border))
                    }
                })
            })
            .child(
                // Path display area
                div()
                    .id("ccf_file_picker_field")
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .min_h(px(52.))
                    .px_3()
                    .py_2()
                    .when(enabled, |d| {
                        d.on_drop(cx.listener(|picker, paths: &ExternalPaths, _window, cx| {
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
                    })
                    // Empty state (enabled)
                    .when(!self.is_editing && self.value.is_empty() && enabled, |d| {
                        d.on_click(cx.listener(|picker, _event, window, cx| {
                            picker.start_editing(window, cx);
                            cx.notify();
                        }))
                        .cursor_pointer()
                        .hover(|d| d.bg(rgb(theme.bg_input_hover)))
                        .child(
                            div()
                                .text_sm()
                                .italic()
                                .text_color(rgb(theme.text_dimmed))
                                .child("No file selected")
                        )
                        .child(
                            div()
                                .text_xs()
                                .italic()
                                .text_color(rgb(theme.text_dimmed))
                                .line_height(relative(1.4))
                                .child(placeholder.clone())
                        )
                    })
                    // Empty state (disabled)
                    .when(!self.is_editing && self.value.is_empty() && !enabled, |d| {
                        d.cursor_default()
                        .child(
                            div()
                                .text_sm()
                                .italic()
                                .text_color(rgb(theme.disabled_text))
                                .child("No file selected")
                        )
                        .child(
                            div()
                                .text_xs()
                                .italic()
                                .text_color(rgb(theme.disabled_text))
                                .line_height(relative(1.4))
                                .child(placeholder.clone())
                        )
                    })
                    // Display mode (enabled)
                    .when(!self.is_editing && !self.value.is_empty() && enabled, |d| {
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
                        .when(!path_display.is_empty(), |d| {
                            d.child(
                                div()
                                    .text_xs()
                                    .min_w_0()
                                    .line_height(relative(1.4))
                                    .child(path_display.to_styled_text())
                            )
                        })
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
                    // Display mode (disabled)
                    .when(!self.is_editing && !self.value.is_empty() && !enabled, |d| {
                        d.cursor_default()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgb(theme.disabled_text))
                                .child(basename.clone().unwrap_or_default())
                        )
                        .when(!path_display.is_empty(), |d| {
                            d.child(
                                div()
                                    .text_xs()
                                    .min_w_0()
                                    .text_color(rgb(theme.disabled_text))
                                    .line_height(relative(1.4))
                                    .child(path_display.full_text.clone())
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
                        let edit_display = self.compute_path_display(&edit_path_info, &theme, &self.validation_display);

                        d.child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(edit_state.clone())
                                .child(
                                    div()
                                        .text_xs()
                                        .min_w_0()
                                        .line_height(relative(1.4))
                                        .when(!edit_display.is_empty(), |d| {
                                            d.child(edit_display.to_styled_text())
                                        })
                                        .when(edit_display.is_empty(), |d| {
                                            d.text_color(rgb(theme.text_dimmed))
                                                .child("(empty path)")
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
                // Icon buttons (Edit and Browse)
                div()
                    .flex()
                    .flex_col()
                    .border_l_1()
                    .border_color(rgb(theme.border_default))
                    .child(
                        // Edit button
                        div()
                            .id("ccf_file_edit_button")
                            .flex_1()
                            .flex()
                            .items_center()
                            .justify_center()
                            .key_context("CcfFilePickerButton")
                            .track_focus(&edit_button_focus_handle)
                            .tab_stop(enabled)
                            .px_2()
                            .when(enabled, |d| d.bg(rgb(theme.bg_input_hover)))
                            .when(!enabled, |d| d.bg(rgb(theme.disabled_bg)))
                            .border_1()
                            .border_color(rgb(if edit_button_is_focused && enabled {
                                theme.border_focus
                            } else if enabled {
                                theme.bg_input_hover // Invisible border when not focused
                            } else {
                                theme.disabled_bg
                            }))
                            .cursor_for_enabled(enabled)
                            .when(enabled, |d| d.hover(|d| d.bg(rgb(theme.bg_hover))))
                            .when(enabled, |d| {
                                d.on_click(cx.listener(|picker, _event, window, cx| {
                                    picker.start_editing(window, cx);
                                    cx.notify();
                                }))
                            })
                            .on_action(cx.listener(|picker, _: &ActivateButton, window, cx| {
                                picker.start_editing(window, cx);
                                cx.notify();
                            }))
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
                            .when(enabled, |d| {
                                d.tooltip(|_window, cx| cx.new(|_cx| Tooltip::new("Edit path")).into())
                            })
                            .child(
                                div()
                                    .text_sm()
                                    .when(enabled, |d| d.text_color(rgb(theme.text_label)))
                                    .when(!enabled, |d| d.text_color(rgb(theme.disabled_text)))
                                    .child("✎")
                            )
                    )
                    .child(
                        // Divider between buttons
                        div()
                            .h(px(1.))
                            .bg(rgb(theme.border_default))
                    )
                    .child(
                        // Browse button
                        div()
                            .id("ccf_file_browse_button")
                            .flex_1()
                            .flex()
                            .items_center()
                            .justify_center()
                            .key_context("CcfFilePickerButton")
                            .track_focus(&browse_button_focus_handle)
                            .tab_stop(enabled)
                            .px_2()
                            .when(enabled, |d| d.bg(rgb(theme.bg_input_hover)))
                            .when(!enabled, |d| d.bg(rgb(theme.disabled_bg)))
                            .border_1()
                            .border_color(rgb(if browse_button_is_focused && enabled {
                                theme.border_focus
                            } else if enabled {
                                theme.bg_input_hover // Invisible border when not focused
                            } else {
                                theme.disabled_bg
                            }))
                            .cursor_for_enabled(enabled)
                            .when(enabled, |d| d.hover(|d| d.bg(rgb(theme.bg_hover))))
                            .when(enabled, |d| {
                                d.on_click(cx.listener(|picker, _event, window, cx| {
                                    picker.open_file_dialog(window, cx);
                                }))
                            })
                            .on_action(cx.listener(|picker, _: &ActivateButton, window, cx| {
                                picker.open_file_dialog(window, cx);
                            }))
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
                            .when(enabled, |d| {
                                d.tooltip(move |_window, cx| {
                                    cx.new(|_cx| Tooltip::new(if is_save_mode { "Save as..." } else { "Select file..." })).into()
                                })
                            })
                            .child(
                                div()
                                    .text_sm()
                                    .when(enabled, |d| d.text_color(rgb(theme.text_label)))
                                    .when(!enabled, |d| d.text_color(rgb(theme.disabled_text)))
                                    .child(if is_save_mode { "💾" } else { "📂" })
                            )
                    )
            )
    }
}

#[cfg(test)]
mod tests {
    use super::{validate_file_path, FilePickerValidation, FileMode, MissingDirectories};
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    fn setup_test_dir() -> TempDir {
        let dir = TempDir::new().unwrap();
        // Create a test file
        let file_path = dir.path().join("existing_file.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "test content").unwrap();
        // Create a subdirectory
        fs::create_dir(dir.path().join("subdir")).unwrap();
        dir
    }

    #[test]
    fn test_validate_empty_path() {
        assert_eq!(
            validate_file_path("", &FileMode::Open, &MissingDirectories::Error),
            FilePickerValidation::Empty
        );
        assert_eq!(
            validate_file_path("", &FileMode::Save, &MissingDirectories::Error),
            FilePickerValidation::Empty
        );
    }

    #[test]
    fn test_validate_open_existing_file() {
        let dir = setup_test_dir();
        let file_path = dir.path().join("existing_file.txt");

        assert_eq!(
            validate_file_path(file_path.to_str().unwrap(), &FileMode::Open, &MissingDirectories::Error),
            FilePickerValidation::Valid
        );
    }

    #[test]
    fn test_validate_open_non_existing_file() {
        let dir = setup_test_dir();
        let file_path = dir.path().join("non_existing.txt");

        assert_eq!(
            validate_file_path(file_path.to_str().unwrap(), &FileMode::Open, &MissingDirectories::Error),
            FilePickerValidation::PathDoesNotExist
        );
    }

    #[test]
    fn test_validate_open_directory() {
        let dir = setup_test_dir();
        let subdir_path = dir.path().join("subdir");

        assert_eq!(
            validate_file_path(subdir_path.to_str().unwrap(), &FileMode::Open, &MissingDirectories::Error),
            FilePickerValidation::IsDirectory
        );
    }

    #[test]
    fn test_validate_save_existing_file() {
        let dir = setup_test_dir();
        let file_path = dir.path().join("existing_file.txt");

        assert_eq!(
            validate_file_path(file_path.to_str().unwrap(), &FileMode::Save, &MissingDirectories::Error),
            FilePickerValidation::WillOverwrite
        );
    }

    #[test]
    fn test_validate_save_new_file_parent_exists() {
        let dir = setup_test_dir();
        let file_path = dir.path().join("new_file.txt");

        assert_eq!(
            validate_file_path(file_path.to_str().unwrap(), &FileMode::Save, &MissingDirectories::Error),
            FilePickerValidation::Valid
        );
    }

    #[test]
    fn test_validate_save_parent_missing_error() {
        let dir = setup_test_dir();
        let file_path = dir.path().join("missing_dir/new_file.txt");

        assert_eq!(
            validate_file_path(file_path.to_str().unwrap(), &FileMode::Save, &MissingDirectories::Error),
            FilePickerValidation::ParentDoesNotExist
        );
    }

    #[test]
    fn test_validate_save_parent_missing_create() {
        let dir = setup_test_dir();
        let file_path = dir.path().join("missing_dir/new_file.txt");

        assert_eq!(
            validate_file_path(file_path.to_str().unwrap(), &FileMode::Save, &MissingDirectories::Create),
            FilePickerValidation::WillCreatePath
        );
    }

    #[test]
    fn test_validate_save_parent_missing_okay() {
        let dir = setup_test_dir();
        let file_path = dir.path().join("missing_dir/new_file.txt");

        assert_eq!(
            validate_file_path(file_path.to_str().unwrap(), &FileMode::Save, &MissingDirectories::Okay),
            FilePickerValidation::WillCreatePath
        );
    }

    #[test]
    fn test_validate_save_directory() {
        let dir = setup_test_dir();
        let subdir_path = dir.path().join("subdir");

        assert_eq!(
            validate_file_path(subdir_path.to_str().unwrap(), &FileMode::Save, &MissingDirectories::Error),
            FilePickerValidation::IsDirectory
        );
    }
}
