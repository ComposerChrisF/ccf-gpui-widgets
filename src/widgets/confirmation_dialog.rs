//! Confirmation dialog widget
//!
//! A modal dialog for confirming user actions or displaying information.
//! Supports different styles and configurable buttons.
//!
//! # Dialog Styles
//!
//! - **Info**: Single primary button. Click-outside, Escape, or Enter dismisses.
//! - **Default**: Primary and secondary buttons. Click-outside or Escape triggers secondary. Enter triggers primary.
//! - **Warning**: Same as Default but with orange title for emphasis.
//! - **Danger**: Red primary button. Click-outside does nothing. Escape triggers secondary.
//!   Enter does NOT trigger primary (must click explicitly).
//!
//! # Button Configuration
//!
//! - **Primary**: Always shown (colored based on style)
//! - **Secondary**: Optional second button (gray). Use `secondary_label()` to enable.
//! - **Tertiary**: Optional third button (gray). Use `tertiary_label()` to enable.
//!
//! # Key Mappings
//!
//! Use `map_key()` to bind keys to buttons. For example, map "y" to Primary and "n" to Secondary.
//!
//! # Example
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::{ConfirmationDialog, DialogStyle, DialogButton};
//!
//! // Simple info dialog
//! let info = cx.new(|cx| {
//!     ConfirmationDialog::new("Success", "Your changes have been saved.", cx)
//!         .style(DialogStyle::Info)
//! });
//!
//! // Two-button confirmation
//! let confirm = cx.new(|cx| {
//!     ConfirmationDialog::new("Confirm", "Are you sure?", cx)
//!         .primary_label("Yes")
//!         .secondary_label("No")
//!         .map_key("y", DialogButton::Primary)
//!         .map_key("n", DialogButton::Secondary)
//! });
//!
//! // Three-button save dialog
//! let save = cx.new(|cx| {
//!     ConfirmationDialog::new("Unsaved Changes", "Save before closing?", cx)
//!         .primary_label("Save")
//!         .secondary_label("Cancel")
//!         .tertiary_label("Don't Save")
//!         .map_key("y", DialogButton::Primary)
//!         .map_key("n", DialogButton::Tertiary)
//! });
//!
//! // Subscribe to dialog events
//! cx.subscribe(&dialog, |this, _dialog, event: &ConfirmationDialogEvent, cx| {
//!     match event {
//!         ConfirmationDialogEvent::Primary => { /* OK/Yes/Save clicked */ }
//!         ConfirmationDialogEvent::Secondary => { /* Cancel/No clicked */ }
//!         ConfirmationDialogEvent::Tertiary => { /* Third button clicked */ }
//!     }
//! }).detach();
//! ```

use std::collections::HashMap;

use gpui::prelude::*;
use gpui::*;

use crate::theme::{get_theme_or, Theme};
use super::button::{primary_button, secondary_button, danger_button};
use super::focus_navigation::{FocusNext, FocusPrev};

/// Dialog style/severity (controls primary button color)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DialogStyle {
    /// Informational dialog (blue primary button, easy to dismiss)
    Info,
    /// Normal confirmation dialog (blue primary button)
    #[default]
    Default,
    /// Warning dialog (orange title, blue primary button)
    Warning,
    /// Danger dialog (red primary button, harder to confirm)
    Danger,
}

/// Which button a key or action should trigger
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DialogButton {
    /// Primary button (colored based on style)
    Primary,
    /// Secondary button (gray)
    Secondary,
    /// Tertiary button (gray)
    Tertiary,
}

/// Events emitted by ConfirmationDialog
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfirmationDialogEvent {
    /// Primary button clicked (OK, Yes, Save, Delete, etc.)
    Primary,
    /// Secondary button clicked (Cancel, No, etc.)
    Secondary,
    /// Tertiary button clicked (Don't Save, etc.)
    Tertiary,
}

/// Confirmation dialog widget
pub struct ConfirmationDialog {
    title: SharedString,
    message: SharedString,
    style: DialogStyle,
    primary_label: SharedString,
    secondary_label: Option<SharedString>,
    tertiary_label: Option<SharedString>,
    key_mappings: HashMap<String, DialogButton>,
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
            style: DialogStyle::default(),
            primary_label: "OK".into(),
            secondary_label: None,
            tertiary_label: None,
            key_mappings: HashMap::new(),
            focus_handle: cx.focus_handle().tab_stop(true),
            custom_theme: None,
        }
    }

    /// Set primary button label (builder pattern)
    #[must_use]
    pub fn primary_label(mut self, label: impl Into<SharedString>) -> Self {
        self.primary_label = label.into();
        self
    }

    /// Set secondary button label (builder pattern)
    /// Setting this enables the secondary button.
    #[must_use]
    pub fn secondary_label(mut self, label: impl Into<SharedString>) -> Self {
        self.secondary_label = Some(label.into());
        self
    }

    /// Set tertiary button label (builder pattern)
    /// Setting this enables the tertiary button.
    #[must_use]
    pub fn tertiary_label(mut self, label: impl Into<SharedString>) -> Self {
        self.tertiary_label = Some(label.into());
        self
    }

    /// Map a key to a button (builder pattern)
    /// Keys are case-insensitive (both "y" and "Y" will match).
    #[must_use]
    pub fn map_key(mut self, key: impl Into<String>, button: DialogButton) -> Self {
        let key_lower = key.into().to_lowercase();
        self.key_mappings.insert(key_lower, button);
        self
    }

    /// Set dialog style (builder pattern)
    #[must_use]
    pub fn style(mut self, style: DialogStyle) -> Self {
        self.style = style;
        self
    }

    /// Set custom theme (builder pattern)
    #[must_use]
    pub fn theme(mut self, theme: Theme) -> Self {
        self.custom_theme = Some(theme);
        self
    }

    /// Get the focus handle
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    fn emit_button(&mut self, button: DialogButton, cx: &mut Context<Self>) {
        let event = match button {
            DialogButton::Primary => ConfirmationDialogEvent::Primary,
            DialogButton::Secondary => ConfirmationDialogEvent::Secondary,
            DialogButton::Tertiary => ConfirmationDialogEvent::Tertiary,
        };
        cx.emit(event);
    }
}

