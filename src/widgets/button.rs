//! Button utility functions for creating styled buttons
//!
//! These functions create pre-styled button elements using the theme system.
//! They return focusable `Stateful<Div>` elements that can be composed with `.on_click()` handlers.
//! Buttons support keyboard activation with Enter or Space when focused.
//!
//! # Example
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::{primary_button, secondary_button};
//!
//! let run_button = primary_button("run_btn", "Run", enabled, cx)
//!     .on_click(cx.listener(|this, _, _, cx| {
//!         this.run_action(cx);
//!     }));
//!
//! let cancel_button = secondary_button("cancel_btn", "Cancel", cx)
//!     .on_click(cx.listener(|this, _, _, cx| {
//!         this.cancel_action(cx);
//!     }));
//! ```

use gpui::prelude::*;
use gpui::*;
use crate::theme::get_theme;
use super::focus_navigation::{FocusNext, FocusPrev};

// Actions for button activation
actions!(ccf_button, [ActivateButton]);

/// Register key bindings for button components
///
/// Call this once at application startup:
/// ```ignore
/// ccf_gpui_widgets::widgets::button::register_keybindings(cx);
/// ```
pub fn register_keybindings(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("enter", ActivateButton, Some("CcfButton")),
        KeyBinding::new("space", ActivateButton, Some("CcfButton")),
    ]);
}

/// Create a primary button with the specified label
///
/// The button is styled using the theme's primary colors when enabled,
/// and disabled colors when not enabled. Buttons are focusable tab stops
/// that can be activated with Enter or Space.
///
/// # Arguments
///
/// * `id` - Element ID for the button (required for click handlers)
/// * `label` - The text to display on the button
/// * `enabled` - Whether the button is enabled (affects styling and cursor)
/// * `cx` - Application context to access the theme
///
/// # Returns
///
/// A focusable `Stateful<Div>` that can be composed with `.on_click()` and other handlers.
pub fn primary_button(
    id: impl Into<ElementId>,
    label: &str,
    enabled: bool,
    cx: &App,
) -> Stateful<Div> {
    let theme = get_theme(cx);

    div()
        .id(id)
        .key_context("CcfButton")
        .focusable()
        .tab_stop(enabled) // Disabled buttons are not tab stops
        // Focus navigation (Tab / Shift+Tab)
        .on_action(|_: &FocusNext, window, _cx| {
            window.focus_next();
        })
        .on_action(|_: &FocusPrev, window, _cx| {
            window.focus_prev();
        })
        .flex()
        .items_center()
        .justify_center()
        .h(px(36.))
        .px_4()
        .rounded_md()
        .cursor_pointer()
        .text_sm()
        .font_weight(FontWeight::MEDIUM)
        .border_2()
        .border_color(rgba(0x00000000)) // Invisible border by default
        .when(enabled, |d| {
            d.bg(rgb(theme.primary))
                .text_color(rgb(theme.text_primary))
                .hover(|d| d.bg(rgb(theme.primary_hover)))
                .active(|d| d.bg(rgb(theme.primary_active)))
        })
        .when(!enabled, |d| {
            d.bg(rgb(theme.disabled_bg))
                .text_color(rgb(theme.disabled_text))
                .cursor_default()
        })
        // Use contrasting color for focus on primary button (theme-aware)
        .focus(|d| d.border_color(rgb(theme.border_focus_on_color)))
        .child(label.to_string())
}

