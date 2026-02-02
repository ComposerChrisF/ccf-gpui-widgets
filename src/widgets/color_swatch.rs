//! Color swatch widget
//!
//! A color preview with hex input and color picker. Displays a colored square alongside
//! a text input for the hex color value. Clicking the swatch opens a full color picker
//! with RGB/HSL sliders and a 2D saturation/lightness selector.
//!
//! # Features
//!
//! - Hex color input (#RGB, #RRGGBB, #RRGGBBAA)
//! - CSS named color support (140 colors: "red", "coral", "darkblue", etc.)
//! - Color picker popup with RGB and HSL modes
//! - 2D saturation/lightness selector canvas
//! - Hue rainbow slider
//! - Optional alpha channel support
//! - Old/new color preview
//!
//! # Example
//!
//! ```ignore
//! use ccf_gpui_widgets::widgets::ColorSwatch;
//!
//! let swatch = cx.new(|cx| {
//!     ColorSwatch::new(cx)
//!         .value("#3b82f6")
//!         .with_alpha(true)
//! });
//!
//! // Subscribe to changes
//! cx.subscribe(&swatch, |this, _swatch, event: &ColorSwatchEvent, cx| {
//!     if let ColorSwatchEvent::Change(hex) = event {
//!         println!("Color: {}", hex);
//!     }
//! }).detach();
//! ```

use std::cell::Cell;
use std::rc::Rc;

use gpui::prelude::*;
use gpui::*;

use crate::theme::{get_theme_or, Theme};
use crate::utils::color::{Rgb, Hsl, Hsv, parse_color, parse_color_alpha};
use super::text_input::{TextInput, TextInputEvent};
use super::focus_navigation::{FocusNext, FocusPrev};

// Actions for keyboard navigation
actions!(ccf_color_swatch, [ClosePicker]);

/// Register key bindings for color swatch components
///
/// Call this once at application startup:
/// ```ignore
/// ccf_gpui_widgets::widgets::color_swatch::register_keybindings(cx);
/// ```
pub fn register_keybindings(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("escape", ClosePicker, Some("CcfColorPicker")),
    ]);
}

/// Drag state for saturation/lightness canvas
#[derive(Clone)]
struct SlDrag {
    canvas_origin: Rc<Cell<Point<Pixels>>>,
    canvas_width: f32,
    canvas_height: f32,
}

/// Drag state for hue slider
#[derive(Clone)]
struct HueDrag {
    origin: Rc<Cell<f32>>,
    width: Rc<Cell<f32>>,
}

/// Drag state for alpha slider
#[derive(Clone)]
struct AlphaDrag {
    origin: Rc<Cell<f32>>,
    width: Rc<Cell<f32>>,
}

/// Drag state for component sliders (R, G, B, S, L)
#[derive(Clone)]
struct ComponentDrag {
    origin: Rc<Cell<f32>>,
    slider_width: f32,
    handle_visual_width: f32,
    max_value: f32,
    update_fn: fn(&mut ColorSwatch, f32, &mut Context<ColorSwatch>),
}

/// Empty view for drag visualization (we don't need a visual)
struct EmptyDragView;

impl Render for EmptyDragView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_0()
    }
}

/// Events emitted by ColorSwatch
#[derive(Clone, Debug)]
pub enum ColorSwatchEvent {
    /// Color value changed
    Change(String),
}

/// Color picker mode (RGB or HSL sliders)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickerMode {
    Rgb,
    Hsl,
}

/// Color swatch widget with hex input and color picker
pub struct ColorSwatch {
    /// Current color value as hex (#RRGGBB or #RRGGBBAA)
    value: String,
    /// Placeholder text
    placeholder: String,
    /// Whether alpha channel is enabled
    with_alpha: bool,
    /// Whether the widget is enabled
    enabled: bool,
    /// Custom theme
    custom_theme: Option<Theme>,
    /// Focus handle (for focus navigation, not key capture)
    focus_handle: FocusHandle,
    /// Focus handle for the picker popup (for ESC key handling)
    picker_focus_handle: FocusHandle,
    /// Text input for hex editing
    hex_input: Entity<TextInput>,
    /// Whether picker popup is open
    is_picker_open: bool,
    /// Current RGB values
    current_rgb: Rgb,
    /// Current HSL values (used for H slider and conversion)
    current_hsl: Hsl,
    /// Current HSV values (used for the S/V canvas)
    current_hsv: Hsv,
    /// Current alpha value (0-255)
    current_alpha: u8,
    /// Original color when picker opened (for comparison)
    original_value: String,
    /// Whether the text input needs to be synced with value
    needs_input_sync: bool,
    /// Whether the current input is valid
    input_is_valid: bool,
    /// Measured hue slider width (persists between frames)
    hue_slider_width: Rc<Cell<f32>>,
    /// Measured hue slider origin (persists between frames)
    hue_slider_origin: Rc<Cell<f32>>,
    /// Measured alpha slider width (persists between frames)
    alpha_slider_width: Rc<Cell<f32>>,
    /// Measured alpha slider origin (persists between frames)
    alpha_slider_origin: Rc<Cell<f32>>,
}

