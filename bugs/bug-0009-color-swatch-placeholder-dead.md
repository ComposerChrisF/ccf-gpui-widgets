# bug-0009: ColorSwatch `placeholder()` builder is dead — the value is stored but never applied

**Severity:** low
**Fix side:** CODE
**Component:** src/widgets/color_swatch.rs:252-255 (builder), 129 (field), 181 (hex input constructed with hard-coded placeholder); src/widgets/text_input.rs:400 (`set_placeholder`, unused here)
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

The `placeholder()` builder (lines `252-255`) writes `self.placeholder`, but the field is read nowhere.  A grep confirms the only occurrences are the field declaration (line `129`, defaulted to `"#000000"`), the builder, and nothing else.  The child hex `TextInput` is constructed with a hard-coded `"#000000"` placeholder (line `181`) and never updated.

`ColorSwatch::new(cx).placeholder("Pick a color…")` therefore compiles, is public API, and has zero effect.

## Reproduction (test-ready)

The dead store is verifiable by inspection/grep alone.  Rendered confirmation needs GPUI test context:

1. Build `ColorSwatch::new(cx).placeholder("custom")`.
2. Clear the hex field.
3. Expected: displayed placeholder is `custom`.
4. Actual: displayed placeholder is `#000000`.

## Suggested Fix

Piggyback on the first-render sync block that bug-0014 repairs: inside the `needs_input_sync` block (lines `562-570`), call `input.set_placeholder(self.placeholder.clone(), cx)` — `TextInput` already has the setter (text_input.rs:400) — and set `self.needs_input_sync = true` in the `placeholder()` builder so the block is armed.

## Why the Fix Works

It routes the stored builder value to the only component that renders placeholders, using an existing setter and an existing sync mechanism — no new plumbing.

## Fix Ordering / Related

Shares the `needs_input_sync` builder-sync mechanism with bug-0014 (`with_enabled` never arming the same block) — fix the two together.
