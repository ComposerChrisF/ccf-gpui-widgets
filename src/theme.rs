//! Theme system for ccf-gpui-widgets
//!
//! Provides a `Theme` struct with sensible defaults and builder pattern for customization.
//! Widgets can access the theme via a global context or per-widget override.
//!
//! # Example
//!
//! ```ignore
//! use ccf_gpui_widgets::Theme;
//!
//! // Use dark theme (default)
//! let theme = Theme::dark();
//!
//! // Use light theme
//! let theme = Theme::light();
//!
//! // Customize
//! let theme = Theme::dark()
//!     .with_accent(0x00ff00)
//!     .with_border_focus(0x00ff00);
//!
//! // Set globally
//! cx.set_global(theme);
//! ```

/// Theme configuration for widgets
///
/// All colors are stored as u32 hex values (0xRRGGBB format).
/// Use with GPUI's `rgb()` macro: `rgb(theme.bg_primary)`
#[derive(Clone, Copy, Debug)]
pub struct Theme {
    // Background colors
    /// Main application background (darkest)
    pub bg_primary: u32,
    /// Panel/section background (slightly lighter)
    pub bg_secondary: u32,
    /// Input field background
    pub bg_input: u32,
    /// Input field background when hovered
    pub bg_input_hover: u32,
    /// Button/interactive element hover background
    pub bg_hover: u32,
    /// Section header background
    pub bg_section_header: u32,
    /// Section header hover background
    pub bg_section_header_hover: u32,
    /// White background for light-themed elements
    pub bg_white: u32,
    /// Light hover background
    pub bg_light_hover: u32,

    // Text colors
    /// Primary text color
    pub text_primary: u32,
    /// Label text color
    pub text_label: u32,
    /// Section header text color
    pub text_section_header: u32,
    /// Value/content text color
    pub text_value: u32,
    /// Muted/secondary text color
    pub text_muted: u32,
    /// Placeholder text color
    pub text_placeholder: u32,
    /// Dimmed/disabled text color
    pub text_dimmed: u32,
    /// Icon text color
    pub text_icon: u32,
    /// Dark text (on light backgrounds)
    pub text_dark: u32,
    /// Black text
    pub text_black: u32,

    // Border colors
    /// Standard border color
    pub border_default: u32,
    /// Checkbox/radio button border
    pub border_checkbox: u32,
    /// Input field border
    pub border_input: u32,
    /// Menu/dropdown border
    pub border_menu: u32,
    /// Focus/active border
    pub border_focus: u32,
    /// Focus border for colored backgrounds (e.g., primary buttons)
    /// Should contrast with primary/accent colored elements
    pub border_focus_on_color: u32,
    /// Error border
    pub border_error: u32,

    // Accent colors
    /// Primary accent color (buttons, checkboxes)
    pub primary: u32,
    /// Primary hover state
    pub primary_hover: u32,
    /// Primary active/pressed state
    pub primary_active: u32,
    /// Accent color (focus rings, selections)
    pub accent: u32,

    // Status colors
    /// Success/positive color (green)
    pub success: u32,
    /// Error/negative color (red)
    pub error: u32,
    /// Warning color (orange)
    pub warning: u32,
    /// Validation error text
    pub error_text: u32,

    // Tooltip colors
    /// Tooltip background
    pub tooltip_bg: u32,
    /// Tooltip border
    pub tooltip_border: u32,
    /// Tooltip text
    pub tooltip_text: u32,

    // Selection color (for text selection)
    pub selection: u32,

    // Button disabled state
    /// Disabled button background
    pub disabled_bg: u32,
    /// Disabled button text
    pub disabled_text: u32,

    // Secondary button colors
    /// Secondary button background
    pub secondary_bg: u32,
    /// Secondary button hover background
    pub secondary_bg_hover: u32,
    /// Secondary button active background
    pub secondary_bg_active: u32,
    /// Secondary button border
    pub secondary_border: u32,

    // Tab colors
    /// Tab hover background
    pub bg_tab_hover: u32,
    /// Active tab border
    pub border_tab_active: u32,

    // Delete/remove button colors
    /// Delete button background
    pub delete_bg: u32,
    /// Delete button hover background
    pub delete_bg_hover: u32,

