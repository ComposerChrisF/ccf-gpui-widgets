# bug-0079: ToggleSwitch emits `Toggle(bool)` while CLAUDE.md, README, and the unified convention say `Change(bool)`

**Severity:** medium
**Fix side:** CODE (decided 2026-08-12; see Decision below)
**Component:** `src/widgets/toggle_switch.rs:43-47` (event enum), `toggle_switch.rs:126` and `151` (emit sites); vs `CLAUDE.md:223` and `README.md:249`
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

> **Decision (Chris, 2026-08-12): Option A — rename `ToggleSwitchEvent::Toggle` → `Change` in code.**  The doc surfaces stay as written; the bug-0004/bug-0057 rewrites document `Change(bool)`.

## Description

The code declares and emits `Toggle`:

```rust
// toggle_switch.rs:43-47
pub enum ToggleSwitchEvent {
    /// Toggle state changed.
    /// The boolean indicates the new on/off state: `true` = on, `false` = off.
    Toggle(bool),
}
```

emitted from `set_on` (`toggle_switch.rs:126`) and `toggle` (`toggle_switch.rs:151`).  But every other form widget emits a variant named `Change`, and both doc surfaces document `Change` for this widget: the `CLAUDE.md:223` widget table row reads `ToggleSwitch | Change(bool)`, and the `README.md:249` example is

```rust
if let ToggleSwitchEvent::Change(is_on) = event {
```

which does not compile against the crate.

Git-history evidence:

- The 2026-02 unification renamed `TabBarEvent::TabSelected` → `Change` and `SidebarNavEvent::Select` → `Change` “for consistency” (`TODO.md:330`), but left `ToggleSwitch` behind.
- Commit `02a44d1` renamed `CollapsibleEvent::Toggle` → `Change` explicitly for consistency with other widgets (`TODO.md:282`), and that same commit edited `ToggleSwitchEvent::Toggle`’s doc comment yet chose not to rename the variant—so the omission may have been deliberate, which is why this is a decision rather than a fix.
- `git log -S "Change(bool)" -- src/widgets/toggle_switch.rs` is empty: the code never had a `Change` variant; the docs were never right.
- clui does not reference `ToggleSwitchEvent` at all (grep of `../clui/src`), so a rename is a near-free breaking change.

## Reproduction (test-ready)

1. Compile-time check (no GPUI runtime): paste the `README.md:249` subscription example into a test.
   Actual: compile error—no variant `Change` on `ToggleSwitchEvent`.  Expected per both docs: compiles.
2. Runtime check (needs GPUI entity context): subscribe to a `ToggleSwitch`, call `set_on(true, cx)`.
   Actual: `ToggleSwitchEvent::Toggle(true)`.  Documented: `Change(true)`.

## Suggested Fix

**Question for Chris:** rename `ToggleSwitchEvent::Toggle` → `Change` in code, or keep `Toggle` as deliberate and fix the docs?

- **Option A (rename in code — reviewer recommendation).**  Rename the variant to `Change` in `toggle_switch.rs:43-47` and the two emit sites (`126`, `151`).  Breaking, but clui—the only known consumer—does not reference `ToggleSwitchEvent`, so the break lands on no one.  This restores the one-event-name convention the 2026-02 unification established and makes both existing doc surfaces correct as written.
- **Option B (keep `Toggle`, fix docs).**  Declare `Toggle` the deliberate exception, then correct `CLAUDE.md:223` (table row to `Toggle(bool)`) and `README.md:249` (example to `ToggleSwitchEvent::Toggle(is_on)`), and note the exception wherever the convention is described.

## Why the Fix Works

Either option ends the current state—where the code and both doc surfaces disagree and the shipped README example cannot compile; Option A does it while removing the convention’s only stray.

## Fix Ordering / Related

- **This decision gates the doc rewrites bug-0004 (`CLAUDE.md` widget table) and bug-0057 (`README` examples)**—their corrected tables/examples must reflect the decided event name.
- Related decisions in the same gate: bug-0055, bug-0050, bug-0042 (other event-surface decisions feeding the same doc rewrites).