impl EventEmitter<ColorSwatchEvent> for ColorSwatch {}

impl Focusable for ColorSwatch {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl ColorSwatch {
    /// Create a new color swatch
    pub fn new(cx: &mut Context<Self>) -> Self {
        let hex_input = cx.new(|cx| {
            TextInput::new(cx)
                .placeholder("#000000")
                .with_value("#000000")
        });

        // Subscribe to text input events
        cx.subscribe(&hex_input, |this, input, event: &TextInputEvent, cx| {
            match event {
                TextInputEvent::Change => {
                    // Get the current input value and update preview
                    let value = input.read(cx).content().to_string();
                    this.handle_input_change(&value, cx);
                }
                TextInputEvent::Enter | TextInputEvent::Blur => {
                    // Try to parse as named color or hex on Enter/Blur
                    let value = input.read(cx).content().to_string();
                    this.handle_input_commit(&value, cx);
                }
                _ => {}
            }
        }).detach();

        Self {
            value: "#000000".to_string(),
            placeholder: "#000000".to_string(),
            with_alpha: false,
            enabled: true,
            custom_theme: None,
            focus_handle: cx.focus_handle(),
            picker_focus_handle: cx.focus_handle(),
            hex_input,
            is_picker_open: false,
            current_rgb: Rgb::new(0, 0, 0),
            current_hsl: Hsl::new(0.0, 0.0, 0.0),
            current_hsv: Hsv::new(0.0, 0.0, 0.0),
            current_alpha: 255,
            original_value: "#000000".to_string(),
            needs_input_sync: false,
            input_is_valid: true,
            // Initial estimates, will be updated by prepaint
            hue_slider_width: Rc::new(Cell::new(200.0)),
            hue_slider_origin: Rc::new(Cell::new(0.0)),
            alpha_slider_width: Rc::new(Cell::new(200.0)),
            alpha_slider_origin: Rc::new(Cell::new(0.0)),
        }
    }

    /// Set initial value (builder pattern)
    /// Accepts hex colors (#RGB, #RRGGBB, #RRGGBBAA) or CSS named colors
    #[must_use]
    pub fn with_value(mut self, color: impl Into<String>) -> Self {
        let color_str = color.into();
        // Try to parse as hex or named color
        if let Some(rgba) = parse_color_alpha(&color_str) {
            self.current_rgb = Rgb::new(rgba.r, rgba.g, rgba.b);
            self.current_hsl = self.current_rgb.to_hsl();
            self.current_hsv = self.current_rgb.to_hsv();
            self.current_alpha = rgba.a;
            self.value = if self.with_alpha && rgba.a != 255 {
                format!("#{:02X}{:02X}{:02X}{:02X}", rgba.r, rgba.g, rgba.b, rgba.a)
            } else {
                format!("#{:02X}{:02X}{:02X}", rgba.r, rgba.g, rgba.b)
            };
            // Flag that text input needs to be synced on first render
            self.needs_input_sync = true;
        }
        self
    }

    /// Set placeholder text (builder pattern)
    #[must_use]
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// Enable or disable alpha channel support (builder pattern)
    #[must_use]
    pub fn with_alpha(mut self, enabled: bool) -> Self {
        self.with_alpha = enabled;
        self
    }

    /// Set enabled state (builder pattern)
    #[must_use]
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set custom theme (builder pattern)
    #[must_use]
    pub fn theme(mut self, theme: Theme) -> Self {
        self.custom_theme = Some(theme);
        self
    }

    /// Get the current hex value
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Get current RGB value
    pub fn rgb(&self) -> Rgb {
        self.current_rgb
    }

    /// Get current HSL value
    pub fn hsl(&self) -> Hsl {
        self.current_hsl
    }

    /// Get current alpha value (0-255)
    pub fn alpha(&self) -> u8 {
        self.current_alpha
    }

    /// Check if the widget is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Set enabled state programmatically
    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.enabled != enabled {
            self.enabled = enabled;
            // Sync enabled state to the hex input
            self.hex_input.update(cx, |input, cx| {
                input.set_enabled(enabled, cx);
            });
            cx.notify();
        }
    }

    /// Set value programmatically
    pub fn set_value(&mut self, color: &str, cx: &mut Context<Self>) {
        self.set_value_internal(color, cx);
        cx.emit(ColorSwatchEvent::Change(self.value.clone()));
    }

    /// Get the focus handle
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    /// Internal set value without emitting event
    fn set_value_internal(&mut self, color: &str, cx: &mut Context<Self>) {
        // Try to parse as hex or named color
        if let Some(rgba) = parse_color_alpha(color) {
            self.current_rgb = Rgb::new(rgba.r, rgba.g, rgba.b);
            self.current_hsl = self.current_rgb.to_hsl();
            self.current_hsv = self.current_rgb.to_hsv();
            self.current_alpha = rgba.a;
            self.value = if self.with_alpha && rgba.a != 255 {
                format!("#{:02X}{:02X}{:02X}{:02X}", rgba.r, rgba.g, rgba.b, rgba.a)
            } else {
                format!("#{:02X}{:02X}{:02X}", rgba.r, rgba.g, rgba.b)
            };
            // Update text input
            self.hex_input.update(cx, |input, cx| {
                input.set_value(&self.value, cx);
            });
        }
        cx.notify();
    }

