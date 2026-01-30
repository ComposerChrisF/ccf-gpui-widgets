//! Widget Gallery - A demo application showcasing all ccf-gpui-widgets
//!
//! Run with: cargo run --example widget_gallery --features full
//! Or without file pickers: cargo run --example widget_gallery

use gpui::prelude::FluentBuilder;
use gpui::*;
use ccf_gpui_widgets::prelude::*;
use ccf_gpui_widgets::Theme;
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

impl TabItem for GalleryTab {
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

/// Main application state
struct WidgetGallery {
    // Theme
    current_theme: ThemeChoice,

    // Collapsible sections
    section_text: Entity<Collapsible>,
    section_checkbox: Entity<Collapsible>,
    section_dropdown: Entity<Collapsible>,
    section_number: Entity<Collapsible>,
    section_radio: Entity<Collapsible>,
    section_checkbox_group: Entity<Collapsible>,
    section_color: Entity<Collapsible>,
    section_tooltip: Entity<Collapsible>,
    section_button: Entity<Collapsible>,
    section_password: Entity<Collapsible>,
    section_tab_bar: Entity<Collapsible>,
    section_repeatable_text: Entity<Collapsible>,
    #[cfg(feature = "file-picker")]
    section_file: Entity<Collapsible>,
    #[cfg(feature = "file-picker")]
    section_repeatable_file: Entity<Collapsible>,

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
    repeatable_text_input: Entity<RepeatableTextInput>,
    #[cfg(feature = "file-picker")]
    repeatable_file_picker: Entity<RepeatableFilePicker>,

    // Button click tracking (buttons are not Entities)
    primary_click_count: usize,
    secondary_click_count: usize,

    // Event log
    event_log: VecDeque<EventLogEntry>,
    log_collapsed: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum ThemeChoice {
    Dark,
    Light,
}

struct EventLogEntry {
    timestamp: String,
    widget: String,
    event: String,
}

impl WidgetGallery {
    fn new(cx: &mut Context<Self>) -> Self {
        // Create collapsible sections
        let section_text = cx.new(|_cx| Collapsible::new("Text Input"));
        let section_checkbox = cx.new(|_cx| Collapsible::new("Checkbox"));
        let section_dropdown = cx.new(|_cx| Collapsible::new("Dropdown"));
        let section_number = cx.new(|_cx| Collapsible::new("Number Stepper"));
        let section_radio = cx.new(|_cx| Collapsible::new("Radio Group"));
        let section_checkbox_group = cx.new(|_cx| Collapsible::new("Checkbox Group"));
        let section_color = cx.new(|_cx| Collapsible::new("Color Swatch"));
        let section_tooltip = cx.new(|_cx| Collapsible::new("Tooltip"));
        let section_button = cx.new(|_cx| Collapsible::new("Button"));
        let section_password = cx.new(|_cx| Collapsible::new("Password Input"));
        let section_tab_bar = cx.new(|_cx| Collapsible::new("Tab Bar"));
        let section_repeatable_text = cx.new(|_cx| Collapsible::new("Repeatable Text Input"));
        #[cfg(feature = "file-picker")]
        let section_file = cx.new(|_cx| Collapsible::new("File Pickers"));
        #[cfg(feature = "file-picker")]
        let section_repeatable_file = cx.new(|_cx| Collapsible::new("Repeatable File Picker"));

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
            &repeatable_text_input,
            #[cfg(feature = "file-picker")]
            &repeatable_file_picker,
        );

