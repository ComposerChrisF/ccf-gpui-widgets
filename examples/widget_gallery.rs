//! Widget Gallery - A demo application showcasing all ccf-gpui-widgets
//!
//! Run with: cargo run --example widget_gallery --features full
//! Or without file pickers: cargo run --example widget_gallery

use gpui::prelude::FluentBuilder;
use gpui::*;
use ccf_gpui_widgets::prelude::*;
use ccf_gpui_widgets::{Theme, Palette};
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

// Define global action for quitting the app
actions!(widget_gallery, [Quit]);

/// Maximum number of events to keep in the log
const MAX_EVENT_LOG: usize = 50;

/// Tab enum for TabBar demo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GalleryTab {
    Overview,
    Details,
    Settings,
}

impl SelectionItem for GalleryTab {
    fn label(&self) -> SharedString {
        match self {
            GalleryTab::Overview => "Overview".into(),
            GalleryTab::Details => "Details".into(),
            GalleryTab::Settings => "Settings".into(),
        }
    }

    fn id(&self) -> ElementId {
        match self {
            GalleryTab::Overview => "tab_overview".into(),
            GalleryTab::Details => "tab_details".into(),
            GalleryTab::Settings => "tab_settings".into(),
        }
    }
}

/// Category enum for main gallery navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GalleryCategory {
    Text,
    Selection,
    Numbers,
    Files,
    Progress,
    Utility,
    Misc,
}

impl SelectionItem for GalleryCategory {
    fn label(&self) -> SharedString {
        match self {
            GalleryCategory::Text => "Text".into(),
            GalleryCategory::Selection => "Selection".into(),
            GalleryCategory::Numbers => "Numbers".into(),
            GalleryCategory::Files => "Files".into(),
            GalleryCategory::Progress => "Progress".into(),
            GalleryCategory::Utility => "Utility".into(),
            GalleryCategory::Misc => "Misc".into(),
        }
    }

    fn id(&self) -> ElementId {
        match self {
            GalleryCategory::Text => "category_text".into(),
            GalleryCategory::Selection => "category_selection".into(),
            GalleryCategory::Numbers => "category_numbers".into(),
            GalleryCategory::Files => "category_files".into(),
            GalleryCategory::Progress => "category_progress".into(),
            GalleryCategory::Utility => "category_utility".into(),
            GalleryCategory::Misc => "category_misc".into(),
        }
    }
}

/// Section enum for SidebarNav sample in Selection category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GallerySidebarSection {
    Dashboard,
    Analytics,
    Reports,
    Settings,
}

impl SelectionItem for GallerySidebarSection {
    fn label(&self) -> SharedString {
        match self {
            GallerySidebarSection::Dashboard => "Dashboard".into(),
            GallerySidebarSection::Analytics => "Analytics".into(),
            GallerySidebarSection::Reports => "Reports".into(),
            GallerySidebarSection::Settings => "Settings".into(),
        }
    }

    fn id(&self) -> ElementId {
        match self {
            GallerySidebarSection::Dashboard => "sidebar_dashboard".into(),
            GallerySidebarSection::Analytics => "sidebar_analytics".into(),
            GallerySidebarSection::Reports => "sidebar_reports".into(),
            GallerySidebarSection::Settings => "sidebar_settings".into(),
        }
    }
}

/// Main application state
struct WidgetGallery {
    // Theme selection
    current_theme: ThemeChoice,
    theme_selector: Entity<SegmentedControl<ThemeChoice>>,

    // Custom palette editor
    show_palette_editor: bool,
    custom_palette: Palette,
    palette_bg_swatch: Entity<ColorSwatch>,
    palette_text_swatch: Entity<ColorSwatch>,
    palette_primary_swatch: Entity<ColorSwatch>,
    palette_accent_swatch: Entity<ColorSwatch>,
    palette_success_swatch: Entity<ColorSwatch>,
    palette_error_swatch: Entity<ColorSwatch>,
    palette_warning_swatch: Entity<ColorSwatch>,

    // Whether widgets are enabled (for disabled state demo)
    widgets_enabled: bool,

    // Category navigation
    category_sidebar_nav: Entity<SidebarNav<GalleryCategory>>,
    active_category: GalleryCategory,

    // Widgets
    text_input: Entity<TextInput>,
    text_input_placeholder: Entity<TextInput>,
    checkbox: Entity<Checkbox>,
    checkbox_labeled: Entity<Checkbox>,
    dropdown: Entity<Dropdown>,
    number_stepper: Entity<NumberStepper>,
    number_stepper_float: Entity<NumberStepper>,
    radio_group: Entity<RadioGroup>,
    checkbox_group: Entity<CheckboxGroup>,
    color_swatch: Entity<ColorSwatch>,
    color_swatch_alpha: Entity<ColorSwatch>,

    #[cfg(feature = "file-picker")]
    file_picker: Entity<FilePicker>,
    #[cfg(feature = "file-picker")]
    directory_picker: Entity<DirectoryPicker>,

    // New widgets
    password_input: Entity<PasswordInput>,
    tab_bar: Entity<TabBar<GalleryTab>>,
    sidebar_nav_sample: Entity<SidebarNav<GallerySidebarSection>>,
    repeatable_text_input: Entity<RepeatableTextInput>,
    #[cfg(feature = "file-picker")]
    repeatable_file_picker: Entity<RepeatableFilePicker>,
    #[cfg(feature = "file-picker")]
    repeatable_directory_picker: Entity<RepeatableDirectoryPicker>,

    // New widgets (toggle, slider, progress, spinner, dialog)
    toggle_switch: Entity<ToggleSwitch>,
    toggle_switch_labeled: Entity<ToggleSwitch>,
    slider: Entity<Slider>,
    slider_with_value: Entity<Slider>,
    progress_bar: Entity<ProgressBar>,
    progress_bar_indeterminate: Entity<ProgressBar>,
    spinner: Entity<Spinner>,
    spinner_small: Entity<Spinner>,
    spinner_medium: Entity<Spinner>,
    spinner_large: Entity<Spinner>,
    // Segmented control
    segmented_control: Entity<SegmentedControl>,

    // Collapsible sections
    collapsible: Entity<Collapsible>,
    collapsible_expanded: Entity<Collapsible>,
    collapsible_static: Entity<Collapsible>,

    // Dialog state
    show_info_dialog: bool,
    info_dialog: Entity<ConfirmationDialog>,
    info_result: Option<&'static str>,
    show_yes_no_dialog: bool,
    yes_no_dialog: Entity<ConfirmationDialog>,
    yes_no_result: Option<&'static str>,
    show_save_dialog: bool,
    save_dialog: Entity<ConfirmationDialog>,
    save_result: Option<&'static str>,
    show_danger_dialog: bool,
    danger_dialog: Entity<ConfirmationDialog>,
    danger_result: Option<&'static str>,

    // Button click tracking (buttons are not Entities)
    primary_click_count: usize,
    secondary_click_count: usize,
    danger_click_count: usize,

    // Event log
    event_log: VecDeque<EventLogEntry>,
    log_collapsed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ThemeChoice {
    Light,
    Dark,
    Custom,
}

impl SelectionItem for ThemeChoice {
    fn label(&self) -> SharedString {
        match self {
            ThemeChoice::Light => "Light".into(),
            ThemeChoice::Dark => "Dark".into(),
            ThemeChoice::Custom => "Custom".into(),
        }
    }

    fn id(&self) -> ElementId {
        match self {
            ThemeChoice::Light => "theme_light".into(),
            ThemeChoice::Dark => "theme_dark".into(),
            ThemeChoice::Custom => "theme_custom".into(),
        }
    }
}

struct EventLogEntry {
    timestamp: String,
    widget: String,
    event: String,
}

/// Helper macro to subscribe a widget to its event type and log events
macro_rules! subscribe_widget {
    ($cx:expr, $widget:expr, $name:expr, $event_type:ty) => {
        $cx.subscribe($widget, |this, _entity, event: &$event_type, cx| {
            this.log_event($name, format!("{:?}", event), cx);
        })
        .detach();
    };
}

/// Helper macro to update multiple widgets with set_enabled
macro_rules! set_enabled_all {
    ($cx:expr, $enabled:expr, $($widget:expr),+ $(,)?) => {
        $( $widget.update($cx, |w, cx| w.set_enabled($enabled, cx)); )+
    };
}

/// Maps dialog event to result label
fn dialog_result_label(
    event: &ConfirmationDialogEvent,
    primary: &'static str,
    secondary: &'static str,
    tertiary: &'static str,
) -> &'static str {
    match event {
        ConfirmationDialogEvent::Primary => primary,
        ConfirmationDialogEvent::Secondary => secondary,
        ConfirmationDialogEvent::Tertiary => tertiary,
    }
}

/// Convert hex string (with or without #) to u32
fn hex_to_u32(hex: &str) -> u32 {
    let hex = hex.trim_start_matches('#');
    // Handle both 6-char (RRGGBB) and 8-char (RRGGBBAA) formats
    let hex = if hex.len() > 6 { &hex[..6] } else { hex };
    u32::from_str_radix(hex, 16).unwrap_or(0)
}

/// Convert u32 to hex string with # prefix
fn u32_to_hex(val: u32) -> String {
    format!("#{:06x}", val & 0xFFFFFF)
}

