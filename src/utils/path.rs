//! Path utilities
//!
//! Provides path canonicalization and validation helpers for file/directory pickers.
//!
//! # Example
//!
//! ```ignore
//! use ccf_gpui_widgets::utils::path::{parse_path, expand_tilde};
//!
//! // Parse and canonicalize a path
//! let info = parse_path("~/Documents/output.txt");
//! if info.fully_exists() {
//!     println!("File exists at: {}", info.full_path_string());
//! } else {
//!     println!("Existing portion: {:?}", info.existing_canonical);
//!     println!("Non-existing suffix: {:?}", info.non_existing_suffix);
//! }
//! ```

use std::path::{Path, PathBuf};

/// Result of parsing and canonicalizing a user-provided path
#[derive(Debug, Clone)]
pub struct PathInfo {
    /// The canonicalized portion that exists on disk
    pub existing_canonical: PathBuf,
    /// The non-existing suffix (empty if entire path exists)
    pub non_existing_suffix: PathBuf,
    /// The full path (canonical + suffix)
    pub full_path: PathBuf,
}

impl PathInfo {
    /// Returns true if the entire path exists
    pub fn fully_exists(&self) -> bool {
        self.non_existing_suffix.as_os_str().is_empty()
    }

    /// Returns the full path as a string
    pub fn full_path_string(&self) -> String {
        self.full_path.to_string_lossy().to_string()
    }

    /// Create an empty PathInfo
    pub fn empty() -> Self {
        Self {
            existing_canonical: PathBuf::new(),
            non_existing_suffix: PathBuf::new(),
            full_path: PathBuf::new(),
        }
    }
}

impl Default for PathInfo {
    fn default() -> Self {
        Self::empty()
    }
}

/// Parse and canonicalize a user-provided path (which may be relative or non-canonical)
///
/// Returns a PathInfo struct containing:
/// - existing_canonical: The longest prefix that exists and can be canonicalized
/// - non_existing_suffix: Any remaining path components that don't exist
/// - full_path: The complete reconstructed path
///
/// # Examples
///
/// ```ignore
/// // Existing file
/// let info = parse_path("/Users/foo/file.txt");
/// assert!(info.fully_exists());
///
/// // Non-existing output file
/// let info = parse_path("/Users/foo/output.txt");
/// // info.existing_canonical = /Users/foo (if it exists)
/// // info.non_existing_suffix = output.txt
///
/// // Relative path
/// let info = parse_path("../data/file.txt");
/// // Resolves relative to current directory
/// ```
pub fn parse_path(input: &str) -> PathInfo {
    if input.is_empty() {
        return PathInfo::empty();
    }

    // Expand ~ to home directory
    let expanded = expand_tilde(input);
    let path = Path::new(&expanded);

    // Convert to absolute path if relative
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        // Resolve relative to current directory
        std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .unwrap_or_else(|_| path.to_path_buf())
    };

    // Find the longest existing prefix
    let (existing, suffix) = find_existing_prefix(&absolute);

    // Try to canonicalize the existing portion
    let canonical = existing
        .canonicalize()
        .unwrap_or_else(|_| existing.clone());

    // Reconstruct full path
    let full = if suffix.as_os_str().is_empty() {
        canonical.clone()
    } else {
        canonical.join(&suffix)
    };

    PathInfo {
        existing_canonical: canonical,
        non_existing_suffix: suffix,
        full_path: full,
    }
}

/// Expand tilde (~) to home directory
#[cfg(feature = "file-picker")]
pub fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") || path == "~" {
        if let Some(home) = dirs::home_dir() {
            if path == "~" {
                return home.to_string_lossy().to_string();
            } else {
                return home.join(&path[2..]).to_string_lossy().to_string();
            }
        }
    }
    path.to_string()
}

/// Expand tilde (~) to home directory (fallback when dirs is not available)
#[cfg(not(feature = "file-picker"))]
pub fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") || path == "~" {
        if let Ok(home) = std::env::var("HOME") {
            if path == "~" {
                return home;
            } else {
                return format!("{}/{}", home, &path[2..]);
            }
        }
    }
    path.to_string()
}

/// Build a path from components in reverse order
fn build_suffix(components: &[std::ffi::OsString]) -> PathBuf {
    components
        .iter()
        .rev()
        .fold(PathBuf::new(), |acc, comp| acc.join(comp))
}

/// Find the longest prefix of the path that exists on disk
fn find_existing_prefix(path: &Path) -> (PathBuf, PathBuf) {
    let mut current = path.to_path_buf();
    let mut suffix_components = Vec::new();

    loop {
        if current.exists() {
            return (current, build_suffix(&suffix_components));
        }

        match current.file_name() {
            Some(component) => {
                suffix_components.push(component.to_os_string());
                current = current
                    .parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| PathBuf::from("/"));
            }
            None => {
                return (PathBuf::from("/"), build_suffix(&suffix_components));
            }
        }

        if current.as_os_str().is_empty() || current == Path::new("/") {
            return (current, build_suffix(&suffix_components));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_tilde() {
        // Test that ~ expands to something (exact value depends on environment)
        let expanded = expand_tilde("~");
        assert!(!expanded.is_empty());
        assert!(!expanded.starts_with('~') || expanded == "~"); // Either expanded or unchanged

        let with_path = expand_tilde("~/Documents");
        assert!(with_path.ends_with("/Documents") || with_path == "~/Documents");
    }

    #[test]
    fn test_parse_existing_path() {
        let temp = std::env::temp_dir();
        let info = parse_path(temp.to_str().unwrap());
        assert!(info.fully_exists());
        assert_eq!(info.non_existing_suffix, PathBuf::new());
    }

    #[test]
    fn test_parse_non_existing_file() {
        let temp = std::env::temp_dir();
        let non_existing = temp.join("this_file_should_not_exist_12345.txt");
        let info = parse_path(non_existing.to_str().unwrap());

        assert!(!info.fully_exists());
        assert_eq!(info.non_existing_suffix, PathBuf::from("this_file_should_not_exist_12345.txt"));
    }

    #[test]
    fn test_empty_path() {
        let info = parse_path("");
        assert!(info.full_path.as_os_str().is_empty());
    }
}