    // Path display
    /// Path display hover background
    pub bg_path_hover: u32,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

// Implement Global trait so Theme can be stored in GPUI context
impl gpui::Global for Theme {}

impl Theme {
    /// Create a dark theme (default)
    pub fn dark() -> Self {
        Self {
            // Background colors
            bg_primary: 0x1e1e1e,
            bg_secondary: 0x252525,
            bg_input: 0x2a2a2a,
            bg_input_hover: 0x3a3a3a,
            bg_hover: 0x4a4a4a,
            bg_section_header: 0x363636,
            bg_section_header_hover: 0x404040,
            bg_white: 0xffffff,
            bg_light_hover: 0xf0f0f0,

            // Text colors
            text_primary: 0xffffff,
            text_label: 0xeeeeee,
            text_section_header: 0xdddddd,
            text_value: 0xcccccc,
            text_muted: 0xaaaaaa,
            text_placeholder: 0x999999,
            text_dimmed: 0x888888,
            text_icon: 0x666666,
            text_dark: 0x333333,
            text_black: 0x000000,

            // Border colors
            border_default: 0x444444,
            border_checkbox: 0x666666,
            border_input: 0x999999,
            border_menu: 0xcccccc,
            border_focus: 0x0078d4,
            border_focus_on_color: 0xffffff, // White for contrast on colored backgrounds
            border_error: 0x662222,

            // Accent colors
            primary: 0x3b82f6,
            primary_hover: 0x2563eb,
            primary_active: 0x1d4ed8,
            accent: 0x0078d4,

            // Status colors
            success: 0x4CAF50,
            error: 0xF44336,
            warning: 0xFFA726,
            error_text: 0xFF6B6B,

            // Tooltip colors
            tooltip_bg: 0x2a2a2a,
            tooltip_border: 0x444444,
            tooltip_text: 0xeeeeee,

            // Selection color (dark blue for contrast with white text)
            selection: 0x264F78,

            // Button disabled state
            disabled_bg: 0x6b7280,
            disabled_text: 0x9ca3af,

            // Secondary button colors
            secondary_bg: 0x374151,
            secondary_bg_hover: 0x4b5563,
            secondary_bg_active: 0x1f2937,
            secondary_border: 0x6b7280,

            // Tab colors
            bg_tab_hover: 0x323232,
            border_tab_active: 0x007acc,

            // Delete/remove button colors
            delete_bg: 0x4a3a3a,
            delete_bg_hover: 0x5a4a4a,

            // Path display
            bg_path_hover: 0x333333,
        }
    }

    /// Create a light theme
    pub fn light() -> Self {
        Self {
            // Background colors
            bg_primary: 0xf5f5f5,
            bg_secondary: 0xffffff,
            bg_input: 0xffffff,
            bg_input_hover: 0xf0f0f0,
            bg_hover: 0xe0e0e0,
            bg_section_header: 0xeeeeee,
            bg_section_header_hover: 0xe5e5e5,
            bg_white: 0xffffff,
            bg_light_hover: 0xf5f5f5,

            // Text colors
            text_primary: 0x1a1a1a,
            text_label: 0x333333,
            text_section_header: 0x444444,
            text_value: 0x555555,
            text_muted: 0x777777,
            text_placeholder: 0x999999,
            text_dimmed: 0xaaaaaa,
            text_icon: 0x888888,
            text_dark: 0x333333,
            text_black: 0x000000,

            // Border colors
            border_default: 0xcccccc,
            border_checkbox: 0xaaaaaa,
            border_input: 0x444444,
            border_menu: 0xdddddd,
            border_focus: 0x0078d4,
            border_focus_on_color: 0xffffff, // White for contrast on colored backgrounds
            border_error: 0xffcccc,

            // Accent colors
            primary: 0x3b82f6,
            primary_hover: 0x2563eb,
            primary_active: 0x1d4ed8,
            accent: 0x0078d4,

            // Status colors
            success: 0x4CAF50,
            error: 0xF44336,
            warning: 0xFFA726,
            error_text: 0xdc3545,

            // Tooltip colors
            tooltip_bg: 0xffffff,
            tooltip_border: 0xaaaaaa,
            tooltip_text: 0x333333,

            // Selection color
            selection: 0xADD6FF,

            // Button disabled state
            disabled_bg: 0xd1d5db,
            disabled_text: 0x9ca3af,

            // Secondary button colors
            secondary_bg: 0xe5e7eb,
            secondary_bg_hover: 0xd1d5db,
            secondary_bg_active: 0xf3f4f6,
            secondary_border: 0x9ca3af,

            // Tab colors
            bg_tab_hover: 0xe5e5e5,
            border_tab_active: 0x0078d4,

            // Delete/remove button colors
            delete_bg: 0xfee2e2,
            delete_bg_hover: 0xfecaca,

            // Path display
            bg_path_hover: 0xf5f5f5,
        }
    }