impl WidgetGallery {
    fn new(cx: &mut Context<Self>) -> Self {
        // Create theme selector
        let theme_selector = cx.new(|cx| {
            SegmentedControl::new_with_items(
                vec![ThemeChoice::Light, ThemeChoice::Dark, ThemeChoice::Custom],
                ThemeChoice::Dark,
                cx,
            )
        });

        // Subscribe to theme selector changes
        cx.subscribe(&theme_selector, |this, _entity, event: &SegmentedControlEvent<ThemeChoice>, cx| {
            let SegmentedControlEvent::Change(choice) = event;
            this.handle_theme_change(*choice, cx);
        })
        .detach();

        // Initialize custom palette with dark defaults
        let custom_palette = Palette::dark();

        // Create palette editor color swatches
        let palette_bg_swatch = cx.new(|cx| {
            ColorSwatch::new(cx).with_value(u32_to_hex(custom_palette.bg))
        });
        let palette_text_swatch = cx.new(|cx| {
            ColorSwatch::new(cx).with_value(u32_to_hex(custom_palette.text))
        });
        let palette_primary_swatch = cx.new(|cx| {
            ColorSwatch::new(cx).with_value(u32_to_hex(custom_palette.primary))
        });
        let palette_accent_swatch = cx.new(|cx| {
            ColorSwatch::new(cx).with_value(u32_to_hex(custom_palette.accent))
        });
        let palette_success_swatch = cx.new(|cx| {
            ColorSwatch::new(cx).with_value(u32_to_hex(custom_palette.success))
        });
        let palette_error_swatch = cx.new(|cx| {
            ColorSwatch::new(cx).with_value(u32_to_hex(custom_palette.error))
        });
        let palette_warning_swatch = cx.new(|cx| {
            ColorSwatch::new(cx).with_value(u32_to_hex(custom_palette.warning))
        });

        // Subscribe to palette swatch changes
        cx.subscribe(&palette_bg_swatch, |this, _entity, event: &ColorSwatchEvent, cx| {
            let ColorSwatchEvent::Change(hex) = event;
            this.custom_palette.bg = hex_to_u32(hex);
            this.log_event("Palette", format!("bg = {}", hex), cx);
        }).detach();
        cx.subscribe(&palette_text_swatch, |this, _entity, event: &ColorSwatchEvent, cx| {
            let ColorSwatchEvent::Change(hex) = event;
            this.custom_palette.text = hex_to_u32(hex);
            this.log_event("Palette", format!("text = {}", hex), cx);
        }).detach();
        cx.subscribe(&palette_primary_swatch, |this, _entity, event: &ColorSwatchEvent, cx| {
            let ColorSwatchEvent::Change(hex) = event;
            this.custom_palette.primary = hex_to_u32(hex);
            this.log_event("Palette", format!("primary = {}", hex), cx);
        }).detach();
        cx.subscribe(&palette_accent_swatch, |this, _entity, event: &ColorSwatchEvent, cx| {
            let ColorSwatchEvent::Change(hex) = event;
            this.custom_palette.accent = hex_to_u32(hex);
            this.log_event("Palette", format!("accent = {}", hex), cx);
        }).detach();
        cx.subscribe(&palette_success_swatch, |this, _entity, event: &ColorSwatchEvent, cx| {
            let ColorSwatchEvent::Change(hex) = event;
            this.custom_palette.success = hex_to_u32(hex);
            this.log_event("Palette", format!("success = {}", hex), cx);
        }).detach();
        cx.subscribe(&palette_error_swatch, |this, _entity, event: &ColorSwatchEvent, cx| {
            let ColorSwatchEvent::Change(hex) = event;
            this.custom_palette.error = hex_to_u32(hex);
            this.log_event("Palette", format!("error = {}", hex), cx);
        }).detach();
        cx.subscribe(&palette_warning_swatch, |this, _entity, event: &ColorSwatchEvent, cx| {
            let ColorSwatchEvent::Change(hex) = event;
            this.custom_palette.warning = hex_to_u32(hex);
            this.log_event("Palette", format!("warning = {}", hex), cx);
        }).detach();

        // Create category sidebar nav for navigation
        let category_sidebar_nav = cx.new(|cx| {
            SidebarNav::new(
                vec![
                    GalleryCategory::Text,
                    GalleryCategory::Selection,
                    GalleryCategory::Numbers,
                    GalleryCategory::Files,
                    GalleryCategory::Progress,
                    GalleryCategory::Utility,
                    GalleryCategory::Misc,
                ],
                GalleryCategory::Text,
                cx,
            )
            .with_width(px(160.0))
        });

        // Subscribe to category nav changes
        cx.subscribe(&category_sidebar_nav, |this, _entity, event: &SidebarNavEvent<GalleryCategory>, cx| {
            let SidebarNavEvent::Change(category) = event;
            this.active_category = *category;
            this.log_event("CategoryNav", format!("Changed to {:?}", category), cx);
        })
        .detach();

        // Create widgets
        let text_input = cx.new(|cx| TextInput::new(cx).placeholder("Type something..."));
        let text_input_placeholder = cx.new(|cx| {
            TextInput::new(cx)
                .placeholder("This input has a longer placeholder")
                .with_value("Pre-filled value")
        });

        let checkbox = cx.new(Checkbox::new);
        let checkbox_labeled = cx.new(|cx| Checkbox::new(cx).label("Enable feature").with_checked(true));

        let dropdown = cx.new(|cx| {
            Dropdown::new(cx).choices(vec![
                "Option 1".to_string(),
                "Option 2".to_string(),
                "Option 3".to_string(),
                "A longer option name".to_string(),
            ])
        });

        let number_stepper = cx.new(|cx| {
            NumberStepper::new(cx)
                .with_value(50.0)
                .min(0.0)
                .max(100.0)
                .step(1.0)
                .resolution(1.0)           // Snap to integers
                .display_precision(0)      // No decimal places
                .drag_sensitivities(0.5, 2.0, 0.1)  // normal, fast, slow
        });
        let number_stepper_float = cx.new(|cx| {
            NumberStepper::new(cx)
                .with_value(2.5)
                .min(-10.0)
                .max(10.0)
                .step(0.1)
                .resolution(0.1)           // Snap to 0.1
                .display_precision(1)      // 1 decimal place
                .drag_sensitivities(0.2, 0.5, 0.05)  // normal, fast, slow
        });

        let radio_group = cx.new(|cx| {
            RadioGroup::new(cx)
                .choices(vec![
                    "Small".to_string(),
                    "Medium".to_string(),
                    "Large".to_string(),
                ])
                .with_selected_value("Medium")
        });

        let checkbox_group = cx.new(|cx| {
            CheckboxGroup::new(cx)
                .choices(vec![
                    "Red".to_string(),
                    "Green".to_string(),
                    "Blue".to_string(),
                    "Yellow".to_string(),
                ])
                .with_selected(vec!["Green".to_string(), "Blue".to_string()])
        });

        let color_swatch = cx.new(|cx| ColorSwatch::new(cx).with_value("#3b82f6"));
        let color_swatch_alpha = cx.new(|cx| {
            ColorSwatch::new(cx)
                .with_value("coral")  // Using a CSS named color!
                .with_alpha(true)
        });

        #[cfg(feature = "file-picker")]
        let file_picker = cx.new(|cx| {
            FilePicker::new(cx)
                .placeholder("Select a file...")
                .extensions(vec!["txt".to_string(), "md".to_string(), "rs".to_string()])
        });

        #[cfg(feature = "file-picker")]
        let directory_picker =
            cx.new(|cx| DirectoryPicker::new(cx).placeholder("Select a directory..."));

        // New widgets
        let password_input = cx.new(|cx| PasswordInput::new(cx).placeholder("Enter password..."));

        let tab_bar = cx.new(|cx| {
            TabBar::new(
                vec![GalleryTab::Overview, GalleryTab::Details, GalleryTab::Settings],
                GalleryTab::Overview,
                cx,
            )
        });

        let sidebar_nav_sample = cx.new(|cx| {
            SidebarNav::new(
                vec![
                    GallerySidebarSection::Dashboard,
                    GallerySidebarSection::Analytics,
                    GallerySidebarSection::Reports,
                    GallerySidebarSection::Settings,
                ],
                GallerySidebarSection::Dashboard,
                cx,
            )
            .with_width(px(140.0))
        });

        let repeatable_text_input = cx.new(|cx| {
            RepeatableTextInput::new(cx)
                .with_values(vec!["tag1".to_string(), "tag2".to_string()])
                .placeholder("Enter tag...")
                .min_entries(1)
        });

        #[cfg(feature = "file-picker")]
        let repeatable_file_picker = cx.new(|cx| {
            RepeatableFilePicker::new(cx)
                .placeholder("Select file...")
                .extensions(vec!["txt".to_string(), "md".to_string()])
                .mode(FileMode::Open)
                .min_entries(1)
        });

        #[cfg(feature = "file-picker")]
        let repeatable_directory_picker = cx.new(|cx| {
            RepeatableDirectoryPicker::new(cx)
                .placeholder("Select directory...")
                .min_entries(1)
        });

        // New widgets
        let toggle_switch = cx.new(ToggleSwitch::new);
        let toggle_switch_labeled = cx.new(|cx| {
            ToggleSwitch::new(cx)
                .with_on(true)
                .label("Enable notifications")
        });

        let slider = cx.new(|cx| {
            Slider::new(cx)
                .with_value(50.0)
                .min(0.0)
                .max(100.0)
                .step(1.0)
        });
        let slider_with_value = cx.new(|cx| {
            Slider::new(cx)
                .with_value(0.5)
                .min(0.0)
                .max(1.0)
                .step(0.01)
                .show_value(true)
                .display_precision(2)
        });

        let progress_bar = cx.new(|_cx| {
            ProgressBar::new()
                .with_value(0.65)
                .show_percentage(true)
                .label("Upload Progress")
        });
        let progress_bar_indeterminate = cx.new(|_cx| {
            ProgressBar::new()
                .indeterminate()
                .label("Loading...")
        });

        let spinner = cx.new(|_cx| Spinner::new().label("Processing..."));
        let spinner_small = cx.new(|_cx| Spinner::new().size(SpinnerSize::Small));
        let spinner_medium = cx.new(|_cx| Spinner::new().size(SpinnerSize::Medium));
        let spinner_large = cx.new(|_cx| Spinner::new().size(SpinnerSize::Large));

        // Segmented control
        let segmented_control = cx.new(|cx| {
            SegmentedControl::new(cx)
                .options(vec![
                    ("fit", "Fit"),
                    ("100", "100%"),
                    ("200", "200%"),
                    ("custom", "Custom"),
                ])
                .with_selected_index(1) // "100%"
        });

        // Collapsible sections
        let collapsible = cx.new(|cx| {
            Collapsible::new("Advanced Options", cx)
                .with_collapsed(true)
        });
        let collapsible_expanded = cx.new(|cx| {
            Collapsible::new("Details Section", cx)
                .with_collapsed(false)
        });
        let collapsible_static = cx.new(|cx| {
            Collapsible::new("Static Section Header", cx)
                .collapsible(false)
        });

        // Subscribe to collapsible events
        subscribe_widget!(cx, &collapsible, "Collapsible", CollapsibleEvent);
        subscribe_widget!(cx, &collapsible_expanded, "Collapsible (expanded)", CollapsibleEvent);

        // Info dialog: single button, easy to dismiss
        let info_dialog = cx.new(|cx| {
            ConfirmationDialog::new(
                "Operation Complete",
                "Your changes have been saved successfully. Dismiss with Enter, Escape, or click outside.",
                cx,
            )
            .style(DialogStyle::Info)
            .primary_label("OK")
        });

        // Two-button Yes/No dialog with Y/N key mappings
        let yes_no_dialog = cx.new(|cx| {
            ConfirmationDialog::new(
                "Confirm Action",
                "Do you want to proceed? Press Y for Yes, N for No.",
                cx,
            )
            .primary_label("Yes")
            .secondary_label("No")
            .map_key("y", DialogButton::Primary)
            .map_key("n", DialogButton::Secondary)
        });

        // Three-button Save dialog with Y/N key mappings
        let save_dialog = cx.new(|cx| {
            ConfirmationDialog::new(
                "Unsaved Changes",
                "Save before closing? Press Y to Save, N to Don't Save, or Escape to Cancel.",
                cx,
            )
            .primary_label("Save")
            .secondary_label("Cancel")
            .tertiary_label("Don't Save")
            .map_key("y", DialogButton::Primary)
            .map_key("n", DialogButton::Tertiary)
        });

        // Danger dialog: red button, harder to confirm
        let danger_dialog = cx.new(|cx| {
            ConfirmationDialog::new(
                "Delete Item",
                "Are you sure you want to delete this item? This action cannot be undone.",
                cx,
            )
            .style(DialogStyle::Danger)
            .primary_label("Delete")
            .secondary_label("Cancel")
        });

        // Subscribe to events
        Self::subscribe_events(
            cx,
            &text_input,
            &text_input_placeholder,
            &checkbox,
            &checkbox_labeled,
            &dropdown,
            &number_stepper,
            &number_stepper_float,
            &radio_group,
            &checkbox_group,
            &color_swatch,
            &color_swatch_alpha,
            #[cfg(feature = "file-picker")]
            &file_picker,
            #[cfg(feature = "file-picker")]
            &directory_picker,
        );

        // Subscribe to new widget events
        Self::subscribe_new_events(
            cx,
            &password_input,
            &tab_bar,
            &sidebar_nav_sample,
            &repeatable_text_input,
            #[cfg(feature = "file-picker")]
            &repeatable_file_picker,
            #[cfg(feature = "file-picker")]
            &repeatable_directory_picker,
        );

        // Subscribe to toggle/slider/progress/dialog events
        Self::subscribe_extra_events(
            cx,
            &toggle_switch,
            &toggle_switch_labeled,
            &slider,
            &slider_with_value,
            &progress_bar,
            &info_dialog,
            &yes_no_dialog,
            &save_dialog,
            &danger_dialog,
        );

        // Subscribe to segmented control events
        subscribe_widget!(cx, &segmented_control, "SegmentedControl", SegmentedControlEvent<SegmentOption>);

        Self {
            current_theme: ThemeChoice::Dark,
            theme_selector,
            show_palette_editor: false,
            custom_palette,
            palette_bg_swatch,
            palette_text_swatch,
            palette_primary_swatch,
            palette_accent_swatch,
            palette_success_swatch,
            palette_error_swatch,
            palette_warning_swatch,
            widgets_enabled: true,
            category_sidebar_nav,
            active_category: GalleryCategory::Text,
            text_input,
            text_input_placeholder,
            checkbox,
            checkbox_labeled,
            dropdown,
            number_stepper,
            number_stepper_float,
            radio_group,
            checkbox_group,
            color_swatch,
            color_swatch_alpha,
            #[cfg(feature = "file-picker")]
            file_picker,
            #[cfg(feature = "file-picker")]
            directory_picker,
            password_input,
            tab_bar,
            sidebar_nav_sample,
            repeatable_text_input,
            #[cfg(feature = "file-picker")]
            repeatable_file_picker,
            #[cfg(feature = "file-picker")]
            repeatable_directory_picker,
            toggle_switch,
            toggle_switch_labeled,
            slider,
            slider_with_value,
            progress_bar,
            progress_bar_indeterminate,
            spinner,
            spinner_small,
            spinner_medium,
            spinner_large,
            segmented_control,
            collapsible,
            collapsible_expanded,
            collapsible_static,
            show_info_dialog: false,
            info_dialog,
            info_result: None,
            show_yes_no_dialog: false,
            yes_no_dialog,
            yes_no_result: None,
            show_save_dialog: false,
            save_dialog,
            save_result: None,
            show_danger_dialog: false,
            danger_dialog,
            danger_result: None,
            primary_click_count: 0,
            secondary_click_count: 0,
            danger_click_count: 0,
            event_log: VecDeque::new(),
            log_collapsed: true,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn subscribe_events(
        cx: &mut Context<Self>,
        text_input: &Entity<TextInput>,
        text_input_placeholder: &Entity<TextInput>,
        checkbox: &Entity<Checkbox>,
        checkbox_labeled: &Entity<Checkbox>,
        dropdown: &Entity<Dropdown>,
        number_stepper: &Entity<NumberStepper>,
        number_stepper_float: &Entity<NumberStepper>,
        radio_group: &Entity<RadioGroup>,
        checkbox_group: &Entity<CheckboxGroup>,
        color_swatch: &Entity<ColorSwatch>,
        color_swatch_alpha: &Entity<ColorSwatch>,
        #[cfg(feature = "file-picker")] file_picker: &Entity<FilePicker>,
        #[cfg(feature = "file-picker")] directory_picker: &Entity<DirectoryPicker>,
    ) {
        subscribe_widget!(cx, text_input, "TextInput", TextInputEvent);
        subscribe_widget!(cx, text_input_placeholder, "TextInput (prefilled)", TextInputEvent);
        subscribe_widget!(cx, checkbox, "Checkbox", CheckboxEvent);
        subscribe_widget!(cx, checkbox_labeled, "Checkbox (labeled)", CheckboxEvent);
        subscribe_widget!(cx, dropdown, "Dropdown", DropdownEvent);
        subscribe_widget!(cx, number_stepper, "NumberStepper (int)", NumberStepperEvent);
        subscribe_widget!(cx, number_stepper_float, "NumberStepper (float)", NumberStepperEvent);
        subscribe_widget!(cx, radio_group, "RadioGroup", RadioGroupEvent<StringItem>);
        subscribe_widget!(cx, checkbox_group, "CheckboxGroup", CheckboxGroupEvent);
        subscribe_widget!(cx, color_swatch, "ColorSwatch", ColorSwatchEvent);
        subscribe_widget!(cx, color_swatch_alpha, "ColorSwatch (alpha)", ColorSwatchEvent);
        #[cfg(feature = "file-picker")]
        {
            subscribe_widget!(cx, file_picker, "FilePicker", FilePickerEvent);
            subscribe_widget!(cx, directory_picker, "DirectoryPicker", DirectoryPickerEvent);
        }
    }

    fn subscribe_new_events(
        cx: &mut Context<Self>,
        password_input: &Entity<PasswordInput>,
        tab_bar: &Entity<TabBar<GalleryTab>>,
        sidebar_nav_sample: &Entity<SidebarNav<GallerySidebarSection>>,
        repeatable_text_input: &Entity<RepeatableTextInput>,
        #[cfg(feature = "file-picker")] repeatable_file_picker: &Entity<RepeatableFilePicker>,
        #[cfg(feature = "file-picker")] repeatable_directory_picker: &Entity<RepeatableDirectoryPicker>,
    ) {
        subscribe_widget!(cx, password_input, "PasswordInput", PasswordInputEvent);
        subscribe_widget!(cx, tab_bar, "TabBar", TabBarEvent<GalleryTab>);
        subscribe_widget!(cx, sidebar_nav_sample, "SidebarNav", SidebarNavEvent<GallerySidebarSection>);
        subscribe_widget!(cx, repeatable_text_input, "RepeatableTextInput", RepeatableTextInputEvent);
        #[cfg(feature = "file-picker")]
        subscribe_widget!(cx, repeatable_file_picker, "RepeatableFilePicker", RepeatableFilePickerEvent);
        #[cfg(feature = "file-picker")]
        subscribe_widget!(cx, repeatable_directory_picker, "RepeatableDirectoryPicker", RepeatableDirectoryPickerEvent);
    }

    #[allow(clippy::too_many_arguments)]
    fn subscribe_extra_events(
        cx: &mut Context<Self>,
        toggle_switch: &Entity<ToggleSwitch>,
        toggle_switch_labeled: &Entity<ToggleSwitch>,
        slider: &Entity<Slider>,
        slider_with_value: &Entity<Slider>,
        progress_bar: &Entity<ProgressBar>,
        info_dialog: &Entity<ConfirmationDialog>,
        yes_no_dialog: &Entity<ConfirmationDialog>,
        save_dialog: &Entity<ConfirmationDialog>,
        danger_dialog: &Entity<ConfirmationDialog>,
    ) {
        subscribe_widget!(cx, toggle_switch, "ToggleSwitch", ToggleSwitchEvent);
        subscribe_widget!(cx, toggle_switch_labeled, "ToggleSwitch (labeled)", ToggleSwitchEvent);
        subscribe_widget!(cx, slider, "Slider", SliderEvent);
        subscribe_widget!(cx, slider_with_value, "Slider (with value)", SliderEvent);
        subscribe_widget!(cx, progress_bar, "ProgressBar", ProgressBarEvent);

        // Dialog subscriptions need custom handlers for result tracking
        cx.subscribe(info_dialog, |this, _entity, event: &ConfirmationDialogEvent, cx| {
            this.log_event("Dialog (Info)", format!("{:?}", event), cx);
            this.info_result = Some(dialog_result_label(event, "OK", "Secondary", "Tertiary"));
            this.show_info_dialog = false;
            cx.notify();
        })
        .detach();

        cx.subscribe(yes_no_dialog, |this, _entity, event: &ConfirmationDialogEvent, cx| {
            this.log_event("Dialog (Yes/No)", format!("{:?}", event), cx);
            this.yes_no_result = Some(dialog_result_label(event, "Yes", "No", "Tertiary"));
            this.show_yes_no_dialog = false;
            cx.notify();
        })
        .detach();

        cx.subscribe(save_dialog, |this, _entity, event: &ConfirmationDialogEvent, cx| {
            this.log_event("Dialog (Save)", format!("{:?}", event), cx);
            this.save_result = Some(dialog_result_label(event, "Save", "Cancel", "Don't Save"));
            this.show_save_dialog = false;
            cx.notify();
        })
        .detach();

        cx.subscribe(danger_dialog, |this, _entity, event: &ConfirmationDialogEvent, cx| {
            this.log_event("Dialog (Danger)", format!("{:?}", event), cx);
            this.danger_result = Some(dialog_result_label(event, "Delete", "Cancel", "Tertiary"));
            this.show_danger_dialog = false;
            cx.notify();
        })
        .detach();
    }

    fn log_event(&mut self, widget: &str, event: String, cx: &mut Context<Self>) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| {
                let secs = d.as_secs() % 86400; // Time of day
                let hours = (secs / 3600) % 24;
                let minutes = (secs % 3600) / 60;
                let seconds = secs % 60;
                let millis = d.subsec_millis();
                format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, millis)
            })
            .unwrap_or_else(|_| "??:??:??".to_string());

