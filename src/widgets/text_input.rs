//! Text input widget
//!
//! A full-featured text input with cursor positioning, selection, and clipboard support.
//! Uses GPUI's text shaping APIs for accurate cursor positioning with variable-width fonts.
//!
//! # Example
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::TextInput;
//!
//! // Register keybindings at app startup
//! ccf_gpui_widgets::widgets::text_input::register_keybindings(cx);
//!
//! // Create a text input
//! let input = cx.new(|cx| TextInput::new(cx).placeholder("Enter text..."));
//!
//! // Subscribe to events
//! cx.subscribe(&input, |this, _input, event: &TextInputEvent, cx| {
//!     match event {
//!         TextInputEvent::Change => {
//!             // Handle content change
//!         }
//!         TextInputEvent::Enter => {
//!             // Handle Enter key
//!         }
//!         _ => {}
//!     }
//! }).detach();
//! ```

use std::time::Duration;

use gpui::prelude::*;
use gpui::*;

use crate::theme::{get_theme_or, Theme};
use super::cursor_blink::CursorBlink;
use super::editing_core::EditingCore;
use super::focus_navigation::{FocusNext, FocusPrev};

// Actions for keyboard handling
actions!(
    ccf_text_input,
    [
        MoveLeft,
        MoveRight,
        MoveWordLeft,
        MoveWordRight,
        MoveToStart,
        MoveToEnd,
        SelectLeft,
        SelectRight,
        SelectWordLeft,
        SelectWordRight,
        SelectToStart,
        SelectToEnd,
        SelectAll,
        DeleteBackward,
        DeleteForward,
        DeleteWordBackward,
        DeleteWordForward,
        Cut,
        Copy,
        Paste,
        Enter,
        Escape,
    ]
);

/// Register key bindings for text input components
///
/// Call this once at application startup:
/// ```ignore
/// ccf_gpui_widgets::widgets::text_input::register_keybindings(cx);
/// ```
pub fn register_keybindings(cx: &mut App) {
    cx.bind_keys([
        // Navigation
        KeyBinding::new("left", MoveLeft, Some("CcfTextInput")),
        KeyBinding::new("right", MoveRight, Some("CcfTextInput")),
        KeyBinding::new("home", MoveToStart, Some("CcfTextInput")),
        KeyBinding::new("end", MoveToEnd, Some("CcfTextInput")),
        // Selection
        KeyBinding::new("shift-left", SelectLeft, Some("CcfTextInput")),
        KeyBinding::new("shift-right", SelectRight, Some("CcfTextInput")),
        KeyBinding::new("shift-home", SelectToStart, Some("CcfTextInput")),
        KeyBinding::new("shift-end", SelectToEnd, Some("CcfTextInput")),
        // Delete
        KeyBinding::new("backspace", DeleteBackward, Some("CcfTextInput")),
        KeyBinding::new("delete", DeleteForward, Some("CcfTextInput")),
        // Actions
        KeyBinding::new("enter", Enter, Some("CcfTextInput")),
        KeyBinding::new("escape", Escape, Some("CcfTextInput")),
        // Note: Tab/ShiftTab not bound here - let GPUI handle tab navigation
    ]);

    // Platform-specific bindings
    #[cfg(target_os = "macos")]
    cx.bind_keys([
        // Clipboard
        KeyBinding::new("cmd-a", SelectAll, Some("CcfTextInput")),
        KeyBinding::new("cmd-c", Copy, Some("CcfTextInput")),
        KeyBinding::new("cmd-x", Cut, Some("CcfTextInput")),
        KeyBinding::new("cmd-v", Paste, Some("CcfTextInput")),
        // Word navigation (Option+Arrow on macOS)
        KeyBinding::new("alt-left", MoveWordLeft, Some("CcfTextInput")),
        KeyBinding::new("alt-right", MoveWordRight, Some("CcfTextInput")),
        KeyBinding::new("shift-alt-left", SelectWordLeft, Some("CcfTextInput")),
        KeyBinding::new("shift-alt-right", SelectWordRight, Some("CcfTextInput")),
        // Word delete
        KeyBinding::new("alt-backspace", DeleteWordBackward, Some("CcfTextInput")),
        KeyBinding::new("alt-delete", DeleteWordForward, Some("CcfTextInput")),
    ]);

    #[cfg(not(target_os = "macos"))]
    cx.bind_keys([
        // Clipboard
        KeyBinding::new("ctrl-a", SelectAll, Some("CcfTextInput")),
        KeyBinding::new("ctrl-c", Copy, Some("CcfTextInput")),
        KeyBinding::new("ctrl-x", Cut, Some("CcfTextInput")),
        KeyBinding::new("ctrl-v", Paste, Some("CcfTextInput")),
        // Word navigation (Ctrl+Arrow on Windows/Linux)
        KeyBinding::new("ctrl-left", MoveWordLeft, Some("CcfTextInput")),
        KeyBinding::new("ctrl-right", MoveWordRight, Some("CcfTextInput")),
        KeyBinding::new("shift-ctrl-left", SelectWordLeft, Some("CcfTextInput")),
        KeyBinding::new("shift-ctrl-right", SelectWordRight, Some("CcfTextInput")),
        // Word delete
        KeyBinding::new("ctrl-backspace", DeleteWordBackward, Some("CcfTextInput")),
        KeyBinding::new("ctrl-delete", DeleteWordForward, Some("CcfTextInput")),
    ]);
}

