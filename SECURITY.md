# Security

This document describes the security model and considerations for `ccf-gpui-widgets`.

## Overview

This is a pure UI widget library with no network I/O, no command execution, and no unsafe code. All memory safety is guaranteed by Rust's type system.

## Password Handling

When the `secure-password` feature is enabled, the library provides strong password security:

### Memory Zeroization
- Password data is wrapped in `Zeroizing<String>` from the `zeroize` crate
- Memory is automatically zeroed when passwords go out of scope
- Prevents password data from lingering in memory after use

### API Boundaries
- `SecretString` from the `secrecy` crate is used at API boundaries
- Prevents accidental logging or serialization of passwords
- Debug output shows `[REDACTED]` instead of actual content

### Clipboard Restrictions
- **Copy is disabled** for password fields
- Cut operations delete content without copying to clipboard
- Prevents passwords from being exposed via clipboard

### Feature Flag

For production password handling, enable the `secure-password` feature:

```toml
[dependencies]
ccf-gpui-widgets = { version = "0.1", features = ["secure-password"] }
```

Without this feature, password fields still mask input visually but do not provide memory zeroization.

## Path Handling

### UI Display Only

The path utilities in this library are designed for **UI display purposes**:
- Color-coding existing vs non-existing path segments
- Showing tilde (`~`) for home directory
- Displaying canonical paths

### Consumer Responsibility

**Important:** Path validation in this library does not constitute a security boundary. Consumers should:

1. Perform their own path validation before file operations
2. Validate paths against allowed directories if sandboxing is required
3. Handle path traversal (`..`) according to their security requirements
4. Verify file permissions before read/write operations

### Known Behaviors

- `to_string_lossy()` replaces invalid UTF-8 with replacement characters
- Existence checks (`is_file()`, `is_dir()`) may race with filesystem changes
- Partial canonicalization only resolves existing path prefixes

## Input Validation

| Widget | Validation Approach |
|--------|---------------------|
| TextInput | Optional filter function via `filter()` builder |
| PasswordInput | None (accepts any input) |
| NumberStepper | `parse::<f64>()` with fallback to previous value |
| ColorSwatch | Hex/RGB parsing validation |
| FilePicker | Existence checks for UI feedback only |

Consumers should implement additional validation as needed for their use case.

## Dependencies

All dependencies are well-maintained crates from the Rust ecosystem:

| Dependency | Purpose | Risk |
|------------|---------|------|
| gpui | Core UI framework | Low |
| smol | Async runtime | Low |
| rfd | Native file dialogs (optional) | Low |
| dirs | Home directory lookup (optional) | Low |
| secrecy | Secret handling (optional) | Low |
| zeroize | Memory zeroization (optional) | Low |

### Auditing Dependencies

Periodically check for known vulnerabilities:

```bash
cargo install cargo-audit
cargo audit
```

## What This Library Does NOT Do

- **No network I/O** - Pure UI components
- **No command execution** - User input never reaches a shell
- **No file operations** - Only displays paths; consumers handle actual I/O
- **No serialization** - No serde usage that could lead to deserialization attacks
- **No unsafe code** - Entire codebase uses safe Rust

## Reporting Security Issues

If you discover a security vulnerability, please report it via:
- GitHub Issues: https://github.com/ccf-tools/ccf-gpui-widgets/issues
- Email: (add contact email if applicable)

Please include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)