        self.event_log.push_front(EventLogEntry {
            timestamp,
            widget: widget.to_string(),
            event,
        });

        // Trim old events
        while self.event_log.len() > MAX_EVENT_LOG {
            self.event_log.pop_back();
        }

        cx.notify();
    }

    fn handle_theme_change(&mut self, choice: ThemeChoice, cx: &mut Context<Self>) {
        self.current_theme = choice;

        match choice {
            ThemeChoice::Light => {
                cx.set_global(Theme::light());
                self.log_event("Theme", "Switched to Light".to_string(), cx);
            }
            ThemeChoice::Dark => {
                cx.set_global(Theme::dark());
                self.log_event("Theme", "Switched to Dark".to_string(), cx);
            }
            ThemeChoice::Custom => {
                self.show_palette_editor = true;
                self.log_event("Theme", "Opening palette editor".to_string(), cx);
            }
        }
        cx.notify();
    }

    fn apply_custom_palette(&mut self, cx: &mut Context<Self>) {
        let theme = Theme::from_palette(self.custom_palette);
        cx.set_global(theme);
        self.show_palette_editor = false;
        self.log_event("Theme", "Applied custom palette".to_string(), cx);
        cx.notify();
    }

    fn close_palette_editor(&mut self, cx: &mut Context<Self>) {
        self.show_palette_editor = false;
        // Revert to previous non-custom theme if canceling
        if self.current_theme == ThemeChoice::Custom {
            self.current_theme = ThemeChoice::Dark;
            self.theme_selector.update(cx, |s, scx| {
                s.set_selected_index(1, scx); // Dark is index 1
            });
            cx.set_global(Theme::dark());
        }
        self.log_event("Theme", "Canceled palette editor".to_string(), cx);
        cx.notify();
    }

    fn reset_palette_to(&mut self, palette: Palette, cx: &mut Context<Self>) {
        self.custom_palette = palette;
        self.palette_bg_swatch.update(cx, |s, scx| s.set_value(&u32_to_hex(palette.bg), scx));
        self.palette_text_swatch.update(cx, |s, scx| s.set_value(&u32_to_hex(palette.text), scx));
        self.palette_primary_swatch.update(cx, |s, scx| s.set_value(&u32_to_hex(palette.primary), scx));
        self.palette_accent_swatch.update(cx, |s, scx| s.set_value(&u32_to_hex(palette.accent), scx));
        self.palette_success_swatch.update(cx, |s, scx| s.set_value(&u32_to_hex(palette.success), scx));
        self.palette_error_swatch.update(cx, |s, scx| s.set_value(&u32_to_hex(palette.error), scx));
        self.palette_warning_swatch.update(cx, |s, scx| s.set_value(&u32_to_hex(palette.warning), scx));
        cx.notify();
    }

    fn render_palette_editor(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = get_theme(cx);

        // Semi-transparent backdrop (rendered without deferred since other dialogs use it)
        div()
            .id("palette-editor-backdrop")
            .absolute()
            .inset_0()
            .bg(rgba(0x00000080)) // 50% black
            .flex()
            .items_center()
            .justify_center()
            .on_mouse_down(MouseButton::Left, cx.listener(|this, _event, _window, cx| {
                this.close_palette_editor(cx);
            }))
            .child(
                // Modal dialog - uses a separate click handler that does nothing
                // to prevent clicks from bubbling to backdrop
                div()
                    .id("palette-editor-modal")
                    .w(px(520.0))
                    .bg(rgb(theme.bg_secondary))
                    .border_1()
                    .border_color(rgb(theme.border_default))
                    .rounded_lg()
                    .shadow_lg()
                    .occlude()
                        // Title bar
                        .child(
                            div()
                                .px_4()
                                .py_3()
                                .bg(rgb(theme.bg_section_header))
                                .border_b_1()
                                .border_color(rgb(theme.border_default))
                                .text_lg()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgb(theme.text_primary))
                                .child("Customize Theme"),
                        )
                        // Content area
                        .child(
                            div()
                                .p_4()
                                .flex()
                                .flex_col()
                                .gap_3()
                                // Background color
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_3()
                                        .child(
                                            div()
                                                .w(px(80.0))
                                                .flex_shrink_0()
                                                .text_sm()
                                                .text_color(rgb(theme.text_label))
                                                .child("Background"),
                                        )
                                        .child(div().flex_1().child(self.palette_bg_swatch.clone())),
                                )
                                // Text color
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_3()
                                        .child(
                                            div()
                                                .w(px(80.0))
                                                .flex_shrink_0()
                                                .text_sm()
                                                .text_color(rgb(theme.text_label))
                                                .child("Text"),
                                        )
                                        .child(div().flex_1().child(self.palette_text_swatch.clone())),
                                )
                                // Primary color
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_3()
                                        .child(
                                            div()
                                                .w(px(80.0))
                                                .flex_shrink_0()
                                                .text_sm()
                                                .text_color(rgb(theme.text_label))
                                                .child("Primary"),
                                        )
                                        .child(div().flex_1().child(self.palette_primary_swatch.clone())),
                                )
                                // Accent color
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_3()
                                        .child(
                                            div()
                                                .w(px(80.0))
                                                .flex_shrink_0()
                                                .text_sm()
                                                .text_color(rgb(theme.text_label))
                                                .child("Accent"),
                                        )
                                        .child(div().flex_1().child(self.palette_accent_swatch.clone())),
                                )
                                // Success color
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_3()
                                        .child(
                                            div()
                                                .w(px(80.0))
                                                .flex_shrink_0()
                                                .text_sm()
                                                .text_color(rgb(theme.text_label))
                                                .child("Success"),
                                        )
                                        .child(div().flex_1().child(self.palette_success_swatch.clone())),
                                )
                                // Error color
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_3()
                                        .child(
                                            div()
                                                .w(px(80.0))
                                                .flex_shrink_0()
                                                .text_sm()
                                                .text_color(rgb(theme.text_label))
                                                .child("Error"),
                                        )
                                        .child(div().flex_1().child(self.palette_error_swatch.clone())),
                                )
                                // Warning color
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap_3()
                                        .child(
                                            div()
                                                .w(px(80.0))
                                                .flex_shrink_0()
                                                .text_sm()
                                                .text_color(rgb(theme.text_label))
                                                .child("Warning"),
                                        )
                                        .child(div().flex_1().child(self.palette_warning_swatch.clone())),
                                ),
                        )
                        // Button row
                        .child(
                            div()
                                .px_4()
                                .py_3()
                                .bg(rgb(theme.bg_section_header))
                                .border_t_1()
                                .border_color(rgb(theme.border_default))
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_2()
                                // Reset buttons on the left
                                .child(
                                    secondary_button("palette-reset-dark", "Reset Dark", cx)
                                        .on_click(cx.listener(|this, _event, _window, cx| {
                                            this.reset_palette_to(Palette::dark(), cx);
                                            this.log_event("Palette", "Reset to dark".to_string(), cx);
                                        })),
                                )
                                .child(
                                    secondary_button("palette-reset-light", "Reset Light", cx)
                                        .on_click(cx.listener(|this, _event, _window, cx| {
                                            this.reset_palette_to(Palette::light(), cx);
                                            this.log_event("Palette", "Reset to light".to_string(), cx);
                                        })),
                                )
                                // Spacer
                                .child(div().flex_1())
                                // Cancel and Apply on the right
                                .child(
                                    secondary_button("palette-cancel", "Cancel", cx)
                                        .on_click(cx.listener(|this, _event, _window, cx| {
                                            this.close_palette_editor(cx);
                                        })),
                                )
                                .child(
                                    primary_button("palette-apply", "Apply", true, cx)
                                        .on_click(cx.listener(|this, _event, _window, cx| {
                                            this.apply_custom_palette(cx);
                                        })),
                                ),
                        ),
                )
    }

    fn toggle_widgets_enabled(&mut self, cx: &mut Context<Self>) {
        self.widgets_enabled = !self.widgets_enabled;
        let enabled = self.widgets_enabled;

        // Update all widgets
        set_enabled_all!(
            cx, enabled,
            self.text_input,
            self.text_input_placeholder,
            self.checkbox,
            self.checkbox_labeled,
            self.dropdown,
            self.number_stepper,
            self.number_stepper_float,
            self.radio_group,
            self.checkbox_group,
            self.color_swatch,
            self.color_swatch_alpha,
            self.password_input,
            self.tab_bar,
            self.sidebar_nav_sample,
            self.repeatable_text_input,
            self.toggle_switch,
            self.toggle_switch_labeled,
            self.slider,
            self.slider_with_value,
            self.segmented_control,
            self.category_sidebar_nav,
        );

        // Feature-gated widgets
        #[cfg(feature = "file-picker")]
        set_enabled_all!(
            cx, enabled,
            self.file_picker,
            self.directory_picker,
            self.repeatable_file_picker,
            self.repeatable_directory_picker,
        );

        self.log_event(
            "Gallery",
            format!("Widgets {}", if enabled { "enabled" } else { "disabled" }),
            cx,
        );
        cx.notify();
    }

    fn render_header(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = get_theme(cx);
        let enabled_button_text = if self.widgets_enabled {
            "Disable Widgets"
        } else {
            "Enable Widgets"
        };
        let enabled_button_bg = if self.widgets_enabled {
            theme.warning
        } else {
            theme.success
        };

        div()
            .flex()
            .flex_row()
            .justify_between()
            .items_center()
            .w_full()
            .px_4()
            .py_3()
            .bg(rgb(theme.bg_section_header))
            .border_b_1()
            .border_color(rgb(theme.border_default))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(theme.text_primary))
                            .child("Widget Gallery"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(theme.text_muted))
                            .child("A showcase of all ccf-gpui-widgets"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_3()
                    .items_center()
                    .child(
                        div()
                            .id("enable-toggle")
                            .px_3()
                            .py_1()
                            .bg(rgb(enabled_button_bg))
                            .hover(|s| s.opacity(0.8))
                            .text_color(rgb(theme.text_primary))
                            .rounded_md()
                            .cursor_pointer()
                            .child(enabled_button_text)
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.toggle_widgets_enabled(cx);
                            })),
                    )
                    .child(self.theme_selector.clone()),
            )
    }

    fn render_widget_row(
        label: &'static str,
        description: &'static str,
        widget: impl IntoElement,
        value_display: Option<String>,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let theme = get_theme(cx);

        div()
            .flex()
            .flex_row()
            .items_start()
            .gap_4()
            .py_2()
            .child(
                div()
                    .w(px(200.0))
                    .flex_shrink_0()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(theme.text_primary))
                            .child(label),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(theme.text_muted))
                            .child(description),
                    ),
            )
            .child(div().flex_1().child(widget))
            .when_some(value_display, |d, value| {
                d.child(
                    div()
                        .w(px(200.0))
                        .flex_shrink_0()
                        .text_xs()
                        .font_family("monospace")
                        .text_color(rgb(theme.text_muted))
                        .child(format!("Value: {}", value)),
                )
            })
    }

    fn render_text_input_section(&self, cx: &Context<Self>) -> impl IntoElement {
        let text_value = self.text_input.read(cx).content().to_string();
        let prefilled_value = self.text_input_placeholder.read(cx).content().to_string();

        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(Self::render_widget_row(
                "Basic TextInput",
                "Simple text input with placeholder",
                self.text_input.clone(),
                Some(format!("\"{}\"", text_value)),
                cx,
            ))
            .child(Self::render_widget_row(
                "Pre-filled TextInput",
                "With initial value",
                self.text_input_placeholder.clone(),
                Some(format!("\"{}\"", prefilled_value)),
                cx,
            ))
    }

    fn render_checkbox_section(&self, cx: &Context<Self>) -> impl IntoElement {
        let checkbox_value = self.checkbox.read(cx).is_checked();
        let labeled_value = self.checkbox_labeled.read(cx).is_checked();

        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(Self::render_widget_row(
                "Basic Checkbox",
                "No label",
                self.checkbox.clone(),
                Some(checkbox_value.to_string()),
                cx,
            ))
            .child(Self::render_widget_row(
                "Labeled Checkbox",
                "With label text",
                self.checkbox_labeled.clone(),
                Some(labeled_value.to_string()),
                cx,
            ))
    }

    fn render_dropdown_section(&self, cx: &Context<Self>) -> impl IntoElement {
        let dropdown_value = self.dropdown.read(cx).selected().to_string();

        div().child(Self::render_widget_row(
            "Dropdown",
            "Select from options",
            self.dropdown.clone(),
            Some(format!("\"{}\"", dropdown_value)),
            cx,
        ))
    }

    fn render_number_section(&self, cx: &Context<Self>) -> impl IntoElement {
        let int_value = self.number_stepper.read(cx).value();
        let float_value = self.number_stepper_float.read(cx).value();

        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(Self::render_widget_row(
                "Integer Stepper",
                "Range: 0-100, step: 1",
                div().w(px(130.0)).child(self.number_stepper.clone()),
                Some(int_value.to_string()),
                cx,
            ))
            .child(Self::render_widget_row(
                "Float Stepper",
                "Range: -10 to 10, step: 0.1",
                div().w(px(130.0)).child(self.number_stepper_float.clone()),
                Some(format!("{:.2}", float_value)),
                cx,
            ))
    }

    fn render_radio_section(&self, cx: &Context<Self>) -> impl IntoElement {
        let radio_value = self.radio_group.read(cx).selected().to_string();

        div().child(Self::render_widget_row(
            "Radio Group",
            "Single selection",
            self.radio_group.clone(),
            Some(format!("\"{}\"", radio_value)),
            cx,
        ))
    }

    fn render_checkbox_group_section(&self, cx: &Context<Self>) -> impl IntoElement {
        let selected = self.checkbox_group.read(cx).get_selected();
        let display = if selected.is_empty() {
            "[]".to_string()
        } else {
            format!("[{}]", selected.join(", "))
        };

        div().child(Self::render_widget_row(
            "Checkbox Group",
            "Multi selection",
            self.checkbox_group.clone(),
            Some(display),
            cx,
        ))
    }

    fn render_color_section(&self, cx: &Context<Self>) -> impl IntoElement {
        let color_value = self.color_swatch.read(cx).value().to_string();
        let alpha_value = self.color_swatch_alpha.read(cx).value().to_string();

        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(Self::render_widget_row(
                "Color Swatch",
                "Click swatch to open picker, type hex or color names (red, coral, etc.)",
                self.color_swatch.clone(),
                Some(color_value),
                cx,
            ))
            .child(Self::render_widget_row(
                "Color (with alpha)",
                "Alpha channel enabled (#RRGGBBAA format)",
                self.color_swatch_alpha.clone(),
                Some(alpha_value),
                cx,
            ))
    }

    fn render_tooltip_section(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = get_theme(cx);

        div().child(Self::render_widget_row(
            "Tooltip",
            "Hover over the box",
            div()
                .id("tooltip-demo")
                .px_4()
                .py_2()
                .bg(rgb(theme.bg_input))
                .border_1()
                .border_color(rgb(theme.border_input))
                .rounded_md()
                .text_color(rgb(theme.text_primary))
                .child("Hover me!")
                .tooltip(|_window, cx| cx.new(|_cx| Tooltip::new("This is a tooltip!")).into()),
            None,
            cx,
        ))
    }

    fn render_button_section(&self, cx: &Context<Self>) -> Div {
        let theme = get_theme(cx);
        let primary_count = self.primary_click_count;
        let secondary_count = self.secondary_click_count;
        let danger_count = self.danger_click_count;

        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(Self::render_widget_row(
                "Primary Button",
                "Main action button",
                div()
                    .w(px(130.0))
                    .child(
                        primary_button("primary_demo", "Click Me", true, cx)
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.primary_click_count += 1;
                                this.log_event(
                                    "Button",
                                    format!("Primary clicked (count: {})", this.primary_click_count),
                                    cx,
                                );
                            })),
                    ),
                Some(format!("clicks: {}", primary_count)),
                cx,
            ))
            .child(Self::render_widget_row(
                "Primary (disabled)",
                "Disabled state",
                div()
                    .w(px(130.0))
                    .child(primary_button("primary_disabled", "Disabled", false, cx)),
                None,
                cx,
            ))
            .child(Self::render_widget_row(
                "Secondary Button",
                "Alternative action",
                div()
                    .w(px(130.0))
                    .child(
                        secondary_button("secondary_demo", "Cancel", cx).on_click(cx.listener(
                            |this, _event, _window, cx| {
                                this.secondary_click_count += 1;
                                this.log_event(
                                    "Button",
                                    format!("Secondary clicked (count: {})", this.secondary_click_count),
                                    cx,
                                );
                            },
                        )),
                    ),
                Some(format!("clicks: {}", secondary_count)),
                cx,
            ))
            .child(Self::render_widget_row(
                "Danger Button",
                "Destructive action",
                div()
                    .w(px(130.0))
                    .child(
                        danger_button("danger_demo", "Delete", true, cx).on_click(cx.listener(
                            |this, _event, _window, cx| {
                                this.danger_click_count += 1;
                                this.log_event(
                                    "Button",
                                    format!("Danger clicked (count: {})", this.danger_click_count),
                                    cx,
                                );
                            },
                        )),
                    ),
                Some(format!("clicks: {}", danger_count)),
                cx,
            ))
            .child(Self::render_widget_row(
                "Danger (disabled)",
                "Disabled state",
                div()
                    .w(px(130.0))
                    .child(danger_button("danger_disabled", "Delete", false, cx)),
                None,
                cx,
            ))
            .p_4()
            .bg(rgb(theme.bg_secondary))
    }

    fn render_password_section(&self, cx: &Context<Self>) -> impl IntoElement {
        #[cfg(feature = "secure-password")]
        let display = {
            use secrecy::ExposeSecret;
            let secret = self.password_input.read(cx).value(cx);
            let value = secret.expose_secret();
            if value.is_empty() {
                "(empty)".to_string()
            } else {
                // Show actual value for demo purposes (this gallery is not collecting real passwords)
                format!("\"{}\"", value)
            }
        };
        #[cfg(not(feature = "secure-password"))]
        let display = {
            let value = self.password_input.read(cx).value(cx);
            if value.is_empty() {
                "(empty)".to_string()
            } else {
                format!("\"{}\"", value)
            }
        };

        div().child(Self::render_widget_row(
            "Password Input",
            "Masked input with visibility toggle",
            self.password_input.clone(),
            Some(display),
            cx,
        ))
    }

    fn render_tab_bar_section(&self, cx: &Context<Self>) -> impl IntoElement {
        let active = format!("{:?}", self.tab_bar.read(cx).selected());

        div().child(Self::render_widget_row(
            "Tab Bar",
            "Click tabs, right-click for context menu",
            self.tab_bar.clone(),
            Some(active),
            cx,
        ))
    }

    fn render_sidebar_nav_section(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = get_theme(cx);
        let active = format!("{:?}", self.sidebar_nav_sample.read(cx).selected());

        div().child(Self::render_widget_row(
            "Sidebar Nav",
            "Vertical navigation, Up/Down keys",
            div()
                .h(px(150.0)) // Fixed height for demo
                .border_1()
                .border_color(rgb(theme.border_default))
                .rounded_md()
                .overflow_hidden()
                .child(self.sidebar_nav_sample.clone()),
            Some(active),
            cx,
        ))
    }

    fn render_repeatable_text_section(&self, cx: &Context<Self>) -> impl IntoElement {
        let values = self.repeatable_text_input.read(cx).values(cx);
        let display = if values.is_empty() {
            "[]".to_string()
        } else {
            format!("[{}]", values.join(", "))
        };

        div().child(Self::render_widget_row(
            "Repeatable Text Input",
            "Add/remove text entries (min: 1)",
            self.repeatable_text_input.clone(),
            Some(display),
            cx,
        ))
    }

    /// Renders a repeatable picker section with value display and widget
    #[cfg(feature = "file-picker")]
    fn render_repeatable_picker_section(
        values: Vec<String>,
        empty_label: &str,
        title: &str,
        description: &str,
        widget: impl IntoElement,
        cx: &Context<Self>,
    ) -> Div {
        let theme = get_theme(cx);

        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(theme.text_muted))
                            .child("Value:"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .font_family("monospace")
                            .text_color(rgb(theme.text_muted))
                            .overflow_x_hidden()
                            .whitespace_nowrap()
                            .when(values.is_empty(), |d| d.child(empty_label.to_string()))
                            .when(!values.is_empty(), |d| {
                                d.flex()
                                    .flex_col()
                                    .children(values.iter().map(|v| {
                                        div()
                                            .overflow_x_hidden()
                                            .text_ellipsis()
                                            .child(v.clone())
                                    }))
                            }),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_start()
                    .gap_4()
                    .py_2()
                    .child(
                        div()
                            .w(px(200.0))
                            .flex_shrink_0()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(theme.text_primary))
                                    .child(title.to_string()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(theme.text_muted))
                                    .child(description.to_string()),
                            ),
                    )
                    .child(div().flex_1().child(widget)),
            )
    }

    #[cfg(feature = "file-picker")]
    fn render_repeatable_file_section(&self, cx: &Context<Self>) -> Div {
        Self::render_repeatable_picker_section(
            self.repeatable_file_picker.read(cx).values(cx),
            "(no files selected)",
            "Repeatable File Picker",
            "Add/remove file selections (min: 1)",
            self.repeatable_file_picker.clone(),
            cx,
        )
    }

    #[cfg(feature = "file-picker")]
    fn render_repeatable_dir_section(&self, cx: &Context<Self>) -> Div {
        Self::render_repeatable_picker_section(
            self.repeatable_directory_picker.read(cx).values(cx),
            "(no directories selected)",
            "Repeatable Directory Picker",
            "Add/remove directory selections (min: 1)",
            self.repeatable_directory_picker.clone(),
            cx,
        )
    }

    #[cfg(feature = "file-picker")]
    fn render_file_section(&self, cx: &Context<Self>) -> impl IntoElement {
        let file_value = self.file_picker.read(cx).value().to_string();
        let dir_value = self.directory_picker.read(cx).value().to_string();

        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(Self::render_widget_row(
                "File Picker",
                "Select .txt, .md, .rs files",
                self.file_picker.clone(),
                Some(if file_value.is_empty() {
                    "(none)".to_string()
                } else {
                    file_value
                }),
                cx,
            ))
            .child(Self::render_widget_row(
                "Directory Picker",
                "Select a directory",
                self.directory_picker.clone(),
                Some(if dir_value.is_empty() {
                    "(none)".to_string()
                } else {
                    dir_value
                }),
                cx,
            ))
    }

    fn render_toggle_section(&self, cx: &Context<Self>) -> impl IntoElement {
        let toggle_value = self.toggle_switch.read(cx).is_on();
        let labeled_value = self.toggle_switch_labeled.read(cx).is_on();

        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(Self::render_widget_row(
                "Basic Toggle",
                "Simple on/off switch",
                self.toggle_switch.clone(),
                Some(toggle_value.to_string()),
                cx,
            ))
            .child(Self::render_widget_row(
                "Labeled Toggle",
                "With label text",
                self.toggle_switch_labeled.clone(),
                Some(labeled_value.to_string()),
                cx,
            ))
    }

    fn render_slider_section(&self, cx: &Context<Self>) -> impl IntoElement {
        let slider_value = self.slider.read(cx).value();
        let slider_with_value_val = self.slider_with_value.read(cx).value();

        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(Self::render_widget_row(
                "Basic Slider",
                "Range: 0-100, step: 1",
                div().w(px(200.0)).child(self.slider.clone()),
                Some(format!("{:.0}", slider_value)),
                cx,
            ))
            .child(Self::render_widget_row(
                "Slider with Value",
                "Range: 0-1, step: 0.01, shows value",
                div().w(px(250.0)).child(self.slider_with_value.clone()),
                Some(format!("{:.2}", slider_with_value_val)),
                cx,
            ))
    }

    fn render_progress_section(&self, cx: &Context<Self>) -> impl IntoElement {
        let progress_value = self.progress_bar.read(cx).percentage();

        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(Self::render_widget_row(
                "Determinate Progress",
                "Shows 65% complete with label",
                div().w(px(300.0)).child(self.progress_bar.clone()),
                progress_value.map(|p| format!("{:.0}%", p * 100.0)),
                cx,
            ))
            .child(Self::render_widget_row(
                "Indeterminate Progress",
                "Animated loading bar",
                div().w(px(300.0)).child(self.progress_bar_indeterminate.clone()),
                Some("indeterminate".to_string()),
                cx,
            ))
    }

    fn render_spinner_section(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = get_theme(cx);

        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(Self::render_widget_row(
                "Spinner with Label",
                "Medium size (default)",
                self.spinner.clone(),
                None,
                cx,
            ))
            .child(Self::render_widget_row(
                "Spinner Sizes",
                "Small, Medium, Large",
                div()
                    .flex()
                    .flex_row()
                    .gap_4()
                    .items_center()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap_1()
                            .child(self.spinner_small.clone())
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(theme.text_muted))
                                    .child("Small"),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap_1()
                            .child(self.spinner_medium.clone())
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(theme.text_muted))
                                    .child("Medium"),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap_1()
                            .child(self.spinner_large.clone())
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(theme.text_muted))
                                    .child("Large"),
                            ),
                    ),
                None,
                cx,
            ))
    }

    fn render_dialog_section(&self, cx: &Context<Self>) -> Div {
        let theme = get_theme(cx);
        let info_result = self.info_result;
        let yes_no_result = self.yes_no_result;
        let save_result = self.save_result;
        let danger_result = self.danger_result;

        div()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .bg(rgb(theme.bg_secondary))
            // Info Dialog (single button)
            .child(Self::render_widget_row(
                "Info Dialog",
                "Enter/Escape/click-outside dismisses",
                div()
                    .flex()
                    .flex_row()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .max_w(px(140.0))
                            .child(
                                primary_button("show_info_dialog_btn", "Show Info...", true, cx)
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        this.info_result = None;
                                        this.show_info_dialog = true;
                                        this.log_event("Dialog (Info)", "Opened".to_string(), cx);
                                    }))
                            )
                    )
                    .when_some(info_result, |d, result| {
                        d.child(
                            div()
                                .text_sm()
                                .text_color(rgb(theme.text_muted))
                                .child(format!("Result: {}", result))
                        )
                    }),
                None,
                cx,
            ))
            // Yes/No Dialog (two buttons with Y/N keys)
            .child(Self::render_widget_row(
                "Yes/No Dialog",
                "Y/N keys, Enter=Yes, Escape=No",
                div()
                    .flex()
                    .flex_row()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .max_w(px(140.0))
                            .child(
                                primary_button("show_yes_no_dialog_btn", "Confirm...", true, cx)
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        this.yes_no_result = None;
                                        this.show_yes_no_dialog = true;
                                        this.log_event("Dialog (Yes/No)", "Opened".to_string(), cx);
                                    }))
                            )
                    )
                    .when_some(yes_no_result, |d, result| {
                        d.child(
                            div()
                                .text_sm()
                                .text_color(rgb(theme.text_muted))
                                .child(format!("Result: {}", result))
                        )
                    }),
                None,
                cx,
            ))
            // Save Dialog (three buttons with Y/N keys)
            .child(Self::render_widget_row(
                "Save Dialog",
                "Y=Save, N=Don't Save, Escape=Cancel",
                div()
                    .flex()
                    .flex_row()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .max_w(px(140.0))
                            .child(
                                primary_button("show_save_dialog_btn", "Save Changes...", true, cx)
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        this.save_result = None;
                                        this.show_save_dialog = true;
                                        this.log_event("Dialog (Save)", "Opened".to_string(), cx);
                                    }))
                            )
                    )
                    .when_some(save_result, |d, result| {
                        d.child(
                            div()
                                .text_sm()
                                .text_color(rgb(theme.text_muted))
                                .child(format!("Result: {}", result))
                        )
                    }),
                None,
                cx,
            ))
            // Danger Dialog (red button, Enter doesn't confirm)
            .child(Self::render_widget_row(
                "Danger Dialog",
                "Enter disabled, must click or Escape",
                div()
                    .flex()
                    .flex_row()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .max_w(px(140.0))
                            .child(
                                danger_button("show_danger_dialog_btn", "Delete Item...", true, cx)
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        this.danger_result = None;
                                        this.show_danger_dialog = true;
                                        this.log_event("Dialog (Danger)", "Opened".to_string(), cx);
                                    }))
                            )
                    )
                    .when_some(danger_result, |d, result| {
                        d.child(
                            div()
                                .text_sm()
                                .text_color(rgb(theme.text_muted))
                                .child(format!("Result: {}", result))
                        )
                    }),
                None,
                cx,
            ))
    }

    fn render_segmented_section(&self, cx: &Context<Self>) -> impl IntoElement {
        let selected = self.segmented_control.read(cx).selected().label.clone();

        div().child(Self::render_widget_row(
            "Segmented Control",
            "Use arrow keys or click to select",
            self.segmented_control.clone(),
            Some(format!("\"{}\"", selected)),
            cx,
        ))
    }

    fn render_collapsible_section(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = get_theme(cx);
        let collapsed_state = self.collapsible.read(cx).is_collapsed();
        let expanded_state = self.collapsible_expanded.read(cx).is_collapsed();

        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(Self::render_widget_row(
                "Collapsible (collapsed)",
                "Click or Enter/Space to toggle, ↑/↓ keys",
                div()
                    .w(px(300.0))
                    .flex()
                    .flex_col()
                    .child(self.collapsible.clone())
                    .when(!collapsed_state, |d| {
                        d.child(
                            div()
                                .p_3()
                                .bg(rgb(theme.bg_input))
                                .border_1()
                                .border_color(rgb(theme.border_default))
                                .border_t_0()
                                .rounded_b_md()
                                .text_sm()
                                .text_color(rgb(theme.text_primary))
                                .child("This is the hidden content that appears when expanded. You can put any content here."),
                        )
                    }),
                Some(if collapsed_state { "collapsed" } else { "expanded" }.to_string()),
                cx,
            ))
            .child(Self::render_widget_row(
                "Collapsible (expanded)",
                "Starts expanded",
                div()
                    .w(px(300.0))
                    .flex()
                    .flex_col()
                    .child(self.collapsible_expanded.clone())
                    .when(!expanded_state, |d| {
                        d.child(
                            div()
                                .p_3()
                                .bg(rgb(theme.bg_input))
                                .border_1()
                                .border_color(rgb(theme.border_default))
                                .border_t_0()
                                .rounded_b_md()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(theme.text_primary))
                                        .child("Details can include multiple elements:"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(theme.text_muted))
                                        .child("• First item"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(theme.text_muted))
                                        .child("• Second item"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(theme.text_muted))
                                        .child("• Third item"),
                                ),
                        )
                    }),
                Some(if expanded_state { "collapsed" } else { "expanded" }.to_string()),
                cx,
            ))
            .child(Self::render_widget_row(
                "Static Section Header",
                "Non-collapsible mode via .collapsible(false)",
                div()
                    .w(px(300.0))
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .rounded_md()
                    .border_1()
                    .border_color(rgb(theme.border_default))
                    .child(self.collapsible_static.clone())
                    .child(
                        div()
                            .p_3()
                            .bg(rgb(theme.bg_input))
                            .text_sm()
                            .text_color(rgb(theme.text_primary))
                            .child("Content is always visible. No chevron, not focusable."),
                    ),
                Some("static".to_string()),
                cx,
            ))
    }

    fn render_scrollable_section(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = get_theme(cx);

        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(Self::render_widget_row(
                "Vertical Scrollable",
                "Visible scrollbar with auto-fade",
                div()
                    .w(px(300.0))
                    .h(px(120.0))
                    .border_1()
                    .border_color(rgb(theme.border_default))
                    .rounded_md()
                    .overflow_hidden()
                    .child(
                        scrollable_vertical(
                            div()
                                .p_2()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .children((1..=20).map(|i| {
                                    div()
                                        .text_sm()
                                        .text_color(rgb(theme.text_primary))
                                        .child(format!("Item {} - Scroll to see more", i))
                                })),
                        )
                        .always_show_scrollbars(),
                    ),
                None,
                cx,
            ))
            .child(Self::render_widget_row(
                "Horizontal Scrollable",
                "Scrollbar at bottom",
                div()
                    .w(px(300.0))
                    .h(px(60.0))
                    .border_1()
                    .border_color(rgb(theme.border_default))
                    .rounded_md()
                    .overflow_hidden()
                    .child(
                        scrollable_horizontal(
                            // Note: Horizontal scroll requires explicit width on content
                            // that exceeds the container width
                            div()
                                .w(px(600.0))
                                .p_2()
                                .flex()
                                .flex_row()
                                .gap_2()
                                .children((1..=15).map(|i| {
                                    div()
                                        .flex_shrink_0()
                                        .px_3()
                                        .py_1()
                                        .bg(rgb(theme.bg_input))
                                        .rounded_md()
                                        .text_sm()
                                        .text_color(rgb(theme.text_primary))
                                        .whitespace_nowrap()
                                        .child(format!("Tag {}", i))
                                })),
                        )
                        .always_show_scrollbars(),
                    ),
                None,
                cx,
            ))
    }

    fn render_event_log(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = get_theme(cx);

        div()
            .w_full()
            .flex()
            .flex_col()
            .border_t_1()
            .border_color(rgb(theme.border_default))
            .child(
                // Header
                div()
                    .id("log-header")
                    .flex()
                    .flex_row()
                    .justify_between()
                    .items_center()
                    .px_4()
                    .py_2()
                    .bg(rgb(theme.bg_section_header))
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _event, _window, cx| {
                        this.log_collapsed = !this.log_collapsed;
                        cx.notify();
                    }))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(theme.text_muted))
                                    .child(if self.log_collapsed { "▶" } else { "▼" }),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(theme.text_primary))
                                    .child("Event Log"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(theme.text_muted))
                                    .child(format!("({} events)", self.event_log.len())),
                            ),
                    ),
            )
            .when(!self.log_collapsed, |d| {
                d.child(
                    div()
                        .id("event-log-content")
                        .h(px(150.0))
                        .overflow_y_scroll()
                        .bg(rgb(theme.bg_primary))
                        .p_2()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .children(self.event_log.iter().map(|entry| {
                                    div()
                                        .flex()
                                        .flex_row()
                                        .gap_2()
                                        .text_xs()
                                        .font_family("monospace")
                                        .child(
                                            div()
                                                .text_color(rgb(theme.text_muted))
                                                .child(entry.timestamp.clone()),
                                        )
                                        .child(
                                            div()
                                                .text_color(rgb(theme.primary))
                                                .min_w(px(150.0))
                                                .child(entry.widget.clone()),
                                        )
                                        .child(
                                            div()
                                                .text_color(rgb(theme.text_primary))
                                                .child(entry.event.clone()),
                                        )
                                })),
                        ),
                )
            })
    }

    // Category navigation methods

    fn render_active_category(&self, cx: &Context<Self>) -> impl IntoElement {
        match self.active_category {
            GalleryCategory::Text => self.render_text_category(cx).into_any_element(),
            GalleryCategory::Selection => self.render_selection_category(cx).into_any_element(),
            GalleryCategory::Numbers => self.render_numbers_category(cx).into_any_element(),
            GalleryCategory::Files => self.render_files_category(cx).into_any_element(),
            GalleryCategory::Progress => self.render_progress_category(cx).into_any_element(),
            GalleryCategory::Utility => self.render_utility_category(cx).into_any_element(),
            GalleryCategory::Misc => self.render_misc_category(cx).into_any_element(),
        }
    }

    fn render_category_section(
        title: &'static str,
        content: impl IntoElement,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let theme = get_theme(cx);

        div()
            .w_full()
            .mb_4()
            .border_1()
            .border_color(rgb(theme.border_default))
            .rounded_md()
            .overflow_hidden()
            .child(
                div()
                    .px_4()
                    .py_2()
                    .bg(rgb(theme.bg_section_header))
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(rgb(theme.text_primary))
                    .child(title),
            )
            .child(
                div()
                    .p_4()
                    .bg(rgb(theme.bg_secondary))
                    .child(content),
            )
    }

    fn render_text_category(&self, cx: &Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(Self::render_category_section(
                "Text Input",
                self.render_text_input_section(cx),
                cx,
            ))
            .child(Self::render_category_section(
                "Password Input",
                self.render_password_section(cx),
                cx,
            ))
            .child(Self::render_category_section(
                "Repeatable Text Input",
                self.render_repeatable_text_section(cx),
                cx,
            ))
    }

    fn render_selection_category(&self, cx: &Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(Self::render_category_section(
                "Checkbox",
                self.render_checkbox_section(cx),
                cx,
            ))
            .child(Self::render_category_section(
                "Toggle Switch",
                self.render_toggle_section(cx),
                cx,
            ))
            .child(Self::render_category_section(
                "Dropdown",
                self.render_dropdown_section(cx),
                cx,
            ))
            .child(Self::render_category_section(
                "Radio Group",
                self.render_radio_section(cx),
                cx,
            ))
            .child(Self::render_category_section(
                "Checkbox Group",
                self.render_checkbox_group_section(cx),
                cx,
            ))
            .child(Self::render_category_section(
                "Segmented Control",
                self.render_segmented_section(cx),
                cx,
            ))
            .child(Self::render_category_section(
                "Tab Bar",
                self.render_tab_bar_section(cx),
                cx,
            ))
            .child(Self::render_category_section(
                "Sidebar Nav",
                self.render_sidebar_nav_section(cx),
                cx,
            ))
    }

    fn render_numbers_category(&self, cx: &Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(Self::render_category_section(
                "Number Stepper",
                self.render_number_section(cx),
                cx,
            ))
            .child(Self::render_category_section(
                "Slider",
                self.render_slider_section(cx),
                cx,
            ))
            .child(Self::render_category_section(
                "Color Swatch",
                self.render_color_section(cx),
                cx,
            ))
    }

    #[cfg(feature = "file-picker")]
    fn render_files_category(&self, cx: &Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(Self::render_category_section(
                "File Pickers",
                self.render_file_section(cx),
                cx,
            ))
            .child(Self::render_category_section(
                "Repeatable File Picker",
                self.render_repeatable_file_section(cx),
                cx,
            ))
            .child(Self::render_category_section(
                "Repeatable Directory Picker",
                self.render_repeatable_dir_section(cx),
                cx,
            ))
    }

    #[cfg(not(feature = "file-picker"))]
    fn render_files_category(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = get_theme(cx);

        div()
            .p_4()
            .bg(rgb(theme.bg_secondary))
            .border_1()
            .border_color(rgb(theme.warning))
            .rounded_md()
            .text_sm()
            .text_color(rgb(theme.warning))
            .child(
                "File picker widgets require the 'file-picker' feature. Run with --features file-picker or --features full to enable.",
            )
    }

    fn render_progress_category(&self, cx: &Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(Self::render_category_section(
                "Progress Bar",
                self.render_progress_section(cx),
                cx,
            ))
            .child(Self::render_category_section(
                "Spinner",
                self.render_spinner_section(cx),
                cx,
            ))
    }

    fn render_utility_category(&self, cx: &Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(Self::render_category_section(
                "Collapsible",
                self.render_collapsible_section(cx),
                cx,
            ))
            .child(Self::render_category_section(
                "Tooltip",
                self.render_tooltip_section(cx),
                cx,
            ))
            .child(Self::render_category_section(
                "Scrollable",
                self.render_scrollable_section(cx),
                cx,
            ))
    }

    fn render_misc_category(&self, cx: &Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(Self::render_category_section(
                "Buttons",
                self.render_button_section(cx),
                cx,
            ))
            .child(Self::render_category_section(
                "Confirmation Dialogs",
                self.render_dialog_section(cx),
                cx,
            ))
    }
}