/// Events emitted by TextInput
#[derive(Clone, Debug)]
pub enum TextInputEvent {
    /// Content changed (use `state.read(cx).content()` to get the new value)
    Change,
    /// Enter key pressed
    Enter,
    /// Escape key pressed (use to cancel editing and return focus to parent)
    Escape,
    /// Input lost focus
    Blur,
    /// Input gained focus
    Focus,
    /// Tab key pressed (only emitted when emit_tab_events is true)
    Tab,
    /// Shift+Tab key pressed (only emitted when emit_tab_events is true)
    ShiftTab,
}

/// Character used to mask password input
const MASK_CHAR: &str = "\u{25CF}"; // ● Black circle

/// Text input widget state
pub struct TextInput {
    /// Core editing logic
    core: EditingCore<String>,
    /// Placeholder text
    placeholder: Option<SharedString>,
    /// Focus handle for keyboard focus
    focus_handle: FocusHandle,
    /// Whether to select all text when focused
    pub select_on_focus: bool,
    /// Horizontal scroll offset in pixels
    scroll_offset: f32,
    /// Visible width of the text area
    visible_width: f32,
    /// Left edge of content area in window coordinates
    content_origin_x: f32,
    /// Track previous focus state
    was_focused: bool,
    /// Whether focus-out subscription has been set up
    focus_out_subscribed: bool,
    /// Cursor blink state
    cursor_blink: CursorBlink,
    /// Whether blink timer is set up
    blink_timer_active: bool,
    /// Optional custom theme
    custom_theme: Option<Theme>,
    /// Whether currently dragging to select text
    is_dragging: bool,
    /// Whether auto-scroll timer is active
    auto_scroll_active: bool,
    /// Current auto-scroll speed (pixels per frame, positive = scroll right)
    auto_scroll_speed: f32,
    /// Whether to render without border/background (for embedding in other controls)
    borderless: bool,
    /// Optional filter for allowed input characters
    input_filter: Option<Box<dyn Fn(char) -> bool>>,
    /// Whether to emit Tab/ShiftTab events instead of handling focus navigation
    emit_tab_events: bool,
}

impl EventEmitter<TextInputEvent> for TextInput {}

