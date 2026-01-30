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

---

## Future Enhancements

### 📦 Publishing
- [ ] Publish to crates.io
- [ ] Set up GitHub repository
- [ ] Add CI/CD (GitHub Actions)
- [ ] Add more comprehensive tests

### 🎨 Theme Improvements
- [ ] Add more built-in themes (e.g., high contrast)
- [ ] Theme hot-reloading support
- [ ] CSS-like theme file loading

### 🧩 New Widgets
- [ ] Slider (horizontal range input)
- [ ] Toggle/Switch (iOS-style toggle)
- [ ] ProgressBar
- [x] Tabs component (TabBar)
- [ ] Modal/Dialog
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
