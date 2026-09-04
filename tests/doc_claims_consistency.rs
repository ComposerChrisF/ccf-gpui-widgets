//! Documentation-claim consistency tests (Phase 1 doc repairs, 2026-08-12).
//!
//! These tests pin the independent doc fixes of bug-0003 (CLAUDE.md drift), bug-0005
//! (COLOR_PICKER.md Cancel/Apply workflow), bug-0023 (color counts), bug-0031
//! (`luminance()` overclaim), bug-0058 (README symlink name), bug-0063 (SECURITY.md
//! errors), bug-0069 (phantom Slider drag modifiers), bug-0078 (TODO.md stale refs),
//! and bug-0080 (non-compiling module-doc examples).
//!
//! Every one of those is a *documentation* defect, so each test asserts on the doc text
//! **and** on the code fact the text is supposed to project.  Reverting a doc edit fails
//! the text half; changing the code out from under a now-correct doc fails the mechanism
//! half, which is what turns these into a standing guard rather than a snapshot.
//!
//! Scope note: no GPUI test context exists (or is needed) here — these read source and
//! Markdown as text and exercise only pure functions.  Where a report's repro needed a
//! live window, the manual check is written out in a comment beside the test.
//!
//! Sibling file: `tests/doc_surface_consistency.rs` covers bug-0004 / 0025 / 0050 / 0057.

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    let path = repo_root().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

/// The `//!` module-doc header of a Rust source file.
fn module_doc(rel: &str) -> String {
    read(rel)
        .lines()
        .take_while(|l| l.trim_start().starts_with("//!") || l.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// The contiguous `///` doc-comment block immediately above `signature`.
fn doc_comment_above(source: &str, signature: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let idx = lines
        .iter()
        .position(|l| l.trim_start().starts_with(signature))
        .unwrap_or_else(|| panic!("no line starting with `{signature}`"));
    let mut out: Vec<&str> = lines[..idx]
        .iter()
        .rev()
        .take_while(|l| l.trim_start().starts_with("///"))
        .copied()
        .collect();
    out.reverse();
    out.join("\n")
}

/// Text of a Markdown/`CLAUDE.md` section: from `heading` up to the next line that
/// starts with `stop_prefix`.
fn section<'a>(doc: &'a str, heading: &str, stop_prefix: &str) -> &'a str {
    let start = doc
        .find(heading)
        .unwrap_or_else(|| panic!("no `{heading}` heading"));
    let rest = &doc[start + heading.len()..];
    let end = rest
        .find(&format!("\n{stop_prefix}"))
        .unwrap_or(rest.len() - 1);
    &rest[..end]
}

/// Filenames named on their own line inside a directory-listing code fence, with the
/// box-drawing prefix stripped.  Substring matching would let `repeatable_file_picker.rs`
/// stand in for a missing `file_picker.rs`, so tokens are compared whole.
fn listed_filenames(block: &str) -> Vec<String> {
    block
        .lines()
        .filter_map(|line| {
            let rest = line.trim_start_matches(|c: char| !c.is_ascii_alphanumeric());
            let token: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '.')
                .collect();
            token.ends_with(".rs").then_some(token)
        })
        .collect()
}

fn source_files(dir: &str) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(repo_root().join(dir))
        .unwrap_or_else(|e| panic!("cannot list {dir}: {e}"))
        .map(|e| e.expect("dir entry").file_name().to_string_lossy().into())
        .filter(|n: &String| n.ends_with(".rs"))
        .collect();
    names.sort();
    names
}

// ---------------------------------------------------------------------------
// bug-0003 — CLAUDE.md stops contradicting the codebase
// ---------------------------------------------------------------------------

