//! Generic tab bar widget for switching between views
//!
//! A tab bar that can display any type implementing the `SelectionItem` trait.
//! Supports left-click tab switching, right-click context menus, and keyboard navigation.
//! Use `register_keybindings()` at app startup to enable keyboard shortcuts.
//!
//! # Example
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::{TabBar, TabBarEvent, SelectionItem};
//! use gpui::*;
//!
//! // Register keybindings at app startup
//! ccf_gpui_widgets::widgets::tab_bar::register_keybindings(cx);
//!
//! #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//! pub enum MyTab {
//!     Overview,
//!     Details,
//!     Settings,
//! }
//!
//! impl SelectionItem for MyTab {
//!     fn label(&self) -> SharedString {
//!         match self {
//!             MyTab::Overview => "Overview".into(),
//!             MyTab::Details => "Details".into(),
//!             MyTab::Settings => "Settings".into(),
//!         }
//!     }
//!
//!     fn id(&self) -> ElementId {
//!         match self {
//!             MyTab::Overview => "tab_overview".into(),
//!             MyTab::Details => "tab_details".into(),
//!             MyTab::Settings => "tab_settings".into(),
//!         }
//!     }
//! }
//!
//! let tab_bar = cx.new(|cx| {
//!     TabBar::new(
//!         vec![MyTab::Overview, MyTab::Details, MyTab::Settings],
//!         MyTab::Overview,
//!         cx,
//!     )
//! });
//!
//! cx.subscribe(&tab_bar, |this, _, event: &TabBarEvent<MyTab>, cx| {
//!     match event {
//!         TabBarEvent::Change(tab) => this.switch_to(*tab, cx),
//!         TabBarEvent::ContextMenu { tab, position } => {
//!             this.show_context_menu(*tab, *position, cx);
//!         }
//!     }
//! }).detach();
//! ```
//!
//! # API Changes (2025-02)
//!
//! - Replaced `TabItem` trait with `SelectionItem` (unified trait across all selection widgets)
//! - Renamed `active` field/methods to `selected` for consistency:
//!   - `active_tab()` → `selected()`
//!   - `set_active_tab()` → `set_selected()`
//! - Added index-based selection: `selected_index()`, `set_selected_index()`
//! - Renamed event: `TabSelected(T)` → `Change(T)`
//! - Note: Navigation widgets (TabBar, SidebarNav) do NOT emit events from set_* methods

use super::focus_navigation::{with_focus_actions, EnabledCursorExt};
use super::selection::SelectionItem;
use crate::theme::{get_theme_or, Theme};
use gpui::prelude::*;
use gpui::*;

// Actions for keyboard navigation
actions!(ccf_tab_bar, [SelectPreviousTab, SelectNextTab]);

/// Register key bindings for tab bar components
///
/// Call this once at application startup:
/// ```ignore
/// ccf_gpui_widgets::widgets::tab_bar::register_keybindings(cx);
/// ```
pub fn register_keybindings(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("left", SelectPreviousTab, Some("CcfTabBar")),
        KeyBinding::new("right", SelectNextTab, Some("CcfTabBar")),
    ]);
}

/// Events emitted by TabBar
///
/// Note: `set_selected()` and `set_selected_index()` do NOT emit events.
/// Navigation widgets represent UI navigation state where the consumer typically
/// controls transitions and doesn't need redundant event notifications.
#[derive(Debug, Clone)]
pub enum TabBarEvent<T> {
    /// A tab was selected (left-click or keyboard navigation)
    ///
    /// Previously named `TabSelected(T)`.
    Change(T),
    /// Context menu was requested (right-click)
    ContextMenu {
        tab: T,
        /// Mouse position for context menu placement
        position: Point<Pixels>,
    },
}

/// Generic tab bar widget
pub struct TabBar<T: SelectionItem> {
    tabs: Vec<T>,
    selected: T,
    focus_handle: FocusHandle,
    custom_theme: Option<Theme>,
    /// Whether the widget is enabled (interactive)
    enabled: bool,
    /// Stores the previously focused element when mouse down occurs,
    /// so we can restore focus after a tab click (preventing focus stealing)
    previous_focus: Option<FocusHandle>,
    /// Horizontal padding for tabs (border extends full width)
    tab_row_padding: Pixels,
}