impl Render for WidgetGallery {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = get_theme(cx);

        div()
            .id("widget-gallery-root")
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(theme.bg_primary))
            // Header
            .child(self.render_header(cx))
            // Main area: Sidebar + Content (horizontal layout)
            .child(
                div()
                    .id("main-area")
                    .flex()
                    .flex_row()
                    .flex_1()
                    .min_h_0() // Important for flex shrinking
                    // Sidebar navigation
                    .child(self.category_sidebar_nav.clone())
                    // Content area
                    .child(
                        div()
                            .id("main-content")
                            .flex_1()
                            .min_w_0()
                            .overflow_y_scroll()
                            .bg(rgb(theme.bg_primary))
                            .p_4()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .max_w(px(900.0))
                                    .mx_auto()
                                    .child(self.render_active_category(cx)),
                            ),
                    ),
            )
            // Event log at bottom
            .child(self.render_event_log(cx))
            // Info dialog overlay (when shown)
            .when(self.show_info_dialog, |d| {
                d.child(self.info_dialog.clone())
            })
            // Yes/No dialog overlay (when shown)
            .when(self.show_yes_no_dialog, |d| {
                d.child(self.yes_no_dialog.clone())
            })
            // Save dialog overlay (when shown)
            .when(self.show_save_dialog, |d| {
                d.child(self.save_dialog.clone())
            })
            // Danger dialog overlay (when shown)
            .when(self.show_danger_dialog, |d| {
                d.child(self.danger_dialog.clone())
            })
            // Palette editor overlay (when shown)
            .when(self.show_palette_editor, |d| {
                d.child(self.render_palette_editor(cx))
            })
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        // Set the global theme
        cx.set_global(Theme::dark());

        // Register all widget keybindings (includes Tab/Shift+Tab navigation)
        register_all_keybindings(cx);

        // Register Quit keybindings
        // Mac: Cmd+Q/Cmd+W, Windows/Linux: Ctrl+Q/Ctrl+W
        #[cfg(target_os = "macos")]
        cx.bind_keys([
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("cmd-w", Quit, None),
        ]);
        #[cfg(not(target_os = "macos"))]
        cx.bind_keys([
            KeyBinding::new("ctrl-q", Quit, None),
            KeyBinding::new("ctrl-w", Quit, None),
        ]);

        // Quit application when all windows are closed
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        // Create the window
        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(1000.0), px(1000.0)),
                cx,
            ))),
            titlebar: Some(TitlebarOptions {
                title: Some(SharedString::from("Widget Gallery - ccf-gpui-widgets")),
                ..Default::default()
            }),
            ..Default::default()
        };

        cx.open_window(window_options, |_window, cx| cx.new(WidgetGallery::new))
            .unwrap();

        // Handle quit action (must be registered after window creation)
        cx.on_action(|_: &Quit, cx| cx.quit());

        cx.activate(true);
    });
}
