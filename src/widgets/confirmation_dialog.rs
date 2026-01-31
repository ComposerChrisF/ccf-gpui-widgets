//! Confirmation dialog widget
//!
//! A modal dialog for confirming user actions or displaying information.
//! Supports different styles for different contexts.
//!
//! # Dialog Styles
//!
//! - **Info**: Single "OK" button for informational messages. Click-outside, Escape, or Enter dismisses.
//! - **Default**: Cancel and Confirm buttons. Click-outside or Escape cancels. Enter confirms.
//! - **Warning**: Same as Default but with orange title for emphasis.
//! - **Danger**: Cancel and Delete/Confirm buttons (red). Click-outside does nothing.
//!   Escape cancels. Enter does NOT confirm (must click the button explicitly).
//!
//! The Danger style is intentionally harder to confirm to prevent accidental
//! destructive actions.
//!
//! # Example
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::{ConfirmationDialog, DialogStyle};
//!
//! // Info dialog (just shows a message with OK button)
//! let info = cx.new(|cx| {
//!     ConfirmationDialog::new("Success", "Your changes have been saved.", cx)
//!         .style(DialogStyle::Info)
//! });
//!
//! // Danger confirmation dialog
//! let dialog = cx.new(|cx| {
//!     ConfirmationDialog::new("Delete Item", "Are you sure? This cannot be undone.", cx)
//!         .style(DialogStyle::Danger)
//!         .confirm_label("Delete")
//! });
//!
//! // Subscribe to dialog events
//! cx.subscribe(&dialog, |this, _dialog, event: &ConfirmationDialogEvent, cx| {
//!     match event {
//!         ConfirmationDialogEvent::Confirm => {
//!             this.delete_item(cx);
//!         }
//!         ConfirmationDialogEvent::Cancel => {
//!             // Dialog was cancelled
//!         }
//!     }
//! }).detach();
//! ```

use gpui::prelude::*;
use gpui::*;

use crate::theme::{get_theme_or, Theme};
use super::button::{primary_button, secondary_button, danger_button};
use super::focus_navigation::{FocusNext, FocusPrev};

/// Dialog style/severity
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DialogStyle {
    /// Informational dialog with single OK button (dismissible with Enter)
    Info,
    /// Normal confirmation dialog (blue primary button)
    #[default]
    Default,
    /// Warning dialog (orange/yellow styling)
    Warning,
    /// Danger dialog (red confirm button, Enter does NOT confirm)
    Danger,
}

/// Events emitted by ConfirmationDialog
#[derive(Clone, Debug)]
pub enum ConfirmationDialogEvent {
    /// User confirmed the action (or dismissed Info dialog)
    Confirm,
    /// User cancelled the dialog
    Cancel,
}

/// Confirmation dialog widget
pub struct ConfirmationDialog {
    title: SharedString,
    message: SharedString,
    confirm_label: SharedString,
    cancel_label: SharedString,
    style: DialogStyle,
    focus_handle: FocusHandle,
    custom_theme: Option<Theme>,
}

impl EventEmitter<ConfirmationDialogEvent> for ConfirmationDialog {}

impl Focusable for ConfirmationDialog {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl ConfirmationDialog {
    /// Create a new confirmation dialog
    pub fn new(
        title: impl Into<SharedString>,
        message: impl Into<SharedString>,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            confirm_label: "OK".into(),
            cancel_label: "Cancel".into(),
            style: DialogStyle::default(),
            focus_handle: cx.focus_handle().tab_stop(true),
            custom_theme: None,
        }
    }

    /// Set confirm button label (builder pattern)
    pub fn confirm_label(mut self, label: impl Into<SharedString>) -> Self {
        self.confirm_label = label.into();
        self
    }

    /// Set cancel button label (builder pattern)
    pub fn cancel_label(mut self, label: impl Into<SharedString>) -> Self {
        self.cancel_label = label.into();
        self
    }

    /// Set dialog style (builder pattern)
    pub fn style(mut self, style: DialogStyle) -> Self {
        self.style = style;
        self
    }

    /// Set custom theme (builder pattern)
    pub fn theme(mut self, theme: Theme) -> Self {
        self.custom_theme = Some(theme);
        self
    }

    /// Get the focus handle
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    fn confirm(&mut self, cx: &mut Context<Self>) {
        cx.emit(ConfirmationDialogEvent::Confirm);
    }

    fn cancel(&mut self, cx: &mut Context<Self>) {
        cx.emit(ConfirmationDialogEvent::Cancel);
    }
}

