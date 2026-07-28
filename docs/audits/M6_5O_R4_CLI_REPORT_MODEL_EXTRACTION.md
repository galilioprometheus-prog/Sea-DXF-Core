# M6.5o-r4 CLI Report Model Extraction Audit

## Scope

M6.5o-r4 is a behavior-preserving cleanup checkpoint. It moves the CLI's
serializable report data and the stable names used at its output boundary out
of `main.rs` and into the private `report.rs` module.

It does not change DXF parsing, semantics, schema rows, provenance, streaming,
allocation limits, source-file handling, JSON schema v1, human output,
localization, exit codes, path redaction, dependencies, public API, or support
claims. It is not part of M6.5p.

## Boundary decision

`report.rs` owns only:

- the data-transfer structs serialized by the CLI;
- exact enum-to-output names for read mode, resource profile, physical format,
  ASCII/Binary conformance, and diagnostic severity;
- a focused test that locks every published core enum variant used by those
  names.

`main.rs` continues to own argument parsing, command execution, file access,
DXF dispatch, report construction, JSON and localized text rendering, errors,
and exit codes. `CliOutcome` remains in `main.rs` because it is an execution
result rather than serialized report data.

All moved items use `pub(super)`, the narrowest visibility that lets the parent
binary module construct and render them. No public module, trait, builder,
factory, wrapper, configuration layer, or extension point is introduced.

## External architecture review

These repositories were read only as architectural references. No code,
fixture, dependency, or implementation was copied, translated, ported, or
linked.

| Reference | License | Applied to SeaCad | Deliberately not adopted |
| --- | --- | --- | --- |
| [rust-lang/rust-analyzer](https://github.com/rust-lang/rust-analyzer) and its [architecture guide](https://rust-analyzer.github.io/book/contributing/architecture.html) | MIT OR Apache-2.0 | Keep protocol/serialization shapes at the outer binary boundary; keep core types independent of JSON; make boundaries directly testable. | Its assumption that the full source input can be held in memory does not fit large CAD files. SeaCad does not add Salsa, Rowan, or an in-memory whole-document requirement. |
| [astral-sh/ruff](https://github.com/astral-sh/ruff) | MIT | Prefer fast deterministic local/CI gates and stable machine-readable output; keep core processing separate from presentation. | SeaCad does not adopt automatic source rewriting, overwrite DXF input, or add Ruff code/dependencies. |
| [helix-editor/helix](https://github.com/helix-editor/helix) | MPL-2.0 | Preserve a core/view/terminal-style direction for later UI work: the format core must not depend on a renderer or GUI. | No GUI, tree-sitter, renderer, or MPL implementation is added in this checkpoint. |

The immediate application is intentionally small: `seacad-dxf-core` remains
unaware of CLI serialization, while the CLI report contract becomes easy to
locate, test, and change without searching through command execution and
localized rendering.

## Test-first and mutation evidence

The initial mutation baseline selected 26 mutations in the six stable-name
functions and caught 16. Ten survived because the CLI integration tests did not
directly exercise both resource profiles, every physical-format result, both
recovered conformance variants, and all diagnostic severities.

A focused variant-coverage test was added before the production move. The
strengthened baseline and post-refactor run have identical results:

| Outcome | Initial baseline | Before extraction | After extraction |
| --- | ---: | ---: | ---: |
| Caught | 16 | 25 | 25 |
| Missed | 10 | 1 | 1 |
| Timeout | 0 | 0 | 0 |
| Unviable | 0 | 0 | 0 |

The one remaining missed mutation deletes the explicit
`DxfPhysicalFormat::Unknown => "unknown"` arm. The non-exhaustive fallback also
returns `"unknown"`, so the mutation is behaviorally equivalent. No distinct
output mutation survives, and mutation quality does not regress.

## Size and complexity evidence

Before extraction, `main.rs` contained 900 physical production lines. After
extraction it contains 795 production lines; `report.rs` contains 120
production lines and 44 test lines.

The reviewed production diff adds 129 lines and removes 114 lines, for 243
production additions/deletions. The set and bodies of moved name functions are
unchanged.

`rust-code-analysis` over the resulting production portions reports:

- maximum cognitive complexity 15 in unchanged `run`;
- maximum cyclomatic complexity 32 in unchanged `error_text`;
- maximum function SLOC 108 in unchanged `render_text`;
- moved report functions at cognitive complexity 0-1.

The maximum complexity of the target does not increase. This checkpoint makes
one responsibility easier to locate; it does not claim that the remaining
`main.rs` work is finished.

## Module evidence

`cargo modules structure --bin seacad --package seacad-cli` shows the private
`report` module and its report structs. `cargo modules orphans --bin seacad
--deny --package seacad-cli` reports no orphans.

The focused dependency graph is one-way from the binary root to `report`; the
new module has no dependency back into command execution, localization, or
rendering. The unfiltered `--acyclic` command continues to stop at the
pre-existing item/method self-cycle between `CliLanguage` and
`CliLanguage::detect`. This is tool noise unrelated to the new module and does
not represent a module dependency cycle.

SplitRS is not applicable: the handwritten production portion of `main.rs` was
900 lines before extraction, below the 1,000-line trigger. Its total physical
length exceeded 1,000 only because tests remain colocated with the binary.

## Public API evidence

`seacad-cli` is a binary crate and this diff changes only private items.
`seacad-dxf-core` is unchanged. Therefore `cargo-semver-checks` against the core
crate is not applicable to this checkpoint; no public library item, visibility,
signature, enum variant, trait contract, or re-export changes.

## Over-engineering review

The post-diff Ponytail review result is `Lean already. Ship.` Keeping
`CliOutcome` in `main.rs` avoids mixing execution control with serializable
report data. The extracted module has one established responsibility and no
speculative abstraction.

## Verification result

Rust 1.97.1 verification:

- `cargo +1.97.1 fmt --all -- --check`;
- deterministic schema generation in `--check` mode;
- Clippy for the workspace and all targets with warnings denied;
- 235 workspace tests;
- `git diff --check`.

No external implementation, fixture, or source code was copied, translated, or
ported.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `crates/seacad-cli/src/main.rs` | `9c393e6661c38093cea390f609789bd000675df47b342ffd90be38c67363d1e8` |
| `crates/seacad-cli/src/report.rs` | `2aff1b521cad59cee0939d151b3b71a487b005c735f5ae7e61c74cdaf570a8bd` |
| `docs/IMPLEMENTATION_PLAN.md` | `f04c81f65fadb8eb32aefed3586f605e32e70777d88ab7fe8de6754cd3780fb1` |
