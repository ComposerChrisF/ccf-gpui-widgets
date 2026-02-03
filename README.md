# ccf-gpui-widgets

Reusable GPUI widgets for building desktop applications.

## Features

- **Themeable**: All widgets support custom themes via global context or per-widget override
- **Accessible**: Keyboard navigation support where applicable
- **Event-driven**: All widgets emit events for state changes
- **Builder pattern**: Fluent API for widget configuration

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ccf-gpui-widgets = "0.1"

# For file/directory pickers (adds rfd and dirs dependencies)
# ccf-gpui-widgets = { version = "0.1", features = ["file-picker"] }
```

## Quick Start

```rust
use gpui::*;
use ccf_gpui_widgets::{Theme, widgets::*};

Application::new().run(|cx: &mut App| {
    // Register keybindings for widgets that need them
    register_all_keybindings(cx);

    // Optionally set a global theme
    cx.set_global(Theme::dark());

    cx.open_window(WindowOptions::default(), |_window, cx| {
        cx.new(|cx| MyView::new(cx))
    }).unwrap();

    cx.activate(true);
});
```

## Available Widgets

### Input Widgets

| Widget | Description |
|--------|-------------|
| `TextInput` | Full-featured text input with cursor, selection, clipboard |
| `PasswordInput` | Text input with visibility toggle |
| `NumberStepper` | Numeric input with +/- buttons |
| `Slider` | Horizontal slider for numeric ranges |

### Selection Widgets

| Widget | Description |
|--------|-------------|
| `Checkbox` | Simple checkbox with optional label |
| `ToggleSwitch` | On/off toggle with configurable label position |
| `Dropdown` | Select/dropdown with keyboard navigation |
| `RadioGroup` | Single-selection from multiple choices |
| `CheckboxGroup` | Multi-selection from multiple choices |
| `ColorSwatch` | Color picker with hex input, HSV canvas |

### Display Widgets

| Widget | Description |
|--------|-------------|
| `Tooltip` | Hover tooltip |
| `ProgressBar` | Progress indicator (determinate/indeterminate) |
| `Spinner` | Loading spinner in multiple sizes |

### Layout & Navigation

| Widget | Description |
|--------|-------------|
| `Collapsible` | Expandable/collapsible section |
| `TabBar` | Tab navigation with keyboard support |
| `ConfirmationDialog` | Modal dialogs (Info/Default/Warning/Danger styles) |

### Repeatable Widgets

| Widget | Description |
|--------|-------------|
| `RepeatableTextInput` | Text input with add/remove for lists |

### File Widgets (requires `file-picker` feature)

| Widget | Description |
|--------|-------------|
| `FilePicker` | File selection with native dialog |
| `DirectoryPicker` | Directory selection with native dialog |
| `RepeatableFilePicker` | File picker with add/remove for lists |
| `RepeatableDirectoryPicker` | Directory picker with add/remove for lists |

### Utilities

| Function | Description |
|----------|-------------|
| `primary_button()` | Blue/accent styled button |
| `secondary_button()` | Gray styled button |
| `danger_button()` | Red styled button |
| `with_focus_actions()` | Add Tab/Shift-Tab focus navigation to elements |

## Theming

Widgets use a `Theme` struct for colors. You can:

1. Set a global theme: `cx.set_global(Theme::dark())`
2. Use per-widget themes: `TextInput::new(cx).theme(my_theme)`
3. Use the default (dark theme) if nothing is set

```rust
use ccf_gpui_widgets::Theme;

// Built-in themes
let dark = Theme::dark();
let light = Theme::light();

// Customize with builder methods
let custom = Theme::dark()
    .with_accent(0x00ff00)
    .with_primary(0xff0000);
```

## Widget Usage Examples

### TextInput

```rust
let input = cx.new(|cx| {
    TextInput::new(cx)
        .placeholder("Enter text...")
        .select_on_focus(true)
});

cx.subscribe(&input, |this, _input, event: &TextInputEvent, cx| {
    match event {
        TextInputEvent::Change => { /* content changed */ }
        TextInputEvent::Enter => { /* enter pressed */ }
        _ => {}
    }
}).detach();
```

### Checkbox

```rust
let checkbox = cx.new(|cx| {
    Checkbox::new(cx)
        .checked(true)
        .label("Enable feature")
});

