//! Generic tab bar widget for switching between views
//!
//! A tab bar that can display any type implementing the `TabItem` trait.
//! Supports left-click tab switching and right-click context menus.
//!
//! # Example
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::{TabBar, TabBarEvent, TabItem};
//! use gpui::*;
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
            focus_handle: cx.focus_handle(),
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
        self.custom_theme.clone().unwrap_or_else(|| get_theme(cx))
    }
}

impl<T: TabItem> EventEmitter<TabBarEvent<T>> for TabBar<T> {}

impl<T: TabItem> Focusable for TabBar<T> {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl<T: TabItem> Render for TabBar<T> {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = self.get_theme(cx);
        let active_tab = self.active.clone();

        div()
            .flex()
            .flex_row()
            .bg(rgb(theme.bg_secondary))
            .children(self.tabs.clone().into_iter().map(|tab| {
                let is_active = tab == active_tab;
                let tab_for_click = tab.clone();
                let tab_for_context = tab.clone();

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