    // Builder methods for customization

    /// Set the accent color
    pub fn with_accent(mut self, color: u32) -> Self {
        self.accent = color;
        self
    }

    /// Set the primary color
    pub fn with_primary(mut self, color: u32) -> Self {
        self.primary = color;
        self
    }

    /// Set the primary hover color
    pub fn with_primary_hover(mut self, color: u32) -> Self {
        self.primary_hover = color;
        self
    }

    /// Set the focus border color
    pub fn with_border_focus(mut self, color: u32) -> Self {
        self.border_focus = color;
        self
    }

    /// Set the focus border color for colored backgrounds
    pub fn with_border_focus_on_color(mut self, color: u32) -> Self {
        self.border_focus_on_color = color;
        self
    }

    /// Set the success color
    pub fn with_success(mut self, color: u32) -> Self {
        self.success = color;
        self
    }

    /// Set the error color
    pub fn with_error(mut self, color: u32) -> Self {
        self.error = color;
        self
    }

    /// Set the warning color
    pub fn with_warning(mut self, color: u32) -> Self {
        self.warning = color;
        self
    }

    /// Set the primary background color
    pub fn with_bg_primary(mut self, color: u32) -> Self {
        self.bg_primary = color;
        self
    }

    /// Set the input background color
    pub fn with_bg_input(mut self, color: u32) -> Self {
        self.bg_input = color;
        self
    }

    /// Set the primary text color
    pub fn with_text_primary(mut self, color: u32) -> Self {
        self.text_primary = color;
        self
    }

    // Additional background color builders

    /// Set the secondary background color
    pub fn with_bg_secondary(mut self, color: u32) -> Self {
        self.bg_secondary = color;
        self
    }

    /// Set the input hover background color
    pub fn with_bg_input_hover(mut self, color: u32) -> Self {
        self.bg_input_hover = color;
        self
    }

    /// Set the hover background color
    pub fn with_bg_hover(mut self, color: u32) -> Self {
        self.bg_hover = color;
        self
    }

    /// Set the section header background color
    pub fn with_bg_section_header(mut self, color: u32) -> Self {
        self.bg_section_header = color;
        self
    }

    /// Set the section header hover background color
    pub fn with_bg_section_header_hover(mut self, color: u32) -> Self {
        self.bg_section_header_hover = color;
        self
    }

    /// Set the white background color
    pub fn with_bg_white(mut self, color: u32) -> Self {
        self.bg_white = color;
        self
    }

    /// Set the light hover background color
    pub fn with_bg_light_hover(mut self, color: u32) -> Self {
        self.bg_light_hover = color;
        self
    }

    // Additional text color builders

    /// Set the label text color
    pub fn with_text_label(mut self, color: u32) -> Self {
        self.text_label = color;
        self
    }

    /// Set the section header text color
    pub fn with_text_section_header(mut self, color: u32) -> Self {
        self.text_section_header = color;
        self
    }

    /// Set the value text color
    pub fn with_text_value(mut self, color: u32) -> Self {
        self.text_value = color;
        self
    }

    /// Set the muted text color
    pub fn with_text_muted(mut self, color: u32) -> Self {
        self.text_muted = color;
        self
    }

    /// Set the placeholder text color
    pub fn with_text_placeholder(mut self, color: u32) -> Self {
        self.text_placeholder = color;
        self
    }

    /// Set the dimmed text color
    pub fn with_text_dimmed(mut self, color: u32) -> Self {
        self.text_dimmed = color;
        self
    }

    /// Set the icon text color
    pub fn with_text_icon(mut self, color: u32) -> Self {
        self.text_icon = color;
        self
    }

