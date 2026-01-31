//! GPUI widgets

mod cursor_blink;
mod editing_core;
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
mod button;
mod password_input;
mod tab_bar;
mod repeatable_text_input;
mod toggle_switch;
mod slider;
mod progress_bar;
mod spinner;
mod confirmation_dialog;

#[cfg(feature = "secure-password")]
mod sensitive_string;

#[cfg(feature = "file-picker")]
mod file_picker;
#[cfg(feature = "file-picker")]
mod directory_picker;
#[cfg(feature = "file-picker")]
mod repeatable_file_picker;
#[cfg(feature = "file-picker")]
mod repeatable_directory_picker;

// Re-exports
pub use text_input::{TextInput, TextInputEvent, register_keybindings as register_text_input_keybindings};
pub use tooltip::Tooltip;
pub use checkbox::{Checkbox, CheckboxEvent};
pub use dropdown::{Dropdown, DropdownEvent, register_keybindings as register_dropdown_keybindings};
pub use number_stepper::{NumberStepper, NumberStepperEvent};
pub use radio_group::{RadioGroup, RadioGroupEvent};
pub use checkbox_group::{CheckboxGroup, CheckboxGroupEvent};
pub use color_swatch::{ColorSwatch, ColorSwatchEvent, PickerMode, register_keybindings as register_color_swatch_keybindings};
pub use collapsible::{Collapsible, CollapsibleEvent};
pub use focus_navigation::{FocusNext, FocusPrev, register_keybindings as register_focus_navigation_keybindings};
pub use button::{primary_button, secondary_button, danger_button, register_keybindings as register_button_keybindings};
pub use toggle_switch::{ToggleSwitch, ToggleSwitchEvent, LabelPosition};
pub use slider::{Slider, SliderEvent};
pub use progress_bar::{ProgressBar, ProgressBarEvent};
pub use spinner::{Spinner, SpinnerSize};
pub use confirmation_dialog::{ConfirmationDialog, ConfirmationDialogEvent, DialogStyle, DialogButton};
pub use password_input::{PasswordInput, PasswordInputEvent};
#[cfg(feature = "secure-password")]
pub use secrecy::SecretString;
pub use tab_bar::{TabBar, TabBarEvent, TabItem, register_keybindings as register_tab_bar_keybindings};
pub use repeatable_text_input::{RepeatableTextInput, RepeatableTextInputEvent, ActivateButton as RepeatableActivateButton, register_keybindings as register_repeatable_text_input_keybindings};

#[cfg(feature = "file-picker")]
pub use file_picker::{
    FilePicker, FilePickerEvent, FileMode, MissingDirectories,
    FilePickerValidation, ValidationDisplay, validate_file_path,
    register_keybindings as register_file_picker_keybindings,
};
#[cfg(feature = "file-picker")]
pub use directory_picker::{
    DirectoryPicker, DirectoryPickerEvent,
    DirectoryPickerValidation, ValidationDisplay as DirectoryValidationDisplay,
    validate_directory_path,
    register_keybindings as register_directory_picker_keybindings,
};
#[cfg(feature = "file-picker")]
pub use repeatable_file_picker::{RepeatableFilePicker, RepeatableFilePickerEvent};
#[cfg(feature = "file-picker")]
pub use repeatable_directory_picker::{RepeatableDirectoryPicker, RepeatableDirectoryPickerEvent};

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
    register_color_swatch_keybindings(cx);
    register_focus_navigation_keybindings(cx);
    register_tab_bar_keybindings(cx);
    register_repeatable_text_input_keybindings(cx);
    register_button_keybindings(cx);
    #[cfg(feature = "file-picker")]
    {
        register_file_picker_keybindings(cx);
        register_directory_picker_keybindings(cx);
    }
}
