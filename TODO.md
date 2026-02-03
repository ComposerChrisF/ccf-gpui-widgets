# ccf-gpui-widgets - TODO

## Current Status: Initial Release ✅

The library is functional and being used by clui.

---

## Completed

### ✅ Initial Implementation (2026-01-25)
- [x] Theme system with dark/light presets and builder pattern
- [x] TextInput with cursor, selection, clipboard, horizontal scrolling
- [x] Tooltip for hover text
- [x] Checkbox with optional label
- [x] Dropdown with keyboard navigation (Up/Down/Enter/Escape)
- [x] NumberStepper with +/- buttons, min/max/step
- [x] RadioGroup for single selection
- [x] CheckboxGroup for multi-selection
- [x] ColorSwatch with hex input and preview (basic version)
- [x] Collapsible section header
- [x] FilePicker with native dialog, drag-drop, path validation
- [x] DirectoryPicker with native dialog, drag-drop
- [x] Path utilities (PathInfo, parse_path, expand_tilde)
- [x] Feature flags for optional dependencies
- [x] README with usage examples
- [x] Unit tests for path utilities

### ✅ NumberStepper Enhancements (2026-01-26)
- [x] Double-click to edit value as text, Enter/Tab/click-away to commit
- [x] Click & drag on value for scrubber-style adjustment
- [x] Mouse capture during drag (continues outside control bounds)
- [x] Three drag sensitivities: normal, fast (Shift), slow (Alt/Option)
- [x] Value resolution (snap to multiples)
- [x] Display precision (independent of stored value)
- [x] Unified visual design (single rounded box with separators)
- [x] Widget gallery: Cmd+Q/Cmd+W (Mac) and Ctrl+Q/Ctrl+W (other) to quit
- [x] Documentation: docs/MOUSE_CAPTURE.md

### ✅ Widget Bug Fixes (2026-01-26)
- [x] Dropdown: Use `.occlude()` to block mouse events from reaching elements below open menu
- [x] Dropdown: Changed to `on_mouse_down` for choice selection (snappier response)
- [x] TextInput: Preserve selection and scroll offset when focus is lost
- [x] TextInput: Restore selection/scroll when focus returns (via Tab or click)
- [x] TextInput: Click on unfocused input with selection restores state instead of repositioning cursor

### ✅ ColorSwatch Enhancements (2026-01-27)
- [x] Full color picker popup with 2D saturation/brightness canvas
- [x] HSV color model (Photoshop-style picker behavior)
- [x] Hue slider (0-359°, capped to prevent wrap-around)
- [x] Alpha slider with checkerboard visualization
- [x] RGB component sliders (R, G, B)
- [x] Old/New color comparison with hex values
- [x] CSS named color support (140 colors)
- [x] Proper drag support using GPUI's drag API
- [x] Persistent slider measurements across render frames
- [x] Color utilities: Rgb, Rgba, Hsl, Hsv with conversions
- [x] Documentation: docs/COLOR_PICKER.md

### ✅ Theme Consistency & Dark/Light Mode Fixes (2026-01-27)
- [x] TextInput: Use `bg_input` background and `border_input` border (match NumberStepper)
- [x] Dropdown button: Use `bg_input` background and `border_input` border
- [x] Dropdown popup menu: Use theme-aware colors for dark/light mode
- [x] Checkbox (unchecked): Use `bg_input` background instead of white
- [x] Checkbox (checked): Use white checkmark for contrast on blue background
- [x] CheckboxGroup: Same checkbox styling fixes
- [x] ColorSwatch popup: Use `bg_secondary` background and `text_primary` for labels
- [x] Text selection: Darker color (`0x264F78`) for contrast with white text
- [x] Light mode: `border_input` changed to `0x444444` for visible borders
- [x] Light mode: Tooltip uses light background with dark text
- [x] Theme: Added `with_*` builder methods for all 30+ fields

