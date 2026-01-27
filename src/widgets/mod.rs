//! GPUI widgets

mod text_input;
mod tooltip;
mod checkbox;
mod dropdown;
mod number_stepper;
mod radio_group;
mod checkbox_group;
mod color_swatch;
mod collapsible;
mod focus_navigation;

#[cfg(feature = "file-picker")]
mod file_picker;
#[cfg(feature = "file-picker")]
mod directory_picker;

// Re-exports
pub use text_input::{TextInput, TextInputEvent, register_keybindings as register_text_input_keybindings};
pub use tooltip::Tooltip;
pub use checkbox::{Checkbox, CheckboxEvent};
pub use dropdown::{Dropdown, DropdownEvent, register_keybindings as register_dropdown_keybindings};
pub use number_stepper::{NumberStepper, NumberStepperEvent, register_keybindings as register_number_stepper_keybindings};
pub use radio_group::{RadioGroup, RadioGroupEvent};
pub use checkbox_group::{CheckboxGroup, CheckboxGroupEvent};
pub use color_swatch::{ColorSwatch, ColorSwatchEvent};
pub use collapsible::{Collapsible, CollapsibleEvent};
pub use focus_navigation::{FocusNext, FocusPrev, register_keybindings as register_focus_navigation_keybindings};

#[cfg(feature = "file-picker")]
pub use file_picker::{FilePicker, FilePickerEvent, FileMode, MissingDirectories};
#[cfg(feature = "file-picker")]
pub use directory_picker::{DirectoryPicker, DirectoryPickerEvent};

/// Register all widget keybindings
///
/// Call this once at application startup to enable keyboard shortcuts
/// for all widgets that require them.
///
/// ```ignore
/// use ccf_gpui_widgets::widgets::register_all_keybindings;
///
/// Application::new().run(|cx: &mut App| {
///     register_all_keybindings(cx);
///     // ... rest of your initialization
/// });
/// ```
pub fn register_all_keybindings(cx: &mut gpui::App) {
    register_text_input_keybindings(cx);
    register_dropdown_keybindings(cx);
    register_number_stepper_keybindings(cx);
    register_focus_navigation_keybindings(cx);
}
