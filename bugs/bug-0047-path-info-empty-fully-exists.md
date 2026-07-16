# bug-0047: `parse_path("")` reports the empty path as fully existing

**Severity:** low
**Fix side:** CODE
**Component:** `src/utils/path.rs:35-37` (`fully_exists`), `src/utils/path.rs:44-51` (`PathInfo::empty`), `src/utils/path.rs:83-86` (empty-input early return)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

`fully_exists()` is defined purely as “no non-existing suffix” (`src/utils/path.rs:35-37`):

```rust
pub fn fully_exists(&self) -> bool {
    self.non_existing_suffix.as_os_str().is_empty()
}
```

`parse_path("")` returns `PathInfo::empty()` (`src/utils/path.rs:83-86`), whose suffix is empty—so the empty path “fully exists”.  The module’s own doc example (`src/utils/path.rs:12-14`) is:

```rust
if info.fully_exists() {
    println!("File exists at: {}", info.full_path_string());
}
```

which would print an empty path as an existing file.  The widgets happen to guard with `value.is_empty()` before calling `parse_path`, but this is a public exported utility (`src/utils/mod.rs:8`), so any consumer following the documented pattern hits it.

The trailing-`..` case compounds it (see bug-0040): `parse_path("<missing>/..")` yields `full_path == "/"` with `fully_exists() == true`.  Both behaviors verified live.

## Reproduction (test-ready)

Plain unit test—no GPUI machinery needed:

```rust
#[test]
fn empty_path_does_not_fully_exist() {
    assert!(!parse_path("").fully_exists()); // fails today
}
```

The existing test at `src/utils/path.rs:234-238` checks only that `full_path` is empty, sidestepping this.

## Suggested Fix

Require a non-empty path in `fully_exists()`:

```rust
pub fn fully_exists(&self) -> bool {
    !self.full_path.as_os_str().is_empty() && self.non_existing_suffix.as_os_str().is_empty()
}
```

and document on `parse_path` that an empty input yields a “nothing” `PathInfo` for which `fully_exists()` is `false`.

## Why the Fix Works

An empty path can then never satisfy the exists branch of any caller, matching the obvious reading of the method name.

## Fix Ordering / Related

Independent one-liner.  Related: bug-0040 removes the trailing-`..` route to a spurious `fully_exists() == true` at `/`; this fix covers the empty-input case, and both are needed.