### ✅ FilePicker & DirectoryPicker Enhancements (2026-01-28)
- [x] Focus indicator on outer control when widget has focus but TextInput is inactive
- [x] ESC key returns focus to picker (instead of losing focus entirely)
- [x] TextInput emits separate `Escape` event (distinct from `Blur`)
- [x] Browse button is keyboard accessible (Tab stop, Enter/Space to activate)
- [x] Cmd+O / Ctrl+O shortcut to open file dialog (configurable via `.browse_shortcut(bool)`)
- [x] Validation API: `validate()` returns enum with detailed state
- [x] Validation API: `is_valid()` convenience method
- [x] Standalone `validate_file_path()` and `validate_directory_path()` functions
- [x] `ValidationDisplay` enum to control feedback visibility (Full/ColorsOnly/MessageOnly/Hidden)
- [x] Unit tests for validation logic (15 new tests)

### ✅ FilePicker & DirectoryPicker UI Redesign (2026-01-29)
- [x] Path display uses StyledText with TextRuns for proper word-wrap at `/` boundaries
- [x] Unified border around entire control (text area + buttons)
- [x] Compact icon buttons (✎ Edit, 📂 Select/💾 Save) with tooltips
- [x] Buttons stretch to match text area height
- [x] Text area no longer focusable (use Edit button instead)
- [x] Empty state uses italic text
- [x] Blue focus outline on buttons (consistent with other widgets)

### ✅ New Widgets (2026-01-30)
- [x] Button: `primary_button()` and `secondary_button()` utility functions with theming
- [x] PasswordInput: Secure password field with bullet masking and visibility toggle
- [x] TabBar: Generic tab navigation with `TabItem` trait, context menu support
- [x] RepeatableTextInput: Dynamic list of text inputs with add/remove buttons
- [x] RepeatableFilePicker: Multi-file selection with validation, drag-drop support
- [x] Theme extensions: 12 new color fields (disabled, secondary, tabs, delete, path)
- [x] Widget gallery: Added demos for Button, PasswordInput, TabBar, RepeatableTextInput, RepeatableFilePicker
- [x] RepeatableTextInput: Fixed borrow conflict panic when typing in newly added entries

### ✅ PasswordInput & TextInput Enhancements (2026-01-30)
- [x] TextInput: Added `masked` mode for password input (displays bullets instead of text)
- [x] TextInput: Added `borderless` mode for embedding in unified containers
- [x] TextInput: Word navigation disabled in masked mode (prevents password structure leak)
- [x] TextInput: Copy disabled in masked mode (prevents password clipboard leak)
- [x] TextInput: Cut deletes but doesn't copy in masked mode
- [x] PasswordInput: Refactored to use TextInput internally (full editing support)
- [x] PasswordInput: Unified visual styling matching NumberStepper (shared border/background)
- [x] PasswordInput: Simpler line-art eye icons (◎ show, ⊖ hide)
- [x] PasswordInput: Fixed-width toggle button (no size change between states)
- [x] PasswordInput: Both text input and toggle button are proper tab stops
- [x] Widget gallery: Shows actual typed value for demo purposes

### ✅ Keyboard Navigation Improvements (2026-01-30)
- [x] Button: Added `.focusable()` and `.tab_stop()` for keyboard navigation
- [x] Button: Added FocusNext/FocusPrev action handlers for Tab/Shift+Tab
- [x] Button: Primary button uses `border_focus_on_color` for visible focus on colored backgrounds
- [x] Button: Disabled buttons excluded from tab order
- [x] TabBar: Added tab stop with focus handle tracking
- [x] TabBar: Added left/right arrow key navigation (wraps around)
- [x] TabBar: Focus ring displays only on active tab, not whole bar
- [x] RepeatableTextInput: +/- buttons are focusable tab stops
- [x] RepeatableTextInput: Button height matches text input (28px)
- [x] RepeatableFilePicker: +/- buttons are focusable tab stops
- [x] Theme: Added `border_focus_on_color` for focus on colored backgrounds

### ✅ Repeatable Picker Refactoring (2026-01-30)
- [x] RepeatableFilePicker: Refactored to use `Entity<FilePicker>` internally (removes code duplication)
- [x] RepeatableFilePicker: Added `browse_shortcut()` and `validation_display()` builder methods
- [x] RepeatableFilePicker: Added validation API (`entries()`, `validate_all()`, `is_all_valid()`, `directories_to_create()`)
- [x] RepeatableFilePicker: **Breaking change**: `values()` now requires `cx` parameter
- [x] RepeatableDirectoryPicker: New widget for multi-directory selection (uses `Entity<DirectoryPicker>`)
- [x] Widget gallery: Added RepeatableDirectoryPicker demo

