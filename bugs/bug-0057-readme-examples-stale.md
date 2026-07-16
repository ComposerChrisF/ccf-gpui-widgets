# bug-0057: README examples are stale — several no longer compile against the shipped API

**Severity:** high
**Fix side:** SPEC (docs)
**Component:** docs/ccf-pgui-widgets-README.md (the file `README.md` symlinks to — see bug-0058); lines cited below.  Secondary: src/lib.rs crate docs
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

> **THE DOCS ARE THE PROBLEM — the code is correct and consumed by clui.  Do NOT change code to match the docs.**

## Description

The README is the first thing a consumer (human or agent) copies from, and several of its examples use APIs that were removed or renamed long ago.  `README.md` is a committed symlink to `docs/ccf-pgui-widgets-README.md` (see bug-0058 for the filename typo), so **all edits happen in the `docs/` file** — line references below are into that file.

1. **TabBar example (lines 255-273) uses the pre-refactor API.**  It builds `TabBar::new(cx)` with a `.tabs(vec![TabItem::new("general", "General"), …])` builder and `.with_selected_index(0)`.  Commit `87e592a` (“Unify selection API across RadioGroup, SegmentedControl, TabBar, and SidebarNav”) removed the `TabItem` trait entirely and changed the constructor to `TabBar::new(tabs, selected, cx)` (`tab_bar.rs:133`).  `git show 87e592a --stat` confirms the commit touched `CLAUDE.md`, `TODO.md`, `examples/widget_gallery.rs`, `src/lib.rs`, `src/widgets/mod.rs`, and the widget sources — but **not** the README.  A repo-wide grep for `TabItem` today matches only the tab_bar module doc describing the removal, and this README example.

2. **ConfirmationDialog example (lines 277-294) uses a constructor and builders that never shipped in this form.**  It calls `ConfirmationDialog::new(cx)` then `.title("Delete Item")` and `.message("…")`.  The actual API is `ConfirmationDialog::new(title, message, cx)` — title and message are required constructor arguments; there are no `.title()`/`.message()` builders.  The gallery shows the real form at `examples/widget_gallery.rs:619-626`.  Cf. commit `24a4e39` (“Simplify ConfirmationDialog API with configurable buttons and key mappings”).

3. **Slider example (line 233) destructures a unit variant.**  `SliderEvent::ChangeComplete(value) => …` — `ChangeComplete` carries no payload, so the example does not compile.  Doc fix: match `SliderEvent::ChangeComplete => …`.  (Option for Chris: adding a payload to `ChangeComplete` is a small, arguably useful code change — but that would be a deliberate API change, not this bug; the default here is the doc fix.)

4. **Checkbox example (lines 150-155) calls `.checked(true)`.**  The builder is `with_checked()` (`checkbox.rs:71`); the gallery uses `with_checked(true)` (`examples/widget_gallery.rs:426`).

5. **Keybinding registration paths (lines 318-323) point at private modules.**  The README shows `ccf_gpui_widgets::widgets::text_input::register_keybindings(cx)` and `…::dropdown::register_keybindings(cx)` — those modules are private.  The public re-exports are the aliases in `src/widgets/mod.rs`: `register_text_input_keybindings` (mod.rs:87-89), `register_dropdown_keybindings` (mod.rs:61), and siblings (`register_color_swatch_keybindings`, `register_tab_bar_keybindings`, etc.).

6. **“Available Widgets” tables (lines 45-105) omit shipped widgets.**  `SidebarNav`, `SegmentedControl`, and `Scrollable` are missing — added by commits `e9f3979` (SidebarNav) and `1a239e9` (Scrollable, Scrollbar, SegmentedControl) without README updates.  The crate-doc widget lists in `src/lib.rs` have the same omissions.

7. **Installation section (lines 12-22) omits the `secure-password` feature**, as does the `src/lib.rs` Feature Flags crate-doc section.  `Cargo.toml:32-36` defines it, and `SECURITY.md:28-37` tells users to enable it for production password handling — a feature the security doc depends on should be discoverable from the installation instructions.

