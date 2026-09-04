# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`ccf-gpui-widgets` is a reusable GPUI widget library for building desktop applications.  It provides themeable, accessible UI components with a consistent builder-pattern API.

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
│   ├── color.rs              # Rgb/Rgba/Hsl/Hsv, hex parsing, NAMED_COLORS, luminance
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
    ├── sidebar_nav.rs        # Vertical sidebar navigation
    ├── segmented_control.rs  # Horizontal button-style options
    ├── scrollable.rs         # Scrollable container with scrollbars
    ├── scrollbar.rs          # Scrollbar component
    ├── confirmation_dialog.rs # Modal confirmation dialogs
    ├── repeatable_text_input.rs # Text input with add/remove
    ├── button.rs             # Button factory functions
    ├── focus_navigation.rs   # Tab/Shift-Tab focus helpers
    ├── cursor_blink.rs       # Shared text-cursor blink timer
    ├── editing_core.rs       # Shared single-line editing/selection engine
    ├── selection.rs          # SelectionItem trait, StringItem, shared selection state
    ├── path_display.rs       # Path formatting/eliding shared by the pickers
    ├── sensitive_string.rs   # Zeroizing string storage for PasswordInput
    ├── file_picker.rs        # File selection (requires file-picker feature)
    ├── directory_picker.rs   # Directory selection (requires file-picker feature)
    ├── repeatable_file_picker.rs      # (requires file-picker feature)
    └── repeatable_directory_picker.rs # (requires file-picker feature)
```

### Feature Flags

```toml
[features]
default = []
file-picker = ["dep:rfd", "dep:dirs"]      # Native file dialogs
secure-password = ["dep:secrecy", "dep:zeroize"]  # Zeroizing password storage
full = ["file-picker", "secure-password"]   # All features
```

## GPUI Patterns

This library uses GPUI 0.2.2.  Key patterns:

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
    enabled: bool,
}

impl MyWidget {
    /// Create new widget
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            value: String::new(),
            // `tab_stop` MUST be set on the handle here.  Calling `.tab_stop(true)`
            // on the div after `.track_focus()` silently does nothing, and the
            // widget becomes invisible to Tab navigation.
            // See docs/TAB_STOP_SOLUTIONS.md.
            focus_handle: cx.focus_handle().tab_stop(true),
            custom_theme: None,
            enabled: true,
        }
    }

    /// Builder: set initial value
    #[must_use]
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    /// Builder: set custom theme
    #[must_use]
    pub fn theme(mut self, theme: Theme) -> Self {
        self.custom_theme = Some(theme);
        self
    }

    /// Builder: set enabled state
    #[must_use]
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Getter: current value
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Getter: enabled state
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Setter: update value programmatically
    pub fn set_value(&mut self, value: &str, cx: &mut Context<Self>) {
        if self.value != value {
            self.value = value.to_string();
            cx.emit(MyWidgetEvent::Change(self.value.clone()));
            cx.notify();
        }
    }

    /// Setter: update enabled state programmatically (no event — see below)
    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.enabled != enabled {
            self.enabled = enabled;
            cx.notify();
        }
    }

    /// Get focus handle
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }
}
```

All 17 interactive widgets carry the `enabled` field and the `with_enabled()` / `is_enabled()` / `set_enabled()` trio; a new interactive widget should too.

**Naming Convention:**
- Builder methods that set initial state: `with_*` (e.g., `with_value`, `with_selected_index`)
- Builder methods for configuration: direct name (e.g., `placeholder`, `theme`, `min`, `max`)
- Getters: direct name (e.g., `value()`, `is_checked()`)
- Setters: `set_*` (e.g., `set_value()`, `set_checked()`)

**Event Emission from Setters:**
- **Form widgets** (RadioGroup, SegmentedControl, Checkbox, etc.): `set_*` methods emit events.  These widgets represent data-bound form fields where programmatic changes should trigger the same event flow as user interaction.
- **Navigation widgets** (TabBar, SidebarNav): `set_*` methods do NOT emit events.  These widgets represent UI navigation state where the consumer typically controls transitions and doesn’t need redundant event notifications from their own calls.