impl<T: SelectionItem> TabBar<T> {
    /// Create a new tab bar with the given tabs
    ///
    /// # Arguments
    ///
    /// * `tabs` - List of tabs to display
    /// * `selected` - The initially selected tab
    /// * `cx` - Context for creating the focus handle
    pub fn new(tabs: Vec<T>, selected: T, cx: &mut Context<Self>) -> Self {
        Self {
            tabs,
            selected,
            focus_handle: cx.focus_handle().tab_stop(true),
            custom_theme: None,
            enabled: true,
            previous_focus: None,
            tab_row_padding: px(0.0),
        }
    }

    /// Set a custom theme for this widget
    #[must_use]
    pub fn theme(mut self, theme: Theme) -> Self {
        self.custom_theme = Some(theme);
        self
    }

    /// Set enabled state (builder pattern)
    #[must_use]
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set horizontal padding for tabs (border spans full width)
    #[must_use]
    pub fn tab_row_padding(mut self, padding: Pixels) -> Self {
        self.tab_row_padding = padding;
        self
    }

    /// Get the currently selected tab
    pub fn selected(&self) -> &T {
        &self.selected
    }

    /// Get the currently selected index
    pub fn selected_index(&self) -> usize {
        self.tabs
            .iter()
            .position(|t| *t == self.selected)
            .unwrap_or(0)
    }

    /// Set the selected tab
    ///
    /// Note: Does NOT emit Change event. Navigation widgets represent UI state
    /// where the consumer controls transitions.
    pub fn set_selected(&mut self, tab: T, cx: &mut Context<Self>) {
        self.selected = tab;
        cx.notify();
    }

    /// Set selected by index
    ///
    /// Note: Does NOT emit Change event.
    pub fn set_selected_index(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(tab) = self.tabs.get(index).cloned() {
            self.selected = tab;
            cx.notify();
        }
    }

    /// Get the focus handle
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    /// Check if the tab bar is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Set enabled state programmatically
    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.enabled != enabled {
            self.enabled = enabled;
            cx.notify();
        }
    }

    /// Select the previous tab (wraps around)
    fn select_previous(&mut self, cx: &mut Context<Self>) {
        if self.tabs.is_empty() {
            return;
        }
        let current_index = self
            .tabs
            .iter()
            .position(|t| *t == self.selected)
            .unwrap_or(0);
        let new_index = if current_index == 0 {
            self.tabs.len() - 1
        } else {
            current_index - 1
        };
        if let Some(tab) = self.tabs.get(new_index) {
            self.selected = tab.clone();
            cx.emit(TabBarEvent::Change(self.selected.clone()));
            cx.notify();
        }
    }

    /// Select the next tab (wraps around)
    fn select_next(&mut self, cx: &mut Context<Self>) {
        if self.tabs.is_empty() {
            return;
        }
        let current_index = self
            .tabs
            .iter()
            .position(|t| *t == self.selected)
            .unwrap_or(0);
        let new_index = if current_index >= self.tabs.len() - 1 {
            0
        } else {
            current_index + 1
        };
        if let Some(tab) = self.tabs.get(new_index) {
            self.selected = tab.clone();
            cx.emit(TabBarEvent::Change(self.selected.clone()));
            cx.notify();
        }
    }
}

impl<T: SelectionItem> EventEmitter<TabBarEvent<T>> for TabBar<T> {}

impl<T: SelectionItem> Focusable for TabBar<T> {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl<T: SelectionItem> Render for TabBar<T> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = get_theme_or(cx, self.custom_theme.as_ref());
        let selected_tab = self.selected.clone();
        let is_focused = self.focus_handle.is_focused(window);
        let enabled = self.enabled;

