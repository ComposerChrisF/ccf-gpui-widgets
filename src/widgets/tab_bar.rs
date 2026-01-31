//! Generic tab bar widget for switching between views
//!
//! A tab bar that can display any type implementing the `TabItem` trait.
//! Supports left-click tab switching, right-click context menus, and keyboard navigation.
//! Use `register_keybindings()` at app startup to enable keyboard shortcuts.
//!
//! # Example
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::{TabBar, TabBarEvent, TabItem};
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
//! impl TabItem for MyTab {
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
//!         TabBarEvent::TabSelected(tab) => this.switch_to(*tab, cx),
//!         TabBarEvent::ContextMenu { tab, position } => {
//!             this.show_context_menu(*tab, *position, cx);
//!         }
//!     }
//! }).detach();
//! ```

use gpui::prelude::*;
use gpui::*;
use crate::theme::{get_theme, Theme};
use super::focus_navigation::{FocusNext, FocusPrev};

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

/// Trait for items that can be displayed as tabs
///
/// Implement this trait for your tab enum or struct to use it with `TabBar`.
pub trait TabItem: Clone + PartialEq + 'static {
    /// The display label for this tab
    fn label(&self) -> SharedString;

    /// A unique element ID for this tab (used for click handling)
    fn id(&self) -> ElementId;
}

/// Events emitted by TabBar
#[derive(Debug, Clone)]
pub enum TabBarEvent<T> {
    /// A tab was selected (left-click)
    TabSelected(T),
    /// Context menu was requested (right-click)
    ContextMenu {
        tab: T,
        /// Mouse position for context menu placement
        position: Point<Pixels>,
    },
}

/// Generic tab bar widget
pub struct TabBar<T: TabItem> {
    tabs: Vec<T>,
    active: T,
    focus_handle: FocusHandle,
    custom_theme: Option<Theme>,
}

impl<T: TabItem> TabBar<T> {
    /// Create a new tab bar with the given tabs
    ///
    /// # Arguments
    ///
    /// * `tabs` - List of tabs to display
    /// * `active` - The initially active tab
    /// * `cx` - Context for creating the focus handle
    pub fn new(tabs: Vec<T>, active: T, cx: &mut Context<Self>) -> Self {
        Self {
            tabs,
            active,
            focus_handle: cx.focus_handle().tab_stop(true),
            custom_theme: None,
        }
    }

    /// Set a custom theme for this widget
    pub fn theme(mut self, theme: Theme) -> Self {
        self.custom_theme = Some(theme);
        self
    }

    /// Get the currently active tab
    pub fn active_tab(&self) -> &T {
        &self.active
    }

    /// Set the active tab
    pub fn set_active_tab(&mut self, tab: T, cx: &mut Context<Self>) {
        self.active = tab;
        cx.notify();
    }

    /// Get the focus handle
    pub fn focus_handle(&self, _cx: &Context<Self>) -> FocusHandle {
        self.focus_handle.clone()
    }

    fn get_theme(&self, cx: &App) -> Theme {
        self.custom_theme.unwrap_or_else(|| get_theme(cx))
    }

    /// Select the previous tab (wraps around)
    fn select_previous(&mut self, cx: &mut Context<Self>) {
        if self.tabs.is_empty() {
            return;
        }
        let current_index = self.tabs.iter().position(|t| *t == self.active).unwrap_or(0);
        let new_index = if current_index == 0 {
            self.tabs.len() - 1
        } else {
            current_index - 1
        };
        self.active = self.tabs[new_index].clone();
        cx.emit(TabBarEvent::TabSelected(self.active.clone()));
        cx.notify();
    }

    /// Select the next tab (wraps around)
    fn select_next(&mut self, cx: &mut Context<Self>) {
        if self.tabs.is_empty() {
            return;
        }
        let current_index = self.tabs.iter().position(|t| *t == self.active).unwrap_or(0);
        let new_index = if current_index >= self.tabs.len() - 1 {
            0
        } else {
            current_index + 1
        };
        self.active = self.tabs[new_index].clone();
        cx.emit(TabBarEvent::TabSelected(self.active.clone()));
        cx.notify();
    }
}

impl<T: TabItem> EventEmitter<TabBarEvent<T>> for TabBar<T> {}

impl<T: TabItem> Focusable for TabBar<T> {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl<T: TabItem> Render for TabBar<T> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = self.get_theme(cx);
        let active_tab = self.active.clone();
        let is_focused = self.focus_handle.is_focused(window);

        div()
            .id("ccf_tab_bar")
            .key_context("CcfTabBar")
            .track_focus(&self.focus_handle)
            .flex()
            .flex_row()
            .bg(rgb(theme.bg_secondary))
            // Focus navigation (Tab / Shift+Tab)
            .on_action(cx.listener(|_this, _: &FocusNext, window, _cx| {
                window.focus_next();
            }))
            .on_action(cx.listener(|_this, _: &FocusPrev, window, _cx| {
                window.focus_prev();
            }))
            // Tab navigation (Left / Right arrows)
            .on_action(cx.listener(|this, _: &SelectPreviousTab, _window, cx| {
                this.select_previous(cx);
            }))
            .on_action(cx.listener(|this, _: &SelectNextTab, _window, cx| {
                this.select_next(cx);
            }))
            .children(self.tabs.clone().into_iter().map(|tab| {
                let is_active = tab == active_tab;
                let tab_for_click = tab.clone();
                let tab_for_context = tab.clone();
                // Show focus ring only on the active tab when the tab bar is focused
                let show_focus_ring = is_active && is_focused;

                div()
                    .id(tab.id())
                    .px_4()
                    .py_2()
                    .cursor_pointer()
                    .border_r_1()
                    .border_color(rgb(theme.border_default))
                    .when(is_active, |d| {
                        d.bg(rgb(theme.bg_primary))
                            .text_color(rgb(theme.text_primary))
                            .border_t_2()
                            .border_color(rgb(theme.border_tab_active))
                    })
                    .when(!is_active, |d| {
                        d.bg(rgb(theme.bg_input))
                            .text_color(rgb(theme.text_dimmed))
                            .border_b_1()
                            .border_color(rgb(theme.border_default))
                            .hover(|d| {
                                d.bg(rgb(theme.bg_tab_hover))
                                    .text_color(rgb(theme.text_muted))
                            })
                    })
                    // Focus ring on active tab only
                    .when(show_focus_ring, |d| {
                        d.border_2()
                            .border_color(rgb(theme.border_focus))
                    })
                    .on_click(cx.listener(move |this, _event: &ClickEvent, _window, cx| {
                        this.active = tab_for_click.clone();
                        cx.emit(TabBarEvent::TabSelected(tab_for_click.clone()));
                        cx.notify();
                    }))
                    .on_mouse_down(MouseButton::Right, cx.listener(move |_this, event: &MouseDownEvent, _window, cx| {
                        cx.emit(TabBarEvent::ContextMenu {
                            tab: tab_for_context.clone(),
                            position: event.position,
                        });
                    }))
                    .child(tab.label())
            }))
            // Filler area to the right of tabs with bottom border
            .child(
                div()
                    .flex_1()
                    .bg(rgb(theme.bg_secondary))
                    .border_b_1()
                    .border_color(rgb(theme.border_default))
            )
    }
}
