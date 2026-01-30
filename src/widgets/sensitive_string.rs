//! Sensitive string storage with zeroization
//!
//! This module provides `SensitiveString`, a secure string storage type that:
//! - Wraps content in `Zeroizing<String>` for automatic memory zeroization on drop
//! - Implements `ContentStorage` for use with `EditingCore`
//! - Provides redacted `Debug` output
//! - Can be converted to `SecretString` for API boundaries

use std::fmt;
use std::ops::Range;

use secrecy::{ExposeSecret, SecretString};
use zeroize::Zeroizing;

use super::editing_core::ContentStorage;

/// A secure string storage type that zeroizes memory on drop
///
/// This type is designed for storing sensitive data like passwords in memory.
/// It automatically zeroizes (overwrites with zeros) its content when dropped,
/// preventing sensitive data from lingering in memory.
///
/// The `Debug` implementation is intentionally redacted to prevent accidental
/// logging of sensitive data.
pub struct SensitiveString {
    inner: Zeroizing<String>,
}

impl Default for SensitiveString {
    fn default() -> Self {
        Self {
            inner: Zeroizing::new(String::new()),
        }
    }
}

impl Clone for SensitiveString {
    fn clone(&self) -> Self {
        Self {
            inner: Zeroizing::new((*self.inner).clone()),
        }
    }
}

impl fmt::Debug for SensitiveString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SensitiveString")
            .field("inner", &"[REDACTED]")
            .finish()
    }
}

#[allow(dead_code)]
impl SensitiveString {
    /// Create a new empty sensitive string
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a sensitive string from a value
    pub fn from_str(value: &str) -> Self {
        Self {
            inner: Zeroizing::new(value.to_string()),
        }
    }

    /// Create a sensitive string from a SecretString
    pub fn from_secret(secret: &SecretString) -> Self {
        Self {
            inner: Zeroizing::new(secret.expose_secret().to_string()),
        }
    }

    /// Convert to a SecretString for API boundaries
    ///
    /// This creates a new `SecretString` that wraps the content.
    /// Use this when passing password values to external APIs.
    pub fn to_secret_string(&self) -> SecretString {
        SecretString::from(self.inner.as_str().to_string())
    }

    /// Get the length in bytes
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl ContentStorage for SensitiveString {
    fn as_str(&self) -> &str {
        &self.inner
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    fn set(&mut self, value: &str) {
        // Clear existing content first (will be zeroized when dropped)
        let old = std::mem::take(&mut self.inner);
        drop(old); // Explicit drop for clarity - zeroizes the old content
        self.inner = Zeroizing::new(value.to_string());
    }

    fn insert_str(&mut self, pos: usize, text: &str) {
        // Get mutable access to the inner string
        self.inner.insert_str(pos, text);
    }

    fn remove_range(&mut self, range: Range<usize>) {
        self.inner.replace_range(range, "");
    }

    fn replace_range(&mut self, range: Range<usize>, replacement: &str) {
        self.inner.replace_range(range, replacement);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let mut s = SensitiveString::new();
        assert!(s.is_empty());

        s.set("password123");
        assert_eq!(s.as_str(), "password123");
        assert_eq!(s.len(), 11);
    }

    #[test]
    fn test_insert() {
        let mut s = SensitiveString::from_str("hello");
        s.insert_str(5, " world");
        assert_eq!(s.as_str(), "hello world");
    }

    #[test]
    fn test_remove_range() {
        let mut s = SensitiveString::from_str("hello world");
        s.remove_range(5..11);
        assert_eq!(s.as_str(), "hello");
    }

    #[test]
    fn test_replace_range() {
        let mut s = SensitiveString::from_str("hello world");
        s.replace_range(0..5, "hi");
        assert_eq!(s.as_str(), "hi world");
    }

    #[test]
    fn test_to_secret_string() {
        let s = SensitiveString::from_str("secret");
        let secret = s.to_secret_string();
        assert_eq!(secret.expose_secret(), "secret");
    }

    #[test]
    fn test_from_secret() {
        let secret = SecretString::from("password".to_string());
        let s = SensitiveString::from_secret(&secret);
        assert_eq!(s.as_str(), "password");
    }

    #[test]
    fn test_debug_redaction() {
        let s = SensitiveString::from_str("secret");
        let debug = format!("{:?}", s);
        assert!(!debug.contains("secret"));
        assert!(debug.contains("REDACTED"));
    }

    #[test]
    fn test_clone() {
        let s1 = SensitiveString::from_str("password");
        let s2 = s1.clone();
        assert_eq!(s1.as_str(), s2.as_str());
    }
}
