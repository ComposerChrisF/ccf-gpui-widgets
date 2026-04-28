//! Scrollable component with visible scrollbars.
//!
//! A wrapper component that adds visible, interactive scrollbars to any content.
//! Unlike native GPUI `overflow_y_scroll()` which enables scrolling but renders no
//! visible scrollbar, this component provides:
//!
//! - Visible, themed scrollbars
//! - `.always_show_scrollbars()` option
//! - Interactive thumb (drag to scroll)
//! - Click-on-track to jump
//! - Auto-fade after inactivity
//! - ScrollHandle integration for programmatic control
//! - Scroll events don't bubble to parent containers
//!
//! # Vertical Scrolling
//!
//! Vertical scrolling works naturally with GPUI's layout system. Content that
//! exceeds the container height will automatically trigger scrolling.
//!
//! ```ignore
//! use ccf_gpui_widgets::scrollable_vertical;
//! use gpui::*;
//!
//! // Container with fixed height - content scrolls when it exceeds this height
//! div()
//!     .h(px(200.0))
//!     .child(
//!         scrollable_vertical(
//!             div().children(many_items)  // Content grows naturally
//!         )
//!     )
//! ```
//!
//! **Vertical scrolling pitfalls:**
//! - The scrollable container needs a constrained height (explicit or from parent)
//! - Without height constraint, content expands infinitely and never scrolls
//! - Use `.h(px(...))`, `.max_h(px(...))`, or ensure parent constrains height
//!
//! # Horizontal Scrolling
//!
//! **Important:** Horizontal scrolling requires explicit width on content due to
//! GPUI layout limitations. Flex items shrink to fit by default, so without
//! explicit width, GPUI cannot detect content overflow.
//!
//! ```ignore
//! use ccf_gpui_widgets::scrollable_horizontal;
//! use gpui::*;
//!
//! // Container with fixed width
//! div()
//!     .w(px(300.0))
//!     .child(
//!         scrollable_horizontal(
//!             div()
//!                 .w(px(800.0))  // REQUIRED: explicit width > container width
//!                 .flex()
//!                 .flex_row()
//!                 .gap_2()
//!                 .children(items)
//!         )
//!     )
//! ```
//!
//! **Horizontal scrolling pitfalls:**
//! - Content MUST have explicit width via `.w(px(...))` that exceeds container
//! - `flex_shrink_0()` on items is NOT sufficient - the container needs explicit width
//! - Without explicit width, scrollbar won't appear and content won't scroll
//! - Calculate required width based on content (e.g., `num_items * item_width + gaps`)
//!
//! **What does NOT work for horizontal:**
//! ```ignore
//! // WRONG: No explicit width - content shrinks to fit container
//! scrollable_horizontal(
//!     div()
//!         .flex()
//!         .flex_row()
//!         .children(items.iter().map(|i| div().flex_shrink_0().child(i)))
//! )
//!
//! // WRONG: flex_shrink_0 on container doesn't help
//! scrollable_horizontal(
//!     div()
//!         .flex_shrink_0()  // This doesn't prevent layout constraint
//!         .flex()
//!         .flex_row()
//!         .children(items)
//! )
//! ```
//!
//! # Bidirectional Scrolling
//!
//! For content that scrolls both horizontally and vertically:
//!
//! ```ignore
//! use ccf_gpui_widgets::scrollable_both;
//!
//! scrollable_both(
//!     div()
//!         .w(px(800.0))  // Explicit width for horizontal
//!         // Height grows naturally for vertical
//!         .children(content)
//! )
//! ```
//!
//! # Options
//!
//! ```ignore
//! scrollable_vertical(content)
//!     .with_scroll_handle(my_handle)  // For programmatic scroll control
//!     .always_show_scrollbars()       // Don't auto-hide scrollbars
//!     .theme(custom_theme)            // Custom scrollbar colors
//!     .id("my-scrollable")            // Custom element ID
//! ```

use super::scrollbar::{Scrollbar, ScrollbarAxis, ScrollbarState};
use crate::theme::Theme;
use gpui::{
    div, relative, AnyElement, App, Bounds, Div, Element, ElementId, GlobalElementId,
    InspectorElementId, InteractiveElement, Interactivity, IntoElement, LayoutId, ParentElement,
    Pixels, Position, ScrollHandle, SharedString, Stateful, StatefulInteractiveElement, Style,
    StyleRefinement, Styled, Window,
};
use std::panic::Location;