    /// Handle input text change (live preview without committing)
    fn handle_input_change(&mut self, value: &str, cx: &mut Context<Self>) {
        // Try to parse and update preview
        if let Some(rgb) = parse_color(value) {
            self.current_rgb = rgb;
            self.current_hsl = rgb.to_hsl();
            self.current_hsv = rgb.to_hsv();
            // Update the value but don't update the text input (user is typing)
            self.value = format!("#{:02X}{:02X}{:02X}", rgb.r, rgb.g, rgb.b);
            self.input_is_valid = true;
            cx.emit(ColorSwatchEvent::Change(self.value.clone()));
        } else {
            self.input_is_valid = false;
        }
        cx.notify();
    }

    /// Handle input commit (Enter/Blur) - parse named colors
    fn handle_input_commit(&mut self, value: &str, cx: &mut Context<Self>) {
        if let Some(rgba) = parse_color_alpha(value) {
            self.current_rgb = Rgb::new(rgba.r, rgba.g, rgba.b);
            self.current_hsl = self.current_rgb.to_hsl();
            self.current_hsv = self.current_rgb.to_hsv();
            self.current_alpha = rgba.a;
            self.value = if self.with_alpha && rgba.a != 255 {
                format!("#{:02X}{:02X}{:02X}{:02X}", rgba.r, rgba.g, rgba.b, rgba.a)
            } else {
                format!("#{:02X}{:02X}{:02X}", rgba.r, rgba.g, rgba.b)
            };
            self.input_is_valid = true;
            // Update text input to show hex value (convert named colors)
            self.hex_input.update(cx, |input, cx| {
                input.set_value(&self.value, cx);
            });
            cx.emit(ColorSwatchEvent::Change(self.value.clone()));
        } else {
            self.input_is_valid = false;
        }
        cx.notify();
    }

    /// Check if the current input is valid
    pub fn is_input_valid(&self) -> bool {
        self.input_is_valid
    }

    /// Update from RGB values
    fn update_from_rgb(&mut self, r: u8, g: u8, b: u8, cx: &mut Context<Self>) {
        self.current_rgb = Rgb::new(r, g, b);
        self.current_hsl = self.current_rgb.to_hsl();
        self.current_hsv = self.current_rgb.to_hsv();
        self.sync_value(cx);
    }

    /// Update from HSL values (preserves hue for H slider)
    #[allow(dead_code)]
    fn update_from_hsl(&mut self, h: f32, s: f32, l: f32, cx: &mut Context<Self>) {
        self.current_hsl = Hsl::new(h, s, l);
        self.current_rgb = self.current_hsl.to_rgb();
        self.current_hsv = self.current_rgb.to_hsv();
        // Preserve hue in HSV to match HSL hue
        self.current_hsv = Hsv::new(h, self.current_hsv.s, self.current_hsv.v);
        self.sync_value(cx);
    }

    /// Update from HSV values (used by the S/V canvas)
    fn update_from_hsv(&mut self, h: f32, s: f32, v: f32, cx: &mut Context<Self>) {
        self.current_hsv = Hsv::new(h, s, v);
        self.current_rgb = self.current_hsv.to_rgb();
        self.current_hsl = self.current_rgb.to_hsl();
        // Preserve hue in HSL to match HSV hue
        self.current_hsl = Hsl::new(h, self.current_hsl.s, self.current_hsl.l);
        self.sync_value(cx);
    }

    /// Sync value string and text input from current RGB
    fn sync_value(&mut self, cx: &mut Context<Self>) {
        let rgb = self.current_rgb;
        self.value = if self.with_alpha && self.current_alpha != 255 {
            format!("#{:02X}{:02X}{:02X}{:02X}", rgb.r, rgb.g, rgb.b, self.current_alpha)
        } else {
            format!("#{:02X}{:02X}{:02X}", rgb.r, rgb.g, rgb.b)
        };
        self.hex_input.update(cx, |input, cx| {
            input.set_value(&self.value, cx);
        });
        cx.emit(ColorSwatchEvent::Change(self.value.clone()));
        cx.notify();
    }

    /// Open the color picker popup
    fn open_picker(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.original_value = self.value.clone();
        self.is_picker_open = true;
        self.picker_focus_handle.focus(window);
        cx.notify();
    }

    /// Close the color picker popup
    fn close_picker(&mut self, cx: &mut Context<Self>) {
        self.is_picker_open = false;
        cx.notify();
    }

    /// Parse the current value to get a GPUI Rgba for display
    fn parse_display_color(&self) -> Rgba {
        let rgb = self.current_rgb;
        let a = self.current_alpha;
        rgba(((rgb.r as u32) << 24) | ((rgb.g as u32) << 16) | ((rgb.b as u32) << 8) | (a as u32))
    }