impl Focusable for TextInput {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl TextInput {
    /// Create a new text input
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            core: EditingCore::new(),
            placeholder: None,
            focus_handle: cx.focus_handle().tab_stop(true),
            select_on_focus: false,
            scroll_offset: 0.0,
            visible_width: 200.0,
            content_origin_x: 0.0,
            was_focused: false,
            focus_out_subscribed: false,
            cursor_blink: CursorBlink::new(),
            blink_timer_active: false,
            custom_theme: None,
            is_dragging: false,
            auto_scroll_active: false,
            auto_scroll_speed: 0.0,
            borderless: false,
            input_filter: None,
            emit_tab_events: false,
        }
    }

    /// Set placeholder text (builder pattern)
    #[must_use]
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// Set initial value (builder pattern)
    #[must_use]
    pub fn with_value(mut self, text: impl Into<String>) -> Self {
        let text = text.into();
        self.core.set_content(&text);
        self
    }

    /// Set custom theme (builder pattern)
    #[must_use]
    pub fn theme(mut self, theme: Theme) -> Self {
        self.custom_theme = Some(theme);
        self
    }

    /// Set select on focus (builder pattern)
    #[must_use]
    pub fn select_on_focus(mut self, select: bool) -> Self {
        self.select_on_focus = select;
        self
    }

    /// Set masked mode for password input (builder pattern)
    ///
    /// When masked, the input displays bullet characters instead of the actual text,
    /// while retaining full editing functionality (cursor movement, selection, etc.)
    #[must_use]
    pub fn masked(mut self, masked: bool) -> Self {
        self.core.set_masked(masked);
        self
    }

    /// Check if this input is in masked mode
    pub fn is_masked(&self) -> bool {
        self.core.is_masked()
    }

    /// Set masked mode programmatically
    pub fn set_masked(&mut self, masked: bool, cx: &mut Context<Self>) {
        self.core.set_masked(masked);
        cx.notify();
    }

    /// Set borderless mode for embedding in other controls (builder pattern)
    ///
    /// When borderless, the input renders without its own border, background,
    /// and rounded corners, allowing it to be embedded in unified containers.
    #[must_use]
    pub fn borderless(mut self, borderless: bool) -> Self {
        self.borderless = borderless;
        self
    }

    /// Set an input filter to restrict allowed characters (builder pattern)
    ///
    /// The filter function receives each character and should return `true` to allow it.
    /// Characters that fail the filter are silently dropped during typing and pasting.
    ///
    /// # Example
    /// ```ignore
    /// TextInput::new(cx)
    ///     .input_filter(|c| c.is_ascii_digit() || c == '.' || c == '-')
    /// ```
    #[must_use]
    pub fn input_filter(mut self, filter: impl Fn(char) -> bool + 'static) -> Self {
        self.input_filter = Some(Box::new(filter));
        self
    }

    /// Emit Tab/ShiftTab events instead of handling focus navigation (builder pattern)
    ///
    /// When true, pressing Tab or Shift+Tab emits `TextInputEvent::Tab` or
    /// `TextInputEvent::ShiftTab` respectively, allowing the parent to handle
    /// focus navigation. This is useful when embedding TextInput in other controls
    /// that need to intercept Tab for custom behavior.
    ///
    /// When false (default), Tab/Shift+Tab directly calls `window.focus_next()`
    /// or `window.focus_prev()`.
    #[must_use]
    pub fn emit_tab_events(mut self, emit: bool) -> Self {
        self.emit_tab_events = emit;
        self
    }

    /// Get the display content (masked or real)
    fn display_content(&self) -> String {
        if self.core.is_masked() {
            MASK_CHAR.repeat(self.core.content().chars().count())
        } else {
            self.core.content().to_string()
        }
    }

    /// Convert a byte index in content to a byte index in display content
    fn content_byte_to_display_byte(&self, content_pos: usize) -> usize {
        if !self.core.is_masked() || self.core.content().is_empty() {
            return content_pos;
        }
        let char_count = self.core.content()[..content_pos].chars().count();
        char_count * MASK_CHAR.len()
    }

    /// Convert a byte index in display content to a byte index in content
    fn display_byte_to_content_byte(&self, display_pos: usize) -> usize {
        if !self.core.is_masked() || self.core.content().is_empty() {
            return display_pos;
        }
        let mask_char_len = MASK_CHAR.len();
        let char_index = display_pos / mask_char_len;
        // Convert char index to byte index in content
        self.core.content()
            .char_indices()
            .nth(char_index)
            .map(|(i, _)| i)
            .unwrap_or(self.core.content().len())
    }

    /// Reset cursor blink timer
    fn reset_cursor_blink(&mut self) {
        self.cursor_blink.reset();
    }

    /// Get the current content
    pub fn content(&self) -> &str {
        self.core.content()
    }

    /// Set the content value
    pub fn set_value(&mut self, value: &str, cx: &mut Context<Self>) {
        self.core.set_content(value);
        self.scroll_offset = 0.0;
        cx.emit(TextInputEvent::Change);
        cx.notify();
    }

    /// Set placeholder text programmatically
    pub fn set_placeholder(&mut self, text: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.placeholder = Some(text.into());
        cx.notify();
    }

    /// Get the focus handle
    pub fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }

    /// Copy selected text to clipboard (disabled when masked)
    fn copy(&self, cx: &mut Context<Self>) {
        if self.core.is_masked() {
            // Don't allow copying password content
            return;
        }
        if let Some(text) = self.core.selected_text() {
            cx.write_to_clipboard(ClipboardItem::new_string(text.to_string()));
        }
    }

    /// Cut selected text to clipboard (delete only when masked, no copy)
    fn cut(&mut self, cx: &mut Context<Self>) {
        if !self.core.is_masked() {
            self.copy(cx);
        }
        if self.core.delete_selection() {
            self.reset_cursor_blink();
            cx.emit(TextInputEvent::Change);
            cx.notify();
        }
    }

    /// Paste from clipboard
    fn paste(&mut self, cx: &mut Context<Self>) {
        if let Some(clipboard) = cx.read_from_clipboard() {
            if let Some(text) = clipboard.text() {
                let clean_text = text.replace(['\n', '\r'], "");

                // Apply input filter if present
                let filtered_text: String = if let Some(ref filter) = self.input_filter {
                    clean_text.chars().filter(|c| filter(*c)).collect()
                } else {
                    clean_text
                };

                if filtered_text.is_empty() {
                    return;
                }

                self.core.insert_text(&filtered_text);
                self.reset_cursor_blink();
                cx.emit(TextInputEvent::Change);
                cx.notify();
            }
        }
    }

    /// Handle focus gained
    fn on_focus(&mut self, cx: &mut Context<Self>) {
        if self.select_on_focus && !self.core.content().is_empty() {
            self.core.select_all();
        }
        self.reset_cursor_blink();
        cx.emit(TextInputEvent::Focus);
        cx.notify();
    }

    /// Handle focus lost
    fn on_blur(&mut self, cx: &mut Context<Self>) {
        // Stop any drag operation
        self.stop_drag();
        // Don't clear selection or scroll_offset - they'll be hidden visually
        // but restored when focus returns
        cx.emit(TextInputEvent::Blur);
        cx.notify();
    }

    /// Shape the display content for measurement
    fn shape_line(&self, window: &Window) -> Option<ShapedLine> {
        let display = self.display_content();
        if display.is_empty() {
            return None;
        }

        let style = window.text_style();
        let font_size = window.rem_size() * 0.875;

        let run = TextRun {
            len: display.len(),
            font: style.font(),
            color: style.color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };

        Some(window.text_system().shape_line(
            SharedString::from(display),
            font_size,
            &[run],
            None,
        ))
    }

    /// Calculate cursor position from click x coordinate
    fn cursor_at_x(&self, x: f32, window: &Window) -> usize {
        let adjusted_x = x + self.scroll_offset;

        if adjusted_x <= 0.0 || self.core.content().is_empty() {
            return 0;
        }

        if let Some(line) = self.shape_line(window) {
            let display_pos = line.closest_index_for_x(px(adjusted_x));
            self.display_byte_to_content_byte(display_pos)
        } else {
            0
        }
    }

    /// Get x position for a character index
    fn x_for_cursor(&self, cursor: usize, window: &Window) -> f32 {
        if self.core.content().is_empty() || cursor == 0 {
            return 0.0;
        }

        if let Some(line) = self.shape_line(window) {
            let display_cursor = self.content_byte_to_display_byte(cursor);
            let pixels = line.x_for_index(display_cursor);
            pixels.into()
        } else {
            0.0
        }
    }

    /// Ensure the cursor is visible within the viewport
    fn ensure_cursor_visible(&mut self, window: &Window) {
        let cursor_x = self.x_for_cursor(self.core.cursor(), window);
        let content_width = self.x_for_cursor(self.core.content().len(), window);
        let margin = 2.0;
        let cursor_width = 1.0;
        let padding = 2.0;

        let actual_visible = self.visible_width - padding;

        if content_width <= actual_visible {
            self.scroll_offset = 0.0;
            return;
        }

        let visual_cursor_x = cursor_x - self.scroll_offset;

        if visual_cursor_x + cursor_width > actual_visible - margin {
            self.scroll_offset = cursor_x + cursor_width - actual_visible + margin;
        }

        if visual_cursor_x < margin {
            self.scroll_offset = (cursor_x - margin).max(0.0);
        }

        let max_scroll = (content_width + cursor_width - actual_visible + margin).max(0.0);
        self.scroll_offset = self.scroll_offset.clamp(0.0, max_scroll);
    }

    /// Handle drag move during text selection
    /// Returns the auto-scroll speed (0 if no scrolling needed)
    fn handle_drag_move(&mut self, mouse_x: f32, window: &Window) -> f32 {
        if !self.is_dragging {
            return 0.0;
        }

        let relative_x = mouse_x - self.content_origin_x;
        let padding = 2.0;
        let actual_visible = self.visible_width - padding;

        // Calculate auto-scroll speed based on distance from edge
        let scroll_speed = if relative_x < 0.0 {
            // Mouse is left of the input - scroll left (negative)
            -self.calculate_scroll_speed(-relative_x)
        } else if relative_x > actual_visible {
            // Mouse is right of the input - scroll right (positive)
            self.calculate_scroll_speed(relative_x - actual_visible)
        } else {
            0.0
        };

        // Update cursor position based on mouse x (clamped to visible area for cursor calculation)
        let clamped_x = relative_x.clamp(0.0, actual_visible);
        let new_cursor = self.cursor_at_x(clamped_x, window);
        self.core.extend_selection_to(new_cursor);
        self.reset_cursor_blink();

        scroll_speed
    }

    /// Calculate scroll speed based on distance from edge
    /// Uses an ease-out curve for acceleration
    fn calculate_scroll_speed(&self, distance: f32) -> f32 {
        let base_speed = 0.5; // pixels per frame at the edge
        let max_speed = 20.0; // max pixels per frame
        let max_distance = 100.0; // distance at which max speed is reached

        let normalized = (distance / max_distance).min(1.0);
        // Ease-out curve: fast start, slow end
        let eased = 1.0 - (1.0 - normalized).powi(2);
        base_speed + eased * (max_speed - base_speed)
    }

    /// Apply auto-scroll during drag
    fn apply_auto_scroll(&mut self, window: &Window) {
        if self.auto_scroll_speed == 0.0 {
            return;
        }

        let content_width = self.x_for_cursor(self.core.content().len(), window);
        let padding = 2.0;
        let actual_visible = self.visible_width - padding;
        let max_scroll = (content_width - actual_visible).max(0.0);

        // Apply scroll
        self.scroll_offset = (self.scroll_offset + self.auto_scroll_speed).clamp(0.0, max_scroll);

        // Update cursor to match scroll direction
        if self.auto_scroll_speed < 0.0 {
            // Scrolling left - move cursor toward start
            let new_cursor = self.cursor_at_x(0.0, window);
            self.core.extend_selection_to(new_cursor);
        } else {
            // Scrolling right - move cursor toward end
            let new_cursor = self.cursor_at_x(actual_visible, window);
            self.core.extend_selection_to(new_cursor);
        }
    }

    /// Stop drag operation
    fn stop_drag(&mut self) {
        self.is_dragging = false;
        self.auto_scroll_active = false;
        self.auto_scroll_speed = 0.0;
    }

    /// Spawn auto-scroll timer if scrolling is needed and timer isn't already active
    fn spawn_auto_scroll_timer_if_needed(&mut self, scroll_speed: f32, window: &mut Window, cx: &mut Context<Self>) {
        self.auto_scroll_speed = scroll_speed;
        if scroll_speed != 0.0 && !self.auto_scroll_active {
            self.auto_scroll_active = true;
            let entity = cx.entity();
            window.spawn(cx, async move |async_cx| {
                loop {
                    smol::Timer::after(Duration::from_millis(32)).await; // ~30fps
                    let should_continue = async_cx
                        .update_entity(&entity, |this, cx| {
                            if !this.auto_scroll_active || !this.is_dragging {
                                this.auto_scroll_active = false;
                                return false;
                            }
                            cx.notify();
                            true
                        })
                        .unwrap_or(false);
                    if !should_continue {
                        break;
                    }
                }
            }).detach();
        }
    }

    // Action handlers that delegate to core and emit events

    fn handle_move_left(&mut self, cx: &mut Context<Self>) {
        self.core.move_left();
        self.reset_cursor_blink();
        cx.notify();
    }

    fn handle_move_right(&mut self, cx: &mut Context<Self>) {
        self.core.move_right();
        self.reset_cursor_blink();
        cx.notify();
    }

    fn handle_move_word_left(&mut self, cx: &mut Context<Self>) {
        self.core.move_word_left();
        self.reset_cursor_blink();
        cx.notify();
    }

    fn handle_move_word_right(&mut self, cx: &mut Context<Self>) {
        self.core.move_word_right();
        self.reset_cursor_blink();
        cx.notify();
    }

    fn handle_move_to_start(&mut self, cx: &mut Context<Self>) {
        self.core.move_to_start();
        self.reset_cursor_blink();
        cx.notify();
    }

    fn handle_move_to_end(&mut self, cx: &mut Context<Self>) {
        self.core.move_to_end();
        self.reset_cursor_blink();
        cx.notify();
    }

    fn handle_select_left(&mut self, cx: &mut Context<Self>) {
        self.core.select_left();
        self.reset_cursor_blink();
        cx.notify();
    }

    fn handle_select_right(&mut self, cx: &mut Context<Self>) {
        self.core.select_right();
        self.reset_cursor_blink();
        cx.notify();
    }

    fn handle_select_word_left(&mut self, cx: &mut Context<Self>) {
        self.core.select_word_left();
        self.reset_cursor_blink();
        cx.notify();
    }

    fn handle_select_word_right(&mut self, cx: &mut Context<Self>) {
        self.core.select_word_right();
        self.reset_cursor_blink();
        cx.notify();
    }

    fn handle_select_to_start(&mut self, cx: &mut Context<Self>) {
        self.core.select_to_start();
        self.reset_cursor_blink();
        cx.notify();
    }

    fn handle_select_to_end(&mut self, cx: &mut Context<Self>) {
        self.core.select_to_end();
        self.reset_cursor_blink();
        cx.notify();
    }

    fn handle_select_all(&mut self, cx: &mut Context<Self>) {
        self.core.select_all();
        self.reset_cursor_blink();
        cx.notify();
    }

    fn handle_delete_backward(&mut self, cx: &mut Context<Self>) {
        if self.core.delete_backward() {
            self.reset_cursor_blink();
            cx.emit(TextInputEvent::Change);
        }
        cx.notify();
    }

    fn handle_delete_forward(&mut self, cx: &mut Context<Self>) {
        if self.core.delete_forward() {
            self.reset_cursor_blink();
            cx.emit(TextInputEvent::Change);
        }
        cx.notify();
    }

    fn handle_delete_word_backward(&mut self, cx: &mut Context<Self>) {
        if self.core.delete_word_backward() {
            self.reset_cursor_blink();
            cx.emit(TextInputEvent::Change);
        }
        cx.notify();
    }

    fn handle_delete_word_forward(&mut self, cx: &mut Context<Self>) {
        if self.core.delete_word_forward() {
            self.reset_cursor_blink();
            cx.emit(TextInputEvent::Change);
        }
        cx.notify();
    }

    fn handle_insert_text(&mut self, text: &str, cx: &mut Context<Self>) {
        // Apply input filter if present
        let filtered_text: String = if let Some(ref filter) = self.input_filter {
            text.chars().filter(|c| filter(*c)).collect()
        } else {
            text.to_string()
        };

        if filtered_text.is_empty() {
            return;
        }

        self.core.insert_text(&filtered_text);
        self.reset_cursor_blink();
        cx.emit(TextInputEvent::Change);
        cx.notify();
    }
}

