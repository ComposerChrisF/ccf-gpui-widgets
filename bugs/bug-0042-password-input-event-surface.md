# bug-0042: PasswordInput event surface — docs promise `Escape`/`Focus`/`select_on_focus()`, code has none, and Escape double-fires `Blur`

**Severity:** medium
**Fix side:** DECISION REQUIRED (Chris)
**Component:** `src/widgets/password_input.rs:52-65` (event enum), `490-499` (`on_focus`/`on_blur`), `893-896` (Escape handler), `163-189` (builders); doc side: `CLAUDE.md` Widget Quick Reference table, PasswordInput row
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

> **DO NOT FIX without Chris’s decision.  Both a code-side and a doc-side resolution exist; the sections below present the options and a recommendation.**

## Description

Two mismatches plus one definite code bug, all in the same surface.

**1.  The doc/code mismatch.**  `CLAUDE.md`’s Widget Quick Reference table claims the PasswordInput row is:

```
| PasswordInput | Change, Enter, Escape, Blur, Focus | placeholder(), select_on_focus() |
```

The code’s `PasswordInputEvent` (`password_input.rs:52-65`) has only `Change`, `Enter`, and `Blur`.  There is no `Escape` variant — the enum doc folds it into `Blur` (`/// Input lost focus (including Escape key)`, lines 62–64) — and no `Focus` variant (`on_focus` at `password_input.rs:490-493` emits nothing).  No `select_on_focus()` builder exists; the builders are `placeholder`, `with_value`, `theme`, `with_enabled` (`password_input.rs:163-189`).

**2.  The definite code bug — Escape double-fires `Blur` — which must be fixed under every option below.**  The Escape action handler is:

```rust
// src/widgets/password_input.rs:893-896
.on_action(cx.listener(|this, _: &Escape, _window, cx| {
    if !this.enabled { return; }
    this.on_blur(cx);
}))
```

and `on_blur` (`password_input.rs:495-499`) emits `PasswordInputEvent::Blur` — but nothing changes window focus.  After Escape the field is still focused, still blinking its cursor, and still accepts typing; the `Blur` event was a lie.  Then, when focus genuinely leaves, the `on_focus_out` subscription calls `on_blur` again — a second `Blur` for one editing session.  A consumer using `Blur` as commit-on-blur (exactly what `NumberStepper` does with TextInput’s `Blur`) acts twice, and the first time at the wrong moment.  Contrast `TextInput`, which emits a dedicated `TextInputEvent::Escape` (`text_input.rs:1047-1052`) and leaves `Blur` to real focus loss.

## Reproduction (test-ready)

Needs GPUI test context (window focus + action dispatch):

1. Subscribe to `PasswordInputEvent`; focus the password field.
2. Press Escape.  `Blur` #1 arrives — while `input_focus_handle.is_focused(window)` is still `true`.
3. Type a character — the input still lands in the field, further evidence that no blur happened.
4. Click elsewhere (real focus loss).  `Blur` #2 arrives.
5. Expected under any sane contract: at most one `Blur` per editing session, delivered at actual focus loss.

## Suggested Fix

**The question for Chris: what is PasswordInput’s event surface supposed to be — the table’s, or the code’s?**  Options:

- **(a)** Make Escape actually release focus (blur the window focus or move it to a parent), so the single `Blur` is truthful.  Smallest code change; keeps the folded contract, but Escape-releases-focus is unusual widget behavior.
- **(b)** Add an `Escape` variant to `PasswordInputEvent` and emit that from the Escape handler instead of the fake `Blur`.  Matches what the table already claims, and matches `TextInput`.
- **(c)** Implement the full table surface: `Escape` variant, `Focus` variant (emitted from `on_focus`), and a `select_on_focus()` builder — full parity with `TextInput`, which already implements identical logic to crib from (`text_input.rs:141-146`, `254-259`).
- **(d)** Correct the table to match the code (drop `Escape`, `Focus`, and `select_on_focus()` from the PasswordInput row) and keep the folded-Blur contract — doc-only for the surface, but the double-`Blur` still must be fixed separately (via option (a)’s focus release, or by simply not emitting on Escape).

**Recommendation: (c).**  PasswordInput presents itself as TextInput-with-masking; consumers will assume the TextInput contract, the table already promises it, and the implementation is a direct port from `text_input.rs`.  If minimalism wins, (b) is the acceptable floor.  **Whatever is chosen, the double-`Blur` must go** — a `Blur` that fires while the field is still focused is a defect under every option.

## Why the Fix Works

Options (b)/(c) route Escape to its own event, leaving `Blur` exclusively to real focus loss, so consumers get one truthful `Blur` per session; option (a) instead makes the emitted `Blur` coincide with actual focus loss.  Either way the two-`Blur`s-per-session defect disappears.

## Fix Ordering / Related

This decision gates the doc rewrites bug-0004 and bug-0057 (the corrected tables/examples must reflect the decided event names).  bug-0041 (missing `Blur` on the toggle-exit path) touches the same `on_blur` machinery and is required under every option here.  bug-0050 (picker event surface) is the same class of decision.