cx.subscribe(&checkbox, |this, _cb, event: &CheckboxEvent, cx| {
    if let CheckboxEvent::Change(checked) = event {
        println!("Checkbox is now: {}", checked);
    }
}).detach();
```

### Dropdown

```rust
let dropdown = cx.new(|cx| {
    Dropdown::new(cx)
        .choices(vec!["Option 1".into(), "Option 2".into()])
        .with_selected_index(0)
});

cx.subscribe(&dropdown, |this, _dd, event: &DropdownEvent, cx| {
    if let DropdownEvent::Change(value) = event {
        println!("Selected: {}", value);
    }
}).detach();
```

### NumberStepper

```rust
let stepper = cx.new(|cx| {
    NumberStepper::new(cx)
        .with_value(50.0)
        .min(0.0)
        .max(100.0)
        .step(5.0)
});
```

### ColorSwatch

```rust
let swatch = cx.new(|cx| {
    ColorSwatch::new(cx)
        .with_value("#3b82f6")  // Initial color (hex or CSS name like "coral")
        .with_alpha(true)       // Enable alpha channel
});

cx.subscribe(&swatch, |this, _swatch, event: &ColorSwatchEvent, cx| {
    if let ColorSwatchEvent::Change(hex) = event {
        println!("Color changed: {}", hex);  // e.g., "#3B82F6" or "#3B82F680"
    }
}).detach();
```

The color picker popup includes:
- 2D saturation/brightness canvas (HSV model)
- Hue slider (0-359°)
- Alpha slider (when enabled)
- RGB component sliders
- Old/New color comparison
- Hex value display

Supports hex input (#RGB, #RRGGBB, #RRGGBBAA) and all 140 CSS named colors.

### Slider

```rust
let slider = cx.new(|cx| {
    Slider::new(cx)
        .with_value(50.0)
        .min(0.0)
        .max(100.0)
        .step(1.0)
        .show_value(true)
});

cx.subscribe(&slider, |this, _slider, event: &SliderEvent, cx| {
    match event {
        SliderEvent::Change(value) => { /* value changing */ }
        SliderEvent::ChangeComplete(value) => { /* drag ended */ }
    }
}).detach();
```

### ToggleSwitch

```rust
let toggle = cx.new(|cx| {
    ToggleSwitch::new(cx)
        .with_on(true)
        .label("Dark mode")
        .label_position(LabelPosition::Left)
});

cx.subscribe(&toggle, |this, _toggle, event: &ToggleSwitchEvent, cx| {
    if let ToggleSwitchEvent::Change(is_on) = event {
        println!("Toggle is now: {}", is_on);
    }
}).detach();
```

### TabBar

```rust
let tabs = cx.new(|cx| {
    TabBar::new(cx)
        .tabs(vec![
            TabItem::new("general", "General"),
            TabItem::new("advanced", "Advanced"),
            TabItem::new("about", "About"),
        ])
        .with_selected_index(0)
});

cx.subscribe(&tabs, |this, _tabs, event: &TabBarEvent, cx| {
    if let TabBarEvent::Change(index) = event {
        println!("Selected tab: {}", index);
    }
}).detach();
```

### ConfirmationDialog

```rust
let dialog = cx.new(|cx| {
    ConfirmationDialog::new(cx)
        .style(DialogStyle::Warning)
        .title("Delete Item")
        .message("Are you sure you want to delete this item? This action cannot be undone.")
        .primary_label("Delete")
        .secondary_label("Cancel")
});

cx.subscribe(&dialog, |this, _dialog, event: &ConfirmationDialogEvent, cx| {
    match event {
        ConfirmationDialogEvent::Primary => { /* confirmed */ }
        ConfirmationDialogEvent::Secondary => { /* cancelled */ }
        ConfirmationDialogEvent::Tertiary => { /* third option */ }
    }
}).detach();
```

### FilePicker (requires `file-picker` feature)

```rust
let picker = cx.new(|cx| {
    FilePicker::new(cx)
        .mode(FileMode::Save)
        .extensions(vec!["json".into(), "yaml".into()])
        .placeholder("Select output file...")
});
```

## Keybindings

Some widgets require keybindings to be registered at startup. Call `register_all_keybindings(cx)` once during app initialization:

```rust
Application::new().run(|cx: &mut App| {
    ccf_gpui_widgets::register_all_keybindings(cx);
    // ...
});
```

Or register individual widgets:

```rust
ccf_gpui_widgets::widgets::text_input::register_keybindings(cx);
ccf_gpui_widgets::widgets::dropdown::register_keybindings(cx);
```

## License

MIT OR Apache-2.0
