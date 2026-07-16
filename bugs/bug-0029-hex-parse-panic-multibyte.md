# bug-0029: `Rgb::from_hex` / `Rgba::from_hex` panic on multi-byte UTF-8 input — reachable from every keystroke in the ColorSwatch hex field

**Severity:** high
**Fix side:** CODE
**Component:** src/utils/color.rs:34-36 and 40-42 (`Rgb::from_hex`), 135-138 and 142-145 (`Rgba::from_hex`); reached from src/widgets/color_swatch.rs:350 (`parse_color` on every `TextInputEvent::Change`)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

Both `from_hex` functions dispatch on `hex.len()` — a _byte_ length — and then byte-slice the `&str`:

```rust
// src/utils/color.rs:34-36 (3-digit arm; the 6-digit and Rgba 4/8-digit arms are the same shape)
let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
```

If a slice boundary lands mid-codepoint, the slice panics.  `Rgb::from_hex("é3")` — `é` is 2 bytes, total 3 bytes, so it enters the 3-digit arm — panics with `byte index 1 is not a char boundary`.  `Rgb::from_hex("€abc")` (6 bytes) panics in the 6-digit arm.  `Rgba::from_hex("€a")` (4 bytes) panics in the 4-digit arm.  Verified by running the exact functions.

This is reachable from ordinary typing: ColorSwatch’s hex `TextInput` has no `input_filter` (text_input.rs:226 defaults it to `None`; color_swatch.rs:179-183 sets none), and `handle_input_change` calls `parse_color` on every keystroke (color_swatch.rs:350).  Typing `é3` — or pasting any string whose byte length is 3, 4, 6, or 8 with a straddling multi-byte character — crashes the whole application.

## Reproduction (test-ready)

Pure unit test, no GPUI machinery:

```rust
assert_eq!(Rgb::from_hex("é3"), None);    // today: PANIC — byte index 1 is not a char boundary
assert_eq!(Rgb::from_hex("€abc"), None);  // today: PANIC in the 6-digit arm
assert_eq!(Rgba::from_hex("€a"), None);   // today: PANIC in the 4-digit arm
```

In-app: focus the ColorSwatch hex field, select-all, type `é3` — the application aborts.

## Suggested Fix

At the top of both functions, immediately after `trim_start_matches('#')`:

```rust
if !hex.is_ascii() {
    return None;
}
```

Alternatively, replace every direct slice with the checked form `hex.get(a..b)?`.

## Why the Fix Works

All valid hex digits are ASCII, so rejecting non-ASCII input loses nothing — and once the string is known to be ASCII, every subsequent byte-slice is guaranteed to land on a char boundary, so the panic path is gone.

## Fix Ordering / Related

Independent.  The in-app trigger path (`handle_input_change` → `parse_color`) is the same code reworked by bug-0006/bug-0007, but this fix lives entirely in `color.rs` and can land alone.