    /// Parse original value for comparison display
    fn parse_original_color(&self) -> Rgba {
        if let Some(rgba_val) = parse_color_alpha(&self.original_value) {
            rgba(((rgba_val.r as u32) << 24) | ((rgba_val.g as u32) << 16) | ((rgba_val.b as u32) << 8) | (rgba_val.a as u32))
        } else {
            rgba(0x000000FF)
        }
    }

    /// Handle S/V canvas interaction at position (HSV model)
    /// X axis = Saturation (0% left to 100% right)
    /// Y axis = Value/Brightness (0% bottom to 100% top)
    fn handle_sl_at_position(&mut self, x: f32, y: f32, origin: Point<Pixels>, canvas_width: f32, canvas_height: f32, cx: &mut Context<Self>) {
        let origin_x: f32 = origin.x.into();
        let origin_y: f32 = origin.y.into();
        let rel_x = (x - origin_x).clamp(0.0, canvas_width);
        let rel_y = (y - origin_y).clamp(0.0, canvas_height);

        let s = (rel_x / canvas_width) * 100.0;
        let v = (1.0 - rel_y / canvas_height) * 100.0;
        self.update_from_hsv(self.current_hsv.h, s, v, cx);
    }

    /// Handle hue slider interaction at position
    /// Note: Hue is clamped to 0-359 to prevent wrap-around (360° = 0° = red)
    fn handle_hue_at_position(&mut self, x: f32, origin_x: f32, slider_width: f32, cx: &mut Context<Self>) {
        // Must match the display calculation
        // Note: border doesn't affect layout width in GPUI, only content width matters
        let handle_width = 4.0f32;
        let usable_width = slider_width - handle_width;
        // Map click position to handle left edge position, then to value
        // Clicking anywhere on the slider should work, with clamping at edges
        let rel_x = (x - origin_x - handle_width / 2.0).clamp(0.0, usable_width);
        // Cap at 359 to prevent wrap-around to pure red (360° = 0°)
        let h = (rel_x / usable_width) * 359.0;
        // Use HSV for hue changes to keep S/V canvas consistent
        self.update_from_hsv(h, self.current_hsv.s, self.current_hsv.v, cx);
    }

    /// Handle alpha slider interaction at position
    fn handle_alpha_at_position(&mut self, x: f32, origin_x: f32, slider_width: f32, cx: &mut Context<Self>) {
        // Must match the display calculation
        // Note: border doesn't affect layout width in GPUI, only content width matters
        let handle_width = 4.0f32;
        let usable_width = slider_width - handle_width;
        // Map click position to handle left edge position, then to value
        let rel_x = (x - origin_x - handle_width / 2.0).clamp(0.0, usable_width);
        self.current_alpha = ((rel_x / usable_width) * 255.0) as u8;
        self.sync_value(cx);
    }
}

impl Render for ColorSwatch {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        // Sync text input if needed (for builder pattern with_value)
        if self.needs_input_sync {
            self.needs_input_sync = false;
            let value = self.value.clone();
            let enabled = self.enabled;
            self.hex_input.update(cx, |input, cx| {
                input.set_value(&value, cx);
                input.set_enabled(enabled, cx);
            });
        }

        let theme = get_theme_or(cx, self.custom_theme.as_ref());
        let color = self.parse_display_color();
        let is_picker_open = self.is_picker_open;
        let hex_input = self.hex_input.clone();
        let enabled = self.enabled;

        let bg_popup = theme.bg_secondary;
        let border_checkbox = theme.border_checkbox;
        let border_input = theme.border_input;
        let text_color = theme.text_primary;
        let picker_focus_handle = self.picker_focus_handle.clone();

