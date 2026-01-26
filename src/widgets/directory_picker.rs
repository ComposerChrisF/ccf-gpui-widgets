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
#[cfg(feature = "file-picker")]
use crate::utils::path::{parse_path, PathInfo};
#[cfg(feature = "file-picker")]
use crate::widgets::{TextInput, TextInputEvent};

/// Events emitted by DirectoryPicker
#[derive(Clone, Debug)]
pub enum DirectoryPickerEvent {
    /// Directory path changed
    Change(String),
}

#[cfg(feature = "file-picker")]
struct PathSegment {
    text: String,
    color: u32,
}

#[cfg(feature = "file-picker")]
struct DirPathDisplayInfo {
    segments: Vec<PathSegment>,
    explanation: Option<(String, u32)>,
}

/// Directory picker widget
#[cfg(feature = "file-picker")]
pub struct DirectoryPicker {
    value: String,
    placeholder: Option<SharedString>,
    focus_handle: FocusHandle,
    is_editing: bool,
    edit_state: Option<Entity<TextInput>>,
    custom_theme: Option<Theme>,
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

    /// Set custom theme (builder pattern)
    pub fn theme(mut self, theme: Theme) -> Self {
        self.custom_theme = Some(theme);
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

    fn compute_path_display(&self, path_info: &PathInfo, theme: &Theme) -> DirPathDisplayInfo {
        let mut segments = Vec::new();
        let mut explanation: Option<(String, u32)> = None;

        if path_info.full_path.as_os_str().is_empty() {
            return DirPathDisplayInfo { segments, explanation };
        }

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

        DirPathDisplayInfo { segments, explanation }
    }

    fn start_editing(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.is_editing = true;

        let value = self.value.clone();
        let edit_state = cx.new(|cx| {
            TextInput::new(cx)
                .value(value)
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
                _ => {}
            }
        }).detach();

        self.edit_state = Some(edit_state.clone());
        edit_state.read(cx).focus_handle(cx).focus(window);
    }

    fn open_directory_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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

        div()
            .id("ccf_directory_picker")
            .flex()
            .flex_row()
            .gap_2()
            .items_start()
            .child(
                // Path display area
                div()
                    .id("ccf_directory_picker_field")
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
                                .child("No directory selected")
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
                                .child(dirname.clone().unwrap_or_default())
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
                    .id("ccf_directory_browse_button")
                    .px_3()
                    .py_2()
                    .bg(rgb(theme.bg_input_hover))
                    .rounded_md()
                    .cursor_pointer()
                    .hover(|d| d.bg(rgb(theme.bg_hover)))
                    .on_click(cx.listener(|picker, _event, window, cx| {
                        picker.open_directory_dialog(window, cx);
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