    /// Set the dark text color
    pub fn with_text_dark(mut self, color: u32) -> Self {
        self.text_dark = color;
        self
    }

    /// Set the black text color
    pub fn with_text_black(mut self, color: u32) -> Self {
        self.text_black = color;
        self
    }

    // Additional border color builders

    /// Set the default border color
    pub fn with_border_default(mut self, color: u32) -> Self {
        self.border_default = color;
        self
    }

    /// Set the checkbox border color
    pub fn with_border_checkbox(mut self, color: u32) -> Self {
        self.border_checkbox = color;
        self
    }

    /// Set the input border color
    pub fn with_border_input(mut self, color: u32) -> Self {
        self.border_input = color;
        self
    }

    /// Set the menu border color
    pub fn with_border_menu(mut self, color: u32) -> Self {
        self.border_menu = color;
        self
    }

    /// Set the error border color
    pub fn with_border_error(mut self, color: u32) -> Self {
        self.border_error = color;
        self
    }

    // Additional accent color builders

    /// Set the primary active color
    pub fn with_primary_active(mut self, color: u32) -> Self {
        self.primary_active = color;
        self
    }

    /// Set the error text color
    pub fn with_error_text(mut self, color: u32) -> Self {
        self.error_text = color;
        self
    }

    // Tooltip color builders

    /// Set the tooltip background color
    pub fn with_tooltip_bg(mut self, color: u32) -> Self {
        self.tooltip_bg = color;
        self
    }

    /// Set the tooltip border color
    pub fn with_tooltip_border(mut self, color: u32) -> Self {
        self.tooltip_border = color;
        self
    }

    /// Set the tooltip text color
    pub fn with_tooltip_text(mut self, color: u32) -> Self {
        self.tooltip_text = color;
        self
    }

    // Selection color builder

    /// Set the selection highlight color
    pub fn with_selection(mut self, color: u32) -> Self {
        self.selection = color;
        self
    }

    // Disabled button state builders

    /// Set the disabled button background color
    pub fn with_disabled_bg(mut self, color: u32) -> Self {
        self.disabled_bg = color;
        self
    }

    /// Set the disabled button text color
    pub fn with_disabled_text(mut self, color: u32) -> Self {
        self.disabled_text = color;
        self
    }

    // Secondary button builders

    /// Set the secondary button background color
    pub fn with_secondary_bg(mut self, color: u32) -> Self {
        self.secondary_bg = color;
        self
    }

    /// Set the secondary button hover background color
    pub fn with_secondary_bg_hover(mut self, color: u32) -> Self {
        self.secondary_bg_hover = color;
        self
    }

    /// Set the secondary button active background color
    pub fn with_secondary_bg_active(mut self, color: u32) -> Self {
        self.secondary_bg_active = color;
        self
    }

    /// Set the secondary button border color
    pub fn with_secondary_border(mut self, color: u32) -> Self {
        self.secondary_border = color;
        self
    }

    // Tab builders

    /// Set the tab hover background color
    pub fn with_bg_tab_hover(mut self, color: u32) -> Self {
        self.bg_tab_hover = color;
        self
    }

    /// Set the active tab border color
    pub fn with_border_tab_active(mut self, color: u32) -> Self {
        self.border_tab_active = color;
        self
    }

    // Delete button builders

    /// Set the delete button background color
    pub fn with_delete_bg(mut self, color: u32) -> Self {
        self.delete_bg = color;
        self
    }

    /// Set the delete button hover background color
    pub fn with_delete_bg_hover(mut self, color: u32) -> Self {
        self.delete_bg_hover = color;
        self
    }

    // Path display builder

    /// Set the path display hover background color
    pub fn with_bg_path_hover(mut self, color: u32) -> Self {
        self.bg_path_hover = color;
        self
    }
}

/// Get the theme from context, falling back to dark theme if not set
pub fn get_theme(cx: &gpui::App) -> Theme {
    cx.try_global::<Theme>()
        .copied()
        .unwrap_or_else(Theme::dark)
}

/// Get the theme from context or use a custom theme
pub fn get_theme_or(cx: &gpui::App, custom: Option<&Theme>) -> Theme {
    custom.copied().unwrap_or_else(|| get_theme(cx))
}