impl Render for TextInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let theme = get_theme_or(cx, self.custom_theme.as_ref());
        let focus_handle = self.focus_handle.clone();
        let is_focused = self.focus_handle.is_focused(window);
        let display_content = self.display_content();
        let placeholder = self.placeholder.clone();
        let has_content = !self.core.content().is_empty();

        // Set up focus-out subscription on first render
        if !self.focus_out_subscribed {
            self.focus_out_subscribed = true;
            let focus_handle = self.focus_handle.clone();
            cx.on_focus_out(&focus_handle, window, |this: &mut Self, _event, _window, cx| {
                this.on_blur(cx);
            }).detach();
        }

        // Detect focus-in
        if is_focused && !self.was_focused {
            self.on_focus(cx);
        }
        self.was_focused = is_focused;

        // Use actual scroll_offset when focused, 0 when unfocused (to show beginning of text)
        let render_scroll_offset = if is_focused { self.scroll_offset } else { 0.0 };

        // Set up blink timer when focused
        if is_focused && !self.blink_timer_active {
            self.blink_timer_active = true;
            let entity = cx.entity();
            let blink_period = CursorBlink::blink_period();
            window.spawn(cx, async move |async_cx| {
                loop {
                    smol::Timer::after(blink_period).await;
                    let should_continue = async_cx
                        .update_entity(&entity, |this, cx| {
                            if !this.blink_timer_active {
                                return false;
                            }
                            cx.notify();
                            true
                        })
                        .unwrap_or(false);
                    if !should_continue {
                        break;
                    }
                }
            }).detach();
        }

        if !is_focused {
            self.blink_timer_active = false;
        }

        // Apply auto-scroll if active
        if self.auto_scroll_active && self.is_dragging {
            self.apply_auto_scroll(window);
        }

        if is_focused {
            self.ensure_cursor_visible(window);
        }

        // Re-capture cursor and selection after potential auto-scroll modification
        let cursor = self.core.cursor();
        let selection = self.core.selection();

        let cursor_x = self.x_for_cursor(cursor, window) - render_scroll_offset;
        let cursor_visible = is_focused && self.cursor_blink.is_visible();

        // Only show selection when focused
        let selection_bounds: Option<(f32, f32)> = if is_focused {
            selection.and_then(|(start, end)| {
                if start != end {
                    let start_x = self.x_for_cursor(start, window) - render_scroll_offset;
                    let end_x = self.x_for_cursor(end, window) - render_scroll_offset;
                    Some((start_x, end_x - start_x))
                } else {
                    None
                }
            })
        } else {
            None
        };

        let scroll_offset = render_scroll_offset;
        let selection_color = theme.selection;
        let text_color = theme.text_primary;
        let text_placeholder = theme.text_placeholder;
        let border_focus = theme.border_focus;
        let border_input = theme.border_input;
        let bg_input = theme.bg_input;

        div()
            .id("ccf_text_input")
            .key_context("CcfTextInput")
            .track_focus(&focus_handle)
            .tab_stop(true)
            // Navigation actions
            .on_action(cx.listener(|this, _: &MoveLeft, _window, cx| {
                this.handle_move_left(cx);
            }))
            .on_action(cx.listener(|this, _: &MoveRight, _window, cx| {
                this.handle_move_right(cx);
            }))
            .on_action(cx.listener(|this, _: &MoveWordLeft, _window, cx| {
                this.handle_move_word_left(cx);
            }))
            .on_action(cx.listener(|this, _: &MoveWordRight, _window, cx| {
                this.handle_move_word_right(cx);
            }))
            .on_action(cx.listener(|this, _: &MoveToStart, _window, cx| {
                this.handle_move_to_start(cx);
            }))
            .on_action(cx.listener(|this, _: &MoveToEnd, _window, cx| {
                this.handle_move_to_end(cx);
            }))
            // Selection actions
            .on_action(cx.listener(|this, _: &SelectLeft, _window, cx| {
                this.handle_select_left(cx);
            }))
            .on_action(cx.listener(|this, _: &SelectRight, _window, cx| {
                this.handle_select_right(cx);
            }))
            .on_action(cx.listener(|this, _: &SelectWordLeft, _window, cx| {
                this.handle_select_word_left(cx);
            }))
            .on_action(cx.listener(|this, _: &SelectWordRight, _window, cx| {
                this.handle_select_word_right(cx);
            }))
            .on_action(cx.listener(|this, _: &SelectToStart, _window, cx| {
                this.handle_select_to_start(cx);
            }))
            .on_action(cx.listener(|this, _: &SelectToEnd, _window, cx| {
                this.handle_select_to_end(cx);
            }))
            .on_action(cx.listener(|this, _: &SelectAll, _window, cx| {
                this.handle_select_all(cx);
            }))
            // Delete actions
            .on_action(cx.listener(|this, _: &DeleteBackward, _window, cx| {
                this.handle_delete_backward(cx);
            }))
            .on_action(cx.listener(|this, _: &DeleteForward, _window, cx| {
                this.handle_delete_forward(cx);
            }))
            .on_action(cx.listener(|this, _: &DeleteWordBackward, _window, cx| {
                this.handle_delete_word_backward(cx);
            }))
            .on_action(cx.listener(|this, _: &DeleteWordForward, _window, cx| {
                this.handle_delete_word_forward(cx);
            }))
            // Clipboard actions
            .on_action(cx.listener(|this, _: &Cut, _window, cx| {
                this.cut(cx);
            }))
            .on_action(cx.listener(|this, _: &Copy, _window, cx| {
                this.copy(cx);
            }))
            .on_action(cx.listener(|this, _: &Paste, _window, cx| {
                this.paste(cx);
            }))
            // Enter/Escape
            .on_action(cx.listener(|_this, _: &Enter, _window, cx| {
                cx.emit(TextInputEvent::Enter);
            }))
            .on_action(cx.listener(|_this, _: &Escape, _window, cx| {
                cx.emit(TextInputEvent::Escape);
            }))
            // Focus navigation (Tab / Shift+Tab)
            .on_action(cx.listener(|_this, _: &FocusNext, window, _cx| {
                window.focus_next();
            }))
            .on_action(cx.listener(|_this, _: &FocusPrev, window, _cx| {
                window.focus_prev();
            }))
            // Character input (Tab handled separately for focus navigation)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                // Handle Tab for focus navigation
                if event.keystroke.key == "tab" {
                    if this.emit_tab_events {
                        // Emit event for parent to handle
                        if event.keystroke.modifiers.shift {
                            cx.emit(TextInputEvent::ShiftTab);
                        } else {
                            cx.emit(TextInputEvent::Tab);
                        }
                    } else {
                        // Handle focus navigation directly
                        if event.keystroke.modifiers.shift {
                            window.focus_prev();
                        } else {
                            window.focus_next();
                        }
                    }
                    return;
                }
                if !event.keystroke.modifiers.alt
                    && !event.keystroke.modifiers.control
                    && !event.keystroke.modifiers.platform
                {
                    if let Some(ref ch) = event.keystroke.key_char {
                        this.handle_insert_text(ch, cx);
                    }
                }
            }))
            // Click to focus and position cursor, start drag
            .on_mouse_down(MouseButton::Left, cx.listener(|this, event: &MouseDownEvent, window, cx| {
                let was_focused = this.focus_handle.is_focused(window);
                this.focus_handle.focus(window);

                // If clicking to restore focus and there's a selection to restore,
                // just restore focus without changing cursor/selection
                if !was_focused && this.core.selection().is_some() {
                    this.reset_cursor_blink();
                    cx.notify();
                    return;
                }

                let click_x: f32 = event.position.x.into();
                let relative_x = (click_x - this.content_origin_x).max(0.0);
                let new_cursor = this.cursor_at_x(relative_x, window);

                if event.modifiers.shift {
                    // Shift+click extends selection
                    this.core.start_selection_from_cursor();
                    this.core.extend_selection_to(new_cursor);
                } else {
                    // Regular click starts a new selection
                    this.core.clear_selection();
                    this.core.set_cursor(new_cursor);
                    // Set anchor for potential drag selection
                    this.core.start_selection_from_cursor();
                }

                // Start drag operation
                this.is_dragging = true;
                this.reset_cursor_blink();
                cx.notify();
            }))
            // Drag to select text
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, window, cx| {
                if !this.is_dragging {
                    return;
                }

                let mouse_x: f32 = event.position.x.into();
                let scroll_speed = this.handle_drag_move(mouse_x, window);
                this.spawn_auto_scroll_timer_if_needed(scroll_speed, window, cx);
                cx.notify();
            }))
            // Mouse up ends drag
            .on_mouse_up(MouseButton::Left, cx.listener(|this, _event: &MouseUpEvent, _window, cx| {
                this.stop_drag();
                cx.notify();
            }))
            // Styling
            .w_full()
            .h(px(28.))
            .px_2()
            // Only apply border/background/rounded corners when not borderless
            .when(!self.borderless, |d| {
                d.border_1()
                    .border_color(if is_focused { rgb(border_focus) } else { rgb(border_input) })
                    .rounded_md()
                    .bg(rgb(bg_input))
            })
            .cursor_text()
            .relative()
            .overflow_hidden()
            .child({
                let entity = cx.entity();
                let entity_paint = entity.clone();
                let is_dragging = self.is_dragging;

                div()
                    .size_full()
                    .flex()
                    .items_center()
                    .relative()
                    // Measurement canvas and window-level drag handlers
                    .child(
                        canvas(
                            move |bounds, _window, cx| {
                                let width: f32 = bounds.size.width.into();
                                let origin_x: f32 = bounds.origin.x.into();
                                // Update measurement values without triggering re-render
                                // to avoid potential render loops when used inside other widgets
                                entity.update(cx, |this: &mut TextInput, _cx| {
                                    this.visible_width = width;
                                    this.content_origin_x = origin_x;
                                });
                            },
                            {
                                let entity = entity_paint;
                                move |_bounds, _, window, _cx| {
                                    // Register window-level mouse handlers when dragging
                                    // This captures mouse events even outside the element bounds
                                    if is_dragging {
                                        // Mouse move handler for drag selection
                                        let entity_move = entity.clone();
                                        window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                                            if phase != DispatchPhase::Capture {
                                                return;
                                            }
                                            let mouse_x: f32 = event.position.x.into();
                                            entity_move.update(cx, |this: &mut TextInput, cx| {
                                                let scroll_speed = this.handle_drag_move(mouse_x, window);
                                                this.spawn_auto_scroll_timer_if_needed(scroll_speed, window, cx);
                                                cx.notify();
                                            });
                                        });

                                        // Mouse up handler to end drag
                                        let entity_up = entity.clone();
                                        window.on_mouse_event(move |_event: &MouseUpEvent, phase, _window, cx| {
                                            if phase != DispatchPhase::Capture {
                                                return;
                                            }
                                            entity_up.update(cx, |this: &mut TextInput, cx| {
                                                this.stop_drag();
                                                cx.notify();
                                            });
                                        });
                                    }
                                }
                            },
                        )
                        .size_full()
                        .absolute()
                    )
                    // Content layer
                    .child(
                        div()
                            .relative()
                            .h_full()
                            .flex()
                            .items_center()
                            .min_w_0()
                            // Selection highlight
                            .when_some(selection_bounds, |d, (start_x, width)| {
                                d.child(
                                    div()
                                        .absolute()
                                        .top_0()
                                        .bottom_0()
                                        .left(px(start_x))
                                        .w(px(width))
                                        .bg(rgb(selection_color))
                                )
                            })
                            // Text content
                            .child(
                                div()
                                    .absolute()
                                    .left(px(-scroll_offset))
                                    .text_sm()
                                    .text_color(rgb(text_color))
                                    .whitespace_nowrap()
                                    .child(display_content.clone())
                            )
                            // Cursor
                            .when(cursor_visible, |d| {
                                d.child(
                                    div()
                                        .absolute()
                                        .top(px(4.))
                                        .bottom(px(4.))
                                        .left(px(cursor_x))
                                        .w(px(1.))
                                        .bg(rgb(text_color))
                                )
                            })
                    )
                    // Placeholder
                    .when(!has_content, |d| {
                        if let Some(ph) = placeholder {
                            d.child(
                                div()
                                    .absolute()
                                    .left_0()
                                    .text_sm()
                                    .text_color(rgb(text_placeholder))
                                    .child(ph)
                            )
                        } else {
                            d
                        }
                    })
            })
    }
}