/// Create a secondary button with the specified label
///
/// Secondary buttons have a more subtle appearance with a border,
/// suitable for less prominent actions. Buttons are focusable tab stops
/// that can be activated with Enter or Space.
///
/// # Arguments
///
/// * `id` - Element ID for the button (required for click handlers)
/// * `label` - The text to display on the button
/// * `cx` - Application context to access the theme
///
/// # Returns
///
/// A focusable `Stateful<Div>` that can be composed with `.on_click()` and other handlers.
pub fn secondary_button(
    id: impl Into<ElementId>,
    label: &str,
    cx: &App,
) -> Stateful<Div> {
    let theme = get_theme(cx);

    div()
        .id(id)
        .key_context("CcfButton")
        .focusable()
        .tab_stop(true)
        // Focus navigation (Tab / Shift+Tab)
        .on_action(|_: &FocusNext, window, _cx| {
            window.focus_next();
        })
        .on_action(|_: &FocusPrev, window, _cx| {
            window.focus_prev();
        })
        .flex()
        .items_center()
        .justify_center()
        .h(px(36.))
        .px_4()
        .rounded_md()
        .cursor_pointer()
        .text_sm()
        .font_weight(FontWeight::MEDIUM)
        .bg(rgb(theme.secondary_bg))
        .text_color(rgb(theme.text_primary))
        .border_2()
        .border_color(rgb(theme.secondary_border))
        .hover(|d| d.bg(rgb(theme.secondary_bg_hover)))
        .active(|d| d.bg(rgb(theme.secondary_bg_active)))
        .focus(|d| d.border_color(rgb(theme.border_focus)))
        .child(label.to_string())
}

/// Create a danger button with the specified label
///
/// Danger buttons are styled with error/red colors to indicate destructive actions
/// like delete, remove, or irreversible operations. Buttons are focusable tab stops
/// that can be activated with Enter or Space.
///
/// # Arguments
///
/// * `id` - Element ID for the button (required for click handlers)
/// * `label` - The text to display on the button
/// * `enabled` - Whether the button is enabled (affects styling and cursor)
/// * `cx` - Application context to access the theme
///
/// # Returns
///
/// A focusable `Stateful<Div>` that can be composed with `.on_click()` and other handlers.
///
/// # Example
///
/// ```ignore
/// use ccf_gpui_widgets::widgets::danger_button;
///
/// let delete_button = danger_button("delete_btn", "Delete", true, cx)
///     .on_click(cx.listener(|this, _, _, cx| {
///         this.delete_item(cx);
///     }));
/// ```
pub fn danger_button(
    id: impl Into<ElementId>,
    label: &str,
    enabled: bool,
    cx: &App,
) -> Stateful<Div> {
    let theme = get_theme(cx);

    // Darker/lighter variants of error color for hover/active states
    let danger_hover = darken_color(theme.error, 0.15);
    let danger_active = darken_color(theme.error, 0.25);

    div()
        .id(id)
        .key_context("CcfButton")
        .focusable()
        .tab_stop(enabled) // Disabled buttons are not tab stops
        // Focus navigation (Tab / Shift+Tab)
        .on_action(|_: &FocusNext, window, _cx| {
            window.focus_next();
        })
        .on_action(|_: &FocusPrev, window, _cx| {
            window.focus_prev();
        })
        .flex()
        .items_center()
        .justify_center()
        .h(px(36.))
        .px_4()
        .rounded_md()
        .cursor_pointer()
        .text_sm()
        .font_weight(FontWeight::MEDIUM)
        .border_2()
        .border_color(rgba(0x00000000)) // Invisible border by default
        .when(enabled, |d| {
            d.bg(rgb(theme.error))
                .text_color(rgb(theme.text_primary))
                .hover(|d| d.bg(rgb(danger_hover)))
                .active(|d| d.bg(rgb(danger_active)))
        })
        .when(!enabled, |d| {
            d.bg(rgb(theme.disabled_bg))
                .text_color(rgb(theme.disabled_text))
                .cursor_default()
        })
        // Use contrasting color for focus on danger button
        .focus(|d| d.border_color(rgb(theme.border_focus_on_color)))
        .child(label.to_string())
}

/// Darken a color by a percentage (0.0 = no change, 1.0 = black)
fn darken_color(color: u32, amount: f32) -> u32 {
    let r = ((color >> 16) & 0xFF) as f32;
    let g = ((color >> 8) & 0xFF) as f32;
    let b = (color & 0xFF) as f32;

    let factor = 1.0 - amount;
    let r = (r * factor) as u32;
    let g = (g * factor) as u32;
    let b = (b * factor) as u32;

    (r << 16) | (g << 8) | b
}