8. **ToggleSwitch example (lines 248-252) matches `ToggleSwitchEvent::Change(is_on)`** — the shipped event is `Toggle(bool)`.  **[GATED on bug-0079 — do not rewrite this example until the event-name decision lands; then write whatever was decided.]**

Additionally, line 216 claims “all 140 CSS named colors” — the actual `NAMED_COLORS` table has 148 entries; that count (and its siblings across the docs) is owned by bug-0023.

## Reproduction (test-ready)

No GPUI machinery needed — each broken example can be shown non-compiling:

1. Copy the TabBar example (lines 255-273) into a consumer crate: fails with unresolved `TabItem` and a wrong-arity `TabBar::new`.
2. Copy the ConfirmationDialog example (lines 277-294): fails — `new` takes `(title, message, cx)`, and no `title`/`message` methods exist.
3. Copy the Slider subscription (line 233): fails with “expected unit struct/variant” on `ChangeComplete(value)`.
4. Copy the Checkbox example (lines 150-155): fails — no method `checked` on `Checkbox`.
5. `ccf_gpui_widgets::widgets::text_input::register_keybindings(cx)` (line 321): fails — module `text_input` is private.
6. Mechanical checks: `grep -n 'SidebarNav\|SegmentedControl\|Scrollable' docs/ccf-pgui-widgets-README.md` (no table rows); `grep -n 'secure-password' docs/ccf-pgui-widgets-README.md` (no hits).

Expected: every README example compiles against the shipped API.  Actual: items 1-5 and 8 do not; items 6-7 omit shipped surface.

## Suggested Fix

In `docs/ccf-pgui-widgets-README.md`:

- **Lines 255-273**: rewrite the TabBar example on the current API — `TabBar::new(vec!["General".to_string(), "Advanced".to_string(), "About".to_string()], "General".to_string(), cx)` (or the `StringItem`/`SelectionItem` form the gallery uses), subscribing to `TabBarEvent::Change(T)`.
- **Lines 277-294**: rewrite to `ConfirmationDialog::new("Delete Item", "Are you sure…", cx).style(DialogStyle::Warning).primary_label("Delete").secondary_label("Cancel")`, mirroring `examples/widget_gallery.rs:619-626`.
- **Line 233**: `SliderEvent::ChangeComplete => { /* drag ended */ }` (note the payload-addition option for Chris above).
- **Lines 150-155**: `.checked(true)` → `.with_checked(true)`.
- **Lines 318-323**: replace the private-module paths with the public aliases, e.g. `ccf_gpui_widgets::widgets::register_text_input_keybindings(cx);` and `ccf_gpui_widgets::widgets::register_dropdown_keybindings(cx);`.
- **Lines 45-105**: add rows for `SidebarNav`, `SegmentedControl`, `Scrollable`; make the same additions to the `src/lib.rs` crate-doc widget lists.
- **Lines 12-22**: add the `secure-password` (and `full`) feature to the installation snippet; add `secure-password` to the `src/lib.rs` Feature Flags section.
- **Lines 248-252**: leave until bug-0079 is decided, then update the event name to the decision.

## Why the Fix Works

Every README example becomes copy-paste-compilable against the shipped crate, and the widget/feature tables regain parity with the actual exports — removing the first-contact surface where consumers currently inherit dead API.

## Fix Ordering / Related

- The ToggleSwitch example is **gated on bug-0079**; everything else can be fixed immediately.
- Coordinate with **bug-0058** (renames `docs/ccf-pgui-widgets-README.md` → `docs/ccf-gpui-widgets-README.md`): do the rename first or in the same pass so edits land in the surviving file.
- The “140 CSS named colors” claim at line 216 is owned by **bug-0023**.
- Corrected event names must agree with the `CLAUDE.md` table rewrite in **bug-0004**.
