//! Password input widget with visibility toggle
//!
//! A secure text input that masks its content with bullet characters and provides
//! a button to toggle password visibility. When the `secure-password` feature is
//! enabled, uses `SensitiveString` internally for automatic memory zeroization
//! and exposes `SecretString` at API boundaries.
//!
//! # Example
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::{PasswordInput, PasswordInputEvent};
//!
//! let password_input = cx.new(|cx| {
//!     PasswordInput::new(cx)
//!         .placeholder("Enter password")
//! });
//!
//! // Subscribe to events
//! cx.subscribe(&password_input, |this, _, event: &PasswordInputEvent, cx| {
//!     match event {
//!         PasswordInputEvent::Change(secret) => {
//!             // Use secret.expose_secret() to access the password
//!             println!("Password changed");
//!         }
//!         PasswordInputEvent::Enter => println!("Enter pressed"),
//!         PasswordInputEvent::Blur => println!("Focus lost"),
//!     }
//! }).detach();
//! ```

use std::time::{Duration, Instant};

use gpui::prelude::*;
use gpui::*;

#[cfg(feature = "secure-password")]
use secrecy::SecretString;

use crate::theme::{get_theme_or, Theme};
use super::editing_core::EditingCore;
use super::focus_navigation::{FocusNext, FocusPrev};
use super::text_input::{
    MoveLeft, MoveRight, MoveWordLeft, MoveWordRight, MoveToStart, MoveToEnd,
    SelectLeft, SelectRight, SelectWordLeft, SelectWordRight, SelectToStart, SelectToEnd, SelectAll,
    DeleteBackward, DeleteForward, DeleteWordBackward, DeleteWordForward,
    Cut, Copy, Paste, Enter, Escape,
};

#[cfg(feature = "secure-password")]
use super::sensitive_string::SensitiveString;

/// Events emitted by PasswordInput
#[derive(Debug, Clone)]
pub enum PasswordInputEvent {
    /// Password value changed
    #[cfg(feature = "secure-password")]
    Change(SecretString),
    /// Password value changed (without secure-password feature)
    #[cfg(not(feature = "secure-password"))]
    Change(String),
    /// Enter key was pressed
    Enter,
    /// Input lost focus (including Escape key)
    Blur,
}

/// Character used to mask password input
const MASK_CHAR: &str = "\u{25CF}"; // ● Black circle

/// Password input widget with visibility toggle
///
/// This widget provides a secure password entry field with:
/// - Masked display by default (bullet characters)
/// - Toggle button to show/hide the actual password
/// - Full cursor/selection support
/// - When `secure-password` feature is enabled:
///   - Automatic memory zeroization when dropped
///   - `SecretString` at API boundaries
///   - Redacted Debug output
pub struct PasswordInput {
    /// Core editing logic with secure storage
    #[cfg(feature = "secure-password")]
    core: EditingCore<SensitiveString>,
    #[cfg(not(feature = "secure-password"))]
    core: EditingCore<String>,
    /// Focus handle for the text input area
    input_focus_handle: FocusHandle,
    /// Focus handle for the toggle button
    toggle_focus_handle: FocusHandle,
    /// Whether password is currently visible
    show_password: bool,
    /// Placeholder text
    placeholder: Option<SharedString>,
    /// Optional custom theme
    custom_theme: Option<Theme>,
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
    /// Time when cursor was last moved (for blink reset)
    cursor_last_moved: Instant,
    /// Whether blink timer is set up
    blink_timer_active: bool,
    /// Whether currently dragging to select text
    is_dragging: bool,
    /// Whether auto-scroll timer is active
    auto_scroll_active: bool,
    /// Current auto-scroll speed
    auto_scroll_speed: f32,
    /// Pending placeholder from builder
    pending_placeholder: Option<SharedString>,
    /// Pending value from builder
    pending_value: Option<String>,
}

impl EventEmitter<PasswordInputEvent> for PasswordInput {}

impl Focusable for PasswordInput {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.input_focus_handle.clone()
    }
}