/// (a) The Widget API Pattern template must not regenerate the solved tab-stop bug.
///
/// Mechanism: GPUI only honors a div's `tab_stop` when it creates the handle itself, so
/// `.tab_stop(true)` after `.track_focus()` is a no-op and the widget vanishes from Tab
/// order.  Every shipped focusable widget therefore sets it on the handle.
///
/// Manual check (needs a GPUI window, so not automated): scaffold a widget from the
/// template, put it in the gallery, and Tab through — it must receive focus.
#[test]
fn widget_template_creates_a_tab_stop_focus_handle() {
    let claude_md = read("CLAUDE.md");
    let template = section(
        &claude_md,
        "## Widget API Pattern",
        "**Naming Convention:**",
    );

    assert!(
        template.contains("cx.focus_handle().tab_stop(true)"),
        "the CLAUDE.md widget template must create the handle as \
         `cx.focus_handle().tab_stop(true)` (bug-0003a)"
    );
    assert!(
        !template.contains("focus_handle: cx.focus_handle(),"),
        "the CLAUDE.md widget template still shows a bare `cx.focus_handle()`, which \
         produces a widget invisible to Tab navigation (bug-0003a)"
    );
    assert!(
        template.contains("TAB_STOP_SOLUTIONS.md"),
        "the template should point at docs/TAB_STOP_SOLUTIONS.md for why (bug-0003a)"
    );

    // The template must also carry the enabled trio that all interactive widgets have.
    for needle in ["enabled: bool", "with_enabled", "is_enabled", "set_enabled"] {
        assert!(
            template.contains(needle),
            "the CLAUDE.md widget template omits `{needle}` (bug-0003a)"
        );
    }

    // The mechanism half: the template projects the shipped idiom, not an invention.
    let checkbox = read("src/widgets/checkbox.rs");
    assert!(
        checkbox.contains("cx.focus_handle().tab_stop(true)"),
        "checkbox.rs no longer sets tab_stop on the handle — if the GPUI behavior \
         changed, revisit docs/TAB_STOP_SOLUTIONS.md and this template together"
    );
    for needle in [
        "pub fn with_enabled",
        "pub fn is_enabled",
        "pub fn set_enabled",
    ] {
        assert!(
            checkbox.contains(needle),
            "checkbox.rs lost `{needle}`; the CLAUDE.md template documents it"
        );
    }
}

/// (a, continued) The "Adding a New Widget" checklist teaches the same two things.
#[test]
fn adding_a_widget_checklist_covers_tab_stop_and_enabled() {
    let claude_md = read("CLAUDE.md");
    let checklist = section(&claude_md, "## Adding a New Widget", "## Testing");
    for needle in [
        "cx.focus_handle().tab_stop(true)",
        "with_enabled()",
        "is_enabled()",
        "set_enabled()",
    ] {
        assert!(
            checklist.contains(needle),
            "the `Adding a New Widget` checklist omits `{needle}` (bug-0003a)"
        );
    }
}

/// (b) The Directory Structure listing names every source file that exists.
#[test]
fn directory_structure_lists_every_source_file() {
    let claude_md = read("CLAUDE.md");
    let block = section(&claude_md, "### Directory Structure", "### Feature Flags");
    let listed = listed_filenames(block);

    for (dir, files) in [
        ("src/widgets", source_files("src/widgets")),
        ("src/utils", source_files("src/utils")),
    ] {
        for file in files {
            assert!(
                listed.contains(&file),
                "`{dir}/{file}` exists but is missing from the CLAUDE.md Directory \
                 Structure listing (bug-0003b)"
            );
        }
    }
}

/// (c) The Testing section describes where tests actually live, and how to run them all.
#[test]
fn testing_section_matches_where_tests_live() {
    let claude_md = read("CLAUDE.md");
    let testing = section(&claude_md, "## Testing", "## Consumer Projects");

    assert!(
        !testing.contains("Tests are in `src/utils/path.rs` for path utilities"),
        "the CLAUDE.md Testing section still claims tests live only in path.rs \
         (bug-0003c)"
    );
    assert!(
        testing.contains("--features full"),
        "the Testing section must document `cargo test --features full`; a bare \
         `cargo test` silently skips the feature-gated tests (bug-0003c)"
    );

    // Mechanism: enumerate the files that really carry a `mod tests`, and require the
    // section to name each one.  Adding tests to a new file fails this until documented.
    let mut with_tests: Vec<String> = Vec::new();
    for dir in ["src/utils", "src/widgets"] {
        for file in source_files(dir) {
            if read(&format!("{dir}/{file}")).contains("mod tests") {
                with_tests.push(file);
            }
        }
    }
    assert!(
        with_tests.len() > 1,
        "expected several test-bearing source files, found {with_tests:?}"
    );
    for file in with_tests {
        assert!(
            testing.contains(&file),
            "`{file}` contains a `mod tests` but the CLAUDE.md Testing section does \
             not mention it (bug-0003c)"
        );
    }
}

