# bug-0067: SensitiveString’s zeroization guarantee is leaky — growth reallocation frees password bytes unzeroized

**Severity:** medium
**Fix side:** CODE
**Component:** `src/widgets/sensitive_string.rs:113-124` (`insert_str`/`remove_range`/`replace_range`); `src/widgets/password_input.rs:279-285` (`display_content`), `333-337` (`shape_line`); doc component: `SECURITY.md:14-16` and the module doc at `sensitive_string.rs:1-24`
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

## Description

`SECURITY.md` claims (lines 14–16):

> - Password data is wrapped in `Zeroizing<String>` from the `zeroize` crate
> - Memory is automatically zeroed when passwords go out of scope
> - Prevents password data from lingering in memory after use

and the module doc says the type prevents “sensitive data from lingering in memory”.  But `insert_str` mutates the inner `String` in place:

```rust
// src/widgets/sensitive_string.rs:113-116
fn insert_str(&mut self, pos: usize, text: &str) {
    // Get mutable access to the inner string
    self.inner.insert_str(pos, text);
}
```

Every keystroke that exceeds the current capacity triggers `String`’s internal `RawVec::grow` → realloc, which may move the buffer and free the old allocation **without zeroization** — `Zeroizing` only zeroes the final buffer on drop (the standard zeroize-on-growable-container caveat).  So during normal typing, prefixes of the password are repeatedly strewn across freed heap.  `replace_range` (`sensitive_string.rs:122-124`) with a longer replacement has the same growth path.

Additionally, two render-side copies escape the guarantee entirely when the user toggles visibility: `display_content()` returns `self.core.content().to_string()` (`password_input.rs:283`) and `shape_line` wraps another copy in a `SharedString` (`password_input.rs:336`) on every render — plain, never-zeroized copies of the full password.

## Reproduction (test-ready)

Code-inspection / heap-instrumentation level.  Type a 40-character password one character at a time into a `PasswordInput` built with the `secure-password` feature; dump the process heap afterward — intermediate prefixes from pre-growth buffers persist unzeroized.  There is no plain Rust-test repro without allocator instrumentation (a custom `GlobalAlloc` that records freed blocks would expose it, at some test-harness cost); the mechanism is fully traced in source above.  A capacity-based unit test _can_ pin the fix indirectly: after the fix, assert that a growth-forcing `insert_str` never leaves `self.inner.capacity()` smaller than needed while routing through the owned-growth path (e.g. by instrumenting the new branch).

## Suggested Fix

**Code — own the growth path** in `insert_str` and `replace_range` (`sensitive_string.rs:113-124`).  When the operation needs more than `self.inner.capacity()`, allocate a fresh `Zeroizing<String>` with headroom (e.g. doubled capacity), build the new content into it, and replace the old one so it drops — and therefore zeroizes.  Sketch for `insert_str`:

```rust
fn insert_str(&mut self, pos: usize, text: &str) {
    let needed = self.inner.len() + text.len();
    if needed > self.inner.capacity() {
        let mut grown =
            Zeroizing::new(String::with_capacity(needed.max(self.inner.capacity() * 2)));
        grown.push_str(&self.inner[..pos]);
        grown.push_str(text);
        grown.push_str(&self.inner[pos..]);
        self.inner = grown; // old Zeroizing drops here → old buffer zeroized
    } else {
        self.inner.insert_str(pos, text);
    }
}
```

(`replace_range` analogously.)

**Docs — soften the absolute claims.**  `SECURITY.md:14-16` and the module doc should state that the zeroization guarantee covers the stored buffer (including, after this fix, its growth path), and that transient render copies in visible-password mode and text-shaping buffers (`display_content`, the `SharedString` in `shape_line`) are outside the guarantee — an honest documented limitation instead of a false claim.

## Why the Fix Works

Owning growth ensures the only deallocation path for password-bearing buffers is `Zeroizing`’s drop, closing the realloc leak; the doc change converts the remaining, unavoidable render-side exposure from a false claim into a documented limitation.

## Fix Ordering / Related

bug-0063 (`SECURITY.md` errors) — coordinate the `SECURITY.md` edits with that report in one doc pass.
