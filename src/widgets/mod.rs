//! GPUI widgets

mod button;
mod checkbox;
mod checkbox_group;
mod collapsible;
mod color_swatch;
mod confirmation_dialog;
mod cursor_blink;
mod dropdown;
mod editing_core;
mod focus_navigation;
mod number_stepper;
mod password_input;
mod progress_bar;
mod radio_group;
mod repeatable_text_input;
mod scrollable;
mod scrollbar;
mod segmented_control;
mod selection;
mod sidebar_nav;
mod slider;
mod spinner;
mod tab_bar;
mod text_input;
mod toggle_switch;
mod tooltip;

#[cfg(feature = "file-picker")]
mod path_display;

#[cfg(feature = "secure-password")]
mod sensitive_string;

#[cfg(feature = "file-picker")]
mod directory_picker;
#[cfg(feature = "file-picker")]
mod file_picker;
#[cfg(feature = "file-picker")]
mod repeatable_directory_picker;
#[cfg(feature = "file-picker")]
mod repeatable_file_picker;

// Re-exports
pub use button::{
    danger_button, primary_button, register_keybindings as register_button_keybindings,
    secondary_button,
};
pub use checkbox::{Checkbox, CheckboxEvent};
pub use checkbox_group::{CheckboxGroup, CheckboxGroupEvent};
pub use collapsible::{Collapsible, CollapsibleEvent};
pub use color_swatch::{
    register_keybindings as register_color_swatch_keybindings, ColorSwatch, ColorSwatchEvent,
    PickerMode,
};
pub use confirmation_dialog::{
    ConfirmationDialog, ConfirmationDialogEvent, DialogButton, DialogStyle,
};
pub use dropdown::{
    register_keybindings as register_dropdown_keybindings, Dropdown, DropdownEvent,
};
pub use focus_navigation::{
    register_keybindings as register_focus_navigation_keybindings, repeatable_add_button,
    repeatable_remove_button, with_focus_actions, EnabledCursorExt, FocusNext, FocusPrev,
};
pub use number_stepper::{NumberStepper, NumberStepperEvent};
pub use password_input::{PasswordInput, PasswordInputEvent};
pub use progress_bar::{ProgressBar, ProgressBarEvent};
pub use radio_group::{RadioGroup, RadioGroupEvent};
pub use repeatable_text_input::{
    register_keybindings as register_repeatable_text_input_keybindings,
    ActivateButton as RepeatableActivateButton, RepeatableTextInput, RepeatableTextInputEvent,
};
pub use scrollable::{scrollable_both, scrollable_horizontal, scrollable_vertical, Scrollable};
pub use scrollbar::ScrollbarAxis;
#[cfg(feature = "secure-password")]
pub use secrecy::SecretString;
pub use segmented_control::{SegmentOption, SegmentedControl, SegmentedControlEvent};
pub use selection::{SelectionItem, StringItem};
pub use sidebar_nav::{
    register_keybindings as register_sidebar_nav_keybindings, SidebarNav, SidebarNavEvent,
};
pub use slider::{Slider, SliderEvent};
pub use spinner::{Spinner, SpinnerSize};
pub use tab_bar::{register_keybindings as register_tab_bar_keybindings, TabBar, TabBarEvent};
pub use text_input::{
    register_keybindings as register_text_input_keybindings, TextInput, TextInputEvent,
};
pub use toggle_switch::{LabelPosition, ToggleSwitch, ToggleSwitchEvent};
pub use tooltip::Tooltip;

#[cfg(feature = "file-picker")]
pub use directory_picker::{
    register_keybindings as register_directory_picker_keybindings, validate_directory_path,
    DirectoryPicker, DirectoryPickerEvent, DirectoryPickerValidation,
    ValidationDisplay as DirectoryValidationDisplay,
};
#[cfg(feature = "file-picker")]
pub use file_picker::{
    register_keybindings as register_file_picker_keybindings, validate_file_path, FileMode,
    FilePicker, FilePickerEvent, FilePickerValidation, MissingDirectories, ValidationDisplay,
};
#[cfg(feature = "file-picker")]
pub use repeatable_directory_picker::{RepeatableDirectoryPicker, RepeatableDirectoryPickerEvent};
#[cfg(feature = "file-picker")]
pub use repeatable_file_picker::{RepeatableFilePicker, RepeatableFilePickerEvent};

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
    register_sidebar_nav_keybindings(cx);
    register_repeatable_text_input_keybindings(cx);
    register_button_keybindings(cx);
    #[cfg(feature = "file-picker")]
    {
        register_file_picker_keybindings(cx);
        register_directory_picker_keybindings(cx);
    }
}
