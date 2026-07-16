# bug-0046: Path display renders a double slash when the missing suffix hangs off root, and a fake leading slash on the relative fallback

**Severity:** low
**Fix side:** CODE
**Component:** `src/widgets/path_display.rs:61-67` (`add_path_prefix`); consumers at `src/widgets/file_picker.rs:477-487` and `src/widgets/directory_picker.rs:317-329`
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

`add_path_prefix` unconditionally pushes a separator (`src/widgets/path_display.rs:61-67`):

```rust
pub fn add_path_prefix(&mut self, text: &str, color: u32) {
    let start = self.full_text.len();
    self.full_text.push('/');
    self.full_text.push_str(text);
    let end = self.full_text.len();
    self.highlights.push(PathHighlight { start, end, color });
}
```

Two degenerate prefixes break it:

- **Root prefix:** when `existing_canonical` is `/`—any path whose first component does not exist, e.g. `/Volumes-typo/x`—the display composes `add_segment("/")` followed by `add_path_prefix(...)`, rendering `//Volumes-typo…`.  Verified live: existing segment `/` plus a prefix renders `//nonexistent_root_xyz_abc`.
- **Empty prefix:** when `existing_canonical` is empty (relative input while `current_dir()` fails—`src/utils/path.rs:99-105` with the empty-prefix return at `src/utils/path.rs:192-194`), `add_segment("")` is skipped (`add_segment` ignores empty text, `src/widgets/path_display.rs:51-58`) but `add_path_prefix` still prepends `/`, displaying a relative path as if absolute: `/rel/file.txt`.

## Reproduction (test-ready)

Widget-level: a `FilePicker` with value `/nonexistent_root_xyz/file.txt` renders a `full_text` beginning with `//` (needs GPUI test context to render).

Crate-internal unit test on the composition itself—no GPUI machinery needed:

```rust
#[test]
fn path_prefix_after_root_has_single_slash() {
    let mut info = PathDisplayInfo::new();
    info.add_segment("/", 0x00AA_AAAA);
    info.add_path_prefix("nonexistent_root_xyz", 0x00FF_0000);
    assert_eq!(info.full_text, "/nonexistent_root_xyz"); // fails today: "//nonexistent_root_xyz"
}

#[test]
fn path_prefix_on_empty_display_stays_relative() {
    let mut info = PathDisplayInfo::new();
    info.add_path_prefix("rel", 0x00FF_0000);
    assert_eq!(info.full_text, "rel"); // fails today: "/rel"
}
```

## Suggested Fix

In `add_path_prefix`, insert the separator only between two real segments:

```rust
if !self.full_text.is_empty() && !self.full_text.ends_with('/') {
    self.full_text.push('/');
}
```

with `start` still captured before the conditional push, so the highlight span keeps covering the separator when one is inserted.

## Why the Fix Works

The separator is only ever inserted between two real segments, which covers both failure cases: a root prefix already ends with `/`, and an empty prefix means there is nothing to separate from.

## Fix Ordering / Related

Independent.  Related: bug-0040—the degenerate `/` existing prefix is frequently manufactured by `find_existing_prefix` fabricating root, so the reproductions overlap, but the fixes are separate.  bug-0020’s new display branch calls `add_path_prefix` and benefits automatically.