/// A scroll view with visible scrollbars
///
/// Wraps content and adds themed scrollbars that appear on scroll
/// and fade out after inactivity.
pub struct Scrollable<E> {
    id: ElementId,
    element: Option<E>,
    axis: ScrollbarAxis,
    always_show_scrollbars: bool,
    external_scroll_handle: Option<ScrollHandle>,
    custom_theme: Option<Theme>,
    _element: Stateful<Div>,
}

impl<E> Scrollable<E>
where
    E: Element,
{
    /// Internal constructor that uses the provided location for ID generation
    fn new_with_location(
        axis: ScrollbarAxis,
        element: E,
        location: &'static Location<'static>,
    ) -> Self {
        // Generate a stable ID based on call site location
        // This ensures the same scrollable gets the same ID across renders
        let id = ElementId::Name(SharedString::from(format!(
            "scrollable-{}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        )));

        Self {
            element: Some(element),
            _element: div().id("fake"),
            id,
            axis,
            always_show_scrollbars: false,
            external_scroll_handle: None,
            custom_theme: None,
        }
    }

    /// Create a vertical scrollable container
    #[track_caller]
    pub fn vertical(element: E) -> Self {
        Self::new_with_location(ScrollbarAxis::Vertical, element, Location::caller())
    }

    /// Create a horizontal scrollable container
    #[track_caller]
    pub fn horizontal(element: E) -> Self {
        Self::new_with_location(ScrollbarAxis::Horizontal, element, Location::caller())
    }

    /// Create a scrollable container with both axes
    #[track_caller]
    pub fn both(element: E) -> Self {
        Self::new_with_location(ScrollbarAxis::Both, element, Location::caller())
    }

    /// Always show scrollbars (don't fade out)
    #[must_use]
    pub fn always_show_scrollbars(mut self) -> Self {
        self.always_show_scrollbars = true;
        self
    }

    /// Attach an external scroll handle for programmatic control
    #[must_use]
    pub fn with_scroll_handle(mut self, handle: ScrollHandle) -> Self {
        self.external_scroll_handle = Some(handle);
        self
    }

    /// Set the element ID
    #[must_use]
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    /// Set custom theme (builder pattern)
    #[must_use]
    pub fn theme(mut self, theme: Theme) -> Self {
        self.custom_theme = Some(theme);
        self
    }

    fn with_element_state<R>(
        &mut self,
        id: &GlobalElementId,
        window: &mut Window,
        cx: &mut App,
        f: impl FnOnce(&mut Self, &mut ScrollViewState, &mut Window, &mut App) -> R,
    ) -> R {
        window.with_optional_element_state::<ScrollViewState, _>(
            Some(id),
            |element_state, window| {
                let mut element_state = element_state.unwrap().unwrap_or_default();
                let result = f(self, &mut element_state, window, cx);
                (result, Some(element_state))
            },
        )
    }
}

/// Internal state for the scroll view
pub struct ScrollViewState {
    state: ScrollbarState,
    handle: ScrollHandle,
}

impl Default for ScrollViewState {
    fn default() -> Self {
        Self {
            handle: ScrollHandle::new(),
            state: ScrollbarState::default(),
        }
    }
}

impl<E> ParentElement for Scrollable<E>
where
    E: Element + ParentElement,
{
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        if let Some(element) = &mut self.element {
            element.extend(elements);
        }
    }
}

impl<E> Styled for Scrollable<E>
where
    E: Element + Styled,
{
    fn style(&mut self) -> &mut StyleRefinement {
        if let Some(element) = &mut self.element {
            element.style()
        } else {
            self._element.style()
        }
    }
}

impl<E> InteractiveElement for Scrollable<E>
where
    E: Element + InteractiveElement,
{
    fn interactivity(&mut self) -> &mut Interactivity {
        if let Some(element) = &mut self.element {
            element.interactivity()
        } else {
            self._element.interactivity()
        }
    }
}

impl<E> StatefulInteractiveElement for Scrollable<E> where E: Element + StatefulInteractiveElement {}

impl<E> IntoElement for Scrollable<E>
where
    E: Element,
{
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl<E> Element for Scrollable<E>
where
    E: Element,
{
    type RequestLayoutState = AnyElement;
    type PrepaintState = ScrollViewState;

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        id: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style {
            flex_grow: 1.0,
            position: Position::Relative,
            ..Default::default()
        };
        style.size.width = relative(1.0).into();
        style.size.height = relative(1.0).into();

        let axis = self.axis;
        let scroll_id = self.id.clone();
        let content = self.element.take().map(|c| c.into_any_element());
        let always_show = self.always_show_scrollbars;

        self.with_element_state(
            id.unwrap(),
            window,
            cx,
            |scrollable, element_state, window, cx| {
                let scroll_handle =
                    if let Some(ref external_handle) = scrollable.external_scroll_handle {
                        external_handle
                    } else {
                        &element_state.handle
                    };

                let mut scrollbar = Scrollbar::new(axis, &element_state.state, scroll_handle);
                if always_show {
                    scrollbar = scrollbar.always_visible();
                }
                if let Some(ref theme) = scrollable.custom_theme {
                    scrollbar = scrollbar.theme(*theme);
                }

                // Build the scroll container with axis-appropriate layout
                let inner_scroll = div()
                    .id(scroll_id.clone())
                    .track_scroll(scroll_handle)
                    .on_scroll_wheel(|_event, _window, cx| {
                        // Stop propagation to prevent parent containers from scrolling
                        cx.stop_propagation();
                    });

                // Apply axis-specific layout and overflow
                let inner_scroll = match axis {
                    ScrollbarAxis::Vertical => {
                        // Vertical: wrap content to allow height growth
                        inner_scroll
                            .size_full()
                            .overflow_y_scroll()
                            .child(div().w_full().children(content))
                    }
                    ScrollbarAxis::Horizontal => {
                        // Horizontal: no wrapper, content directly in scroll container
                        inner_scroll
                            .size_full()
                            .overflow_x_scroll()
                            .children(content)
                    }
                    ScrollbarAxis::Both => {
                        // Both: wrap content to allow growth in both directions
                        inner_scroll
                            .size_full()
                            .overflow_scroll()
                            .child(div().flex_shrink_0().children(content))
                    }
                };

                let mut element = div()
                    .relative()
                    .size_full()
                    .overflow_hidden()
                    .child(inner_scroll)
                    .child(
                        div()
                            .absolute()
                            .top_0()
                            .left_0()
                            .right_0()
                            .bottom_0()
                            .child(scrollbar),
                    )
                    .into_any_element();

                let element_id = element.request_layout(window, cx);
                let layout_id = window.request_layout(style, vec![element_id], cx);

                (layout_id, element)
            },
        )
    }

    fn prepaint(
        &mut self,
        id: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        element.prepaint(window, cx);

        // Access the cached state to preserve scroll position
        self.with_element_state(id.unwrap(), window, cx, |_, state, _, _| ScrollViewState {
            handle: state.handle.clone(),
            state: state.state.clone(),
        })
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        _: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        element.paint(window, cx)
    }
}

/// Create a vertical scrollable container
///
/// # Example
///
/// ```ignore
/// scrollable_vertical(
///     div()
///         .flex()
///         .flex_col()
///         .children(items)
/// )
/// ```
#[track_caller]
pub fn scrollable_vertical<E>(element: E) -> Scrollable<E>
where
    E: Element,
{
    Scrollable::new_with_location(ScrollbarAxis::Vertical, element, Location::caller())
}

/// Create a horizontal scrollable container
///
/// # Example
///
/// ```ignore
/// scrollable_horizontal(
///     div()
///         .flex()
///         .flex_row()
///         .children(items)
/// )
/// ```
#[track_caller]
pub fn scrollable_horizontal<E>(element: E) -> Scrollable<E>
where
    E: Element,
{
    Scrollable::new_with_location(ScrollbarAxis::Horizontal, element, Location::caller())
}

/// Create a scrollable container with both axes
///
/// # Example
///
/// ```ignore
/// scrollable_both(
///     div().children(items)
/// )
/// ```
#[track_caller]
pub fn scrollable_both<E>(element: E) -> Scrollable<E>
where
    E: Element,
{
    Scrollable::new_with_location(ScrollbarAxis::Both, element, Location::caller())
}