### ✅ Secure PasswordInput with secrecy/zeroize (2026-01-30)
- [x] New `secure-password` feature flag with `secrecy` and `zeroize` dependencies
- [x] EditingCore: Generic editing engine with `ContentStorage` trait abstraction
- [x] TextInput: Refactored to use `EditingCore<String>` internally (non-breaking)
- [x] SensitiveString: Secure storage type wrapping `Zeroizing<String>` with redacted Debug
- [x] PasswordInput: Complete rewrite using `EditingCore<SensitiveString>` directly
- [x] PasswordInput: `value()` returns `SecretString` (with feature), `set_value_secret()` method
- [x] PasswordInput: `PasswordInputEvent::Change` carries `SecretString` (with feature)
- [x] PasswordInput: Memory zeroization on drop, type-level enforcement of sensitive data
- [x] PasswordInput: Full cursor/selection/editing support (copy still disabled for security)
- [x] Widget gallery: Updated to handle conditional `SecretString` API
- [x] Backward compatible: Without feature, API unchanged; `full` feature includes `secure-password`

### ✅ NumberStepper Refactoring (2026-01-30)
- [x] NumberStepper: Refactored to use embedded TextInput for edit mode (removes ~130 lines of code)
- [x] NumberStepper: Gains TextInput features: selection, copy/paste, word navigation, proper scrolling
- [x] NumberStepper: No longer needs separate keybinding registration (uses TextInput keybindings)
- [x] TextInput: Added `input_filter()` builder method to restrict allowed characters
- [x] TextInput: Input filter applied to both typing and pasting
- [x] TextInput: Added `emit_tab_events()` for parent controls to intercept Tab key
- [x] TextInput: Added `Tab` and `ShiftTab` events to `TextInputEvent`
- [x] **Breaking change**: `register_number_stepper_keybindings()` removed from exports

### ✅ Code Review Improvements (2026-02-01)
- [x] Added `#[must_use]` attribute to all builder methods across all widgets
- [x] Collapsible: Now implements `Focusable` trait with keyboard support (Tab/Space/Enter)
- [x] Collapsible: **Breaking change**: `new()` now requires `cx` parameter
- [x] EditingCore: Word boundary functions use iterators instead of Vec allocation
- [x] PasswordInput: Removed unused `get_theme()` method
- [x] Slider: Removed unused `multiplier` variable

### ✅ Keyboard Event Fixes (2026-02-01)
- [x] Collapsible: Fixed double-toggle on Space/Enter (was responding to both keydown and synthetic click)
- [x] Collapsible: Added Up/Down arrow keys to collapse/expand
- [x] ConfirmationDialog: Saves and restores focus when dialog is dismissed
- [x] ConfirmationDialog: Dismissal keys (Enter/Escape/custom) now respond on keyup instead of keydown
- [x] ConfirmationDialog: Prevents race condition where keydown launches dialog, same keydown dismisses it, keyup relaunches

### ✅ Disabled State Support (2026-02-01)
- [x] Added disabled state to all 17 interactive widgets
- [x] Each widget has: `enabled: bool` field, `with_enabled()` builder, `is_enabled()` getter, `set_enabled()` setter
- [x] Disabled widgets: grayed styling, no hover effects, removed from tab order, interactions blocked
- [x] Widgets updated: TextInput, Checkbox, Collapsible, Dropdown, RadioGroup, CheckboxGroup, ToggleSwitch, TabBar, Slider, NumberStepper, PasswordInput, ColorSwatch, FilePicker, DirectoryPicker, RepeatableTextInput, RepeatableFilePicker, RepeatableDirectoryPicker
- [x] **Breaking change**: ToggleSwitch renamed `enabled`→`on`, `with_enabled()`→`with_on()`, `is_enabled()`→`is_on()`, `set_enabled()`→`set_on()` (to avoid conflict with disabled state)
- [x] Added missing getters: NumberStepper (`get_min/max/step/resolution/display_precision`), Slider (`get_min/max/step/display_precision`), Dropdown (`is_open`)
- [x] Widget gallery: Added "Enable/Disable Widgets" toggle button to demonstrate disabled state