        with_focus_actions(
            div()
                .id("ccf_tab_bar")
                .key_context("CcfTabBar")
                .track_focus(&self.focus_handle)
                .tab_stop(enabled),
            cx,
        )
        .flex()
        .flex_row()
        .when(enabled, |d| d.bg(rgb(theme.bg_secondary)))
        .when(!enabled, |d| d.bg(rgb(theme.disabled_bg)))
        // Tab navigation (Left / Right arrows)
        .on_action(cx.listener(|this, _: &SelectPreviousTab, _window, cx| {
            if this.enabled {
                this.select_previous(cx);
            }
        }))
        .on_action(cx.listener(|this, _: &SelectNextTab, _window, cx| {
            if this.enabled {
                this.select_next(cx);
            }
        }))
        // Left filler area (draws bottom border for left padding area)
        .when(self.tab_row_padding > px(0.0), |d| {
            d.child(
                div()
                    .w(self.tab_row_padding)
                    .when(enabled, |d| {
                        d.bg(rgb(theme.bg_secondary))
                            .border_b_1()
                            .border_color(rgb(theme.border_default))
                    })
                    .when(!enabled, |d| {
                        d.bg(rgb(theme.disabled_bg))
                            .border_b_1()
                            .border_color(rgb(theme.disabled_bg))
                    }),
            )
        })
        .children(self.tabs.iter().map(|tab| {
            let tab = tab.clone();
            let is_selected = tab == selected_tab;
            let show_focus = is_selected && is_focused && enabled;

            // Tab container - handles clicks and identification
            div()
                .id(tab.id())
                .cursor_for_enabled(enabled)
                .when(enabled, |d| {
                    let tab_clone = tab.clone();
                    d.on_mouse_down(MouseButton::Left, {
                        cx.listener(move |this, _event: &MouseDownEvent, window, cx| {
                            this.previous_focus = window.focused(cx);
                            cx.notify();
                        })
                    })
                    .on_click({
                        let tab = tab.clone();
                        cx.listener(move |this, _event: &ClickEvent, window, cx| {
                            this.selected = tab.clone();
                            cx.emit(TabBarEvent::Change(tab.clone()));
                            if let Some(focus_handle) = this.previous_focus.take() {
                                focus_handle.focus(window);
                            } else {
                                window.blur();
                            }
                            cx.notify();
                        })
                    })
                    .on_mouse_down(MouseButton::Right, {
                        cx.listener(move |_this, event: &MouseDownEvent, _window, cx| {
                            cx.emit(TabBarEvent::ContextMenu {
                                tab: tab_clone.clone(),
                                position: event.position,
                            });
                        })
                    })
                })
                // Tab content
                .child(
                    div()
                        .px_4()
                        .pb_2()
                        // Active tab: py_2 top + border_t_2 (always accent), no other borders
                        .when(is_selected, |d| {
                            d.pt_2() // Standard top padding
                                .border_t_2()
                        })
                        // Inactive tabs: pt = py_2 + 2px to match active height
                        .when(!is_selected, |d| {
                            d.pt(px(10.0)) // 8px (py_2) + 2px (border_t_2)
                                .border_r_1()
                                .border_b_1()
                        })
                        // Colors based on active/enabled state
                        .when(is_selected && enabled, |d| {
                            d.bg(rgb(theme.bg_primary))
                                .text_color(rgb(theme.text_primary))
                                .border_color(rgb(theme.border_focus)) // Always accent for active tab
                        })
                        .when(is_selected && !enabled, |d| {
                            d.bg(rgb(theme.disabled_bg))
                                .text_color(rgb(theme.disabled_text))
                                .border_color(rgb(theme.disabled_bg))
                        })
                        .when(!is_selected && enabled, |d| {
                            d.bg(rgb(theme.bg_input))
                                .text_color(rgb(theme.text_dimmed))
                                .border_color(rgb(theme.border_default))
                                .hover(|d| {
                                    d.bg(rgb(theme.bg_tab_hover))
                                        .text_color(rgb(theme.text_muted))
                                })
                        })
                        .when(!is_selected && !enabled, |d| {
                            d.bg(rgb(theme.disabled_bg))
                                .text_color(rgb(theme.disabled_text))
                                .border_color(rgb(theme.disabled_bg))
                        })
                        // Text with focus ring (border always present to prevent layout shift)
                        .child(
                            div()
                                .px_1()
                                .border_1()
                                .rounded_sm()
                                .when(show_focus, |d| d.border_color(rgb(theme.border_focus)))
                                .when(!show_focus, |d| d.border_color(rgba(0x00000000)))
                                .child(tab.label()),
                        ),
                )
        }))
        // Filler area to the right of tabs (draws its own bottom border)
        .child(
            div()
                .flex_1()
                .when(enabled, |d| {
                    d.bg(rgb(theme.bg_secondary))
                        .border_b_1()
                        .border_color(rgb(theme.border_default))
                })
                .when(!enabled, |d| {
                    d.bg(rgb(theme.disabled_bg))
                        .border_b_1()
                        .border_color(rgb(theme.disabled_bg))
                }),
        )
        // Right filler area (draws bottom border for right padding area)
        .when(self.tab_row_padding > px(0.0), |d| {
            d.child(
                div()
                    .w(self.tab_row_padding)
                    .when(enabled, |d| {
                        d.bg(rgb(theme.bg_secondary))
                            .border_b_1()
                            .border_color(rgb(theme.border_default))
                    })
                    .when(!enabled, |d| {
                        d.bg(rgb(theme.disabled_bg))
                            .border_b_1()
                            .border_color(rgb(theme.disabled_bg))
                    }),
            )
        })
    }
}