impl Render for ConfirmationDialog {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = get_theme_or(cx, self.custom_theme.as_ref());
        let title = self.title.clone();
        let message = self.message.clone();
        let primary_label = self.primary_label.clone();
        let secondary_label = self.secondary_label.clone();
        let tertiary_label = self.tertiary_label.clone();
        let style = self.style;
        let focus_handle = self.focus_handle.clone();
        let key_mappings = self.key_mappings.clone();
        let is_danger = style == DialogStyle::Danger;
        let is_info = style == DialogStyle::Info;
        let has_secondary = secondary_label.is_some();
        let has_tertiary = tertiary_label.is_some();

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

        // Build primary button based on style
        let primary_button_element = match style {
            DialogStyle::Danger => {
                danger_button("dialog_primary", &primary_label, true, cx)
                    .on_click(cx.listener(|dialog, _event: &ClickEvent, _window, cx| {
                        dialog.emit_button(DialogButton::Primary, cx);
                    }))
            }
            _ => {
                primary_button("dialog_primary", &primary_label, true, cx)
                    .on_click(cx.listener(|dialog, _event: &ClickEvent, _window, cx| {
                        dialog.emit_button(DialogButton::Primary, cx);
                    }))
            }
        };

        // Build buttons container
        let mut buttons = div()
            .w_full()
            .flex()
            .flex_row()
            .gap_3()
            .justify_end();

        // Add tertiary button (leftmost of the optional buttons)
        if let Some(label) = &tertiary_label {
            buttons = buttons.child(
                secondary_button("dialog_tertiary", label, cx)
                    .on_click(cx.listener(|dialog, _event: &ClickEvent, _window, cx| {
                        dialog.emit_button(DialogButton::Tertiary, cx);
                    }))
            );
        }

        // Add secondary button
        if let Some(label) = &secondary_label {
            buttons = buttons.child(
                secondary_button("dialog_secondary", label, cx)
                    .on_click(cx.listener(|dialog, _event: &ClickEvent, _window, cx| {
                        dialog.emit_button(DialogButton::Secondary, cx);
                    }))
            );
        }

        // Add primary button (rightmost)
        buttons = buttons.child(primary_button_element);

        // Dialog box
        let dialog_box = div()
            .id("ccf_confirmation_dialog_box")
            .track_focus(&focus_handle)
            .tab_stop(true)
            .occlude()
            .on_action(cx.listener(|_this, _: &FocusNext, window, _cx| {
                window.focus_next();
            }))
            .on_action(cx.listener(|_this, _: &FocusPrev, window, _cx| {
                window.focus_prev();
            }))
            .on_key_down(cx.listener(move |dialog, event: &KeyDownEvent, window, cx| {
                let key = event.keystroke.key.as_str().to_lowercase();

                // Check custom key mappings first
                if let Some(&button) = key_mappings.get(&key) {
                    // Only trigger if the button exists
                    let can_trigger = match button {
                        DialogButton::Primary => true,
                        DialogButton::Secondary => has_secondary,
                        DialogButton::Tertiary => has_tertiary,
                    };
                    if can_trigger {
                        dialog.emit_button(button, cx);
                        return;
                    }
                }

                // Default key behaviors
                match key.as_str() {
                    "escape" => {
                        // Escape: triggers secondary if exists, otherwise primary (for Info)
                        if has_secondary {
                            dialog.emit_button(DialogButton::Secondary, cx);
                        } else {
                            dialog.emit_button(DialogButton::Primary, cx);
                        }
                    }
                    "enter" => {
                        // Enter: triggers primary (except for Danger style)
                        if !is_danger {
                            dialog.emit_button(DialogButton::Primary, cx);
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
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(title_color))
                    .child(title)
            )
            .child(
                div()
                    .mt_4()
                    .text_sm()
                    .text_color(rgb(theme.text_muted))
                    .child(message)
            )
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
                .occlude()
                .flex()
                .items_center()
                .justify_center()
                .bg(rgba(0x000000aa))
                .on_mouse_down(MouseButton::Left, cx.listener(move |dialog, _event, _window, cx| {
                    // Click-outside behavior
                    if is_info {
                        // Info: click-outside dismisses (Primary)
                        dialog.emit_button(DialogButton::Primary, cx);
                    } else if !is_danger && has_secondary {
                        // Default/Warning with secondary: click-outside triggers Secondary
                        dialog.emit_button(DialogButton::Secondary, cx);
                    }
                    // Danger: click-outside does nothing
                }))
                .child(dialog_box)
        )
    }
}