impl Render for ConfirmationDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = get_theme_or(cx, self.custom_theme.as_ref());
        let title = self.title.clone();
        let message = self.message.clone();
        let confirm_label = self.confirm_label.clone();
        let cancel_label = self.cancel_label.clone();
        let style = self.style;
        let focus_handle = self.focus_handle.clone();
        let is_danger = style == DialogStyle::Danger;
        let is_info = style == DialogStyle::Info;

        // Focus the dialog when it renders
        if !focus_handle.is_focused(window) {
            focus_handle.focus(window);
        }

        // Title color based on style
        let title_color = match style {
            DialogStyle::Info => theme.primary,
            DialogStyle::Default => theme.text_primary,
            DialogStyle::Warning => theme.warning,
            DialogStyle::Danger => theme.error,
        };

        // Build buttons based on style
        let buttons = if is_info {
            // Info dialog: single OK button
            div()
                .w_full()
                .flex()
                .flex_row()
                .gap_3()
                .justify_end()
                .child(
                    primary_button("dialog_ok", &confirm_label, true, cx)
                        .on_click(cx.listener(|dialog, _event: &ClickEvent, _window, cx| {
                            dialog.confirm(cx);
                        }))
                )
        } else {
            // Other dialogs: Cancel and Confirm buttons
            let confirm_button = match style {
                DialogStyle::Default | DialogStyle::Warning => {
                    primary_button("dialog_confirm", &confirm_label, true, cx)
                        .on_click(cx.listener(|dialog, _event: &ClickEvent, _window, cx| {
                            dialog.confirm(cx);
                        }))
                }
                DialogStyle::Danger => {
                    danger_button("dialog_confirm", &confirm_label, true, cx)
                        .on_click(cx.listener(|dialog, _event: &ClickEvent, _window, cx| {
                            dialog.confirm(cx);
                        }))
                }
                DialogStyle::Info => unreachable!(),
            };

            let cancel_button = secondary_button("dialog_cancel", &cancel_label, cx)
                .on_click(cx.listener(|dialog, _event: &ClickEvent, _window, cx| {
                    dialog.cancel(cx);
                }));

            div()
                .w_full()
                .flex()
                .flex_row()
                .gap_3()
                .justify_end()
                .child(cancel_button)
                .child(confirm_button)
        };

        // Dialog box
        let dialog_box = div()
            .id("ccf_confirmation_dialog_box")
            .track_focus(&focus_handle)
            .tab_stop(true)
            .occlude() // Block mouse events from reaching elements below
            // Focus navigation
            .on_action(cx.listener(|_this, _: &FocusNext, window, _cx| {
                window.focus_next();
            }))
            .on_action(cx.listener(|_this, _: &FocusPrev, window, _cx| {
                window.focus_prev();
            }))
            .on_key_down(cx.listener(move |dialog, event: &KeyDownEvent, window, cx| {
                match event.keystroke.key.as_str() {
                    "escape" => {
                        // Escape always cancels (or dismisses for Info)
                        if is_info {
                            dialog.confirm(cx);
                        } else {
                            dialog.cancel(cx);
                        }
                    }
                    "enter" => {
                        // For Danger dialogs, Enter does NOT confirm
                        // For Info/Default/Warning, Enter confirms
                        if !is_danger {
                            dialog.confirm(cx);
                        }
                    }
                    "tab" => {
                        if event.keystroke.modifiers.shift {
                            window.focus_prev();
                        } else {
                            window.focus_next();
                        }
                    }
                    _ => {}
                }
            }))
            .bg(rgb(theme.bg_secondary))
            .border_1()
            .border_color(rgb(theme.border_default))
            .rounded_lg()
            .shadow_lg()
            .min_w(px(320.0))
            .max_w(px(480.0))
            .p(px(24.0))
            // Title
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(title_color))
                    .child(title)
            )
            // Message
            .child(
                div()
                    .mt_4()
                    .text_sm()
                    .text_color(rgb(theme.text_muted))
                    .child(message)
            )
            // Buttons
            .child(
                div()
                    .mt_4()
                    .child(buttons)
            );

        // Use deferred for proper overlay behavior
        deferred(
            div()
                .id("ccf_confirmation_dialog")
                .absolute()
                .inset_0()
                .occlude() // Block all mouse events from reaching elements below
                .flex()
                .items_center()
                .justify_center()
                // Semi-transparent overlay background
                .bg(rgba(0x000000aa))
                // Handle click on overlay (outside dialog box)
                .on_mouse_down(MouseButton::Left, cx.listener(move |dialog, _event, _window, cx| {
                    // For Info/Default/Warning: click-outside dismisses/cancels
                    // For Danger: click-outside does nothing
                    if is_info {
                        dialog.confirm(cx);
                    } else if !is_danger {
                        dialog.cancel(cx);
                    }
                }))
                .child(dialog_box)
        )
    }
}
