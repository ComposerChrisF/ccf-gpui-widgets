//! Core editing logic for text input widgets
//!
//! This module provides `EditingCore<S>`, a generic editing engine that handles
//! cursor movement, selection, and text manipulation. The storage type `S` must
//! implement `ContentStorage`, allowing different backing stores (e.g., `String`
//! for regular text, or a zeroizing type for sensitive data).

use std::ops::Range;

/// Trait for content storage backends
///
/// This trait abstracts over the string storage, allowing different implementations
/// for regular text (`String`) and sensitive data (`Zeroizing<String>`).
#[allow(dead_code)]
pub trait ContentStorage: Default {
    /// Get the content as a string slice
    fn as_str(&self) -> &str;

    /// Get the length in bytes
    fn len(&self) -> usize {
        self.as_str().len()
    }

    /// Check if empty
    fn is_empty(&self) -> bool {
        self.as_str().is_empty()
    }

    /// Set the content from a string slice
    fn set(&mut self, value: &str);

    /// Insert a string at the given byte position
    fn insert_str(&mut self, pos: usize, text: &str);

    /// Remove a range of bytes
    fn remove_range(&mut self, range: Range<usize>);

    /// Replace a range with new text
    fn replace_range(&mut self, range: Range<usize>, replacement: &str);

    /// Get a substring
    fn get(&self, range: Range<usize>) -> Option<&str> {
        self.as_str().get(range)
    }

    /// Iterate over char indices
    fn char_indices(&self) -> std::str::CharIndices<'_> {
        self.as_str().char_indices()
    }

    /// Count characters
    fn chars_count(&self) -> usize {
        self.as_str().chars().count()
    }
}

impl ContentStorage for String {
    fn as_str(&self) -> &str {
        self
    }

    fn set(&mut self, value: &str) {
        self.clear();
        self.push_str(value);
    }

    fn insert_str(&mut self, pos: usize, text: &str) {
        String::insert_str(self, pos, text);
    }

    fn remove_range(&mut self, range: Range<usize>) {
        self.replace_range(range, "");
    }

    fn replace_range(&mut self, range: Range<usize>, replacement: &str) {
        String::replace_range(self, range, replacement);
    }
}

/// Core editing state and operations
///
/// This struct handles all the cursor, selection, and text manipulation logic
/// independent of the UI framework. It's generic over the storage type to support
/// both regular strings and secure/zeroizing storage for passwords.
pub struct EditingCore<S: ContentStorage> {
    /// The text content
    content: S,
    /// Cursor position (byte index into content)
    cursor: usize,
    /// Selection range (start, end) where start <= end
    selection: Option<(usize, usize)>,
    /// The anchor point for selection extension
    selection_anchor: Option<usize>,
    /// Whether this input masks content (for password fields)
    masked: bool,
}

impl<S: ContentStorage> Default for EditingCore<S> {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl<S: ContentStorage> EditingCore<S> {
    /// Create a new editing core with empty content
    pub fn new() -> Self {
        Self {
            content: S::default(),
            cursor: 0,
            selection: None,
            selection_anchor: None,
            masked: false,
        }
    }

    /// Create with initial content
    #[must_use]
    pub fn with_content(mut self, content: &str) -> Self {
        self.content.set(content);
        self.cursor = self.content.len();
        self
    }

    /// Set masked mode (for password input)
    #[must_use]
    pub fn with_masked(mut self, masked: bool) -> Self {
        self.masked = masked;
        self
    }

    // ========================================================================
    // Getters
    // ========================================================================

    /// Get the content as a string slice
    pub fn content(&self) -> &str {
        self.content.as_str()
    }

    /// Get the cursor position
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Get the selection range
    pub fn selection(&self) -> Option<(usize, usize)> {
        self.selection
    }

    /// Check if masked mode is enabled
    pub fn is_masked(&self) -> bool {
        self.masked
    }

    /// Get selected text
    pub fn selected_text(&self) -> Option<&str> {
        if let Some((start, end)) = self.selection {
            if start != end {
                return self.content.get(start..end);
            }
        }
        None
    }

    // ========================================================================
    // Setters
    // ========================================================================