/// (d) Known Limitation #3 scopes the validation claim instead of denying shipped API.
#[test]
fn known_limitation_scopes_the_validation_claim() {
    let claude_md = read("CLAUDE.md");
    let limitations = section(&claude_md, "## Known Limitations", "## Code Style");

    assert!(
        !limitations.contains("**No built-in validation**"),
        "CLAUDE.md still claims there is no built-in validation, contradicting the \
         shipped picker validation API and TextInput::input_filter() (bug-0003d)"
    );
    for needle in ["input_filter()", "ValidationDisplay", "SECURITY.md"] {
        assert!(
            limitations.contains(needle),
            "the reworded validation limitation should mention `{needle}` (bug-0003d)"
        );
    }

    // Mechanism: the API the limitation now acknowledges really exists.
    assert!(read("src/widgets/text_input.rs").contains("pub fn input_filter"));
    assert!(read("src/widgets/file_picker.rs").contains("ValidationDisplay"));
    assert!(read("src/widgets/file_picker.rs").contains("pub fn validate_file_path"));
    assert!(read("src/widgets/directory_picker.rs").contains("pub fn validate_directory_path"));
}

/// (e) The Widget Quick Reference preamble names the six constructors that differ.
#[test]
fn quick_reference_preamble_names_the_constructor_exceptions() {
    let claude_md = read("CLAUDE.md");
    let preamble = section(&claude_md, "## Widget Quick Reference", "| Widget |");

    assert!(
        !preamble.contains("All widgets follow the pattern"),
        "the Widget Quick Reference preamble still claims *all* widgets take only `cx`; \
         six do not (bug-0003e)"
    );

    // Mechanism: derive the exceptions from the source, then require each to be named.
    let no_cx = [("ProgressBar", "progress_bar"), ("Spinner", "spinner")];
    for (widget, module) in no_cx {
        let source = read(&format!("src/widgets/{module}.rs"));
        assert!(
            source.contains("pub fn new() -> Self"),
            "{widget}::new() gained parameters — update the CLAUDE.md preamble"
        );
        assert!(
            preamble.contains(widget),
            "the preamble must name `{widget}`, whose `new()` takes no `cx` (bug-0003e)"
        );
    }
    let args_before_cx = [
        ("Collapsible", "collapsible", "pub fn new(title:"),
        ("TabBar", "tab_bar", "pub fn new(tabs:"),
        ("SidebarNav", "sidebar_nav", "pub fn new(items:"),
        (
            "ConfirmationDialog",
            "confirmation_dialog",
            "pub fn new(\n        title:",
        ),
    ];
    for (widget, module, signature) in args_before_cx {
        let source = read(&format!("src/widgets/{module}.rs"));
        assert!(
            source.contains(signature),
            "{widget}::new() no longer matches `{signature}` — update the preamble"
        );
        assert!(
            preamble.contains(widget),
            "the preamble must name `{widget}`, whose `new()` takes required arguments \
             before `cx` (bug-0003e)"
        );
    }
}

// ---------------------------------------------------------------------------
// bug-0005 — COLOR_PICKER.md describes the shipped Cancel/Apply workflow
// ---------------------------------------------------------------------------

