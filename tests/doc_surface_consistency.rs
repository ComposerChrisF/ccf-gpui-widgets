//! Documentation-surface consistency tests.
//!
//! These tests pin the doc repairs made on 2026-08-12 for bug-0004 (CLAUDE.md widget
//! table), bug-0050 (picker event surface), bug-0025 (`extensions()` semantics), and
//! bug-0057 (README examples).  They are mechanical: the docs claim event variants and
//! builder methods, and every claim is checked against the shipped source.  Reverting
//! any of those doc repairs fails a test here.
//!
//! Scope note: this is a *documentation* check, so it reads files as text rather than
//! constructing widgets — no GPUI test context is needed or available.  The one
//! behavioral assertion (extensions are not part of validation) uses the standalone
//! `validate_file_path` function, which needs no context.
//!
//! Two surfaces are deliberately exempted, because Chris's 2026-08-12 decisions put the
//! doc ahead of the code and the matching code change lands later in the same campaign:
//! `ToggleSwitchEvent::Change` (bug-0079, rename of `Toggle`) and `PasswordInput`'s
//! `Escape` / `Focus` / `select_on_focus()` (bug-0042, TextInput parity).  See
//! `PENDING_CODE_SURFACES` below; each entry may be removed once its code fix lands.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(rel: &str) -> String {
    let path = repo_root().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

/// Widget-surface claims that the docs make ahead of the code, by decision.
///
/// `(widget, symbol)` — removing the entry after the code change lands is safe; the
/// check simply becomes enforced.
const PENDING_CODE_SURFACES: &[(&str, &str)] = &[
    // bug-0079: `ToggleSwitchEvent::Toggle` is renamed to `Change`.
    ("ToggleSwitch", "Change"),
    // bug-0042: PasswordInput gains full TextInput parity.
    ("PasswordInput", "Escape"),
    ("PasswordInput", "Focus"),
    ("PasswordInput", "select_on_focus"),
];

fn is_pending(widget: &str, symbol: &str) -> bool {
    PENDING_CODE_SURFACES
        .iter()
        .any(|(w, s)| *w == widget && *s == symbol)
}