        Self {
            current_theme: ThemeChoice::Dark,
            section_text,
            section_checkbox,
            section_dropdown,
            section_number,
            section_radio,
            section_checkbox_group,
            section_color,
            section_tooltip,
            section_button,
            section_password,
            section_tab_bar,
            section_repeatable_text,
            #[cfg(feature = "file-picker")]
            section_file,
            #[cfg(feature = "file-picker")]
            section_repeatable_file,
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
            repeatable_text_input,
            #[cfg(feature = "file-picker")]
            repeatable_file_picker,
            primary_click_count: 0,
            secondary_click_count: 0,
            event_log: VecDeque::new(),
            log_collapsed: false,
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
        // TextInput events
        cx.subscribe(text_input, |this, _entity, event: &TextInputEvent, cx| {
            this.log_event("TextInput", format!("{:?}", event), cx);
        })
        .detach();

        cx.subscribe(
            text_input_placeholder,
            |this, _entity, event: &TextInputEvent, cx| {
                this.log_event("TextInput (prefilled)", format!("{:?}", event), cx);
            },
        )
        .detach();

        // Checkbox events
        cx.subscribe(checkbox, |this, _entity, event: &CheckboxEvent, cx| {
            this.log_event("Checkbox", format!("{:?}", event), cx);
        })
        .detach();

        cx.subscribe(
            checkbox_labeled,
            |this, _entity, event: &CheckboxEvent, cx| {
                this.log_event("Checkbox (labeled)", format!("{:?}", event), cx);
            },
        )
        .detach();

        // Dropdown events
        cx.subscribe(dropdown, |this, _entity, event: &DropdownEvent, cx| {
            this.log_event("Dropdown", format!("{:?}", event), cx);
        })
        .detach();

        // NumberStepper events
        cx.subscribe(
            number_stepper,
            |this, _entity, event: &NumberStepperEvent, cx| {
                this.log_event("NumberStepper (int)", format!("{:?}", event), cx);
            },
        )
        .detach();

        cx.subscribe(
            number_stepper_float,
            |this, _entity, event: &NumberStepperEvent, cx| {
                this.log_event("NumberStepper (float)", format!("{:?}", event), cx);
            },
        )
        .detach();

        // RadioGroup events
        cx.subscribe(radio_group, |this, _entity, event: &RadioGroupEvent, cx| {
            this.log_event("RadioGroup", format!("{:?}", event), cx);
        })
        .detach();

        // CheckboxGroup events
        cx.subscribe(
            checkbox_group,
            |this, _entity, event: &CheckboxGroupEvent, cx| {
                this.log_event("CheckboxGroup", format!("{:?}", event), cx);
            },
        )
        .detach();

        // ColorSwatch events
        cx.subscribe(color_swatch, |this, _entity, event: &ColorSwatchEvent, cx| {
            this.log_event("ColorSwatch", format!("{:?}", event), cx);
        })
        .detach();

        cx.subscribe(
            color_swatch_alpha,
            |this, _entity, event: &ColorSwatchEvent, cx| {
                this.log_event("ColorSwatch (alpha)", format!("{:?}", event), cx);
            },
        )
        .detach();

        // File picker events
        #[cfg(feature = "file-picker")]
        {
            cx.subscribe(file_picker, |this, _entity, event: &FilePickerEvent, cx| {
                this.log_event("FilePicker", format!("{:?}", event), cx);
            })
            .detach();

            cx.subscribe(
                directory_picker,
                |this, _entity, event: &DirectoryPickerEvent, cx| {
                    this.log_event("DirectoryPicker", format!("{:?}", event), cx);
                },
            )
            .detach();
        }
    }

