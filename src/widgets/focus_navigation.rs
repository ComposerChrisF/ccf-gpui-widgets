//! Focus navigation support for Tab/Shift+Tab between widgets
//!
//! This module provides actions and key bindings for tab navigation.
//! Call `register_focus_navigation_keybindings` at application startup
//! to enable Tab/Shift+Tab navigation between widgets.

use gpui::prelude::*;
use gpui::*;

// Define actions for focus navigation
actions!(ccf_focus, [FocusNext, FocusPrev]);

/// Register Tab and Shift+Tab key bindings for focus navigation.
///
/// Call this once at application startup:
///
/// ```ignore
/// use ccf_gpui_widgets::widgets::register_focus_navigation_keybindings;
///
/// Application::new().run(|cx: &mut App| {
///     register_focus_navigation_keybindings(cx);
///     // ... rest of your initialization
/// });
/// ```
pub fn register_keybindings(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("tab", FocusNext, None),
        KeyBinding::new("shift-tab", FocusPrev, None),
    ]);
}

/// Handle tab key navigation in on_key_down handlers.
/// Returns true if the event was handled (tab key was pressed).
pub fn handle_tab_navigation(event: &KeyDownEvent, window: &mut Window) -> bool {
    if event.keystroke.key == "tab" {
        if event.keystroke.modifiers.shift {
            window.focus_prev();
        } else {
            window.focus_next();
        }
        true
    } else {
        false
    }
}

/// Apply standard focus navigation actions (Tab / Shift+Tab) to an element.
///
/// This helper reduces boilerplate in widget render methods by adding
/// the common FocusNext and FocusPrev action handlers.
///
/// # Example
///
/// ```ignore
/// use ccf_gpui_widgets::widgets::focus_navigation::with_focus_actions;
///
/// with_focus_actions(
///     div()
///         .id("my_widget")
///         .track_focus(&focus_handle),
///     cx,
/// )
/// ```
pub fn with_focus_actions<V: 'static, E: InteractiveElement>(element: E, cx: &mut Context<V>) -> E {
    element
        .on_action(cx.listener(|_this, _: &FocusNext, window, _cx| {
            window.focus_next();
        }))
        .on_action(cx.listener(|_this, _: &FocusPrev, window, _cx| {
            window.focus_prev();
        }))
}

/// Extension trait for applying cursor styling based on enabled state.
///
/// This reduces the common pattern of:
/// ```ignore
/// .when(enabled, |d| d.cursor_pointer())
/// .when(!enabled, |d| d.cursor_default())
/// ```
///
/// To simply:
/// ```ignore
/// .cursor_for_enabled(enabled)
/// ```
pub trait EnabledCursorExt: Styled + Sized + FluentBuilder {
    /// Apply cursor styling based on enabled state.
    /// When enabled, uses pointer cursor; otherwise uses default cursor.
    fn cursor_for_enabled(self, enabled: bool) -> Self {
        self.when(enabled, |d| d.cursor_pointer())
            .when(!enabled, |d| d.cursor_default())
    }
}

impl<E: Styled + Sized + FluentBuilder> EnabledCursorExt for E {}

use crate::theme::Theme;
use super::repeatable_text_input::ActivateButton as RepeatableActivateButton;

// ============================================================================
// CRITICAL BUG WARNING - DO NOT REMOVE action_just_handled MECHANISM
// ============================================================================
//
// The repeatable button helpers below use an `action_just_handled` flag to
// prevent DOUBLE-TRIGGERING when Space/Enter is pressed on a focused button.
//
// WHY THIS HAPPENS:
// When Space/Enter is pressed on a focused button, GPUI fires BOTH:
//   1. on_action() - from the ActivateButton keybinding
//   2. on_click()  - because keyboard activation counts as a "click"
//
// Without the flag, pressing Space/Enter once would add/remove TWO entries!
//
// HOW THE FIX WORKS:
// - The action handler sets `action_just_handled = true` before calling the callback
// - The click handler checks this flag and skips if true, then resets it
// - This ensures only ONE handler actually performs the action
//
// THIS BUG HAS BEEN REINTRODUCED AT LEAST 3 TIMES. DO NOT:
// - Remove the action_just_handled field from widget structs
// - Remove the flag-setting logic from callbacks
// - "Simplify" by removing what looks like unused state
//
// If you're refactoring this code, TEST by pressing Space/Enter on the +/-
// buttons and verify only ONE entry is added/removed per keypress.
// ============================================================================