        div()
            .id("ccf_color_swatch")
            .relative()
            // Focus navigation (Tab / Shift+Tab) - but don't track focus, let TextInput handle it
            .on_action(cx.listener(|this, _: &FocusNext, window, _cx| {
                if !this.enabled {
                    return;
                }
                window.focus_next();
            }))
            .on_action(cx.listener(|this, _: &FocusPrev, window, _cx| {
                if !this.enabled {
                    return;
                }
                window.focus_prev();
            }))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_2()
                    .items_center()
                    .child(
                        // Color preview box (clickable to open picker)
                        div()
                            .id("ccf_color_preview")
                            .relative()
                            .w(px(40.))
                            .h(px(32.))
                            .border_1()
                            .border_color(rgb(border_checkbox))
                            .rounded_md()
                            .overflow_hidden()
                            .when(enabled, |d| d.cursor_pointer())
                            .when(!enabled, |d| d.cursor_default().opacity(0.5))
                            // Checkerboard background for alpha visualization
                            .when(self.with_alpha, |d| d.child(Self::render_checkerboard()))
                            // Color overlay
                            .child(
                                div()
                                    .size_full()
                                    .absolute()
                                    .bg(color)
                            )
                            .on_click(cx.listener(|this, _event, window, cx| {
                                if !this.enabled {
                                    return;
                                }
                                if this.is_picker_open {
                                    this.close_picker(cx);
                                } else {
                                    this.open_picker(window, cx);
                                }
                            }))
                    )
                    .child(
                        // Hex color text input with error border
                        div()
                            .flex_1()
                            .border_2()
                            .rounded_md()
                            .border_color(if self.input_is_valid {
                                rgba(0x00000000)
                            } else {
                                rgb(theme.border_error)
                            })
                            .child(hex_input)
                    )
            )
            // Color picker popup
            .when(is_picker_open, |parent| {
                let current_rgb = self.current_rgb;
                let current_hsv = self.current_hsv;
                let current_alpha = self.current_alpha;
                let with_alpha = self.with_alpha;
                let original_color = self.parse_original_color();
                let new_color = self.parse_display_color();
                let original_hex = self.original_value.clone();
                let new_hex = self.value.clone();

                // 2D S/V canvas dimensions (HSV model)
                let canvas_width = 200.0f32;
                let canvas_height = 150.0f32;
                let hue = current_hsv.h;

                // Canvas origin for mouse handling (shared via Rc<Cell<>>)
                let canvas_origin = Rc::new(Cell::new(Point::default()));
                let canvas_origin_for_paint = canvas_origin.clone();
                let canvas_origin_for_drag = canvas_origin.clone();

                // Hue slider - use persistent fields from struct
                let hue_origin = self.hue_slider_origin.clone();
                let hue_origin_for_paint = hue_origin.clone();
                let hue_origin_for_drag = hue_origin.clone();
                let hue_width = self.hue_slider_width.clone();
                let hue_width_for_paint = hue_width.clone();
                let hue_width_for_drag = hue_width.clone();

                // Alpha slider - use persistent fields from struct
                let alpha_origin = self.alpha_slider_origin.clone();
                let alpha_origin_for_paint = alpha_origin.clone();
                let alpha_origin_for_drag = alpha_origin.clone();
                let alpha_width = self.alpha_slider_width.clone();
                let alpha_width_for_paint = alpha_width.clone();
                let alpha_width_for_drag = alpha_width.clone();

                parent.child(
                    deferred(
                        anchored()
                            .anchor(Corner::TopLeft)
                            .child(
                                div()
                                    .id("ccf_color_picker")
                                    .key_context("CcfColorPicker")
                                    .track_focus(&picker_focus_handle)
                                    .on_action(cx.listener(|this, _: &ClosePicker, _window, cx| {
                                        this.close_picker(cx);
                                    }))
                                    .occlude()
                                    .absolute()
                                    .top(px(4.))  // Small gap below the main control
                                    .left_0()
                                    .w(px(280.))
                                    .p_3()
                                    .bg(rgb(bg_popup))
                                    .border_1()
                                    .border_color(rgb(border_input))
                                    .rounded_lg()
                                    .shadow_lg()
                                    .flex()
                                    .flex_col()
                                    .gap_3()
                                    // 2D Saturation/Lightness canvas
                                    .child(
                                        div()
                                            .id("sl_canvas")
                                            .relative()
                                            .w(px(canvas_width))
                                            .h(px(canvas_height))
                                            .rounded_md()
                                            .border_1()
                                            .border_color(rgb(border_input))
                                            .overflow_hidden()
                                            .cursor_crosshair()
                                            // Background is the hue at full saturation (HSV: S=100%, V=100%)
                                            .bg(rgb(Hsv::new(hue, 100.0, 100.0).to_rgb().to_u32()))
                                            .child(
                                                // Canvas for gradients
                                                canvas(
                                                    move |bounds, _window, _cx| {
                                                        canvas_origin_for_paint.set(bounds.origin);
                                                        bounds
                                                    },
                                                    move |bounds, _prepaint_result, window, _cx| {
                                                        // Paint white gradient (left to right): 90 degrees
                                                        let white_start = linear_color_stop(white(), 0.0);
                                                        let white_end = linear_color_stop(transparent_white(), 1.0);
                                                        let white_gradient = linear_gradient(90.0, white_start, white_end);
                                                        window.paint_quad(fill(bounds, white_gradient));

                                                        // Paint black gradient (top to bottom): 180 degrees
                                                        let black_start = linear_color_stop(transparent_black(), 0.0);
                                                        let black_end = linear_color_stop(black(), 1.0);
                                                        let black_gradient = linear_gradient(180.0, black_start, black_end);
                                                        window.paint_quad(fill(bounds, black_gradient));
                                                    },
                                                )
                                                .size_full()
                                                .absolute()
                                            )
                                            // Crosshair indicator
                                            .child({
                                                // Position based on current S/V (HSV model)
                                                let s = current_hsv.s / 100.0;
                                                let v = current_hsv.v / 100.0;
                                                let x = s * canvas_width;
                                                let y = (1.0 - v) * canvas_height;

                                                div()
                                                    .absolute()
                                                    .left(px(x - 6.0))
                                                    .top(px(y - 6.0))
                                                    .w(px(12.))
                                                    .h(px(12.))
                                                    .rounded_full()
                                                    .border_2()
                                                    .border_color(rgb(0xFFFFFF))
                                                    .shadow_sm()
                                            })
                                            .on_drag(
                                                SlDrag {
                                                    canvas_origin: canvas_origin_for_drag.clone(),
                                                    canvas_width,
                                                    canvas_height,
                                                },
                                                |_drag, _offset, _window, cx| cx.new(|_| EmptyDragView),
                                            )
                                            .on_drag_move(cx.listener(move |this, event: &DragMoveEvent<SlDrag>, _window, cx| {
                                                let x: f32 = event.event.position.x.into();
                                                let y: f32 = event.event.position.y.into();
                                                let drag = event.drag(cx);
                                                this.handle_sl_at_position(x, y, drag.canvas_origin.get(), drag.canvas_width, drag.canvas_height, cx);
                                            }))
                                    )
                                    // Hue slider
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .w(px(16.))
                                                    .text_xs()
                                                    .text_color(rgb(text_color))
                                                    .child("H")
                                            )
                                            .child(
                                                div()
                                                    .id("hue_slider")
                                                    .relative()
                                                    .flex_1()
                                                    .h(px(20.))
                                                    .rounded_sm()
                                                    .border_1()
                                                    .border_color(rgb(border_input))
                                                    .overflow_hidden()
                                                    .cursor_pointer()
                                                    .child(
                                                        canvas(
                                                            move |bounds, _window, _cx| {
                                                                hue_origin_for_paint.set(bounds.origin.x.into());
                                                                hue_width_for_paint.set(bounds.size.width.into());
                                                                bounds
                                                            },
                                                            move |bounds, _prepaint_result, window, _cx| {
                                                                // Paint rainbow gradient using multiple quads (90 degrees = left to right)
                                                                let width: f32 = bounds.size.width.into();
                                                                let segment_count = 6;
                                                                let segment_width = width / segment_count as f32;
                                                                let hue_colors = [
                                                                    0xFF0000u32, // Red
                                                                    0xFFFF00,    // Yellow
                                                                    0x00FF00,    // Green
                                                                    0x00FFFF,    // Cyan
                                                                    0x0000FF,    // Blue
                                                                    0xFF00FF,    // Magenta
                                                                    0xFF0000,    // Red (wrap)
                                                                ];

                                                                for i in 0..segment_count {
                                                                    let start_x = bounds.origin.x + px(i as f32 * segment_width);
                                                                    let segment_bounds = Bounds {
                                                                        origin: point(start_x, bounds.origin.y),
                                                                        size: size(px(segment_width + 1.0), bounds.size.height),
                                                                    };
                                                                    let start_color = rgb(hue_colors[i]);
                                                                    let end_color = rgb(hue_colors[i + 1]);
                                                                    let start_stop = linear_color_stop(start_color, 0.0);
                                                                    let end_stop = linear_color_stop(end_color, 1.0);
                                                                    let gradient = linear_gradient(90.0, start_stop, end_stop);
                                                                    window.paint_quad(fill(segment_bounds, gradient));
                                                                }
                                                            },
                                                        )
                                                        .size_full()
                                                        .absolute()
                                                    )
                                                    // Handle - use measured width, accounting for handle width
                                                    .child({
                                                        let measured_width = hue_width.get();
                                                        let handle_width = 4.0f32;
                                                        // Handle moves within (0, measured_width - handle_width)
                                                        // Use 359.0 as max to match the clamped hue range
                                                        let handle_x = (current_hsv.h / 359.0).min(1.0) * (measured_width - handle_width);
                                                        div()
                                                            .absolute()
                                                            .top_0()
                                                            .bottom_0()
                                                            .left(px(handle_x))
                                                            .w(px(handle_width))
                                                            .bg(rgb(0xFFFFFF))
                                                            .border_1()
                                                            .border_color(rgb(0x333333))
                                                            .rounded_sm()
                                                    })
                                                    .on_drag(
                                                        HueDrag {
                                                            origin: hue_origin_for_drag.clone(),
                                                            width: hue_width_for_drag.clone(),
                                                        },
                                                        |_drag, _offset, _window, cx| cx.new(|_| EmptyDragView),
                                                    )
                                                    .on_drag_move(cx.listener(move |this, event: &DragMoveEvent<HueDrag>, _window, cx| {
                                                        let x: f32 = event.event.position.x.into();
                                                        let drag = event.drag(cx);
                                                        this.handle_hue_at_position(x, drag.origin.get(), drag.width.get(), cx);
                                                    }))
                                            )
                                    )
                                    // Alpha slider (if enabled)
                                    .when(with_alpha, |parent| {
                                        parent.child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .gap_2()
                                                .child(
                                                    div()
                                                        .w(px(16.))
                                                        .text_xs()
                                                        .text_color(rgb(text_color))
                                                        .child("A")
                                                )
                                                .child(
                                                    div()
                                                        .id("alpha_slider")
                                                        .relative()
                                                        .flex_1()
                                                        .h(px(20.))
                                                        .rounded_sm()
                                                        .border_1()
                                                        .border_color(rgb(border_input))
                                                        .overflow_hidden()
                                                        .cursor_pointer()
                                                        // Checkerboard background for alpha visualization
                                                        .child(Self::render_checkerboard())
                                                        .child(
                                                            canvas(
                                                                move |bounds, _window, _cx| {
                                                                    alpha_origin_for_paint.set(bounds.origin.x.into());
                                                                    alpha_width_for_paint.set(bounds.size.width.into());
                                                                    bounds
                                                                },
                                                                move |bounds, _prepaint_result, window, _cx| {
                                                                    // Paint color gradient with transparency (90 degrees = left to right)
                                                                    let color = rgb(current_rgb.to_u32());
                                                                    let start_stop = linear_color_stop(transparent_white(), 0.0);
                                                                    let end_stop = linear_color_stop(color, 1.0);
                                                                    let gradient = linear_gradient(90.0, start_stop, end_stop);
                                                                    window.paint_quad(fill(bounds, gradient));
                                                                },
                                                            )
                                                            .size_full()
                                                            .absolute()
                                                        )
                                                        // Handle - use measured width, accounting for handle width
                                                        .child({
                                                            let measured_width = alpha_width.get();
                                                            let handle_width = 4.0f32;
                                                            // Handle moves within (0, measured_width - handle_width)
                                                            let handle_x = (current_alpha as f32 / 255.0) * (measured_width - handle_width);
                                                            div()
                                                                .absolute()
                                                                .top_0()
                                                                .bottom_0()
                                                                .left(px(handle_x))
                                                                .w(px(handle_width))
                                                                .bg(rgb(0xFFFFFF))
                                                                .border_1()
                                                                .border_color(rgb(0x333333))
                                                                .rounded_sm()
                                                        })
                                                        .on_drag(
                                                            AlphaDrag {
                                                                origin: alpha_origin_for_drag.clone(),
                                                                width: alpha_width_for_drag.clone(),
                                                            },
                                                            |_drag, _offset, _window, cx| cx.new(|_| EmptyDragView),
                                                        )
                                                        .on_drag_move(cx.listener(move |this, event: &DragMoveEvent<AlphaDrag>, _window, cx| {
                                                            let x: f32 = event.event.position.x.into();
                                                            let drag = event.drag(cx);
                                                            this.handle_alpha_at_position(x, drag.origin.get(), drag.width.get(), cx);
                                                        }))
                                                )
                                        )
                                    })
                                    // RGB sliders
                                    .child(
                                        Self::render_component_slider(
                                            "R", current_rgb.r as f32, 255.0,
                                            (0x000000, 0xFF0000),
                                            |this, v, cx| this.update_from_rgb(v as u8, this.current_rgb.g, this.current_rgb.b, cx),
                                            &theme, cx
                                        )
                                    )
                                    .child(
                                        Self::render_component_slider(
                                            "G", current_rgb.g as f32, 255.0,
                                            (0x000000, 0x00FF00),
                                            |this, v, cx| this.update_from_rgb(this.current_rgb.r, v as u8, this.current_rgb.b, cx),
                                            &theme, cx
                                        )
                                    )
                                    .child(
                                        Self::render_component_slider(
                                            "B", current_rgb.b as f32, 255.0,
                                            (0x000000, 0x0000FF),
                                            |this, v, cx| this.update_from_rgb(this.current_rgb.r, this.current_rgb.g, v as u8, cx),
                                            &theme, cx
                                        )
                                    )
                                    // Old / New color comparison
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap_4()
                                            .child(
                                                div()
                                                    .flex()
                                                    .flex_col()
                                                    .items_center()
                                                    .gap_1()
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(rgb(text_color))
                                                            .child("Old")
                                                    )
                                                    .child(
                                                        div()
                                                            .relative()
                                                            .w(px(60.))
                                                            .h(px(30.))
                                                            .border_1()
                                                            .border_color(rgb(border_input))
                                                            .rounded_md()
                                                            .overflow_hidden()
                                                            // Checkerboard for alpha
                                                            .when(with_alpha, |d| d.child(Self::render_checkerboard()))
                                                            .child(
                                                                div()
                                                                    .size_full()
                                                                    .absolute()
                                                                    .bg(original_color)
                                                            )
                                                    )
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(rgb(text_color))
                                                            .child(original_hex)
                                                    )
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .flex_col()
                                                    .items_center()
                                                    .gap_1()
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(rgb(text_color))
                                                            .child("New")
                                                    )
                                                    .child(
                                                        div()
                                                            .relative()
                                                            .w(px(60.))
                                                            .h(px(30.))
                                                            .border_1()
                                                            .border_color(rgb(border_input))
                                                            .rounded_md()
                                                            .overflow_hidden()
                                                            // Checkerboard for alpha
                                                            .when(with_alpha, |d| d.child(Self::render_checkerboard()))
                                                            .child(
                                                                div()
                                                                    .size_full()
                                                                    .absolute()
                                                                    .bg(new_color)
                                                            )
                                                    )
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(rgb(text_color))
                                                            .child(new_hex)
                                                    )
                                            )
                                    )
                                    // Close on click outside
                                    .on_mouse_down_out(cx.listener(|this, _event, _window, cx| {
                                        this.close_picker(cx);
                                    }))
                            )
                    )
                )
            })
    }
}

