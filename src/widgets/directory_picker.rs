//! Directory picker widget
//!
//! A directory selection widget with native folder dialog support, drag-and-drop,
//! and color-coded path display showing existing vs non-existing portions.
//!
//! Requires the `file-picker` feature.
//!
//! # Example
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::DirectoryPicker;
//!
//! let picker = cx.new(|cx| {
//!     DirectoryPicker::new(cx)
//!         .placeholder("Select output directory...")
//! });
//!
//! // Subscribe to changes
//! cx.subscribe(&picker, |this, _picker, event: &DirectoryPickerEvent, cx| {
//!     if let DirectoryPickerEvent::Change(path) = event {
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
use super::path_display::PathDisplayInfo;

// Actions for keyboard handling
#[cfg(feature = "file-picker")]
actions!(
    ccf_directory_picker,
    [
        BrowseDirectory,
        ActivateButton,
    ]
);

/// Register key bindings for directory picker
///
/// Call this once at application startup:
/// ```ignore
/// ccf_gpui_widgets::widgets::directory_picker::register_keybindings(cx);
/// ```
#[cfg(feature = "file-picker")]
pub fn register_keybindings(cx: &mut App) {
    // Browse shortcut (Cmd+O / Ctrl+O)
    #[cfg(target_os = "macos")]
    cx.bind_keys([
        KeyBinding::new("cmd-o", BrowseDirectory, Some("CcfDirectoryPicker")),
    ]);

    #[cfg(not(target_os = "macos"))]
    cx.bind_keys([
        KeyBinding::new("ctrl-o", BrowseDirectory, Some("CcfDirectoryPicker")),
    ]);

    // Button activation (Enter/Space when button is focused)
    cx.bind_keys([
        KeyBinding::new("enter", ActivateButton, Some("CcfDirectoryPickerButton")),
        KeyBinding::new("space", ActivateButton, Some("CcfDirectoryPickerButton")),
    ]);
}

/// Events emitted by DirectoryPicker
#[derive(Clone, Debug)]
pub enum DirectoryPickerEvent {
    /// Directory path changed
    Change(String),
}

/// Validation state for DirectoryPicker
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DirectoryPickerValidation {
    /// No path entered
    Empty,
    /// Path exists and is a directory
    Valid,
    /// Path does not exist
    PathDoesNotExist,
    /// Path points to a file instead of a directory
    IsFile,
}

// Re-export ValidationDisplay from shared module
pub use super::path_display::ValidationDisplay;

/// Validate a directory path
///
/// This is a standalone function that can be used without a DirectoryPicker instance,
/// useful for validation logic and testing.
pub fn validate_directory_path(path: &str) -> DirectoryPickerValidation {
    use std::path::Path;

    if path.is_empty() {
        return DirectoryPickerValidation::Empty;
    }

    let path = Path::new(path);

    if path.is_dir() {
        DirectoryPickerValidation::Valid
    } else if path.is_file() {
        DirectoryPickerValidation::IsFile
    } else {
        DirectoryPickerValidation::PathDoesNotExist
    }
}


/// Directory picker widget
#[cfg(feature = "file-picker")]
pub struct DirectoryPicker {
    value: String,
    placeholder: Option<SharedString>,
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
    /// Whether the widget is enabled
    enabled: bool,
}

#[cfg(feature = "file-picker")]
impl EventEmitter<DirectoryPickerEvent> for DirectoryPicker {}

#[cfg(feature = "file-picker")]
impl Focusable for DirectoryPicker {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

#[cfg(feature = "file-picker")]
impl DirectoryPicker {
    /// Create a new directory picker
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            value: String::new(),
            placeholder: None,
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
    /// When disabled, the widget cannot be edited or used to browse for directories.
    /// Default is `true`.
    #[must_use]
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Get the current directory path
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Set value programmatically
    pub fn set_value(&mut self, path: &str, cx: &mut Context<Self>) {
        if self.value != path {
            self.value = path.to_string();
            cx.emit(DirectoryPickerEvent::Change(self.value.clone()));
            cx.notify();
        }
    }

