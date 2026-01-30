//! Button utility functions for creating styled buttons
//!
//! These functions create pre-styled button elements using the theme system.
//! They return `Stateful<Div>` elements that can be composed with `.on_click()` handlers.
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

/// Create a primary button with the specified label
///
/// The button is styled using the theme's primary colors when enabled,
/// and disabled colors when not enabled.
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
/// A `Stateful<Div>` that can be composed with `.on_click()` and other handlers.
pub fn primary_button(
    id: impl Into<ElementId>,
    label: &str,
    enabled: bool,
    cx: &App,
) -> Stateful<Div> {
    let theme = get_theme(cx);

    div()
        .id(id)
        .flex()
        .items_center()
        .justify_center()
        .h(px(36.))
        .px_4()
        .rounded_md()
        .cursor_pointer()
        .text_sm()
        .font_weight(FontWeight::MEDIUM)
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
        .child(label.to_string())
}

/// Create a secondary button with the specified label
///
/// Secondary buttons have a more subtle appearance with a border,
/// suitable for less prominent actions.
///
/// # Arguments
///
/// * `id` - Element ID for the button (required for click handlers)
/// * `label` - The text to display on the button
/// * `cx` - Application context to access the theme
///
/// # Returns
///
/// A `Stateful<Div>` that can be composed with `.on_click()` and other handlers.
pub fn secondary_button(
    id: impl Into<ElementId>,
    label: &str,
    cx: &App,
) -> Stateful<Div> {
    let theme = get_theme(cx);

    div()
        .id(id)
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
        .border_1()
        .border_color(rgb(theme.secondary_border))
        .hover(|d| d.bg(rgb(theme.secondary_bg_hover)))
        .active(|d| d.bg(rgb(theme.secondary_bg_active)))
        .child(label.to_string())
}