    fn subscribe_new_events(
        cx: &mut Context<Self>,
        password_input: &Entity<PasswordInput>,
        tab_bar: &Entity<TabBar<GalleryTab>>,
        repeatable_text_input: &Entity<RepeatableTextInput>,
        #[cfg(feature = "file-picker")] repeatable_file_picker: &Entity<RepeatableFilePicker>,
    ) {
        cx.subscribe(password_input, |this, _entity, event: &PasswordInputEvent, cx| {
            this.log_event("PasswordInput", format!("{:?}", event), cx);
        })
        .detach();

        cx.subscribe(tab_bar, |this, _entity, event: &TabBarEvent<GalleryTab>, cx| {
            this.log_event("TabBar", format!("{:?}", event), cx);
        })
        .detach();

        cx.subscribe(
            repeatable_text_input,
            |this, _entity, event: &RepeatableTextInputEvent, cx| {
                this.log_event("RepeatableTextInput", format!("{:?}", event), cx);
            },
        )
        .detach();

        #[cfg(feature = "file-picker")]
        cx.subscribe(
            repeatable_file_picker,
            |this, _entity, event: &RepeatableFilePickerEvent, cx| {
                this.log_event("RepeatableFilePicker", format!("{:?}", event), cx);
            },
        )
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

    fn toggle_theme(&mut self, cx: &mut Context<Self>) {
        self.current_theme = match self.current_theme {
            ThemeChoice::Dark => ThemeChoice::Light,
            ThemeChoice::Light => ThemeChoice::Dark,
        };

        let theme = match self.current_theme {
            ThemeChoice::Dark => Theme::dark(),
            ThemeChoice::Light => Theme::light(),
        };

        cx.set_global(theme);
        cx.notify();
    }

    fn render_header(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = get_theme(cx);
        let theme_button_text = match self.current_theme {
            ThemeChoice::Dark => "Switch to Light",
            ThemeChoice::Light => "Switch to Dark",
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
                    .id("theme-toggle")
                    .px_3()
                    .py_1()
                    .bg(rgb(theme.primary))
                    .hover(|s| s.bg(rgb(theme.primary_hover)))
                    .text_color(rgb(theme.text_primary))
                    .rounded_md()
                    .cursor_pointer()
                    .child(theme_button_text)
                    .on_click(cx.listener(|this, _event, _window, cx| {
                        this.toggle_theme(cx);
                    })),
            )
    }

    fn render_section<V: IntoElement + 'static>(
        &self,
        section: &Entity<Collapsible>,
        content: impl FnOnce() -> V,
        cx: &Context<Self>,
    ) -> impl IntoElement
    {
        let theme = get_theme(cx);
        let is_collapsed = section.read(cx).is_collapsed();

        div()
            .w_full()
            .mb_2()
            .border_1()
            .border_color(rgb(theme.border_default))
            .rounded_md()
            .overflow_hidden()
            .child(section.clone())
            .when(!is_collapsed, |d| {
                d.child(
                    div()
                        .p_4()
                        .bg(rgb(theme.bg_secondary))
                        .child(content()),
                )
            })
    }

