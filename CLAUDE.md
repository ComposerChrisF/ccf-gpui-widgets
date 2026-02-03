# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`ccf-gpui-widgets` is a reusable GPUI widget library for building desktop applications. It provides themeable, accessible UI components with a consistent builder-pattern API.

**Key Features:**
- Themeable widgets via global context or per-widget override
- EventEmitter pattern for state changes
- Keyboard navigation support
- Builder pattern for configuration
- Feature flags for optional dependencies

## Build Commands

```bash
# Build the library
cargo build

# Build with all features
cargo build --features full

# Run tests
cargo test

# Check for compilation errors
cargo check                 # Default features
cargo check --features full # All features

# Build documentation
cargo doc --open

# Running Clippy
cargo clippy                 # Default features
cargo clippy --features full # All features

# Testing Clui with latest `ccf-gpui-widgets` changes
cd ../clui && cargo check && cargo test
```

### Crates dependent on `ccf-gpui-widgets`
Since clui depends on this library, changes need coordination:                
  - Test changes with cd ../clui && cargo check && cargo test                   

## Architecture

### Directory Structure

```
src/
├── lib.rs                    # Crate root, re-exports
├── theme.rs                  # Theme struct with dark/light presets
├── utils/
│   ├── mod.rs
│   └── path.rs               # PathInfo, parse_path, expand_tilde
└── widgets/
    ├── mod.rs                # Widget re-exports, register_all_keybindings()
    ├── text_input.rs         # Full-featured text input
    ├── password_input.rs     # Text input with visibility toggle
    ├── number_stepper.rs     # Numeric input with +/- buttons
    ├── slider.rs             # Horizontal slider for numeric ranges
    ├── checkbox.rs           # Checkbox with optional label
    ├── toggle_switch.rs      # On/off toggle switch
    ├── dropdown.rs           # Dropdown with keyboard navigation
    ├── radio_group.rs        # Single-selection radio buttons
    ├── checkbox_group.rs     # Multi-selection checkboxes
    ├── color_swatch.rs       # Color picker with hex input, HSV canvas
    ├── tooltip.rs            # Simple tooltip
    ├── progress_bar.rs       # Determinate/indeterminate progress
    ├── spinner.rs            # Loading spinner
    ├── collapsible.rs        # Expandable/collapsible section
    ├── tab_bar.rs            # Tab navigation
    ├── confirmation_dialog.rs # Modal confirmation dialogs
    ├── repeatable_text_input.rs # Text input with add/remove
    ├── button.rs             # Button factory functions
    ├── focus_navigation.rs   # Tab/Shift-Tab focus helpers
    ├── file_picker.rs        # File selection (requires file-picker feature)
    ├── directory_picker.rs   # Directory selection (requires file-picker feature)
    ├── repeatable_file_picker.rs      # (requires file-picker feature)
    └── repeatable_directory_picker.rs # (requires file-picker feature)
```

### Feature Flags

```toml
[features]
default = []
file-picker = ["dep:rfd", "dep:dirs"]  # Native file dialogs
full = ["file-picker"]                  # All features
```

## GPUI Patterns

This library uses GPUI 0.2.2. Key patterns:

### Views and Rendering
```rust
impl Render for MyWidget {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        div()
            .id("my_widget")
            .child("content")
    }
}
```

### EventEmitter Pattern
All widgets emit events for state changes:
```rust
#[derive(Clone, Debug)]
pub enum MyWidgetEvent {
    Change(String),
}

impl EventEmitter<MyWidgetEvent> for MyWidget {}

// In widget methods:
cx.emit(MyWidgetEvent::Change(new_value));
```

### Focusable Widgets
```rust
impl Focusable for MyWidget {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
```

### Key Context for Keyboard Shortcuts
```rust
div()
    .id("my_widget")
    .key_context("MyWidgetContext")  // Must match KeyBinding context
    .track_focus(&self.focus_handle)
    .on_action(cx.listener(|this, _: &MyAction, _window, cx| {
        // Handle action
    }))
```

## Widget API Pattern

All widgets follow this consistent pattern:

```rust
pub struct MyWidget {
    value: String,
    focus_handle: FocusHandle,
    custom_theme: Option<Theme>,
}

impl MyWidget {
    /// Create new widget
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            value: String::new(),
            focus_handle: cx.focus_handle(),
            custom_theme: None,
        }
    }

    /// Builder: set initial value
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    /// Builder: set custom theme
    pub fn theme(mut self, theme: Theme) -> Self {
        self.custom_theme = Some(theme);
        self
    }

    /// Getter: current value
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Setter: update value programmatically
    pub fn set_value(&mut self, value: &str, cx: &mut Context<Self>) {
        if self.value != value {
            self.value = value.to_string();
            cx.emit(MyWidgetEvent::Change(self.value.clone()));
            cx.notify();
        }
    }

    /// Get focus handle
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }
}
```

