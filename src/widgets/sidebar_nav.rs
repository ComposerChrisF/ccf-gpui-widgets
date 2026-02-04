//! Generic sidebar navigation widget for switching between sections
//!
//! A vertical navigation sidebar that can display any type implementing the `SidebarItem` trait.
//! Supports click-to-select and keyboard navigation with Up/Down arrows.
//! Use `register_keybindings()` at app startup to enable keyboard shortcuts.
//!
//! # Example
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::{SidebarNav, SidebarNavEvent, SidebarItem};
//! use gpui::*;
//!
//! // Register keybindings at app startup
//! ccf_gpui_widgets::widgets::sidebar_nav::register_keybindings(cx);
//!
//! #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//! pub enum MySection {
//!     Overview,
//!     Details,
//!     Settings,
//! }
//!
//! impl SidebarItem for MySection {
//!     fn label(&self) -> SharedString {
//!         match self {
//!             MySection::Overview => "Overview".into(),
//!             MySection::Details => "Details".into(),
//!             MySection::Settings => "Settings".into(),
//!         }
//!     }
//!
//!     fn id(&self) -> ElementId {
//!         match self {
//!             MySection::Overview => "sidebar_overview".into(),
//!             MySection::Details => "sidebar_details".into(),
//!             MySection::Settings => "sidebar_settings".into(),
//!         }
//!     }
//! }
//!
//! let sidebar_nav = cx.new(|cx| {
//!     SidebarNav::new(
//!         vec![MySection::Overview, MySection::Details, MySection::Settings],
//!         MySection::Overview,
//!         cx,
//!     )
//! });
//!
//! cx.subscribe(&sidebar_nav, |this, _, event: &SidebarNavEvent<MySection>, cx| {
//!     match event {
//!         SidebarNavEvent::Select(section) => this.switch_to(*section, cx),
//!     }
//! }).detach();
//! ```

use gpui::prelude::*;
use gpui::*;
use crate::theme::{get_theme_or, Theme};
use super::focus_navigation::{with_focus_actions, EnabledCursorExt};

// Actions for keyboard navigation
actions!(ccf_sidebar_nav, [SelectPrevious, SelectNext]);

/// Register key bindings for sidebar nav components
///
/// Call this once at application startup:
/// ```ignore
/// ccf_gpui_widgets::widgets::sidebar_nav::register_keybindings(cx);
/// ```
pub fn register_keybindings(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("up", SelectPrevious, Some("CcfSidebarNav")),
        KeyBinding::new("down", SelectNext, Some("CcfSidebarNav")),
    ]);
}

/// Trait for items that can be displayed in the sidebar navigation
///
/// Implement this trait for your section enum or struct to use it with `SidebarNav`.
pub trait SidebarItem: Clone + PartialEq + 'static {
    /// The display label for this item
    fn label(&self) -> SharedString;

    /// A unique element ID for this item (used for click handling)
    fn id(&self) -> ElementId;
}

/// Events emitted by SidebarNav
#[derive(Debug, Clone)]
pub enum SidebarNavEvent<T> {
    /// An item was selected
    Select(T),
}

/// Generic sidebar navigation widget
pub struct SidebarNav<T: SidebarItem> {
    items: Vec<T>,
    selected: T,
    focus_handle: FocusHandle,
    custom_theme: Option<Theme>,
    /// Whether the widget is enabled (interactive)
    enabled: bool,
    /// Fixed width for the sidebar
    width: Option<Pixels>,
}