impl ColorSwatch {
    /// Render a checkerboard pattern canvas (for alpha transparency visualization)
    fn render_checkerboard() -> impl IntoElement {
        canvas(
            |bounds, _window, _cx| bounds,
            |bounds, _prepaint_result, window, _cx| {
                let cell_size = 8.0f32;
                let light = rgb(0xFFFFFF);
                let dark = rgb(0xCCCCCC);
                let width: f32 = bounds.size.width.into();
                let height: f32 = bounds.size.height.into();
                let cols = (width / cell_size).ceil() as i32;
                let rows = (height / cell_size).ceil() as i32;

                for row in 0..rows {
                    for col in 0..cols {
                        let is_light = (row + col) % 2 == 0;
                        let color = if is_light { light } else { dark };
                        let x = bounds.origin.x + px(col as f32 * cell_size);
                        let y = bounds.origin.y + px(row as f32 * cell_size);
                        let cell_w = (cell_size).min(width - col as f32 * cell_size);
                        let cell_h = (cell_size).min(height - row as f32 * cell_size);
                        let cell_bounds = Bounds {
                            origin: point(x, y),
                            size: size(px(cell_w), px(cell_h)),
                        };
                        window.paint_quad(fill(cell_bounds, color));
                    }
                }
            },
        )
        .size_full()
        .absolute()
    }

