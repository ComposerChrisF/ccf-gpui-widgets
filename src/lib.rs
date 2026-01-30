//! ccf-gpui-widgets - Reusable GPUI widgets for building desktop applications
//!
//! This crate provides a collection of ready-to-use UI widgets built on top of GPUI,
//! the GPU-accelerated UI framework from Zed Industries.
//!
//! # Features
//!
//! - **Themeable**: All widgets support custom themes via a global context or per-widget override
//! - **Accessible**: Keyboard navigation support where applicable
//! - **Event-driven**: All widgets emit events for state changes
//! - **Builder pattern**: Fluent API for widget configuration
//!
//! # Quick Start
//!
//! ```ignore
//! use gpui::*;
//! use ccf_gpui_widgets::{Theme, widgets::*};
//!
//! Application::new().run(|cx: &mut App| {
//!     // Register keybindings for widgets that need them
//!     register_all_keybindings(cx);
//!
//!     // Optionally set a global theme
//!     cx.set_global(Theme::dark());
//!
//!     cx.open_window(WindowOptions::default(), |_window, cx| {
//!         cx.new(|cx| {
//!             // Create your widgets
//!             let input = cx.new(|cx| TextInput::new(cx).placeholder("Enter text..."));
//!             let checkbox = cx.new(|cx| Checkbox::new(cx).label("Enable feature"));
//!             // ...
//!         })
//!     }).unwrap();
//!
//!     cx.activate(true);
//! });
//! ```
//!
//! # Available Widgets
//!
//! ## Core Widgets
//!
//! - [`TextInput`](widgets::TextInput) - Full-featured text input with cursor, selection, clipboard
//! - [`Checkbox`](widgets::Checkbox) - Simple checkbox with optional label
//! - [`Dropdown`](widgets::Dropdown) - Select/dropdown with keyboard navigation
//! - [`NumberStepper`](widgets::NumberStepper) - Numeric input with +/- buttons
//! - [`RadioGroup`](widgets::RadioGroup) - Single-selection from multiple choices
//! - [`CheckboxGroup`](widgets::CheckboxGroup) - Multi-selection from multiple choices
//! - [`ColorSwatch`](widgets::ColorSwatch) - Color preview with hex input
//! - [`Collapsible`](widgets::Collapsible) - Expandable/collapsible section header
//! - [`Tooltip`](widgets::Tooltip) - Simple tooltip for hover text
//!
//! ## File Widgets (requires `file-picker` feature)
//!
//! - [`FilePicker`](widgets::FilePicker) - File selection with native dialog, drag-drop
//! - [`DirectoryPicker`](widgets::DirectoryPicker) - Directory selection with native dialog
//!
//! # Feature Flags
//!
//! - `file-picker` - Enables FilePicker and DirectoryPicker widgets (adds `rfd` and `dirs` dependencies)
//! - `full` - Enables all optional features
//!
//! # Theming
//!
//! Widgets use a [`Theme`] struct for colors. You can:
//!
//! 1. Set a global theme: `cx.set_global(Theme::dark())`
//! 2. Use per-widget themes: `TextInput::new(cx).theme(my_theme)`
//! 3. Use the default (dark theme) if nothing is set
//!
//! ```ignore
//! use ccf_gpui_widgets::Theme;
//!
//! // Built-in themes
//! let dark = Theme::dark();
//! let light = Theme::light();
//!
//! // Customize with builder methods (all ~30 fields have with_* methods)
//! let custom = Theme::dark()
//!     .with_accent(0x00ff00)
//!     .with_primary(0xff0000)
//!     .with_bg_input(0x333333)
//!     .with_border_input(0x666666)
//!     .with_tooltip_bg(0x222222)
//!     .with_selection(0x264F78);
//! ```

pub mod theme;
pub mod utils;
pub mod widgets;

// Convenient re-exports
pub use theme::Theme;
pub use widgets::register_all_keybindings;

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::theme::{get_theme, get_theme_or, Theme};
    pub use crate::widgets::{
        register_all_keybindings,
        TextInput, TextInputEvent,
        Checkbox, CheckboxEvent,
        Dropdown, DropdownEvent,
        NumberStepper, NumberStepperEvent,
        RadioGroup, RadioGroupEvent,
        CheckboxGroup, CheckboxGroupEvent,
        ColorSwatch, ColorSwatchEvent,
        Collapsible, CollapsibleEvent,
        Tooltip,
        primary_button, secondary_button,
        PasswordInput, PasswordInputEvent,
        TabBar, TabBarEvent, TabItem,
        RepeatableTextInput, RepeatableTextInputEvent,
    };

    #[cfg(feature = "file-picker")]
    pub use crate::widgets::{
        FilePicker, FilePickerEvent, FileMode, MissingDirectories,
        DirectoryPicker, DirectoryPickerEvent,
        RepeatableFilePicker, RepeatableFilePickerEvent,
    };

    pub use crate::utils::{parse_path, expand_tilde, PathInfo};
}