/// The design doc must describe all three popup exits, not the pre-`6772468` single one.
///
/// Manual check (needs a GPUI window): open the picker, drag the hue slider, press ESC —
/// the swatch reverts to the color the picker opened with.  Repeat with Enter, and with
/// a click outside: both keep the new color.
#[test]
fn color_picker_doc_describes_cancel_apply_workflow() {
    let doc = read("docs/COLOR_PICKER.md");

    assert!(
        !doc.contains("User clicks outside → `close_picker()`"),
        "COLOR_PICKER.md still says clicking outside merely closes the picker; it \
         applies the current color (bug-0005)"
    );
    for needle in [
        "cancel_picker()",
        "apply_picker()",
        "ClosePicker",
        "ApplyPicker",
    ] {
        assert!(
            doc.contains(needle),
            "COLOR_PICKER.md must document `{needle}` (bug-0005)"
        );
    }

    let overview = section(&doc, "## Overview", "## Color Model");
    for needle in ["Cancel", "Apply", "ESC", "Enter", "Old"] {
        assert!(
            overview.contains(needle),
            "the COLOR_PICKER.md overview omits `{needle}` — the Cancel/Apply workflow, \
             the ESC/Enter bindings, and the clickable Old swatch all ship (bug-0005)"
        );
    }

    // Mechanism: the three exits, and the outside-click commit, exist in the widget.
    let source = read("src/widgets/color_swatch.rs");
    for needle in [
        "fn close_picker(",
        "fn cancel_picker(",
        "fn apply_picker(",
        "KeyBinding::new(\"escape\", ClosePicker",
        "KeyBinding::new(\"enter\", ApplyPicker",
    ] {
        assert!(
            source.contains(needle),
            "color_swatch.rs no longer has `{needle}`; COLOR_PICKER.md now documents it \
             (bug-0005)"
        );
    }
    let outside = source
        .find("on_mouse_down_out")
        .expect("the picker popup has an outside-click handler");
    assert!(
        source[outside..outside + 200].contains("apply_picker"),
        "clicking outside the picker no longer applies the current color — \
         COLOR_PICKER.md documents that it does (bug-0005)"
    );
}

// ---------------------------------------------------------------------------
// bug-0023 — every color count is a projection of a countable code fact
// ---------------------------------------------------------------------------

fn theme_field_count() -> usize {
    let theme = read("src/theme.rs");
    let start = theme
        .find("pub struct Theme")
        .expect("`pub struct Theme` exists");
    theme[start..]
        .lines()
        .skip(1)
        .take_while(|l| *l != "}")
        .filter(|l| {
            let t = l.trim_start();
            t.starts_with("pub ") && t.ends_with(": u32,")
        })
        .count()
}

fn named_color_count() -> usize {
    let color = read("src/utils/color.rs");
    let start = color
        .find("const NAMED_COLORS")
        .expect("NAMED_COLORS exists");
    color[start..]
        .lines()
        .skip(1)
        .take_while(|l| !l.starts_with("];"))
        .filter(|l| l.trim_start().starts_with("(\""))
        .count()
}

#[test]
fn theme_field_count_claims_match_the_struct() {
    let fields = theme_field_count();
    assert_eq!(
        fields, 49,
        "the Theme struct changed size; update every doc claim below and this number"
    );

    for (doc, claim) in [
        ("CLAUDE.md", format!("{fields} semantic color fields")),
        ("src/lib.rs", format!("all {fields} fields have with_*")),
        ("src/theme.rs", format!("all {fields} colors from 7 seeds")),
        ("src/theme.rs", format!("all {fields} theme colors")),
        ("TODO.md", format!("all {fields} theme colors")),
    ] {
        assert!(
            read(doc).contains(&claim),
            "{doc} must say {claim:?} — the Theme struct has {fields} `u32` fields \
             (bug-0023)"
        );
    }

    // The three mutually inconsistent claims that bug-0023 removed must stay gone.
    for doc in [
        "CLAUDE.md",
        "src/lib.rs",
        "src/theme.rs",
        "TODO.md",
        "docs/COLOR_PICKER.md",
        "src/widgets/color_swatch.rs",
        "docs/ccf-gpui-widgets-README.md",
    ] {
        let text = read(doc);
        for stale in ["~30", "52 theme colors", "all 52 colors", "30+ fields"] {
            assert!(
                !text.contains(stale),
                "{doc} still carries the stale claim {stale:?} (bug-0023)"
            );
        }
    }
}

