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