    fn render_widget_row(
        label: &str,
        description: &str,
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
                            .child(label.to_string()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(theme.text_muted))
                            .child(description.to_string()),
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

    fn render_button_section(&mut self, cx: &mut Context<Self>) -> Div {
        let theme = get_theme(cx);
        let primary_count = self.primary_click_count;
        let secondary_count = self.secondary_click_count;

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
            .p_4()
            .bg(rgb(theme.bg_secondary))
    }

    fn render_button_section_wrapper(&mut self, cx: &mut Context<Self>) -> Div {
        let theme = get_theme(cx);
        let is_collapsed = self.section_button.read(cx).is_collapsed();

        let content = if !is_collapsed {
            Some(self.render_button_section(cx))
        } else {
            None
        };

        div()
            .w_full()
            .mb_2()
            .border_1()
            .border_color(rgb(theme.border_default))
            .rounded_md()
            .overflow_hidden()
            .child(self.section_button.clone())
            .when_some(content, |d, c| d.child(c))
    }

    fn render_password_section(&self, cx: &Context<Self>) -> impl IntoElement {
        let value = self.password_input.read(cx).value(cx).to_string();
        let display = if value.is_empty() {
            "(empty)".to_string()
        } else {
            // Show actual value for demo purposes (this gallery is not collecting real passwords)
            format!("\"{}\"", value)
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
        let active = format!("{:?}", self.tab_bar.read(cx).active_tab());

        div().child(Self::render_widget_row(
            "Tab Bar",
            "Click tabs, right-click for context menu",
            self.tab_bar.clone(),
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

    #[cfg(feature = "file-picker")]
    fn render_repeatable_file_section(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = get_theme(cx);
        let values = self.repeatable_file_picker.read(cx).values();
        let display = if values.is_empty() {
            "(no files selected)".to_string()
        } else {
            values.join("\n")
        };

        div()
            .flex()
            .flex_col()
            .gap_2()
            // Value display at top with wrapping
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
                            .when(values.is_empty(), |d| d.child(display.clone()))
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
            // Widget row below
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
                                    .child("Repeatable File Picker"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(theme.text_muted))
                                    .child("Add/remove file selections (min: 1)"),
                            ),
                    )
                    .child(div().flex_1().child(self.repeatable_file_picker.clone())),
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

    /// Renders the file pickers section, or a notice if the feature is disabled
    #[cfg(feature = "file-picker")]
    fn render_file_pickers_section(&self, cx: &Context<Self>) -> Div {
        let theme = get_theme(cx);
        let is_collapsed = self.section_file.read(cx).is_collapsed();

        div()
            .w_full()
            .mb_2()
            .border_1()
            .border_color(rgb(theme.border_default))
            .rounded_md()
            .overflow_hidden()
            .child(self.section_file.clone())
            .when(!is_collapsed, |d| {
                d.child(
                    div()
                        .p_4()
                        .bg(rgb(theme.bg_secondary))
                        .child(self.render_file_section(cx)),
                )
            })
    }

    #[cfg(not(feature = "file-picker"))]
    fn render_file_pickers_section(&self, cx: &Context<Self>) -> Div {
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
                "Note: File pickers not shown. Run with --features file-picker or --features full to enable.",
            )
    }

    #[cfg(feature = "file-picker")]
    fn render_repeatable_file_pickers_section(&self, cx: &Context<Self>) -> Div {
        let theme = get_theme(cx);
        let is_collapsed = self.section_repeatable_file.read(cx).is_collapsed();

        div()
            .w_full()
            .mb_2()
            .border_1()
            .border_color(rgb(theme.border_default))
            .rounded_md()
            .overflow_hidden()
            .child(self.section_repeatable_file.clone())
            .when(!is_collapsed, |d| {
                d.child(
                    div()
                        .p_4()
                        .bg(rgb(theme.bg_secondary))
                        .child(self.render_repeatable_file_section(cx)),
                )
            })
    }

    #[cfg(not(feature = "file-picker"))]
    fn render_repeatable_file_pickers_section(&self, _cx: &Context<Self>) -> Div {
        div() // Empty when feature disabled
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
            // Main content area with scrolling
            .child(
                div()
                    .id("main-content")
                    .w_full()
                    .min_w_0()
                    .flex_1()
                    .overflow_y_scroll()
                    .p_4()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .max_w(px(900.0))
                            .mx_auto()
                            // Text Input Section
                            .child(self.render_section(
                                &self.section_text,
                                || self.render_text_input_section(cx),
                                cx,
                            ))
                            // Checkbox Section
                            .child(self.render_section(
                                &self.section_checkbox,
                                || self.render_checkbox_section(cx),
                                cx,
                            ))
                            // Dropdown Section
                            .child(self.render_section(
                                &self.section_dropdown,
                                || self.render_dropdown_section(cx),
                                cx,
                            ))
                            // Number Stepper Section
                            .child(self.render_section(
                                &self.section_number,
                                || self.render_number_section(cx),
                                cx,
                            ))
                            // Radio Group Section
                            .child(self.render_section(
                                &self.section_radio,
                                || self.render_radio_section(cx),
                                cx,
                            ))
                            // Checkbox Group Section
                            .child(self.render_section(
                                &self.section_checkbox_group,
                                || self.render_checkbox_group_section(cx),
                                cx,
                            ))
                            // Color Swatch Section
                            .child(self.render_section(
                                &self.section_color,
                                || self.render_color_section(cx),
                                cx,
                            ))
                            // Tooltip Section
                            .child(self.render_section(
                                &self.section_tooltip,
                                || self.render_tooltip_section(cx),
                                cx,
                            ))
                            // Button Section
                            .child(self.render_button_section_wrapper(cx))
                            // Password Section
                            .child(self.render_section(
                                &self.section_password,
                                || self.render_password_section(cx),
                                cx,
                            ))
                            // Tab Bar Section
                            .child(self.render_section(
                                &self.section_tab_bar,
                                || self.render_tab_bar_section(cx),
                                cx,
                            ))
                            // Repeatable Text Input Section
                            .child(self.render_section(
                                &self.section_repeatable_text,
                                || self.render_repeatable_text_section(cx),
                                cx,
                            ))
                            // File Pickers Section (conditional)
                            .child(self.render_file_pickers_section(cx))
                            // Repeatable File Picker Section (conditional)
                            .child(self.render_repeatable_file_pickers_section(cx)),
                    ),
            )
            // Event log at bottom
            .child(self.render_event_log(cx))
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
                size(px(1000.0), px(800.0)),
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