#[test]
fn named_color_count_claims_match_the_table() {
    let count = named_color_count();
    assert_eq!(
        count, 148,
        "NAMED_COLORS changed length; update the doc claims below and this number"
    );

    for doc in [
        "docs/COLOR_PICKER.md",
        "docs/ccf-gpui-widgets-README.md",
        "src/widgets/color_swatch.rs",
        "TODO.md",
    ] {
        let text = read(doc);
        assert!(
            text.contains(&count.to_string()),
            "{doc} must state the real CSS named-color count ({count}) (bug-0023)"
        );
        assert!(
            !text.contains("140 "),
            "{doc} still claims 140 CSS named colors (bug-0023)"
        );
    }
}

// ---------------------------------------------------------------------------
// bug-0031 — `luminance()` is documented as luma, not WCAG relative luminance
// ---------------------------------------------------------------------------

/// The numeric mechanism lives in `src/utils/color.rs`
/// (`test_luminance_is_gamma_encoded_luma_not_wcag`); this pins the doc comment that
/// describes it.
#[test]
fn luminance_doc_does_not_claim_wcag_relative_luminance() {
    let source = read("src/utils/color.rs");
    let doc = doc_comment_above(&source, "pub fn luminance");

    assert!(
        !doc.contains("formula for relative luminance"),
        "the `luminance()` doc still claims the ITU-R BT.709 *relative luminance* \
         formula, but the coefficients are applied to gamma-encoded channels \
         (bug-0031)"
    );
    for needle in ["luma", "WCAG", "0.502", "0.216"] {
        assert!(
            doc.contains(needle),
            "the `luminance()` doc should mention `{needle}` so a consumer reaching for \
             contrast math is warned off (bug-0031)"
        );
    }

    let is_dark_doc = doc_comment_above(&source, "pub fn is_dark");
    assert!(
        is_dark_doc.contains("mid-gray"),
        "the `is_dark()` doc should say its 0.5 threshold is calibrated to the luma \
         scale, where 0.5 is perceptual mid-gray (bug-0031)"
    );

    // Mechanism: no linearization step was introduced behind the doc's back.
    assert!(
        !source[source.find("pub fn luminance").unwrap()..]
            .lines()
            .take(8)
            .any(|l| l.contains("powf") || l.contains("12.92")),
        "luminance() appears to linearize now; that flips is_dark() around mid-gray and \
         changes Theme::from_palette mode detection — see bug-0031's optional branch"
    );
}

// ---------------------------------------------------------------------------
// bug-0058 — the README symlink target spells the crate name
// ---------------------------------------------------------------------------

#[test]
fn readme_symlink_target_spells_gpui() {
    let target = fs::read_link(repo_root().join("README.md")).expect("README.md is a symlink");
    assert_eq!(
        target,
        PathBuf::from("docs/ccf-gpui-widgets-README.md"),
        "README.md must point at the correctly spelled canonical README (bug-0058)"
    );
    assert!(
        repo_root().join(&target).is_file(),
        "README.md is a dangling symlink — the rename and the relink must land together \
         (bug-0058)"
    );
    // The misspelling is assembled at runtime so this file does not trip its own sweep.
    let typo = format!("ccf-{}gui", 'p');
    assert!(
        !repo_root()
            .join(format!("docs/{typo}-widgets-README.md"))
            .exists(),
        "the misspelled README target is back (bug-0058)"
    );

    // Nothing may reference the old spelling any more.
    let mut checked = 0usize;
    for dir in ["src/widgets", "src/utils", "docs", "tests"] {
        for entry in fs::read_dir(repo_root().join(dir)).expect("readable dir") {
            let path = entry.expect("dir entry").path();
            if !path.is_file() {
                continue;
            }
            let text = fs::read_to_string(&path).unwrap_or_default();
            assert!(
                !text.contains(&typo),
                "{} still references the misspelled README path (bug-0058)",
                path.display()
            );
            checked += 1;
        }
    }
    for file in ["CLAUDE.md", "TODO.md", "SECURITY.md", "src/lib.rs"] {
        assert!(!read(file).contains(&typo), "{file} (bug-0058)");
        checked += 1;
    }
    assert!(checked > 20, "the sweep checked only {checked} files");
}