/// `SegmentedControl` -> `segmented_control`
fn snake_case(camel: &str) -> String {
    let mut out = String::new();
    for (i, ch) in camel.char_indices() {
        if ch.is_ascii_uppercase() {
            if i != 0 {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

fn widget_source(widget: &str) -> String {
    read(&format!("src/widgets/{}.rs", snake_case(widget)))
}

/// One row of the CLAUDE.md "Widget Quick Reference" table.
struct Row {
    widget: String,
    events: String,
    builders: String,
}

/// Parse the Widget Quick Reference table out of CLAUDE.md.
fn quick_reference_rows() -> Vec<Row> {
    let claude_md = read("CLAUDE.md");
    let mut lines = claude_md
        .lines()
        .skip_while(|l| !l.starts_with("## Widget Quick Reference"));
    assert!(
        lines.next().is_some(),
        "CLAUDE.md has no `## Widget Quick Reference` section"
    );

    let mut rows = Vec::new();
    let mut seen_table = false;
    for line in lines {
        if !line.starts_with('|') {
            if seen_table {
                break; // table ended
            }
            continue; // preamble before the table
        }
        seen_table = true;
        let cells: Vec<&str> = line.trim_matches('|').split('|').map(str::trim).collect();
        assert_eq!(cells.len(), 3, "unexpected table row shape: {line}");
        // Skip the header row and the `|---|---|---|` separator.
        if cells[0] == "Widget" || cells[0].chars().all(|c| c == '-' || c == ':') {
            continue;
        }
        rows.push(Row {
            widget: cells[0].trim_matches('`').to_string(),
            events: cells[1].to_string(),
            builders: cells[2].to_string(),
        });
    }
    assert!(
        rows.len() > 15,
        "parsed only {} widget rows — the table parser is out of step with CLAUDE.md",
        rows.len()
    );
    rows
}

/// Identifiers named at the top level of a table cell, each flagged with whether it is
/// immediately followed by `(`.
///
/// Nesting is skipped, so payloads and parenthetical prose do not leak in:
/// `` `Change(Vec<String>)`, `EntryAdded(usize)` `` yields `Change` and `EntryAdded`,
/// and `` `ChangeComplete` (no payload) `` yields only `ChangeComplete`.  Backticks are
/// ignored rather than required, so an unformatted table is checked just as strictly.
fn top_level_idents(cell: &str) -> Vec<(String, bool)> {
    let chars: Vec<char> = cell.chars().filter(|c| !matches!(c, '`' | '*')).collect();
    let mut depth = 0i32;
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if matches!(ch, '(' | '{' | '[') {
            depth += 1;
        } else if matches!(ch, ')' | '}' | ']') {
            depth -= 1;
        } else if depth == 0 && (ch.is_ascii_alphabetic() || ch == '_') {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let ident: String = chars[start..i].iter().collect();
            out.push((ident, chars.get(i) == Some(&'(')));
            continue;
        }
        i += 1;
    }
    out
}

fn starts_upper(ident: &str) -> bool {
    ident.chars().next().is_some_and(|c| c.is_ascii_uppercase())
}

/// Body of `pub enum <Widget>Event`, with doc comments stripped.
fn event_enum_body(source: &str, widget: &str) -> Option<String> {
    let needle = format!("pub enum {widget}Event");
    let start = source.find(&needle)?;
    let body: String = source[start..]
        .lines()
        .skip(1)
        .take_while(|l| *l != "}")
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    Some(body)
}

fn enum_has_variant(body: &str, variant: &str) -> bool {
    body.lines().any(|line| {
        let trimmed = line.trim_start();
        trimmed.strip_prefix(variant).is_some_and(|rest| {
            rest.is_empty() || rest.starts_with(['(', '{', ',', ' ']) || rest.starts_with("<'")
        })
    })
}

// ---------------------------------------------------------------------------
// bug-0004 — the CLAUDE.md widget table is a faithful projection of the source
// ---------------------------------------------------------------------------

#[test]
fn quick_reference_events_exist_in_source() {
    for row in quick_reference_rows() {
        let source = widget_source(&row.widget);
        // Event cells also cite builders (e.g. `emit_tab_events(true)`); only
        // capitalized identifiers are event variants.
        let variants: Vec<String> = top_level_idents(&row.events)
            .into_iter()
            .filter(|(ident, _)| starts_upper(ident))
            .map(|(ident, _)| ident)
            .collect();
        if variants.is_empty() {
            assert!(
                row.events.contains("no events"),
                "{} lists no event variants but is not marked `(no events)`",
                row.widget
            );
            continue;
        }
        let body = event_enum_body(&source, &row.widget)
            .unwrap_or_else(|| panic!("no `pub enum {}Event` in source", row.widget));
        for variant in variants {
            if is_pending(&row.widget, &variant) {
                continue;
            }
            assert!(
                enum_has_variant(&body, &variant),
                "CLAUDE.md documents `{}Event::{}`, which does not exist in the source \
                 (bug-0004 / bug-0050: the table must project the shipped enums)",
                row.widget,
                variant
            );
        }
    }
}

#[test]
fn quick_reference_builders_exist_in_source() {
    for row in quick_reference_rows() {
        let source = widget_source(&row.widget);
        // A builder is a lower-case identifier written as a call: `with_checked()`.
        // Bare prose words ("free functions:") carry no parentheses and are ignored.
        for (name, is_call) in top_level_idents(&row.builders) {
            if !is_call || starts_upper(&name) || is_pending(&row.widget, &name) {
                continue;
            }
            assert!(
                source.contains(&format!("pub fn {name}")),
                "CLAUDE.md lists `{}::{}()`, which does not exist in {} \
                 (bug-0004: the table must project the shipped builders)",
                row.widget,
                name,
                snake_case(&row.widget)
            );
        }
    }
}

#[test]
fn quick_reference_covers_every_widget_module() {
    // Modules under src/widgets/ that are not themselves widgets.
    const NON_WIDGET_MODULES: &[&str] = &[
        "button",
        "cursor_blink",
        "editing_core",
        "focus_navigation",
        "path_display",
        "scrollbar",
        "selection",
        "sensitive_string",
    ];

    let documented: BTreeSet<String> = quick_reference_rows()
        .iter()
        .map(|r| snake_case(&r.widget))
        .collect();

    for line in read("src/widgets/mod.rs").lines() {
        let Some(module) = line
            .trim()
            .strip_prefix("mod ")
            .and_then(|rest| rest.strip_suffix(';'))
        else {
            continue;
        };
        if NON_WIDGET_MODULES.contains(&module) {
            continue;
        }
        assert!(
            documented.contains(module),
            "widget module `{module}` has no row in the CLAUDE.md Widget Quick Reference \
             (bug-0004: Tooltip and the repeatable pickers were the original omissions)"
        );
    }
}

// ---------------------------------------------------------------------------
// bug-0050 — pickers emit only `Change(String)`; `Validated` is fiction
// ---------------------------------------------------------------------------

/// Lines inside fenced code blocks — the copy-paste surface of a Markdown doc.
fn code_fence_lines(markdown: &str) -> Vec<&str> {
    let mut inside = false;
    let mut out = Vec::new();
    for line in markdown.lines() {
        if line.trim_start().starts_with("```") {
            inside = !inside;
            continue;
        }
        if inside {
            out.push(line);
        }
    }
    out
}

#[test]
fn no_doc_claims_a_validated_picker_event() {
    assert!(
        !read("src/widgets/file_picker.rs").contains("Validated")
            && !read("src/widgets/directory_picker.rs").contains("Validated"),
        "a `Validated` event appeared in the picker sources — if it is now real, \
         update the docs and drop this test (bug-0050 decided doc-side)"
    );

    // CLAUDE.md: the claim lives in the table's event column.
    for row in quick_reference_rows() {
        assert!(
            !row.events.contains("Validated") && !row.events.contains("PathBuf"),
            "the {} row still documents a `Validated` event or a `PathBuf` payload; the \
             shipped surface is `Change(String)` only (bug-0050, decided 2026-08-12: \
             correct the docs to the code)",
            row.widget
        );
    }

    // README: the claim would live in an example a consumer copies.
    for line in code_fence_lines(&readme()) {
        assert!(
            !line.contains("Validated") && !line.contains("Change(PathBuf)"),
            "README example line still uses a nonexistent picker surface (bug-0050): {line}"
        );
    }
}

#[test]
fn picker_and_repeatable_event_rows_match_the_source() {
    let rows = quick_reference_rows();
    let row = |name: &str| {
        rows.iter()
            .find(|r| r.widget == name)
            .unwrap_or_else(|| panic!("no `{name}` row in the CLAUDE.md widget table"))
    };

    for picker in ["FilePicker", "DirectoryPicker"] {
        let events = &row(picker).events;
        assert!(
            events.contains("`Change(String)`"),
            "{picker} row must document `Change(String)` (bug-0050); found: {events}"
        );
    }

    for repeatable in [
        "RepeatableTextInput",
        "RepeatableFilePicker",
        "RepeatableDirectoryPicker",
    ] {
        let events = &row(repeatable).events;
        assert!(
            events.contains("`EntryAdded(usize)`") && events.contains("`EntryRemoved(usize)`"),
            "{repeatable} row must document `EntryAdded(usize)` / `EntryRemoved(usize)`, \
             not `Add` / `Remove` (bug-0050); found: {events}"
        );
    }
}

// ---------------------------------------------------------------------------
// bug-0025 — `extensions()` is a dialog-only filter, never a validation constraint
// ---------------------------------------------------------------------------

#[test]
fn extensions_documented_as_dialog_only() {
    for doc in ["CLAUDE.md", "docs/ccf-gpui-widgets-README.md"] {
        assert!(
            read(doc).contains("native file dialog only"),
            "{doc} must state that `extensions()` filters the native file dialog only \
             (bug-0025, decided 2026-08-12: option (a), document dialog-only)"
        );
    }
    for source in [
        "src/widgets/file_picker.rs",
        "src/widgets/repeatable_file_picker.rs",
    ] {
        let text = read(source);
        let fn_line = text
            .lines()
            .position(|l| l.trim_start().starts_with("pub fn extensions"))
            .expect("extensions() builder exists");
        // The contiguous doc-comment block immediately above the builder.
        let lines: Vec<&str> = text.lines().collect();
        let doc_comment: String = lines[..fn_line]
            .iter()
            .rev()
            .take_while(|l| {
                let t = l.trim_start();
                t.starts_with("///") || t.starts_with("#[")
            })
            .copied()
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            doc_comment.contains("Dialog-only"),
            "{source}: the `extensions()` doc comment must say it is dialog-only (bug-0025)"
        );
    }
}

#[test]
fn extensions_are_not_part_of_path_validation() {
    // The mechanism behind the doc claim: validation never sees an extension list.
    // `validate_file_path` is the whole validation surface (`FilePicker::validate`
    // delegates to it), and it takes no extension parameter — so a picker configured
    // with `extensions(vec!["json"])` reports Valid for `photo.png`.
    //
    // Manual check (needs a GPUI window, so it is not automated here): build
    // `FilePicker::new(cx).extensions(vec!["json".into()]).with_value("<some>.png")`,
    // confirm `is_valid()` is true, then confirm the native browse dialog still shows
    // only *.json.  Both halves are what the doc now promises.
    #[cfg(feature = "file-picker")]
    {
        use ccf_gpui_widgets::widgets::{
            validate_file_path, FileMode, FilePickerValidation, MissingDirectories,
        };

        let dir = tempfile::tempdir().expect("temp dir");
        let file = dir.path().join("photo.png");
        std::fs::write(&file, b"not json").expect("write fixture");

        assert_eq!(
            validate_file_path(
                file.to_str().unwrap(),
                &FileMode::Open,
                &MissingDirectories::Error,
            ),
            FilePickerValidation::Valid,
            "validation must ignore file extensions entirely (bug-0025 decision (a)); \
             if this now fails, extension enforcement was added and the dialog-only \
             wording in CLAUDE.md, the README, and the doc comments is stale"
        );
    }

    // The signature check holds with or without the feature: no extension parameter.
    let source = read("src/widgets/file_picker.rs");
    let sig_start = source
        .find("pub fn validate_file_path")
        .expect("validate_file_path exists");
    let signature = &source[sig_start..sig_start + source[sig_start..].find('{').unwrap()];
    assert!(
        !signature.contains("extension"),
        "validate_file_path now takes an extension list — bug-0025 was decided doc-side \
         (dialog-only); revisit the docs before changing this"
    );
}

// ---------------------------------------------------------------------------
// bug-0057 — README examples compile against the shipped API
// ---------------------------------------------------------------------------

fn readme() -> String {
    // README.md is a committed symlink to this file (bug-0058 fixed the typo'd name).
    read("docs/ccf-gpui-widgets-README.md")
}

#[test]
fn readme_has_no_removed_api() {
    // (needle, what it was replaced by)
    const REMOVED: &[(&str, &str)] = &[
        (
            "TabItem",
            "TabBar::new(tabs, selected, cx) over SelectionItem",
        ),
        ("TabBar::new(cx)", "TabBar::new(tabs, selected, cx)"),
        (
            "ConfirmationDialog::new(cx)",
            "ConfirmationDialog::new(title, message, cx)",
        ),
        (".checked(true)", ".with_checked(true)"),
        ("ChangeComplete(value)", "ChangeComplete (a unit variant)"),
        (
            "widgets::text_input::register_keybindings",
            "widgets::register_text_input_keybindings",
        ),
        (
            "widgets::dropdown::register_keybindings",
            "widgets::register_dropdown_keybindings",
        ),
    ];
    let readme = readme();
    for (needle, replacement) in REMOVED {
        assert!(
            !readme.contains(needle),
            "README still shows `{needle}`, which does not compile — use {replacement} \
             (bug-0057)"
        );
    }
}

#[test]
fn readme_shows_the_shipped_api() {
    const REQUIRED: &[&str] = &[
        "TabBar::new(",
        "StringItem::new(",
        "TabBarEvent::ContextMenu",
        "ConfirmationDialog::new(\n",
        ".with_checked(true)",
        "SliderEvent::ChangeComplete =>",
        // bug-0079 decision: the event is `Change(bool)`.
        "ToggleSwitchEvent::Change",
        // bug-0050 decision: `Change(String)`, pull-style validation.
        "FilePickerEvent::Change(path)",
        // Widgets shipped but previously undocumented.
        "`SidebarNav`",
        "`SegmentedControl`",
        "`Scrollable`",
        // Feature flag the security doc depends on.
        "secure-password",
    ];
    let readme = readme();
    for needle in REQUIRED {
        assert!(
            readme.contains(needle),
            "README is missing {needle:?} (bug-0057)"
        );
    }
}

#[test]
fn readme_keybinding_helpers_are_public_aliases() {
    let readme = readme();
    let mod_rs = read("src/widgets/mod.rs");
    let mut checked = 0usize;
    for span in readme.split("ccf_gpui_widgets::widgets::").skip(1) {
        let name: String = span
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if !name.starts_with("register_") {
            continue;
        }
        assert!(
            mod_rs.contains(&format!("register_keybindings as {name}"))
                || mod_rs.contains(&format!("pub fn {name}")),
            "README calls `{name}`, which is not re-exported from src/widgets/mod.rs \
             (bug-0057: the per-widget modules are private)"
        );
        checked += 1;
    }
    assert!(
        checked >= 2,
        "expected the README to document individual keybinding registration"
    );
}

#[test]
fn crate_docs_list_shipped_widgets_and_features() {
    let lib_rs = read("src/lib.rs");
    for needle in [
        "SidebarNav",
        "SegmentedControl",
        "Scrollable",
        "secure-password",
    ] {
        assert!(
            lib_rs.contains(needle),
            "src/lib.rs crate docs omit {needle:?} (bug-0057, secondary surface)"
        );
    }
}

#[test]
fn readme_symlink_still_points_at_the_edited_file() {
    // Guards against editing the wrong copy: README.md must resolve to the docs file.
    let link = repo_root().join("README.md");
    let target = fs::read_link(&link).expect("README.md is a symlink");
    assert_eq!(
        target,
        Path::new("docs/ccf-gpui-widgets-README.md"),
        "README.md points somewhere unexpected; the doc fixes must land in the file it \
         resolves to (bug-0058 renamed the target to spell `gpui`)"
    );
}
