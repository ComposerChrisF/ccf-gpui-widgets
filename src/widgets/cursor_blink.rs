//! Cursor blink utility
//!
//! Provides reusable cursor blink timing logic used by text input widgets.

use std::time::{Duration, Instant};

/// Default blink period in milliseconds
const BLINK_PERIOD_MS: u128 = 530;

/// Tracks cursor blink state for text inputs
#[derive(Clone)]
pub struct CursorBlink {
    /// Time when cursor was last moved
    last_moved: Instant,
}

impl Default for CursorBlink {
    fn default() -> Self {
        Self::new()
    }
}

impl CursorBlink {
    /// Create a new cursor blink tracker
    pub fn new() -> Self {
        Self {
            last_moved: Instant::now(),
        }
    }

    /// Reset the blink timer (call when cursor moves)
    pub fn reset(&mut self) {
        self.last_moved = Instant::now();
    }

    /// Check if cursor should be visible based on blink cycle
    pub fn is_visible(&self) -> bool {
        let elapsed = self.last_moved.elapsed();
        let cycle_position = elapsed.as_millis() % (BLINK_PERIOD_MS * 2);
        cycle_position < BLINK_PERIOD_MS
    }

    /// Get the blink period duration (for timer setup)
    pub fn blink_period() -> Duration {
        Duration::from_millis(BLINK_PERIOD_MS as u64)
    }
}