// ---------------------------------------------------------------------------
// bug-0063 — SECURITY.md reaches a repository that exists
// ---------------------------------------------------------------------------

#[test]
fn security_md_reporting_channel_and_api_names_are_right() {
    let security = read("SECURITY.md");
    let cargo_toml = read("Cargo.toml");

    // (a) the reporting URL is derived from the real repository, not a guessed org.
    let repo_url = cargo_toml
        .lines()
        .find_map(|l| l.strip_prefix("repository = "))
        .expect("Cargo.toml declares a repository")
        .trim()
        .trim_matches('"');
    assert!(
        security.contains(&format!("{repo_url}/issues")),
        "SECURITY.md must point vulnerability reports at {repo_url}/issues (bug-0063a)"
    );
    assert!(
        !security.contains("ccf-tools"),
        "SECURITY.md still sends vulnerability reports to the nonexistent `ccf-tools` \
         org (bug-0063a)"
    );

    // (b) the validation table names the method that ships.
    assert!(
        security.contains("`input_filter()` builder"),
        "SECURITY.md must name `input_filter()`, the shipped TextInput filter builder \
         (bug-0063b)"
    );
    assert!(
        !security.contains("`filter()` builder"),
        "SECURITY.md still names a `filter()` builder that does not exist (bug-0063b)"
    );
    let text_input = read("src/widgets/text_input.rs");
    assert!(text_input.contains("pub fn input_filter"));
    assert!(
        !text_input.contains("pub fn filter("),
        "a bare `filter()` builder appeared — if it is real, SECURITY.md may name it"
    );

    // (c) no unfinished placeholder in the document that most needs to look maintained.
    assert!(
        !security.contains("add contact email"),
        "SECURITY.md still ships the `(add contact email if applicable)` placeholder \
         (bug-0063c)"
    );
}

// ---------------------------------------------------------------------------
// bug-0069 — the Slider docs claim only the modifiers that exist
// ---------------------------------------------------------------------------

/// Manual check (needs a GPUI window): hold Alt and drag the thumb, then hold Alt and
/// press Right — both must behave identically to the unmodified gesture.  Shift + Right
/// must step 10x.
#[test]
fn slider_doc_claims_only_the_modifiers_that_exist() {
    let doc = module_doc("src/widgets/slider.rs");
    let source = read("src/widgets/slider.rs");

    // Mechanism: `alt` is consulted nowhere, and `shift` only in the key handler.
    assert!(
        !source.contains("modifiers.alt"),
        "the Slider now reads `modifiers.alt`; if a fine-adjust mode was implemented, \
         document it (bug-0069 offered that as Chris's alternative option)"
    );
    assert_eq!(
        source.matches("modifiers.shift").count(),
        1,
        "expected exactly one `modifiers.shift` read (the arrow-key multiplier)"
    );

    assert!(
        !doc.contains("Alt"),
        "the Slider module doc still claims an Alt/Option fine-adjustment mode that \
         does not exist anywhere in the widget (bug-0069)"
    );
    assert!(
        !doc.contains("0.1x"),
        "the Slider module doc still claims a 0.1x step (bug-0069)"
    );

    // Shift is real, but only for the arrow keys — so the bullet must sit under them.
    let arrows = doc
        .find("**Arrow keys**")
        .expect("the doc documents arrow-key stepping");
    let shift = doc
        .find("Hold **Shift**")
        .expect("the doc documents the 10x Shift multiplier");
    assert!(
        shift > arrows,
        "the Shift 10x bullet must sit under the arrow-keys bullet, not under drag — \
         dragging is purely positional (bug-0069)"
    );
}

// ---------------------------------------------------------------------------
// bug-0078 — TODO.md references resolve and its checkboxes reflect reality
// ---------------------------------------------------------------------------

