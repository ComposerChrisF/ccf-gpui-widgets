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
- [x] ColorSwatch with hex input and preview
- [x] Collapsible section header
- [x] FilePicker with native dialog, drag-drop, path validation
- [x] DirectoryPicker with native dialog, drag-drop
- [x] Path utilities (PathInfo, parse_path, expand_tilde)
- [x] Feature flags for optional dependencies
- [x] README with usage examples
- [x] Unit tests for path utilities

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
- [ ] Tabs component
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
