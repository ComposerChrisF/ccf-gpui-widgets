//! Focus navigation support for Tab/Shift+Tab between widgets
//!
//! This module provides actions and key bindings for tab navigation.
//! Call `register_focus_navigation_keybindings` at application startup
//! to enable Tab/Shift+Tab navigation between widgets.

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