### ✅ TabBar Focus Behavior Improvements (2026-02-02)
- [x] TabBar: Mouse clicks no longer steal focus from other widgets
- [x] TabBar: Captures previous focus on mouse_down, restores it after tab selection
- [x] TabBar: Blurs if nothing was previously focused (prevents unwanted focus acquisition)
- [x] TabBar: Focus ring uses always-present 2px transparent border to prevent layout shift
- [x] TabBar: Restructured to outer focus-ring container + inner content div

### ✅ Code Review Fixes (2026-02-02)
- [x] Fixed conditional doc links in lib.rs - FilePicker/DirectoryPicker docs only included when file-picker feature enabled
- [x] Added `log` crate dependency for path resolution error logging
- [x] path.rs: Added `log::warn!` for `current_dir()` and `canonicalize()` failures
- [x] Extracted shared path display types to `widgets/path_display.rs` module
- [x] Unified `ValidationDisplay`, `PathHighlight`, `PathDisplayInfo` types used by both pickers
- [x] Removed ~60 lines of duplicated code from file_picker.rs and directory_picker.rs
- [x] Added `proptest` dev-dependency for property-based testing
- [x] Added RGB↔HSL and RGB↔HSV roundtrip property tests in color.rs
- [x] Added `#[doc(hidden)]` to internal drag state types (NumberDragState, SliderDragState, SlDrag, HueDrag, AlphaDrag, ComponentDrag, EmptyDragView)

### ✅ Code Simplification (2026-02-01)
- [x] Extracted duplicate `format_display_value()` utility function to `utils/mod.rs`
- [x] NumberStepper and Slider now use shared `format_display_value()` function
- [x] TextInput: Simplified selection bounds calculation using iterator chaining
- [x] Slider: Simplified `go_to_min()`/`go_to_max()` to reuse `set_value()`
- [x] Added unit tests for `format_display_value()` function
- [x] Extracted `handle_tab_navigation()` helper to focus_navigation module
- [x] Simplified tab key handling across 9 widgets using the shared helper
- [x] Widget gallery: Added `subscribe_widget!` macro to reduce event subscription boilerplate
- [x] Widget gallery: Extracted `dialog_result_label()` helper for dialog event handlers
- [x] Widget gallery: Consolidated repeatable picker rendering with `render_repeatable_picker_section()`
- [x] Widget gallery: Reduced file size by ~170 lines (~8% reduction)
- [x] Widget gallery: Pre-create spinner entities to avoid allocation on each render
- [x] Widget gallery: Changed `render_widget_row` to use `&'static str` (avoids string allocations)
- [x] Widget gallery: Changed render methods to use `&Context` instead of `&mut Context` where possible
- [x] Fixed clippy `approx_constant` errors in format_display_value tests

### ✅ Focus Navigation & Cursor Helpers (2026-02-02)
- [x] Added `with_focus_actions()` generic helper function to focus_navigation module
- [x] Added `EnabledCursorExt` trait with `.cursor_for_enabled(bool)` method
- [x] Simplified FocusNext/FocusPrev action handlers across 13 widgets
- [x] Simplified cursor_pointer/cursor_default patterns across 16 widgets
- [x] Exported `with_focus_actions` and `EnabledCursorExt` from widgets module