    /// Render a component slider (R, G, B, S, L)
    fn render_component_slider(
        label: &str,
        value: f32,
        max: f32,
        gradient_colors: (u32, u32),
        update_fn: fn(&mut ColorSwatch, f32, &mut Context<Self>),
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let slider_width = 180.0f32;
        let handle_content_width = 4.0f32;
        let handle_visual_width = 6.0f32; // 4px content + 2px border
        // Handle moves within (0, track_width - visual_width)
        let handle_pos = (value / max) * (slider_width - handle_visual_width);
        let value_display = value.round() as i32;
        let text_color = theme.text_primary;
        let border_input = theme.border_input;
        let (start_color, end_color) = gradient_colors;

        let slider_origin = Rc::new(Cell::new(0.0f32));
        let slider_origin_for_paint = slider_origin.clone();
        let slider_origin_for_drag = slider_origin.clone();

        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .child(
                div()
                    .w(px(16.))
                    .text_xs()
                    .text_color(rgb(text_color))
                    .child(label.to_string())
            )
            .child(
                div()
                    .id(SharedString::from(format!("comp_slider_{}", label)))
                    .relative()
                    .w(px(slider_width))
                    .h(px(16.))
                    .rounded_sm()
                    .border_1()
                    .border_color(rgb(border_input))
                    .overflow_hidden()
                    .cursor_pointer()
                    .child(
                        canvas(
                            move |bounds, _window, _cx| {
                                slider_origin_for_paint.set(bounds.origin.x.into());
                                bounds
                            },
                            move |bounds, _prepaint_result, window, _cx| {
                                // 90 degrees = left to right
                                let start_stop = linear_color_stop(rgb(start_color), 0.0);
                                let end_stop = linear_color_stop(rgb(end_color), 1.0);
                                let gradient = linear_gradient(90.0, start_stop, end_stop);
                                window.paint_quad(fill(bounds, gradient));
                            },
                        )
                        .size_full()
                        .absolute()
                    )
                    // Handle
                    .child(
                        div()
                            .absolute()
                            .top_0()
                            .bottom_0()
                            .left(px(handle_pos))
                            .w(px(handle_content_width))
                            .bg(rgb(0xFFFFFF))
                            .border_1()
                            .border_color(rgb(0x333333))
                            .rounded_sm()
                    )
                    .on_drag(
                        ComponentDrag {
                            origin: slider_origin_for_drag.clone(),
                            slider_width,
                            handle_visual_width,
                            max_value: max,
                            update_fn,
                        },
                        |_drag, _offset, _window, cx| cx.new(|_| EmptyDragView),
                    )
                    .on_drag_move(cx.listener(move |this, event: &DragMoveEvent<ComponentDrag>, _window, cx| {
                        let x: f32 = event.event.position.x.into();
                        let drag = event.drag(cx);
                        let origin = drag.origin.get();
                        let usable_width = drag.slider_width - drag.handle_visual_width;
                        // Map click position to handle left edge position, then to value
                        let rel_x = (x - origin - drag.handle_visual_width / 2.0).clamp(0.0, usable_width);
                        let new_value = (rel_x / usable_width) * drag.max_value;
                        (drag.update_fn)(this, new_value, cx);
                    }))
            )
            .child(
                div()
                    .w(px(28.))
                    .text_xs()
                    .text_color(rgb(text_color))
                    .child(format!("{}", value_display))
            )
    }
}