impl<T: SidebarItem> SidebarNav<T> {
    /// Create a new sidebar nav with the given items
    ///
    /// # Arguments
    ///
    /// * `items` - List of items to display
    /// * `selected` - The initially selected item
    /// * `cx` - Context for creating the focus handle
    pub fn new(items: Vec<T>, selected: T, cx: &mut Context<Self>) -> Self {
        Self {
            items,
            selected,
            focus_handle: cx.focus_handle().tab_stop(true),
            custom_theme: None,
            enabled: true,
            width: None,
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

    /// Set a fixed width for the sidebar
    #[must_use]
    pub fn with_width(mut self, width: Pixels) -> Self {
        self.width = Some(width);
        self
    }

    /// Get the currently selected item
    pub fn selected_item(&self) -> &T {
        &self.selected
    }

    /// Set the selected item
    pub fn set_selected(&mut self, item: T, cx: &mut Context<Self>) {
        self.selected = item;
        cx.notify();
    }

    /// Get the focus handle
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    /// Check if the sidebar is enabled
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

    /// Select the previous item (wraps around)
    fn select_previous(&mut self, cx: &mut Context<Self>) {
        if self.items.is_empty() {
            return;
        }
        let current_index = self.items.iter().position(|t| *t == self.selected).unwrap_or(0);
        let new_index = if current_index == 0 {
            self.items.len() - 1
        } else {
            current_index - 1
        };
        if let Some(item) = self.items.get(new_index) {
            self.selected = item.clone();
            cx.emit(SidebarNavEvent::Select(self.selected.clone()));
            cx.notify();
        }
    }

    /// Select the next item (wraps around)
    fn select_next(&mut self, cx: &mut Context<Self>) {
        if self.items.is_empty() {
            return;
        }
        let current_index = self.items.iter().position(|t| *t == self.selected).unwrap_or(0);
        let new_index = if current_index >= self.items.len() - 1 {
            0
        } else {
            current_index + 1
        };
        if let Some(item) = self.items.get(new_index) {
            self.selected = item.clone();
            cx.emit(SidebarNavEvent::Select(self.selected.clone()));
            cx.notify();
        }
    }
}

impl<T: SidebarItem> EventEmitter<SidebarNavEvent<T>> for SidebarNav<T> {}

impl<T: SidebarItem> Focusable for SidebarNav<T> {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl<T: SidebarItem> Render for SidebarNav<T> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = get_theme_or(cx, self.custom_theme.as_ref());
        let selected_item = self.selected.clone();
        let is_focused = self.focus_handle.is_focused(window);
        let enabled = self.enabled;

        with_focus_actions(
            div()
                .id("ccf_sidebar_nav")
                .key_context("CcfSidebarNav")
                .track_focus(&self.focus_handle)
                .tab_stop(enabled),
            cx,
        )
        .flex()
        .flex_col()
        .when_some(self.width, |d, w| d.w(w))
        .when(enabled, |d| d.bg(rgb(theme.bg_input)))
        .when(!enabled, |d| d.bg(rgb(theme.disabled_bg)))
        .border_r_1()
        .border_color(rgb(theme.border_default))
        .p_2()
        // Keyboard navigation (Up / Down arrows)
        .on_action(cx.listener(|this, _: &SelectPrevious, _window, cx| {
            if this.enabled {
                this.select_previous(cx);
            }
        }))
        .on_action(cx.listener(|this, _: &SelectNext, _window, cx| {
            if this.enabled {
                this.select_next(cx);
            }
        }))
        .children(self.items.iter().map(|item| {
            let item = item.clone();
            let is_selected = item == selected_item;
            let show_focus = is_selected && is_focused && enabled;

            div()
                .id(item.id())
                .cursor_for_enabled(enabled)
                .px_2()
                .py_1()
                .mb_1()
                .rounded(px(4.0))
                .when(enabled, |d| {
                    let item_clone = item.clone();
                    d.on_click({
                        cx.listener(move |this, _event: &ClickEvent, _window, cx| {
                            this.selected = item_clone.clone();
                            cx.emit(SidebarNavEvent::Select(item_clone.clone()));
                            cx.notify();
                        })
                    })
                })
                // Selected state
                .when(is_selected && enabled, |d| {
                    d.bg(rgb(theme.bg_hover))
                        .text_color(rgb(theme.accent))
                })
                // Unselected state
                .when(!is_selected && enabled, |d| {
                    d.bg(rgb(theme.bg_input))
                        .text_color(rgb(theme.text_primary))
                        .hover(|d| {
                            d.bg(rgb(theme.bg_secondary))
                        })
                })
                // Disabled states
                .when(is_selected && !enabled, |d| {
                    d.bg(rgb(theme.disabled_bg))
                        .text_color(rgb(theme.disabled_text))
                })
                .when(!is_selected && !enabled, |d| {
                    d.bg(rgb(theme.disabled_bg))
                        .text_color(rgb(theme.disabled_text))
                })
                // Text content with focus ring
                .child(
                    div()
                        .px_1()
                        .border_1()
                        .rounded_sm()
                        .when(show_focus, |d| d.border_color(rgb(theme.border_focus)))
                        .when(!show_focus, |d| d.border_color(rgba(0x00000000)))
                        .child(item.label())
                )
        }))
    }
}