    /// Get the focus handle
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    /// Validate the current path and return the validation state
    ///
    /// Use this to check if the path is valid before taking action.
    pub fn validate(&self) -> DirectoryPickerValidation {
        validate_directory_path(&self.value)
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

    /// Returns true if the current path is valid (exists and is a directory)
    pub fn is_valid(&self) -> bool {
        self.validate() == DirectoryPickerValidation::Valid
    }

    fn compute_path_display(&self, path_info: &PathInfo, theme: &Theme, validation_display: &ValidationDisplay) -> PathDisplayInfo {
        let mut info = PathDisplayInfo::new();

        if path_info.full_path.as_os_str().is_empty() {
            return info;
        }

        // Check if colors and/or messages should be shown
        let show_colors = matches!(validation_display, ValidationDisplay::Full | ValidationDisplay::ColorsOnly);
        let show_message = matches!(validation_display, ValidationDisplay::Full | ValidationDisplay::MessageOnly);

        // Helper to get the appropriate color (respects show_colors setting)
        let color_or_muted = |color: u32| -> u32 {
            if show_colors { color } else { theme.text_muted }
        };

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

        cx.subscribe(&edit_state, |this, edit_state, event: &TextInputEvent, cx| {
            match event {
                TextInputEvent::Enter | TextInputEvent::Blur => {
                    this.is_editing = false;
                    let text = edit_state.read(cx).content().to_string();
                    let path_info = parse_path(&text);
                    let new_value = path_info.full_path_string();
                    if this.value != new_value {
                        this.value = new_value;
                        cx.emit(DirectoryPickerEvent::Change(this.value.clone()));
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
        edit_state.read(cx).focus_handle().focus(window);
    }

    fn open_directory_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.enabled {
            return;
        }
        let entity = cx.entity().clone();

        let initial_dir = if !self.value.is_empty() {
            let path = std::path::Path::new(&self.value);
            if path.exists() {
                Some(path.to_path_buf())
            } else {
                path.ancestors()
                    .find(|p| p.exists() && !p.as_os_str().is_empty())
                    .map(|p| p.to_path_buf())
            }
        } else {
            None
        }.or_else(|| std::env::current_dir().ok());

        window.spawn(cx, async move |async_cx| {
            let result = async_cx.background_executor().spawn(async move {
                let mut dialog = rfd::AsyncFileDialog::new();

                if let Some(dir) = initial_dir {
                    dialog = dialog.set_directory(&dir);
                }

                dialog.pick_folder().await
            }).await;

            if let Some(folder) = result {
                let path = folder.path().to_string_lossy().to_string();
                let _ = async_cx.update_entity(&entity, |this: &mut DirectoryPicker, cx| {
                    if this.value != path {
                        this.value = path;
                        cx.emit(DirectoryPickerEvent::Change(this.value.clone()));
                    }
                    cx.notify();
                });
            }
        }).detach();
    }
}

#[cfg(feature = "file-picker")]
impl Render for DirectoryPicker {
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

        let dirname = if !self.value.is_empty() {
            std::path::Path::new(&self.value)
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
        let edit_button_focus_handle = self.edit_button_focus_handle.clone();
        let edit_button_is_focused = edit_button_focus_handle.is_focused(window);
        let browse_button_focus_handle = self.browse_button_focus_handle.clone();
        let browse_button_is_focused = browse_button_focus_handle.is_focused(window);

        div()
            .id("ccf_directory_picker")
            .key_context("CcfDirectoryPicker")
            .flex()
            .flex_row()
            .w_full()
            .when(enabled, |d| d.bg(rgb(theme.bg_input)))
            .when(!enabled, |d| d.bg(rgb(theme.disabled_bg)))
            .rounded_md()
            .border_1()
            .border_color(rgb(theme.border_default))
            // Handle Cmd+O / Ctrl+O to open directory dialog (when enabled)
            .when(browse_shortcut_enabled && enabled, |d| {
                d.on_action(cx.listener(|picker, _: &BrowseDirectory, window, cx| {
                    picker.open_directory_dialog(window, cx);
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
                    .id("ccf_directory_picker_field")
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
                                let dir_path = if path.is_dir() {
                                    path.to_string_lossy().to_string()
                                } else if path.is_file() {
                                    path.parent()
                                        .map(|p| p.to_string_lossy().to_string())
                                        .unwrap_or_default()
                                } else {
                                    String::new()
                                };

                                if !dir_path.is_empty() && picker.value != dir_path {
                                    picker.value = dir_path;
                                    cx.emit(DirectoryPickerEvent::Change(picker.value.clone()));
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
                                .child("No directory selected")
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
                                .child("No directory selected")
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
                                .child(dirname.clone().unwrap_or_default())
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
                                .child(dirname.clone().unwrap_or_default())
                        )
                        .when(!path_display.is_empty(), |d| {
                            d.child(
                                div()
                                    .text_xs()
                                    .min_w_0()
                                    .line_height(relative(1.4))
                                    .text_color(rgb(theme.disabled_text))
                                    .child(self.value.clone())
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
                            .id("ccf_directory_edit_button")
                            .flex_1()
                            .flex()
                            .items_center()
                            .justify_center()
                            .key_context("CcfDirectoryPickerButton")
                            .track_focus(&edit_button_focus_handle)
                            .tab_stop(enabled)
                            .px_2()
                            .when(enabled, |d| d.bg(rgb(theme.bg_input_hover)))
                            .when(!enabled, |d| d.bg(rgb(theme.disabled_bg)))
                            .border_1()
                            .when(enabled, |d| {
                                d.border_color(rgb(if edit_button_is_focused {
                                    theme.border_focus
                                } else {
                                    theme.bg_input_hover // Invisible border when not focused
                                }))
                            })
                            .when(!enabled, |d| d.border_color(rgb(theme.disabled_bg)))
                            .cursor_for_enabled(enabled)
                            .when(enabled, |d| d.hover(|d| d.bg(rgb(theme.bg_hover))))
                            .when(enabled, |d| {
                                d.on_click(cx.listener(|picker, _event, window, cx| {
                                    picker.start_editing(window, cx);
                                    cx.notify();
                                }))
                                .on_action(cx.listener(|picker, _: &ActivateButton, window, cx| {
                                    picker.start_editing(window, cx);
                                    cx.notify();
                                }))
                            })
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
                            .id("ccf_directory_browse_button")
                            .flex_1()
                            .flex()
                            .items_center()
                            .justify_center()
                            .key_context("CcfDirectoryPickerButton")
                            .track_focus(&browse_button_focus_handle)
                            .tab_stop(enabled)
                            .px_2()
                            .when(enabled, |d| d.bg(rgb(theme.bg_input_hover)))
                            .when(!enabled, |d| d.bg(rgb(theme.disabled_bg)))
                            .border_1()
                            .when(enabled, |d| {
                                d.border_color(rgb(if browse_button_is_focused {
                                    theme.border_focus
                                } else {
                                    theme.bg_input_hover // Invisible border when not focused
                                }))
                            })
                            .when(!enabled, |d| d.border_color(rgb(theme.disabled_bg)))
                            .cursor_for_enabled(enabled)
                            .when(enabled, |d| d.hover(|d| d.bg(rgb(theme.bg_hover))))
                            .when(enabled, |d| {
                                d.on_click(cx.listener(|picker, _event, window, cx| {
                                    picker.open_directory_dialog(window, cx);
                                }))
                                .on_action(cx.listener(|picker, _: &ActivateButton, window, cx| {
                                    picker.open_directory_dialog(window, cx);
                                }))
                            })
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
                                d.tooltip(|_window, cx| cx.new(|_cx| Tooltip::new("Select directory...")).into())
                            })
                            .child(
                                div()
                                    .text_sm()
                                    .when(enabled, |d| d.text_color(rgb(theme.text_label)))
                                    .when(!enabled, |d| d.text_color(rgb(theme.disabled_text)))
                                    .child("📂")
                            )
                    )
            )
    }
}

#[cfg(test)]
mod tests {
    use super::{validate_directory_path, DirectoryPickerValidation};
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    fn setup_test_dir() -> TempDir {
        let dir = TempDir::new().unwrap();
        // Create a test file
        let file_path = dir.path().join("test_file.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "test content").unwrap();
        // Create a subdirectory
        fs::create_dir(dir.path().join("subdir")).unwrap();
        dir
    }

    #[test]
    fn test_validate_empty_path() {
        assert_eq!(
            validate_directory_path(""),
            DirectoryPickerValidation::Empty
        );
    }

    #[test]
    fn test_validate_existing_directory() {
        let dir = setup_test_dir();
        let subdir_path = dir.path().join("subdir");

        assert_eq!(
            validate_directory_path(subdir_path.to_str().unwrap()),
            DirectoryPickerValidation::Valid
        );
    }

    #[test]
    fn test_validate_root_directory() {
        let dir = setup_test_dir();

        assert_eq!(
            validate_directory_path(dir.path().to_str().unwrap()),
            DirectoryPickerValidation::Valid
        );
    }

    #[test]
    fn test_validate_non_existing_path() {
        let dir = setup_test_dir();
        let missing_path = dir.path().join("missing_dir");

        assert_eq!(
            validate_directory_path(missing_path.to_str().unwrap()),
            DirectoryPickerValidation::PathDoesNotExist
        );
    }

    #[test]
    fn test_validate_file_instead_of_directory() {
        let dir = setup_test_dir();
        let file_path = dir.path().join("test_file.txt");

        assert_eq!(
            validate_directory_path(file_path.to_str().unwrap()),
            DirectoryPickerValidation::IsFile
        );
    }
}