## Widget Quick Reference

Most widgets follow the pattern: `cx.new(|cx| Widget::new(cx).builder_methods())`.  Six do not: `ProgressBar::new()` and `Spinner::new()` take no `cx` at all, and `Collapsible::new(title, cx)`, `ConfirmationDialog::new(title, message, cx)`, `TabBar::new(tabs, selected, cx)`, and `SidebarNav::new(items, selected, cx)` take required arguments before `cx`.  The `Key Builder Methods` column below spells out each such constructor.

| Widget | Event Type | Key Builder Methods |
|--------|-----------|---------------------|
| `TextInput` | `Change`, `Enter`, `Escape`, `Blur`, `Focus`, `Tab`, `ShiftTab` (last two only with `emit_tab_events(true)`) | `placeholder()`, `with_value()`, `select_on_focus()` |
| `PasswordInput` | `Change`, `Enter`, `Escape`, `Blur`, `Focus` | `placeholder()`, `with_value()`, `select_on_focus()` |
| `NumberStepper` | `Change(f64)` | `with_value()`, `min()`, `max()`, `step()` |
| `Slider` | `Change(f64)`, `ChangeComplete` (no payload) | `with_value()`, `min()`, `max()`, `step()`, `show_value()` |
| `Checkbox` | `Change(bool)` | `with_checked()`, `label()` |
| `ToggleSwitch` | `Change(bool)` | `with_on()`, `label()`, `label_position()` |
| `Dropdown` | `Change(String)`, `Open`, `Close` | `choices()`, `with_selected_index()`, `with_selected_value()` |
| `RadioGroup` | `Change(T)` | `choices()`, `with_selected_index()`, `with_selected_value()` |
| `CheckboxGroup` | `Change(Vec<String>)` | `choices()`, `with_selected()` |
| `SegmentedControl` | `Change(T)` | `options()`, `with_selected_index()`, `with_selected_value()` |
| `ColorSwatch` | `Change(String)` | `with_value()`, `with_alpha()` |
| `Tooltip` | (no events) | `new(text)`, `theme()` |
| `ProgressBar` | `Complete` | `with_value()`, `min()`, `max()`, `indeterminate()` |
| `Spinner` | (no events) | `size()`, `label()` |
| `Collapsible` | `Change(bool)` (payload = collapsed) | `new(title, cx)`, `with_collapsed()`, `collapsible()` |
| `TabBar` | `Change(T)`, `ContextMenu { tab, position }` | `new(tabs, selected, cx)`, `with_enabled()`, `tab_row_padding()` |
| `SidebarNav` | `Change(T)` | `new(items, selected, cx)`, `with_width()`, `with_enabled()` |
| `Scrollable` | (no events) | free functions: `scrollable_vertical()`, `scrollable_horizontal()`, `scrollable_both()` |
| `ConfirmationDialog` | `Primary`, `Secondary`, `Tertiary` | `new(title, message, cx)`, `style()`, `primary_label()`, `secondary_label()` |
| `FilePicker` | `Change(String)` | `with_value()`, `placeholder()`, `mode()`, `extensions()` |
| `DirectoryPicker` | `Change(String)` | `with_value()`, `placeholder()`, `browse_shortcut()` |
| `RepeatableTextInput` | `Change(Vec<String>)`, `EntryAdded(usize)`, `EntryRemoved(usize)` | `with_values()`, `placeholder()`, `min_entries()` |
| `RepeatableFilePicker` | `Change(Vec<String>)`, `EntryAdded(usize)`, `EntryRemoved(usize)` | `with_values()`, `placeholder()`, `extensions()`, `mode()` |
| `RepeatableDirectoryPicker` | `Change(Vec<String>)`, `EntryAdded(usize)`, `EntryRemoved(usize)` | `with_values()`, `placeholder()`, `min_entries()` |

**Notes on the table**

