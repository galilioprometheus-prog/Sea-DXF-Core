# M6.5o-r5 CLI Output Extraction Audit

## Scope

M6.5o-r5 is a behavior-preserving cleanup checkpoint. It moves JSON rendering,
localized human rendering, stable CLI error codes, and their private output
mapping helpers out of `main.rs` and into the private `output.rs` module.

It does not change DXF parsing, semantics, schema rows, provenance, streaming,
allocation limits, source-file handling, report construction, JSON schema v1,
human output, localization text, exit codes, path redaction, dependencies,
public API, or support claims. It is not part of M6.5p.

## Boundary decision

`output.rs` owns one established outer-boundary responsibility:

- machine-readable JSON serialization;
- localized human rendering;
- stable `CLI-E0001` through `CLI-E0005` identifiers;
- private label and error-message mappings;
- direct contract tests for every published mapping.

`main.rs` continues to own argument parsing, command execution, file access,
DXF dispatch, report construction, failures, and exit routing. `report.rs`
continues to own only serializable report data and stable core-enum names.

The dependency direction is:

```text
main -> output -> report
               -> locale
```

Only the two render functions and five codes use `pub(super)`. Mapping helpers
remain private. No public module, trait, builder, factory, wrapper,
configuration layer, dependency, or extension point is introduced.

## Exact-move evidence

A mechanical comparison extracts the old renderer block from checkpoint
`m6.5o-r4-cli-report-model-extraction`, removes only the new `pub(super)`
visibility tokens, and compares it with the new block:

```text
renderer_exact_match=true
old_chars=8066
new_chars=8066
```

The function bodies, labels, translations, match order, JSON formatting, and
write order are unchanged.

## Test-first and mutation evidence

The initial mutation baseline selected 73 mutations in the two renderers and
seven mapping helpers. Existing integration tests caught 17 and allowed 56 to
survive. Missing direct evidence covered English statuses, several Vietnamese
statuses and labels, physical formats, and most localized error codes.

Two focused contract tests were added before moving production code. They lock
every published status, physical format, read mode, resource profile,
conformance, severity, and localized error-code mapping.

| Outcome | Initial baseline | Before extraction | After extraction |
| --- | ---: | ---: | ---: |
| Caught | 17 | 73 | 73 |
| Missed | 56 | 0 | 0 |
| Timeout | 0 | 0 | 0 |
| Unviable | 0 | 0 | 0 |

No selected output mutation survives, and mutation quality does not regress.

## Size and complexity evidence

Before extraction, `main.rs` contained 795 physical production lines. After
extraction it contains 575 production lines; `output.rs` contains 232
production lines and 149 test lines.

The reviewed production diff adds 237 lines and removes 225 lines, for 462
production additions/deletions. This remains within the 200-500 line
micro-milestone limit.

`rust-code-analysis` over the resulting production portions reports unchanged
maximums:

- maximum cognitive complexity 15 in `run`;
- maximum cyclomatic complexity 32 in `error_text`;
- maximum function SLOC 108 in `render_text`.

The moved `render_text` remains cognitive complexity 9 and cyclomatic
complexity 24. Complexity does not increase; the improvement is that output
changes are now localized to one file with direct tests.

## Module evidence

`cargo modules structure --bin seacad --package seacad-cli` shows the private
`output` module. `cargo modules orphans --bin seacad --deny --package
seacad-cli` reports no orphans.

The focused dependency graph shows only `main -> output`, with `output` using
`CliReport` and `CliLanguage`; neither module depends back on `output`. The
unfiltered `--acyclic` command continues to stop at the pre-existing
item/method self-cycle between `CliLanguage` and `CliLanguage::detect`. This is
tool noise unrelated to the new module and does not represent a module
dependency cycle.

SplitRS is not applicable: the handwritten production portion of `main.rs` was
795 lines before extraction and `output.rs` is 232 production lines, both below
the 1,000-line trigger.

## Public API evidence

`seacad-cli` is a binary crate and this diff changes only private items.
`seacad-dxf-core` is unchanged. Therefore `cargo-semver-checks` against the core
crate is not applicable; no public library item, visibility, signature, enum
variant, trait contract, or re-export changes.

## Over-engineering review

The whole-repository Ponytail audit found no production dependency, trait, or
wrapper that could be safely removed without weakening streaming, provenance,
format parity, or error handling. The post-diff Ponytail review result is
`Lean already. Ship.` The new module is an established output boundary rather
than a speculative abstraction.

## Verification result

Rust 1.97.1 verification:

- `cargo +1.97.1 fmt --all -- --check`;
- deterministic schema generation in `--check` mode;
- Clippy for the workspace and all targets with warnings denied;
- 237 workspace tests;
- `git diff --check`.

No external implementation, fixture, or source code was copied, translated, or
ported. The external architecture references and license review used for this
CLI boundary remain recorded in the M6.5o-r4 audit.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `crates/seacad-cli/src/main.rs` | `dc901e7a10fe4df21022d0ffa85f9ea6d22c1dd9ea87ca279ec646ec2706e392` |
| `crates/seacad-cli/src/output.rs` | `fbe2f8bee063ba6761bcae4883ed72be5f851a47773be8c09e0090bc0424c96c` |
| `docs/IMPLEMENTATION_PLAN.md` | `09caa33444f77e0d413ad60b31417f0f0bd928eaef6fefcc35383902b5dd1647` |