#[test]
fn todo_doc_references_resolve() {
    let todo = read("TODO.md");
    for rel in ["docs/GPUI-LAYOUT-PATTERNS.md", "docs/TAB_STOP_SOLUTIONS.md"] {
        assert!(
            todo.contains(rel),
            "TODO.md must reference `{rel}` by its post-consolidation path (bug-0078a)"
        );
        assert!(
            repo_root().join(rel).is_file(),
            "`{rel}` does not exist, so the TODO.md reference dangles (bug-0078a)"
        );
    }
    assert!(
        !todo.contains("doc/TAB_STOP_SOLUTIONS.md"),
        "TODO.md still uses the pre-move `doc/` path (bug-0078a)"
    );
    assert!(
        !todo.contains("`GPUI-LAYOUT-PATTERNS.md`"),
        "TODO.md still references GPUI-LAYOUT-PATTERNS.md without its `docs/` directory \
         (bug-0078a)"
    );
}

#[test]
fn todo_checkboxes_match_reality() {
    let todo = read("TODO.md");

    // (b) work that is demonstrably done is checked off.
    assert!(
        repo_root().join(".github/workflows/ci.yml").is_file(),
        "CI workflow missing — then line 362 was right after all"
    );
    assert!(
        todo.contains("- [x] Set up GitHub repository"),
        "the GitHub repository and its CI both exist; the entry must be checked \
         (bug-0078b)"
    );

    assert!(repo_root().join("examples/widget_gallery.rs").is_file());
    assert!(read("Cargo.toml").contains("widget_gallery"));
    for entry in ["- [x] Example application", "- [x] Widget showcase/gallery"] {
        assert!(
            todo.contains(entry),
            "`examples/widget_gallery.rs` ships and is registered in Cargo.toml, so \
             {entry:?} must be checked (bug-0078b)"
        );
    }

    // ...and work that is genuinely open stays open.  This guards the opposite error.
    assert!(
        todo.contains("- [ ] Publish to crates.io"),
        "publishing to crates.io has not happened; bug-0078 explicitly said to leave \
         that entry open"
    );

    // The coordinating session's bug queue is not this report's business to touch.
    assert!(
        todo.contains("OPEN BUGS"),
        "the OPEN BUGS section disappeared; bug-0078's scope note excluded it"
    );
}

#[test]
fn todo_known_issue_describes_the_feature_gate_not_a_build_defect() {
    let todo = read("TODO.md");
    assert!(
        !todo.contains("won’t compile without it"),
        "TODO.md Known Issue #2 still reads as a compilation defect; the library \
         compiles cleanly without the `file-picker` feature (bug-0078c)"
    );
    assert!(
        todo.contains("compiles cleanly without it"),
        "Known Issue #2 should say the library compiles without the feature and only \
         consumer code naming the pickers needs it enabled (bug-0078c)"
    );

    // Mechanism: the pickers really are cleanly cfg-gated out of the default build.
    let mod_rs = read("src/widgets/mod.rs");
    for module in ["file_picker", "directory_picker"] {
        let decl = mod_rs
            .find(&format!("mod {module};"))
            .unwrap_or_else(|| panic!("`mod {module};` in src/widgets/mod.rs"));
        assert!(
            mod_rs[decl.saturating_sub(120)..decl].contains("#[cfg(feature = \"file-picker\")]"),
            "`{module}` is no longer gated behind the `file-picker` feature — Known \
             Issue #2 describes that gate (bug-0078c)"
        );
    }
}

// ---------------------------------------------------------------------------
// bug-0080 — module-doc examples name builders that exist
// ---------------------------------------------------------------------------

