# bug-0063: SECURITY.md errors — vulnerability-report URL points at the wrong GitHub org, plus a wrong method name and an unfinished placeholder

**Severity:** medium
**Fix side:** SPEC (docs)
**Component:** SECURITY.md:67, 108, 109
**Found:** 2026-07-15 deep review (verified against source; baseline: 63/63 tests pass, clippy clean)

> **THE DOCS ARE THE PROBLEM — the code is correct and consumed by clui.  Do NOT change code to match the docs.**

## Description

Three defects in `SECURITY.md`:

**(a) The vulnerability-reporting URL points at a nonexistent org.**  Line 108, under “Reporting Security Issues”:

```
- GitHub Issues: https://github.com/ccf-tools/ccf-gpui-widgets/issues
```

The repository actually lives at `github.com/ComposerChrisF/ccf-gpui-widgets` — confirmed by `Cargo.toml:7` (`repository = "https://github.com/ComposerChrisF/ccf-gpui-widgets"`) and the live `git remote -v`.  A vulnerability report sent to the `ccf-tools` org goes nowhere.  This is the one URL in the project that must not be wrong, which is what drives the medium severity.

**(b) The validation table names a method that does not exist.**  Line 67 says TextInput filtering is via a “`filter()` builder” — the actual method is `input_filter()` (`src/widgets/text_input.rs:303`).

**(c) An unfinished placeholder shipped.**  Line 109:

```
- Email: (add contact email if applicable)
```

Either fill it in or delete the line; as shipped it reads as an abandoned TODO in the document that most needs to look maintained.

## Reproduction (test-ready)

```bash
grep -n 'ccf-tools' SECURITY.md            # line 108
grep -n 'repository' Cargo.toml            # line 7 — ComposerChrisF
git remote -v                              # ComposerChrisF/ccf-gpui-widgets
grep -n 'filter()' SECURITY.md             # line 67
grep -n 'pub fn filter\|pub fn input_filter' src/widgets/text_input.rs   # only input_filter, line 303
grep -n 'add contact email' SECURITY.md    # line 109
```

Expected: the reporting URL resolves to the real repo, the table names the shipped method, no placeholders.  Actual: all three fail.  (Optionally confirm `https://github.com/ccf-tools/ccf-gpui-widgets` is a 404.)

## Suggested Fix

In `SECURITY.md`:

- **Line 108**: change the URL to `https://github.com/ComposerChrisF/ccf-gpui-widgets/issues`.
- **Line 67**: “Optional filter function via `filter()` builder” → “Optional filter function via `input_filter()` builder”.
- **Line 109**: fill in a real contact address or delete the line (deleting is fine — the GitHub Issues channel suffices; decide per preference, but do not leave the placeholder).

## Why the Fix Works

Security reports reach a repository that exists, and the validation table names the method a consumer can actually call — restoring the document’s core property of being trustworthy.

## Fix Ordering / Related

None — independent.  (SECURITY.md’s `secure-password` install snippet at lines 28-37 is correct; the missing cross-reference from the README installation section is owned by **bug-0057**.)