**Naming Convention:**
- Builder methods that set initial state: `with_*` (e.g., `with_value`, `with_selected_index`)
- Builder methods for configuration: direct name (e.g., `placeholder`, `theme`, `min`, `max`)
- Getters: direct name (e.g., `value()`, `is_checked()`)
- Setters: `set_*` (e.g., `set_value()`, `set_checked()`)

## Widget Quick Reference

All widgets follow the pattern: `cx.new(|cx| Widget::new(cx).builder_methods())`

| Widget | Event Type | Key Builder Methods |
|--------|-----------|---------------------|
| TextInput | Change, Enter, Escape, Blur, Focus | placeholder(), select_on_focus(), with_value() |
| PasswordInput | Change, Enter, Escape, Blur, Focus | placeholder(), select_on_focus() |
| NumberStepper | Change(f64) | with_value(), min(), max(), step() |
| Slider | Change(f64), ChangeComplete | with_value(), min(), max(), step(), show_value() |
| Checkbox | Change(bool) | checked(), label() |
| ToggleSwitch | Change(bool) | with_on(), label(), label_position() |
| Dropdown | Change(String), Open, Close | choices(), with_selected_index(), placeholder() |
| RadioGroup | Change(usize) | choices(), with_selected_index() |
| CheckboxGroup | Change(Vec<usize>) | choices(), with_selected_indices() |
| ColorSwatch | Change(String) | with_value(), with_alpha() |
| ProgressBar | Change(f64) | with_value(), indeterminate() |
| Spinner | (no events) | size() |
| Collapsible | Change(bool) | with_expanded(), title() |
| TabBar | Change(usize) | tabs(), with_selected_index() |
| ConfirmationDialog | Primary, Secondary, Tertiary | style(), primary_label(), secondary_label() |
| FilePicker | Change(PathBuf), Validated | mode(), extensions(), placeholder() |
| DirectoryPicker | Change(PathBuf), Validated | placeholder() |
| RepeatableTextInput | Change(Vec<String>), Add, Remove | with_values(), placeholder() |

### Subscription Pattern
```rust
cx.subscribe(&widget, |this, _widget, event: &WidgetEvent, cx| {
    match event {
        WidgetEvent::Change(value) => { /* handle */ }
        _ => {}
    }
}).detach();
```

## Theming System

### Theme Structure
The `Theme` struct contains ~30 semantic color fields as u32 hex values:
- Background colors: `bg_primary`, `bg_input`, `bg_hover`, etc.
- Text colors: `text_primary`, `text_muted`, `text_placeholder`, etc.
- Border colors: `border_default`, `border_focus`, `border_error`, etc.
- Status colors: `success`, `error`, `warning`
- Accent colors: `primary`, `accent`

### Theme Usage in Widgets
```rust
use crate::theme::{get_theme_or, Theme};

impl Render for MyWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = get_theme_or(cx, self.custom_theme.as_ref());

        div()
            .bg(rgb(theme.bg_input))
            .text_color(rgb(theme.text_primary))
            .border_color(rgb(theme.border_default))
    }
}
```

### Consumer Setup
Consumers should set a global theme at app startup:
```rust
Application::new().run(|cx: &mut App| {
    cx.set_global(ccf_gpui_widgets::Theme::dark());
    // ...
});
```

## Adding a New Widget

1. Create `src/widgets/my_widget.rs`
2. Define the widget struct with `focus_handle` and `custom_theme: Option<Theme>`
3. Implement `EventEmitter<MyWidgetEvent>`
4. Implement `Focusable` if the widget needs keyboard focus
5. Implement `Render`
6. Add to `src/widgets/mod.rs`:
   - Add `mod my_widget;`
   - Add `pub use my_widget::{MyWidget, MyWidgetEvent};`
7. If widget needs keybindings:
   - Add `register_keybindings` function
   - Update `register_all_keybindings` in mod.rs

## Testing

Tests are in `src/utils/path.rs` for path utilities. Run with:
```bash
cargo test
```

Widget testing requires GPUI test support which is complex to set up. Most widget behavior is verified through integration with consumer applications.

## Consumer Projects

This library is currently used by:
- **clui** (`../clui/`) - CLI wrapper application with form UI

When making changes, verify compatibility:
```bash
cd ../clui && cargo check && cargo test
```

## Known Limitations

1. **TextInput key context**: Uses "CcfTextInput" context, not "TextInput" (to avoid conflicts)
2. **FilePicker/DirectoryPicker**: Require `file-picker` feature flag
3. **No built-in validation**: Consumers handle validation logic
4. **Single-line text input only**: TextInput doesn't support multi-line

## Code Style

- Use `rgb()` macro for colors (from gpui)
- Use `rgba()` for colors with alpha
- Prefer `.when()` over if/else for conditional styling
- Use `cx.listener()` for event handlers
- Call `cx.notify()` after state changes that affect rendering
- Emit events before `cx.notify()` so subscribers see updated state

## General Formatting Requirements
- All dates in code, comments, todo lists, etc. should be in YYYY-MM-DD format.