/// The three `ignore`-fenced examples are copy-paste surfaces; each called a getter
/// where a builder was meant, so the chain could not compile.
///
/// Each case asserts both halves: the doc now names the builder, and the identifier it
/// used to name is still a `&self` getter (i.e. the rename was the right repair, not a
/// coincidence).
#[test]
fn module_doc_examples_use_builders_not_getters() {
    // (a) Dropdown
    let dropdown_doc = module_doc("src/widgets/dropdown.rs");
    let dropdown = read("src/widgets/dropdown.rs");
    assert!(
        dropdown_doc.contains(".with_selected_index(0)"),
        "the Dropdown example must call the builder `with_selected_index()` (bug-0080a)"
    );
    assert!(
        !dropdown_doc.contains(".selected_index(0)"),
        "the Dropdown example still calls the getter `selected_index()` in a builder \
         chain, which does not compile (bug-0080a)"
    );
    assert!(dropdown.contains("pub fn with_selected_index(mut self"));
    assert!(dropdown.contains("pub fn selected_index(&self)"));

    // (b) CheckboxGroup
    let group_doc = module_doc("src/widgets/checkbox_group.rs");
    let group = read("src/widgets/checkbox_group.rs");
    assert!(
        group_doc.contains(".with_selected(vec!"),
        "the CheckboxGroup example must call the builder `with_selected()` (bug-0080b)"
    );
    assert!(
        !group_doc.contains(".selected(vec!"),
        "the CheckboxGroup example still calls `selected()`, which does not exist at \
         all (bug-0080b)"
    );
    assert!(group.contains("pub fn with_selected(mut self"));
    assert!(
        !group.contains("pub fn selected("),
        "a `selected()` method appeared; if it is a real builder, revisit bug-0080b"
    );

    // (c) ColorSwatch
    let swatch_doc = module_doc("src/widgets/color_swatch.rs");
    let swatch = read("src/widgets/color_swatch.rs");
    assert!(
        swatch_doc.contains(".with_value(\"#3b82f6\")"),
        "the ColorSwatch example must call the builder `with_value()` (bug-0080c)"
    );
    assert!(
        !swatch_doc.contains(".value(\"#3b82f6\")"),
        "the ColorSwatch example still calls the getter `value()` in a builder chain \
         (bug-0080c)"
    );
    assert!(swatch.contains("pub fn with_value(mut self"));
    assert!(swatch.contains("pub fn value(&self)"));
}

/// (d) The color-model wording matches the canvas that ships.
///
/// Note the deliberate asymmetry, per Chris's 2026-08-12 decision on **bug-0051**
/// (option 2): the "RGB and HSL modes" claim **stays**, because the RGB/HSL slider-mode
/// switch is being implemented at the end of this campaign — the doc is ahead of the
/// code on purpose, exactly like `PENDING_CODE_SURFACES` in the sibling test file.  Only
/// the HSV/HSL *canvas* wording, which was simply wrong, is corrected here.
#[test]
fn color_swatch_doc_calls_the_canvas_hsv() {
    let doc = module_doc("src/widgets/color_swatch.rs");

    assert!(
        !doc.contains("saturation/lightness"),
        "the ColorSwatch module doc still calls the 2D canvas a saturation/lightness \
         selector; it is HSV saturation/brightness (bug-0080d)"
    );
    assert!(
        doc.contains("saturation/brightness (HSV)"),
        "the ColorSwatch module doc should say `saturation/brightness (HSV)` — the \
         wording docs/COLOR_PICKER.md already uses (bug-0080d)"
    );

    // The canvas really is HSV: the widget carries HSV state and converts through it.
    let source = read("src/widgets/color_swatch.rs");
    assert!(source.contains("current_hsv"));
    assert!(read("docs/COLOR_PICKER.md").contains("HSV"));

    // The RGB/HSL mode claim is retained by decision; `PickerMode` is the type that
    // will back it (bug-0051 option 2).  If that feature is abandoned, delete this
    // assertion together with lines 4-5 and 11 of the module doc.
    assert!(
        doc.contains("RGB and HSL modes"),
        "the RGB/HSL mode claim was deleted, but Chris's bug-0051 decision was to \
         implement the feature and keep the claim (bug-0080d)"
    );
    assert!(
        source.contains("pub enum PickerMode"),
        "`PickerMode` disappeared, so the retained RGB/HSL module-doc claim has nothing \
         behind it — revisit bug-0051 / bug-0080d together"
    );
}