    /// Set the content, moving cursor to end
    pub fn set_content(&mut self, value: &str) {
        self.content.set(value);
        self.cursor = self.content.len();
        self.selection = None;
        self.selection_anchor = None;
    }

    /// Set cursor position (clamped to valid range)
    pub fn set_cursor(&mut self, pos: usize) {
        self.cursor = pos.min(self.content.len());
    }

    /// Set masked mode
    pub fn set_masked(&mut self, masked: bool) {
        self.masked = masked;
    }

    /// Clear selection state
    pub fn clear_selection(&mut self) {
        self.selection = None;
        self.selection_anchor = None;
    }

    // ========================================================================
    // Text modification
    // ========================================================================

    /// Delete selected text and return whether deletion occurred
    pub fn delete_selection(&mut self) -> bool {
        if let Some((start, end)) = self.selection {
            if start != end {
                self.content.remove_range(start..end);
                self.cursor = start;
                self.selection = None;
                self.selection_anchor = None;
                return true;
            }
        }
        false
    }

    /// Insert text at cursor (replacing selection if any)
    /// Returns true if content changed
    pub fn insert_text(&mut self, text: &str) -> bool {
        self.delete_selection();
        self.content.insert_str(self.cursor, text);
        self.cursor += text.len();
        true
    }

    /// Delete character before cursor
    /// Returns true if content changed
    pub fn delete_backward(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }
        if self.cursor > 0 {
            let prev = self.prev_char_boundary(self.cursor);
            self.content.remove_range(prev..self.cursor);
            self.cursor = prev;
            return true;
        }
        false
    }

