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
//! ## Input Widgets
//!
//! - [`TextInput`](widgets::TextInput) - Full-featured text input with cursor, selection, clipboard
//! - [`PasswordInput`](widgets::PasswordInput) - Text input with visibility toggle
//! - [`NumberStepper`](widgets::NumberStepper) - Numeric input with +/- buttons
//! - [`Slider`](widgets::Slider) - Horizontal slider for numeric ranges
//!
//! ## Selection Widgets
//!
//! - [`Checkbox`](widgets::Checkbox) - Simple checkbox with optional label
//! - [`ToggleSwitch`](widgets::ToggleSwitch) - On/off toggle with configurable label position
//! - [`Dropdown`](widgets::Dropdown) - Select/dropdown with keyboard navigation
//! - [`RadioGroup`](widgets::RadioGroup) - Single-selection from multiple choices
//! - [`CheckboxGroup`](widgets::CheckboxGroup) - Multi-selection from multiple choices
//! - [`ColorSwatch`](widgets::ColorSwatch) - Color picker with hex input, HSV canvas
//!
//! ## Display Widgets
//!
//! - [`Tooltip`](widgets::Tooltip) - Hover tooltip
//! - [`ProgressBar`](widgets::ProgressBar) - Progress indicator (determinate/indeterminate)
//! - [`Spinner`](widgets::Spinner) - Loading spinner in multiple sizes
//!
//! ## Layout & Navigation
//!
//! - [`Collapsible`](widgets::Collapsible) - Expandable/collapsible section
//! - [`TabBar`](widgets::TabBar) - Tab navigation with keyboard support
//! - [`ConfirmationDialog`](widgets::ConfirmationDialog) - Modal dialogs (Info/Default/Warning/Danger styles)
//!
//! ## Repeatable Widgets
//!
//! - [`RepeatableTextInput`](widgets::RepeatableTextInput) - Text input with add/remove for lists
//!
#![cfg_attr(
    feature = "file-picker",
    doc = "## File Widgets (requires `file-picker` feature)"
)]
#![cfg_attr(feature = "file-picker", doc = "")]
#![cfg_attr(
    feature = "file-picker",
    doc = "- [`FilePicker`](widgets::FilePicker) - File selection with native dialog"
)]
#![cfg_attr(
    feature = "file-picker",
    doc = "- [`DirectoryPicker`](widgets::DirectoryPicker) - Directory selection with native dialog"
)]
#![cfg_attr(
    feature = "file-picker",
    doc = "- [`RepeatableFilePicker`](widgets::RepeatableFilePicker) - File picker with add/remove for lists"
)]
#![cfg_attr(
    feature = "file-picker",
    doc = "- [`RepeatableDirectoryPicker`](widgets::RepeatableDirectoryPicker) - Directory picker with add/remove for lists"
)]
//!
//! ## Utilities
//!
//! - [`primary_button`](widgets::primary_button) - Blue/accent styled button
//! - [`secondary_button`](widgets::secondary_button) - Gray styled button
//! - [`danger_button`](widgets::danger_button) - Red styled button
//! - [`with_focus_actions`](widgets::with_focus_actions) - Add Tab/Shift-Tab focus navigation to elements
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
pub use theme::{Palette, Theme};
pub use widgets::register_all_keybindings;

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::theme::{get_theme, get_theme_or, Palette, Theme};
    pub use crate::widgets::{
        danger_button, primary_button, register_all_keybindings, scrollable_both,
        scrollable_horizontal, scrollable_vertical, secondary_button, Checkbox, CheckboxEvent,
        CheckboxGroup, CheckboxGroupEvent, Collapsible, CollapsibleEvent, ColorSwatch,
        ColorSwatchEvent, ConfirmationDialog, ConfirmationDialogEvent, DialogButton, DialogStyle,
        Dropdown, DropdownEvent, LabelPosition, NumberStepper, NumberStepperEvent, PasswordInput,
        PasswordInputEvent, ProgressBar, ProgressBarEvent, RadioGroup, RadioGroupEvent,
        RepeatableTextInput, RepeatableTextInputEvent, Scrollable, ScrollbarAxis, SegmentOption,
        SegmentedControl, SegmentedControlEvent, SelectionItem, SidebarNav, SidebarNavEvent,
        Slider, SliderEvent, Spinner, SpinnerSize, StringItem, TabBar, TabBarEvent, TextInput,
        TextInputEvent, ToggleSwitch, ToggleSwitchEvent, Tooltip,
    };

    #[cfg(feature = "file-picker")]
    pub use crate::widgets::{
        DirectoryPicker, DirectoryPickerEvent, FileMode, FilePicker, FilePickerEvent,
        MissingDirectories, RepeatableDirectoryPicker, RepeatableDirectoryPickerEvent,
        RepeatableFilePicker, RepeatableFilePickerEvent,
    };

    pub use crate::utils::{expand_tilde, parse_path, PathInfo};
}