### ✅ New Widgets: Toggle, Slider, Progress, Spinner, Dialog (2026-01-30)
- [x] ToggleSwitch: iOS-style toggle switch with pill-shaped track and circular thumb
- [x] ToggleSwitch: Label support with configurable position (left or right)
- [x] ToggleSwitch: Space/Enter to toggle, full keyboard navigation
- [x] Slider: Horizontal slider with draggable thumb, min/max/step constraints
- [x] Slider: Keyboard support: Left/Right arrows, Shift for 10x step, Home/End for min/max
- [x] Slider: Optional value display
- [x] ProgressBar: Determinate mode with filled bar based on percentage
- [x] ProgressBar: Indeterminate mode with animated sliding bar
- [x] ProgressBar: Optional percentage text and label
- [x] ProgressBar: Complete event when reaching 100%
- [x] Spinner: Animated 8-dot loading spinner with size presets (Small/Medium/Large/Custom)
- [x] Spinner: Optional label text
- [x] ConfirmationDialog: Modal dialog with semi-transparent overlay backdrop
- [x] ConfirmationDialog: Four styles - Info, Default, Warning, Danger
- [x] ConfirmationDialog: Configurable primary/secondary/tertiary buttons
- [x] ConfirmationDialog: Custom key mappings via `map_key()` (e.g., Y/N for Yes/No)
- [x] ConfirmationDialog: Uses `.occlude()` to block mouse events from reaching controls below
- [x] Button: Added `danger_button()` utility function with error color styling
- [x] Widget gallery: Shows dialog results next to launch buttons

---

## Future Enhancements

### 📦 Publishing
- [ ] Publish to crates.io
- [ ] Set up GitHub repository
- [x] Add CI/CD (GitHub Actions) - includes cargo audit for security scanning
- [ ] Add more comprehensive tests

### 🔐 Security Documentation (2026-01-30)
- [x] SECURITY.md documenting security model
- [x] Password handling with memory zeroization
- [x] Path validation caveats (UI display vs security boundary)
- [x] CI workflow with cargo audit for dependency vulnerability scanning

### 🎨 Theme Improvements
- [ ] Add more built-in themes (e.g., high contrast)
- [ ] Theme hot-reloading support
- [ ] CSS-like theme file loading

### 🧩 New Widgets
- [x] Slider (horizontal range input)
- [x] Toggle/Switch (iOS-style toggle)
- [x] ProgressBar
- [x] Tabs component (TabBar)
- [x] Modal/Dialog (ConfirmationDialog)
- [x] Spinner (loading indicator)
- [ ] Toast/Notification
- [ ] TreeView
- [ ] DataTable

### ✨ Widget Enhancements
- [ ] TextInput: Multi-line support (TextArea)
- [ ] TextInput: Input masking (phone, credit card)
- [ ] TextInput: Auto-complete/suggestions
- [ ] Dropdown: Search/filter in dropdown
- [ ] Dropdown: Multi-select dropdown
- [ ] NumberStepper: Slider mode
- [ ] FilePicker: Recent files list
- [ ] FilePicker: Thumbnail preview for images

### ♿ Accessibility
- [ ] ARIA attribute support
- [ ] Screen reader announcements
- [ ] High contrast mode
- [ ] Reduced motion support

### 📖 Documentation
- [ ] API documentation (rustdoc)
- [ ] Example application
- [ ] Widget showcase/gallery
- [ ] Migration guide for consumers

### 🧪 Testing
- [ ] Widget unit tests with GPUI test support
- [ ] Visual regression tests
- [ ] Accessibility tests

---

## Known Issues

1. **Key context naming**: TextInput uses "CcfTextInput" to avoid conflicts with consumers who might have their own "TextInput" context.

2. **FilePicker/DirectoryPicker compilation**: These require the `file-picker` feature flag and won't compile without it.

3. **Theme mismatch**: If consumer doesn't set global theme, widgets use default dark theme which may not match consumer's UI.

4. **Flexbox layout instability**: Scrollable flex containers need `w_full()` and `min_w_0()` to prevent layout issues. See `GPUI-LAYOUT-PATTERNS.md` for details.

---

## Resolved Issues

### Tab Navigation Fix (2026-01-26)

**Issue**: Tab key navigation between widgets was not working.

**Root Cause**: When using `.track_focus(&focus_handle)` on a div, the subsequent `.tab_stop(true)` call does not affect the already-tracked focus handle. GPUI only applies the div's tab_stop setting when creating a new handle (when `tracked_focus_handle.is_none()`).

**Solution**: Set `tab_stop(true)` directly on the FocusHandle when creating it:
```rust
focus_handle: cx.focus_handle().tab_stop(true),
```

See `doc/TAB_STOP_SOLUTIONS.md` for full details.

---

## Consumer Compatibility

When making breaking changes:
1. Update version number appropriately (semver)
2. Document migration steps
3. Test with clui: `cd ../clui && cargo check && cargo test`
