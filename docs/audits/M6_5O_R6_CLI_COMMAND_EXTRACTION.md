# M6.5o-r6 CLI Command Extraction Audit

## Scope

M6.5o-r6 is a behavior-preserving cleanup checkpoint. It moves the Clap command
tree, parsed `CliOptions`, `CliAction`, and root help writer out of `main.rs`
and into the private `command.rs` module.

It does not change argument names, defaults, validation, help text, usage text,
DXF parsing, semantics, schema rows, provenance, streaming, allocation limits,
source-file handling, report construction, output, localization, exit codes,
path redaction, dependencies, public API, or support claims. It is not part of
M6.5p.

## Boundary decision

`command.rs` owns one established input-boundary responsibility:

- the root and inspect/verify Clap command definitions;
- global and subcommand option definitions;
- conversion from validated `ArgMatches` into `CliOptions`;
- `CliAction` and its stable command name;
- root help writing and direct help/usage contract tests.

`main.rs` continues to own process orchestration, execution, file access, DXF
dispatch, report construction, failures, and exit routing. `output.rs`,
`report.rs`, and `locale.rs` retain their existing responsibilities.

The dependency direction is:

```text
main -> command -> locale
```

Only `CliAction`, `CliOptions`, their members needed by execution, and the three
command entry points use `pub(super)`. Subcommand and help-argument builders
remain private. No public module, trait, builder abstraction, factory, wrapper,
configuration layer, dependency, or extension point is introduced.

## Exact-move evidence

A mechanical comparison extracts the old command block from checkpoint
`m6.5o-r5-cli-output-extraction`, removes only the new `pub(super)` visibility
tokens, and compares it with the new block:

```text
command_exact_match=true
old_chars=7477
new_chars=7477
```

Command names, argument order, defaults, accepted values, help text, usage
text, and parser-state errors are unchanged.

## Test-first and mutation evidence

The initial mutation baseline selected 16 mutations in the command contract.
Existing tests caught ten, one replacement was unviable, and five survived:
the root help writer plus inspect/verify usage selection in English and
Vietnamese.

A focused contract test was added before moving production code. It directly
locks the root usage, terminal newline, and all four subcommand usage strings.

| Outcome | Initial baseline | Before extraction | After extraction |
| --- | ---: | ---: | ---: |
| Caught | 10 | 15 | 15 |
| Missed | 5 | 0 | 0 |
| Timeout | 0 | 0 | 0 |
| Unviable | 1 | 1 | 1 |

The unviable mutation tries to replace `CliOptions::from_matches` with
`Ok(Default::default())`; `CliOptions` intentionally has no fabricated default.
Every viable selected mutation is caught before and after extraction.

## Size and complexity evidence

Before extraction, `main.rs` contained 575 physical production lines. After
extraction it contains 364 production lines; `command.rs` contains 224
production lines and 45 test lines.

The reviewed production diff adds 229 lines and removes 216 lines, for 445
production additions/deletions. This remains within the 200-500 line
micro-milestone limit.

`rust-code-analysis` over `main.rs` and `command.rs` reports unchanged target
maximums:

- maximum cognitive complexity 15 in `run`;
- maximum cyclomatic complexity 12 in `run`;
- maximum function SLOC 83 in `cli_command`;
- command parsing maximum cognitive complexity 4 and cyclomatic complexity 10
  in `CliOptions::from_matches`.

Complexity does not increase. Command definition and parser-state debugging are
now localized without changing any function body.

## Module evidence

`cargo modules structure --bin seacad --package seacad-cli` shows the private
`command` module. `cargo modules orphans --bin seacad --deny --package
seacad-cli` reports no orphans.

The focused dependency graph shows `main` using `command`, with `command` using
only `CliLanguage` inside the binary plus the existing Clap and core types. No
module depends back on `command`. The unfiltered `--acyclic` command continues
to stop at pre-existing item/method self-cycles such as `CliLanguage` and
`CliLanguage::detect`; this tool noise does not represent a module cycle.

SplitRS is not applicable: the handwritten production portion of `main.rs` was
575 lines before extraction and `command.rs` is 224 production lines, both
below the 1,000-line trigger.

## Public API evidence

`seacad-cli` is a binary crate and this diff changes only private items.
`seacad-dxf-core` is unchanged. Therefore `cargo-semver-checks` against the core
crate is not applicable; no public library item, visibility, signature, enum
variant, trait contract, or re-export changes.

## Over-engineering review

The post-diff Ponytail review result is `Lean already. Ship.` The new module
contains a concrete command boundary and its parsed data, not a speculative
interface. Existing Clap builders are retained directly rather than wrapped in
another abstraction.

## Verification result

Rust 1.97.1 verification:

- `cargo +1.97.1 fmt --all -- --check`;
- deterministic schema generation in `--check` mode;
- Clippy for the workspace and all targets with warnings denied;
- 238 workspace tests;
- `git diff --check`.

No external implementation, fixture, or source code was copied, translated, or
ported.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `crates/seacad-cli/src/main.rs` | `3af832ebf9a41f003fd1c0b013595348da56288a375275581fa445e5dfe3dc3b` |
| `crates/seacad-cli/src/command.rs` | `d2983d254a40262314389dbac039e934463e1c3256545fb53defffb1835a7a46` |
| `docs/IMPLEMENTATION_PLAN.md` | `7c1104d4e1381cd927e98d6384febcf1bdbf47315488d7c9db7a423d71beb0ac` |
