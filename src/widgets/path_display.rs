//! Shared path display types for file and directory pickers
//!
//! This module contains common types and utilities used by both `FilePicker`
//! and `DirectoryPicker` for displaying path information with color highlighting.

#[cfg(feature = "file-picker")]
use gpui::*;

/// Controls how validation feedback is displayed in file/directory pickers
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ValidationDisplay {
    /// Show colored path segments and explanation message (default)
    #[default]
    Full,
    /// Show colored path segments only, hide explanation message
    ColorsOnly,
    /// Show explanation message only, no path coloring
    MessageOnly,
    /// Hide all validation feedback (no colors, no message)
    Hidden,
}

/// A single highlighted segment of a path display
#[cfg(feature = "file-picker")]
pub(crate) struct PathHighlight {
    pub start: usize,
    pub end: usize,
    pub color: u32,
}

/// Path display information with styled segments and optional explanation
#[cfg(feature = "file-picker")]
pub(crate) struct PathDisplayInfo {
    pub full_text: String,
    pub highlights: Vec<PathHighlight>,
    pub explanation: Option<(String, u32)>,
}

#[cfg(feature = "file-picker")]
impl PathDisplayInfo {
    /// Create a new empty path display info
    pub fn new() -> Self {
        Self {
            full_text: String::new(),
            highlights: Vec::new(),
            explanation: None,
        }
    }

    /// Add a segment of text with the given color
    pub fn add_segment(&mut self, text: &str, color: u32) {
        if !text.is_empty() {
            let start = self.full_text.len();
            self.full_text.push_str(text);
            let end = self.full_text.len();
            self.highlights.push(PathHighlight { start, end, color });
        }
    }

    /// Add a segment with a leading path separator (/)
    pub fn add_path_prefix(&mut self, text: &str, color: u32) {
        let start = self.full_text.len();
        self.full_text.push('/');
        self.full_text.push_str(text);
        let end = self.full_text.len();
        self.highlights.push(PathHighlight { start, end, color });
    }

    /// Set the explanation message with color
    pub fn set_explanation(&mut self, msg: &str, color: u32) {
        self.explanation = Some((msg.to_string(), color));
    }

    /// Convert to a StyledText for rendering
    pub fn to_styled_text(&self) -> StyledText {
        let highlights: Vec<(std::ops::Range<usize>, HighlightStyle)> = self
            .highlights
            .iter()
            .map(|h| (h.start..h.end, HighlightStyle::color(rgb(h.color).into())))
            .collect();

        StyledText::new(self.full_text.clone()).with_highlights(highlights)
    }

    /// Check if the display info is empty
    pub fn is_empty(&self) -> bool {
        self.full_text.is_empty()
    }
}

#[cfg(feature = "file-picker")]
impl Default for PathDisplayInfo {
    fn default() -> Self {
        Self::new()
    }
}