impl PasswordInput {
    /// Create a new password input
    pub fn new(cx: &mut Context<Self>) -> Self {
        #[cfg(feature = "secure-password")]
        let core = EditingCore::<SensitiveString>::new().with_masked(true);
        #[cfg(not(feature = "secure-password"))]
        let core = EditingCore::<String>::new().with_masked(true);

        Self {
            core,
            input_focus_handle: cx.focus_handle().tab_stop(true),
            toggle_focus_handle: cx.focus_handle().tab_stop(true),
            show_password: false,
            placeholder: None,
            custom_theme: None,
            scroll_offset: 0.0,
            visible_width: 200.0,
            content_origin_x: 0.0,
            was_focused: false,
            focus_out_subscribed: false,
            cursor_last_moved: Instant::now(),
            blink_timer_active: false,
            is_dragging: false,
            auto_scroll_active: false,
            auto_scroll_speed: 0.0,
            pending_placeholder: None,
            pending_value: None,
        }
    }

    /// Set placeholder text shown when empty (builder pattern)
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.pending_placeholder = Some(text.into());
        self
    }

    /// Set an initial value (builder pattern)
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.pending_value = Some(value.into());
        self
    }

    /// Set a custom theme for this widget (builder pattern)
    pub fn theme(mut self, theme: Theme) -> Self {
        self.custom_theme = Some(theme);
        self
    }

    /// Apply any pending builder values (called on first render)
    fn apply_pending(&mut self) {
        if let Some(placeholder) = self.pending_placeholder.take() {
            self.placeholder = Some(placeholder);
        }
        if let Some(value) = self.pending_value.take() {
            self.core.set_content(&value);
        }
    }

    /// Get the current password value as a SecretString
    #[cfg(feature = "secure-password")]
    pub fn value(&self, _cx: &App) -> SecretString {
        // Access the sensitive string through the core and convert to SecretString
        // The core stores SensitiveString which has to_secret_string()
        // We need to get at the underlying storage
        self.create_secret_from_content()
    }

    /// Get the current password value
    #[cfg(not(feature = "secure-password"))]
    pub fn value<'a>(&'a self, _cx: &'a App) -> &'a str {
        self.core.content()
    }

    /// Create a SecretString from the current content
    #[cfg(feature = "secure-password")]
    fn create_secret_from_content(&self) -> SecretString {
        SecretString::from(self.core.content().to_string())
    }

    /// Set the password value programmatically
    pub fn set_value(&mut self, value: &str, cx: &mut Context<Self>) {
        self.core.set_content(value);
        self.scroll_offset = 0.0;
        self.emit_change(cx);
        cx.notify();
    }

    /// Set the password value from a SecretString
    #[cfg(feature = "secure-password")]
    pub fn set_value_secret(&mut self, secret: &SecretString, cx: &mut Context<Self>) {
        use secrecy::ExposeSecret;
        self.core.set_content(secret.expose_secret());
        self.scroll_offset = 0.0;
        self.emit_change(cx);
        cx.notify();
    }

    /// Get the focus handle for this input
    pub fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.input_focus_handle.clone()
    }

    fn emit_change(&self, cx: &mut Context<Self>) {
        #[cfg(feature = "secure-password")]
        cx.emit(PasswordInputEvent::Change(self.create_secret_from_content()));
        #[cfg(not(feature = "secure-password"))]
        cx.emit(PasswordInputEvent::Change(self.core.content().to_string()));
    }

    fn toggle_visibility(&mut self, cx: &mut Context<Self>) {
        self.show_password = !self.show_password;
        self.core.set_masked(!self.show_password);
        cx.notify();
    }

    #[allow(dead_code)]
    fn get_theme(&self, cx: &App) -> Theme {
        self.custom_theme.unwrap_or_else(|| crate::theme::get_theme(cx))
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
        self.core.content()
            .char_indices()
            .nth(char_index)
            .map(|(i, _)| i)
            .unwrap_or(self.core.content().len())
    }

    fn reset_cursor_blink(&mut self) {
        self.cursor_last_moved = Instant::now();
    }

    fn is_cursor_visible(&self) -> bool {
        let elapsed = self.cursor_last_moved.elapsed();
        let blink_period = Duration::from_millis(530);
        let cycle_position = elapsed.as_millis() % (blink_period.as_millis() * 2);
        cycle_position < blink_period.as_millis()
    }

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

    fn handle_drag_move(&mut self, mouse_x: f32, window: &Window) -> f32 {
        if !self.is_dragging {
            return 0.0;
        }

        let relative_x = mouse_x - self.content_origin_x;
        let padding = 2.0;
        let actual_visible = self.visible_width - padding;

        let scroll_speed = if relative_x < 0.0 {
            -self.calculate_scroll_speed(-relative_x)
        } else if relative_x > actual_visible {
            self.calculate_scroll_speed(relative_x - actual_visible)
        } else {
            0.0
        };

        let clamped_x = relative_x.clamp(0.0, actual_visible);
        let new_cursor = self.cursor_at_x(clamped_x, window);
        self.core.extend_selection_to(new_cursor);
        self.reset_cursor_blink();

        scroll_speed
    }

    fn calculate_scroll_speed(&self, distance: f32) -> f32 {
        let base_speed = 0.5;
        let max_speed = 20.0;
        let max_distance = 100.0;

        let normalized = (distance / max_distance).min(1.0);
        let eased = 1.0 - (1.0 - normalized).powi(2);
        base_speed + eased * (max_speed - base_speed)
    }

    fn apply_auto_scroll(&mut self, window: &Window) {
        if self.auto_scroll_speed == 0.0 {
            return;
        }

        let content_width = self.x_for_cursor(self.core.content().len(), window);
        let padding = 2.0;
        let actual_visible = self.visible_width - padding;
        let max_scroll = (content_width - actual_visible).max(0.0);

        self.scroll_offset = (self.scroll_offset + self.auto_scroll_speed).clamp(0.0, max_scroll);

        if self.auto_scroll_speed < 0.0 {
            let new_cursor = self.cursor_at_x(0.0, window);
            self.core.extend_selection_to(new_cursor);
        } else {
            let new_cursor = self.cursor_at_x(actual_visible, window);
            self.core.extend_selection_to(new_cursor);
        }
    }

    fn stop_drag(&mut self) {
        self.is_dragging = false;
        self.auto_scroll_active = false;
        self.auto_scroll_speed = 0.0;
    }

    fn spawn_auto_scroll_timer_if_needed(&mut self, scroll_speed: f32, window: &mut Window, cx: &mut Context<Self>) {
        self.auto_scroll_speed = scroll_speed;
        if scroll_speed != 0.0 && !self.auto_scroll_active {
            self.auto_scroll_active = true;
            let entity = cx.entity();
            window.spawn(cx, async move |async_cx| {
                loop {
                    smol::Timer::after(Duration::from_millis(32)).await;
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

    fn on_focus(&mut self, cx: &mut Context<Self>) {
        self.reset_cursor_blink();
        cx.notify();
    }

    fn on_blur(&mut self, cx: &mut Context<Self>) {
        self.stop_drag();
        cx.emit(PasswordInputEvent::Blur);
        cx.notify();
    }

    // Clipboard operations - password content should NEVER be copied
    fn cut(&mut self, cx: &mut Context<Self>) {
        // Delete selection but don't copy to clipboard (security)
        if self.core.delete_selection() {
            self.reset_cursor_blink();
            self.emit_change(cx);
            cx.notify();
        }
    }

    fn paste(&mut self, cx: &mut Context<Self>) {
        if let Some(clipboard) = cx.read_from_clipboard() {
            if let Some(text) = clipboard.text() {
                let clean_text = text.replace(['\n', '\r'], "");
                self.core.insert_text(&clean_text);
                self.reset_cursor_blink();
                self.emit_change(cx);
                cx.notify();
            }
        }
    }

    // Action handlers
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
            self.emit_change(cx);
        }
        cx.notify();
    }

    fn handle_delete_forward(&mut self, cx: &mut Context<Self>) {
        if self.core.delete_forward() {
            self.reset_cursor_blink();
            self.emit_change(cx);
        }
        cx.notify();
    }

    fn handle_delete_word_backward(&mut self, cx: &mut Context<Self>) {
        if self.core.delete_word_backward() {
            self.reset_cursor_blink();
            self.emit_change(cx);
        }
        cx.notify();
    }

    fn handle_delete_word_forward(&mut self, cx: &mut Context<Self>) {
        if self.core.delete_word_forward() {
            self.reset_cursor_blink();
            self.emit_change(cx);
        }
        cx.notify();
    }

    fn handle_insert_text(&mut self, text: &str, cx: &mut Context<Self>) {
        self.core.insert_text(text);
        self.reset_cursor_blink();
        self.emit_change(cx);
        cx.notify();
    }
}

impl Render for PasswordInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        // Apply any pending builder values on first render
        self.apply_pending();

        let theme = get_theme_or(cx, self.custom_theme.as_ref());
        let input_focus_handle = self.input_focus_handle.clone();
        let toggle_focus_handle = self.toggle_focus_handle.clone();
        let input_is_focused = self.input_focus_handle.is_focused(window);
        let toggle_is_focused = self.toggle_focus_handle.is_focused(window);
        let either_focused = input_is_focused || toggle_is_focused;

        // Set up focus-out subscription on first render
        if !self.focus_out_subscribed {
            self.focus_out_subscribed = true;
            let input_fh = self.input_focus_handle.clone();
            cx.on_focus_out(&input_fh, window, |this: &mut Self, _event, window, cx| {
                // Only blur if neither input nor toggle has focus
                if !this.input_focus_handle.is_focused(window) && !this.toggle_focus_handle.is_focused(window) {
                    this.on_blur(cx);
                }
            }).detach();
        }

        // Detect focus-in
        if input_is_focused && !self.was_focused {
            self.on_focus(cx);
        }
        self.was_focused = input_is_focused;

        let render_scroll_offset = if input_is_focused { self.scroll_offset } else { 0.0 };

        // Set up blink timer when focused
        if input_is_focused && !self.blink_timer_active {
            self.blink_timer_active = true;
            let entity = cx.entity();
            window.spawn(cx, async move |async_cx| {
                loop {
                    smol::Timer::after(Duration::from_millis(530)).await;
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

        if !input_is_focused {
            self.blink_timer_active = false;
        }

        // Apply auto-scroll if active
        if self.auto_scroll_active && self.is_dragging {
            self.apply_auto_scroll(window);
        }

        if input_is_focused {
            self.ensure_cursor_visible(window);
        }

        let display_content = self.display_content();
        let has_content = !self.core.content().is_empty();
        let placeholder = self.placeholder.clone();

        let cursor = self.core.cursor();
        let selection = self.core.selection();

        let cursor_x = self.x_for_cursor(cursor, window) - render_scroll_offset;
        let cursor_visible = input_is_focused && self.is_cursor_visible();

        let selection_bounds: Option<(f32, f32)> = if input_is_focused {
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

        // Colors
        let bg_color = theme.bg_input;
        let border_color = if either_focused {
            theme.border_focus
        } else {
            theme.border_input
        };
        let separator_color = theme.text_muted;
        let button_text_color = theme.text_muted;
        let selection_color = theme.selection;
        let text_color = theme.text_primary;
        let text_placeholder = theme.text_placeholder;

        // Eye icons
        let eye_icon = if self.show_password { "\u{2296}" } else { "\u{25CE}" }; // ⊖ / ◎

        // Vertical separator
        let separator = div()
            .w(px(1.0))
            .h_full()
            .bg(rgb(separator_color));

        // Main container
        div()
            .id("ccf_password_input")
            .flex()
            .flex_row()
            .items_center()
            .h(px(28.0))
            .bg(rgb(bg_color))
            .border_1()
            .border_color(rgb(border_color))
            .rounded_md()
            .overflow_hidden()
            // Input area
            .child(
                div()
                    .id("ccf_password_input_field")
                    .key_context("CcfTextInput")
                    .track_focus(&input_focus_handle)
                    .tab_stop(true)
                    .flex_1()
                    .h_full()
                    .px_2()
                    .cursor_text()
                    .relative()
                    .overflow_hidden()
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
                    // Clipboard actions - Copy is disabled for security
                    .on_action(cx.listener(|this, _: &Cut, _window, cx| {
                        this.cut(cx);
                    }))
                    .on_action(cx.listener(|_this, _: &Copy, _window, _cx| {
                        // Intentionally empty - don't allow copying password
                    }))
                    .on_action(cx.listener(|this, _: &Paste, _window, cx| {
                        this.paste(cx);
                    }))
                    // Enter/Escape
                    .on_action(cx.listener(|_this, _: &Enter, _window, cx| {
                        cx.emit(PasswordInputEvent::Enter);
                    }))
                    .on_action(cx.listener(|this, _: &Escape, _window, cx| {
                        this.on_blur(cx);
                    }))
                    // Focus navigation
                    .on_action(cx.listener(|_this, _: &FocusNext, window, _cx| {
                        window.focus_next();
                    }))
                    .on_action(cx.listener(|_this, _: &FocusPrev, window, _cx| {
                        window.focus_prev();
                    }))
                    // Character input
                    .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                        if event.keystroke.key == "tab" {
                            if event.keystroke.modifiers.shift {
                                window.focus_prev();
                            } else {
                                window.focus_next();
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
                    // Click to focus and position cursor
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, event: &MouseDownEvent, window, cx| {
                        let was_focused = this.input_focus_handle.is_focused(window);
                        this.input_focus_handle.focus(window);

                        if !was_focused && this.core.selection().is_some() {
                            this.reset_cursor_blink();
                            cx.notify();
                            return;
                        }

                        let click_x: f32 = event.position.x.into();
                        let relative_x = (click_x - this.content_origin_x).max(0.0);
                        let new_cursor = this.cursor_at_x(relative_x, window);

                        if event.modifiers.shift {
                            this.core.start_selection_from_cursor();
                            this.core.extend_selection_to(new_cursor);
                        } else {
                            this.core.clear_selection();
                            this.core.set_cursor(new_cursor);
                            this.core.start_selection_from_cursor();
                        }

                        this.is_dragging = true;
                        this.reset_cursor_blink();
                        cx.notify();
                    }))
                    // Drag to select
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
                    // Content
                    .child({
                        let entity = cx.entity();
                        let entity_paint = entity.clone();
                        let is_dragging = self.is_dragging;

                        div()
                            .size_full()
                            .flex()
                            .items_center()
                            .relative()
                            // Measurement canvas
                            .child(
                                canvas(
                                    move |bounds, _window, cx| {
                                        let width: f32 = bounds.size.width.into();
                                        let origin_x: f32 = bounds.origin.x.into();
                                        entity.update(cx, |this: &mut PasswordInput, _cx| {
                                            this.visible_width = width;
                                            this.content_origin_x = origin_x;
                                        });
                                    },
                                    {
                                        let entity = entity_paint;
                                        move |_bounds, _, window, _cx| {
                                            if is_dragging {
                                                let entity_move = entity.clone();
                                                window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                                                    if phase != DispatchPhase::Capture {
                                                        return;
                                                    }
                                                    let mouse_x: f32 = event.position.x.into();
                                                    entity_move.update(cx, |this: &mut PasswordInput, cx| {
                                                        let scroll_speed = this.handle_drag_move(mouse_x, window);
                                                        this.spawn_auto_scroll_timer_if_needed(scroll_speed, window, cx);
                                                        cx.notify();
                                                    });
                                                });

                                                let entity_up = entity.clone();
                                                window.on_mouse_event(move |_event: &MouseUpEvent, phase, _window, cx| {
                                                    if phase != DispatchPhase::Capture {
                                                        return;
                                                    }
                                                    entity_up.update(cx, |this: &mut PasswordInput, cx| {
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
            )
            .child(separator)
            // Toggle visibility button
            .child(
                div()
                    .id("password_toggle_button")
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(28.0))
                    .h_full()
                    .cursor_pointer()
                    .text_color(rgb(button_text_color))
                    .when(toggle_is_focused, |d| d.bg(rgb(theme.bg_hover)))
                    .hover(|d| d.bg(rgb(theme.bg_hover)))
                    .track_focus(&toggle_focus_handle)
                    .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                        let key = event.keystroke.key.as_str();
                        match key {
                            "enter" | "space" => {
                                this.toggle_visibility(cx);
                            }
                            "tab" => {
                                if event.keystroke.modifiers.shift {
                                    window.focus_prev();
                                } else {
                                    window.focus_next();
                                }
                            }
                            _ => {}
                        }
                    }))
                    .on_click(cx.listener(|this, _event, _window, cx| {
                        this.toggle_visibility(cx);
                    }))
                    .child(
                        div()
                            .text_sm()
                            .child(eye_icon)
                    )
            )
    }
}