- `T` in `Change(T)` is the widget’s `SelectionItem` type — use `StringItem` for plain string lists, or implement `SelectionItem` on your own enum.
- The pickers emit **only** `Change(String)`; there is no `Validated` event.  Validation is pull-style: `validate()`, `is_valid()`, and the standalone `validate_file_path()` / `validate_directory_path()` functions.
- `FilePicker::extensions()` (and `RepeatableFilePicker::extensions()`) filters the **native file dialog only**.  Paths that arrive by typing, drag-and-drop, or `set_value()` are not checked against it, and `validate()` / `is_valid()` never consider the extension.
- The table is a faithful projection of the shipped enums and builders.  When a widget’s event enum or builders change, update this table in the same commit — `tests/doc_surface_consistency.rs` fails otherwise.

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
The `Theme` struct contains 49 semantic color fields as u32 hex values (as of v0.1.2; `tests/doc_claims_consistency.rs` keeps this number honest):
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
2. Define the widget struct with `focus_handle`, `custom_theme: Option<Theme>`, and — for any interactive widget — `enabled: bool` (default `true`)
3. Create the focus handle as `cx.focus_handle().tab_stop(true)` in `new()`.  Setting `.tab_stop(true)` on the div after `.track_focus()` does nothing, and the widget silently drops out of Tab navigation — see `docs/TAB_STOP_SOLUTIONS.md`
4. Add the enabled trio: `with_enabled()` (builder), `is_enabled()` (getter), `set_enabled()` (setter, `cx.notify()` only, no event), and make every interaction path return early when `!self.enabled`
5. Implement `EventEmitter<MyWidgetEvent>`
6. Implement `Focusable` if the widget needs keyboard focus
7. Implement `Render`
8. Add to `src/widgets/mod.rs`:
   - Add `mod my_widget;`
   - Add `pub use my_widget::{MyWidget, MyWidgetEvent};`
9. If widget needs keybindings:
   - Add `register_keybindings` function
   - Update `register_all_keybindings` in mod.rs
10. Add a row to the Widget Quick Reference table above — `tests/doc_surface_consistency.rs` fails if a widget module has no row

## Testing

Tests live in eight places, not one — `src/utils/path.rs`, `src/utils/mod.rs`, `src/utils/color.rs` (two modules, including a proptest), `src/widgets/sensitive_string.rs`, `src/widgets/editing_core.rs`, `src/widgets/file_picker.rs`, and `src/widgets/directory_picker.rs` — plus the doc-surface tests under `tests/`.

```bash
cargo test                  # default features: skips the feature-gated tests
cargo test --features full  # runs everything, including the doc-consistency tests under `tests/` — use this one
```

The picker and sensitive-string tests are feature-gated (`file-picker`, `secure-password`), so a bare `cargo test` silently runs a subset.  Always use `--features full` before claiming the suite is green.

Widget testing requires GPUI test support which is complex to set up, so widget _rendering_ and _interaction_ are still verified through the gallery example and integration with consumer applications.  Pure logic (editing, path parsing, color math, validation) is unit-tested directly.

## Consumer Projects

This library is currently used by:
- **clui** (`../clui/`) - CLI wrapper application with form UI

When making changes, verify compatibility:
```bash
cd ../clui && cargo check && cargo test
```

## Known Limitations

1. **TextInput key context**: Uses “CcfTextInput” context, not “TextInput” (to avoid conflicts)
2. **FilePicker/DirectoryPicker**: Require `file-picker` feature flag
3. **No general validation framework**: There is no crate-wide validation layer, so consumers own their own validation logic.  Two narrower facilities do ship: the pickers provide path-validation UI feedback (`validate()`, `is_valid()`, `ValidationDisplay`, and the standalone `validate_file_path()` / `validate_directory_path()`), and `TextInput` has `input_filter()`.  Neither is a security boundary — see `SECURITY.md`.
4. **Single-line text input only**: TextInput doesn’t support multi-line

## Code Style

- Use `rgb()` macro for colors (from gpui)
- Use `rgba()` for colors with alpha
- Prefer `.when()` over if/else for conditional styling
- Use `cx.listener()` for event handlers
- Call `cx.notify()` after state changes that affect rendering
- Emit events before `cx.notify()` so subscribers see updated state

## General Formatting Requirements
- All dates in code, comments, todo lists, etc. should be in YYYY-MM-DD format.
