//! Selection item trait for generic selection widgets
//!
//! This module defines the `SelectionItem` trait used by RadioGroup, SegmentedControl,
//! TabBar, and SidebarNav widgets. It also provides `StringItem` as a simple default
//! implementation for string-based selections.
//!
//! # Example: Custom Selection Type
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::SelectionItem;
//! use gpui::*;
//!
//! #[derive(Clone, PartialEq)]
//! enum Size {
//!     Small,
//!     Medium,
//!     Large,
//! }
//!
//! impl SelectionItem for Size {
//!     fn label(&self) -> SharedString {
//!         match self {
//!             Size::Small => "Small".into(),
//!             Size::Medium => "Medium".into(),
//!             Size::Large => "Large".into(),
//!         }
//!     }
//!
//!     fn id(&self) -> ElementId {
//!         match self {
//!             Size::Small => "size_small".into(),
//!             Size::Medium => "size_medium".into(),
//!             Size::Large => "size_large".into(),
//!         }
//!     }
//! }
//! ```

use gpui::{ElementId, SharedString};

/// Trait for items that can be displayed in selection widgets
///
/// Implement this trait to use custom types with RadioGroup, SegmentedControl,
/// TabBar, and SidebarNav widgets.
///
/// # Required Methods
///
/// - `label()`: Display text for the item
/// - `id()`: Unique element ID for the item (used for click handling and GPUI rendering)
pub trait SelectionItem: Clone + PartialEq + 'static {
    /// The display label for this item
    fn label(&self) -> SharedString;

    /// A unique element ID for this item (used for click handling)
    fn id(&self) -> ElementId;
}

/// A simple string-based selection item
///
/// Use this type for simple string selections without defining a custom enum.
/// The label and id are both derived from the string value.
///
/// # Example
///
/// ```ignore
/// use ccf_gpui_widgets::widgets::{RadioGroup, StringItem};
///
/// let radio = cx.new(|cx| {
///     RadioGroup::new(
///         vec![
///             StringItem::new("small"),
///             StringItem::new("medium"),
///             StringItem::new("large"),
///         ],
///         StringItem::new("medium"),
///         cx,
///     )
/// });
/// ```
#[derive(Clone, PartialEq, Debug)]
pub struct StringItem {
    value: String,
}

impl StringItem {
    /// Create a new StringItem from a string value
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    /// Get the underlying string value
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Consume and return the underlying string value
    pub fn into_value(self) -> String {
        self.value
    }
}

impl SelectionItem for StringItem {
    fn label(&self) -> SharedString {
        self.value.clone().into()
    }

    fn id(&self) -> ElementId {
        // Create stable ID from value - lowercase with underscores
        let id_str = format!("item_{}", self.value.to_lowercase().replace(' ', "_"));
        ElementId::Name(id_str.into())
    }
}

impl From<String> for StringItem {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for StringItem {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl std::fmt::Display for StringItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}