    /// Delete character after cursor
    /// Returns true if content changed
    pub fn delete_forward(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }
        if self.cursor < self.content.len() {
            let next = self.next_char_boundary(self.cursor);
            self.content.remove_range(self.cursor..next);
            return true;
        }
        false
    }

    /// Delete word before cursor (falls back to single char when masked)
    /// Returns true if content changed
    pub fn delete_word_backward(&mut self) -> bool {
        if self.masked {
            return self.delete_backward();
        }
        if self.delete_selection() {
            return true;
        }
        if self.cursor > 0 {
            let prev = self.prev_word_boundary(self.cursor);
            self.content.remove_range(prev..self.cursor);
            self.cursor = prev;
            return true;
        }
        false
    }

    /// Delete word after cursor (falls back to single char when masked)
    /// Returns true if content changed
    pub fn delete_word_forward(&mut self) -> bool {
        if self.masked {
            return self.delete_forward();
        }
        if self.delete_selection() {
            return true;
        }
        if self.cursor < self.content.len() {
            let next = self.next_word_boundary(self.cursor);
            self.content.remove_range(self.cursor..next);
            return true;
        }
        false
    }

    // ========================================================================
    // Cursor movement
    // ========================================================================

    /// Move cursor left by one character
    pub fn move_left(&mut self) {
        self.clear_selection();
        if self.cursor > 0 {
            self.cursor = self.prev_char_boundary(self.cursor);
        }
    }

    /// Move cursor right by one character
    pub fn move_right(&mut self) {
        self.clear_selection();
        if self.cursor < self.content.len() {
            self.cursor = self.next_char_boundary(self.cursor);
        }
    }

    /// Move cursor to start
    pub fn move_to_start(&mut self) {
        self.clear_selection();
        self.cursor = 0;
    }

    /// Move cursor to end
    pub fn move_to_end(&mut self) {
        self.clear_selection();
        self.cursor = self.content.len();
    }

    /// Move cursor to previous word (falls back to single char when masked)
    pub fn move_word_left(&mut self) {
        if self.masked {
            self.move_left();
            return;
        }
        self.clear_selection();
        self.cursor = self.prev_word_boundary(self.cursor);
    }

    /// Move cursor to next word (falls back to single char when masked)
    pub fn move_word_right(&mut self) {
        if self.masked {
            self.move_right();
            return;
        }
        self.clear_selection();
        self.cursor = self.next_word_boundary(self.cursor);
    }

    // ========================================================================
    // Selection
    // ========================================================================

    /// Ensure selection anchor is set
    fn ensure_selection_anchor(&mut self) {
        if self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        }
    }

    /// Update selection from anchor to cursor
    fn update_selection(&mut self) {
        if let Some(anchor) = self.selection_anchor {
            if anchor == self.cursor {
                self.selection = None;
            } else {
                self.selection = Some((anchor.min(self.cursor), anchor.max(self.cursor)));
            }
        }
    }

    /// Extend selection left by one character
    pub fn select_left(&mut self) {
        if self.cursor > 0 {
            self.ensure_selection_anchor();
            self.cursor = self.prev_char_boundary(self.cursor);
            self.update_selection();
        }
    }

    /// Extend selection right by one character
    pub fn select_right(&mut self) {
        if self.cursor < self.content.len() {
            self.ensure_selection_anchor();
            self.cursor = self.next_char_boundary(self.cursor);
            self.update_selection();
        }
    }

    /// Extend selection left by one word (falls back to single char when masked)
    pub fn select_word_left(&mut self) {
        if self.masked {
            self.select_left();
            return;
        }
        if self.cursor > 0 {
            self.ensure_selection_anchor();
            self.cursor = self.prev_word_boundary(self.cursor);
            self.update_selection();
        }
    }

    /// Extend selection right by one word (falls back to single char when masked)
    pub fn select_word_right(&mut self) {
        if self.masked {
            self.select_right();
            return;
        }
        if self.cursor < self.content.len() {
            self.ensure_selection_anchor();
            self.cursor = self.next_word_boundary(self.cursor);
            self.update_selection();
        }
    }

    /// Select to start of line
    pub fn select_to_start(&mut self) {
        self.ensure_selection_anchor();
        self.cursor = 0;
        self.update_selection();
    }

    /// Select to end of line
    pub fn select_to_end(&mut self) {
        self.ensure_selection_anchor();
        self.cursor = self.content.len();
        self.update_selection();
    }

    /// Select all text
    pub fn select_all(&mut self) {
        self.selection_anchor = Some(0);
        self.cursor = self.content.len();
        self.selection = Some((0, self.content.len()));
    }

    /// Set selection range and update cursor/anchor accordingly
    pub fn set_selection(&mut self, start: usize, end: usize) {
        let start = start.min(self.content.len());
        let end = end.min(self.content.len());
        if start == end {
            self.selection = None;
            self.selection_anchor = None;
            self.cursor = start;
        } else {
            let (min, max) = (start.min(end), start.max(end));
            self.selection = Some((min, max));
            self.selection_anchor = Some(min);
            self.cursor = max;
        }
    }

    /// Start selection at current cursor position (for shift+click)
    pub fn start_selection_from_cursor(&mut self) {
        self.ensure_selection_anchor();
    }

    /// Extend selection to new cursor position
    pub fn extend_selection_to(&mut self, pos: usize) {
        self.cursor = pos.min(self.content.len());
        self.update_selection();
    }

    // ========================================================================
    // Character boundary helpers
    // ========================================================================

    /// Find the previous character boundary
    pub fn prev_char_boundary(&self, pos: usize) -> usize {
        if pos == 0 {
            return 0;
        }
        self.content
            .as_str()[..pos]
            .char_indices()
            .last()
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    /// Find the next character boundary
    pub fn next_char_boundary(&self, pos: usize) -> usize {
        if pos >= self.content.len() {
            return self.content.len();
        }
        self.content.as_str()[pos..]
            .char_indices()
            .nth(1)
            .map(|(i, _)| pos + i)
            .unwrap_or(self.content.len())
    }

    /// Find the start of the previous word
    /// Uses an iterator-based approach to avoid Vec allocation
    pub fn prev_word_boundary(&self, pos: usize) -> usize {
        if pos == 0 {
            return 0;
        }

        let s = &self.content.as_str()[..pos];
        if s.is_empty() {
            return 0;
        }

        // Iterate backwards by collecting positions, then walking back
        // Find the last word start before pos
        let mut last_word_start = 0;
        let mut in_word = false;
        let mut last_non_alnum_after_word = None;

        for (i, c) in s.char_indices() {
            if c.is_alphanumeric() && !in_word {
                last_word_start = i;
                in_word = true;
            } else if !c.is_alphanumeric() {
                if in_word {
                    last_non_alnum_after_word = Some(i);
                }
                in_word = false;
            }
        }

        // If we're currently in a word (at the end of s), return the start of that word
        if in_word {
            return last_word_start;
        }

        // Otherwise, we're in non-alphanumeric chars, return start of previous word
        // If there was a word before, return its start
        if last_non_alnum_after_word.is_some() {
            return last_word_start;
        }

        // No word found, return 0
        0
    }

    /// Find the end of the next word
    /// Uses an iterator-based approach to avoid Vec allocation
    pub fn next_word_boundary(&self, pos: usize) -> usize {
        if pos >= self.content.len() {
            return self.content.len();
        }

        let s = &self.content.as_str()[pos..];
        if s.is_empty() {
            return self.content.len();
        }

        let mut in_word = false;

        for (i, c) in s.char_indices() {
            if c.is_alphanumeric() {
                in_word = true;
            } else if in_word {
                return pos + i; // Just exited a word
            }
        }

        // If we went through a word (or any text) and reached the end, return end
        self.content.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_insert() {
        let mut core = EditingCore::<String>::new();
        core.insert_text("hello");
        assert_eq!(core.content(), "hello");
        assert_eq!(core.cursor(), 5);
    }

    #[test]
    fn test_cursor_movement() {
        let mut core = EditingCore::<String>::new().with_content("hello");
        assert_eq!(core.cursor(), 5);

        core.move_to_start();
        assert_eq!(core.cursor(), 0);

        core.move_right();
        assert_eq!(core.cursor(), 1);

        core.move_to_end();
        assert_eq!(core.cursor(), 5);

        core.move_left();
        assert_eq!(core.cursor(), 4);
    }

    #[test]
    fn test_selection() {
        let mut core = EditingCore::<String>::new().with_content("hello world");

        core.select_all();
        assert_eq!(core.selection(), Some((0, 11)));
        assert_eq!(core.selected_text(), Some("hello world"));

        core.clear_selection();
        assert_eq!(core.selection(), None);
    }

    #[test]
    fn test_delete_selection() {
        let mut core = EditingCore::<String>::new().with_content("hello world");

        core.set_selection(0, 6); // Select "hello "
        assert!(core.delete_selection());
        assert_eq!(core.content(), "world");
        assert_eq!(core.cursor(), 0);
    }

    #[test]
    fn test_insert_replaces_selection() {
        let mut core = EditingCore::<String>::new().with_content("hello world");

        core.set_selection(0, 5); // Select "hello"
        core.insert_text("hi");
        assert_eq!(core.content(), "hi world");
    }

    #[test]
    fn test_word_boundaries() {
        let core = EditingCore::<String>::new().with_content("hello world test");

        assert_eq!(core.next_word_boundary(0), 5); // "hello" ends at 5
        assert_eq!(core.next_word_boundary(5), 11); // "world" ends at 11
        assert_eq!(core.prev_word_boundary(11), 6); // "world" starts at 6
        assert_eq!(core.prev_word_boundary(6), 0); // "hello" starts at 0
    }

    #[test]
    fn test_masked_mode_disables_word_operations() {
        let mut core = EditingCore::<String>::new()
            .with_content("hello world")
            .with_masked(true);

        core.move_to_start();
        core.move_word_right(); // Should move one char, not one word
        assert_eq!(core.cursor(), 1);

        core.move_to_end();
        core.move_word_left(); // Should move one char, not one word
        assert_eq!(core.cursor(), 10);
    }

    #[test]
    fn test_delete_backward() {
        let mut core = EditingCore::<String>::new().with_content("hello");

        assert!(core.delete_backward());
        assert_eq!(core.content(), "hell");
        assert_eq!(core.cursor(), 4);
    }

    #[test]
    fn test_delete_forward() {
        let mut core = EditingCore::<String>::new().with_content("hello");
        core.move_to_start();

        assert!(core.delete_forward());
        assert_eq!(core.content(), "ello");
        assert_eq!(core.cursor(), 0);
    }

    #[test]
    fn test_unicode_handling() {
        let mut core = EditingCore::<String>::new().with_content("cafe\u{0301}"); // café with combining accent

        // Should handle multi-byte characters correctly
        core.move_left();
        assert!(core.cursor() < 5); // Cursor moved before the combining character
    }
}