/// Render a repeatable widget remove button (minus icon).
///
/// Creates a 28x28 button with a horizontal minus sign, styled according to the theme.
/// Handles focus, hover states, and disabled styling.
///
/// # Arguments
/// * `id` - Unique element ID for the button
/// * `focus_handle` - Focus handle for keyboard navigation
/// * `theme` - Theme for colors
/// * `enabled` - Whether the button is interactive
/// * `is_focused` - Whether the button currently has focus
/// * `on_action_activate` - Callback for action handler (should set action_just_handled = true)
/// * `on_click_activate` - Callback for click handler (should check/reset action_just_handled)
/// * `cx` - Context for creating listeners
///
/// # Double-Trigger Prevention
/// The two separate callbacks allow the caller to implement the action_just_handled pattern.
/// See the module-level comment for why this is critical.
#[allow(clippy::too_many_arguments)] // Required for double-trigger prevention pattern
pub fn repeatable_remove_button<V: 'static>(
    id: impl Into<SharedString>,
    focus_handle: &FocusHandle,
    theme: &Theme,
    enabled: bool,
    is_focused: bool,
    on_action_activate: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static,
    on_click_activate: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static,
    cx: &mut Context<V>,
) -> Stateful<Div> {
    let line_color = if enabled { theme.text_label } else { theme.disabled_text };

    let mut button = with_focus_actions(
        div()
            .id(id.into())
            .key_context("CcfRepeatableButton")
            .track_focus(focus_handle)
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
    .child(
        div()
            .w(px(10.))
            .h(px(2.))
            .rounded_sm()
            .bg(rgb(line_color))
    );

    if enabled {
        button = button
            .on_action(cx.listener(move |this, _: &RepeatableActivateButton, window, cx| {
                on_action_activate(this, window, cx);
            }))
            .on_click(cx.listener(move |this, _event, window, cx| {
                on_click_activate(this, window, cx);
            }));
    }

    button
}

/// Render a repeatable widget add button (plus icon).
///
/// Creates a 28x28 button with a plus sign (crossed lines), styled according to the theme.
/// Handles focus, hover states, and disabled styling.
///
/// # Arguments
/// * `id` - Unique element ID for the button
/// * `focus_handle` - Focus handle for keyboard navigation
/// * `theme` - Theme for colors
/// * `enabled` - Whether the button is interactive
/// * `is_focused` - Whether the button currently has focus
/// * `on_action_activate` - Callback for action handler (should set action_just_handled = true)
/// * `on_click_activate` - Callback for click handler (should check/reset action_just_handled)
/// * `cx` - Context for creating listeners
///
/// # Double-Trigger Prevention
/// The two separate callbacks allow the caller to implement the action_just_handled pattern.
/// See the module-level comment for why this is critical.
#[allow(clippy::too_many_arguments)] // Required for double-trigger prevention pattern
pub fn repeatable_add_button<V: 'static>(
    id: impl Into<SharedString>,
    focus_handle: &FocusHandle,
    theme: &Theme,
    enabled: bool,
    is_focused: bool,
    on_action_activate: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static,
    on_click_activate: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static,
    cx: &mut Context<V>,
) -> Stateful<Div> {
    let line_color = if enabled { theme.text_label } else { theme.disabled_text };

    let mut button = with_focus_actions(
        div()
            .id(id.into())
            .key_context("CcfRepeatableButton")
            .track_focus(focus_handle)
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
            .border_color(if is_focused { rgb(theme.border_focus) } else { rgba(0x00000000) })
    })
    .when(!enabled, |d| {
        d.bg(rgb(theme.disabled_bg))
            .border_color(rgba(0x00000000))
    })
    // Draw plus sign with crossed lines
    .child(
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
    );

    if enabled {
        button = button
            .on_action(cx.listener(move |this, _: &RepeatableActivateButton, window, cx| {
                on_action_activate(this, window, cx);
            }))
            .on_click(cx.listener(move |this, _event, window, cx| {
                on_click_activate(this, window, cx);
            }));
    }

    button
}
