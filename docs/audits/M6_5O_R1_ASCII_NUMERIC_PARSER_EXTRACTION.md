# M6.5o-r1 ASCII Numeric Parser Extraction Audit

## Scope

M6.5o-r1 is a behavior-preserving cleanup checkpoint. It moves the existing
strict ASCII signed-16-bit and floating-point token parsing responsibility out
of `header_numeric.rs` and into the private `ascii_numeric.rs` module.

It does not add or reinterpret any HEADER variable, default, range, unit,
encoding, version applicability, support claim, dependency, allocation policy,
streaming policy, provenance rule, or writer behavior. It is not part of
M6.5p.

## Boundary decision

The extracted module owns:

- the public, crate-root-re-exported `DxfAsciiNumericIssue`;
- strict signed `i16` token parsing;
- strict finite `f64` token parsing, including overflow and underflow
  rejection;
- syntax-offset reporting and mantissa inspection;
- two focused unit tests for signed boundaries, exact negative-zero bits,
  invalid syntax, overflow, and underflow.

`header_numeric.rs` continues to own document reads, cancellation, bounded
stack/heap selection, raw provenance, invalid semantic-state mapping, Binary
decoding, `DxfDouble`, and the schema-driven HEADER directory.

The ASCII parser returns the primitive `f64`; the caller immediately constructs
`DxfDouble` with `from_f64`, exactly where the prior parser did so. This keeps
the low-level parser independent of HEADER semantics and avoids a reverse
module dependency.

The public crate-root path and shape of `DxfAsciiNumericIssue` remain unchanged.
No new public item is introduced.

## Size and complexity evidence

Before extraction, `header_numeric.rs` contained 1,286 physical source lines.
After extraction it contains 1,185, a reduction of 101 lines and one complete
responsibility. The new module contains 107 production lines before its
`#[cfg(test)]` section and 32 lines of focused tests.

The production diff adds 118 lines and removes 111 lines, for 229 reviewed
production additions/deletions. The file was not fragmented into generated
`types`, `constants`, `traits`, and `functions` buckets proposed by the SplitRS
dry-run.

`rust-code-analysis` reports unchanged target complexity:

| Function | Cognitive before | Cognitive after | Cyclomatic before | Cyclomatic after | SLOC |
| --- | ---: | ---: | ---: | ---: | ---: |
| signed `i16` parser | 13 | 13 | 12 | 12 | 29 |
| double syntax validator | 14 | 14 | 14 | 14 | 38 |

The refactor reduces file responsibility; it does not claim that moving a
function reduces the function's intrinsic complexity.

## Module evidence

`cargo modules structure --lib` includes the new private `ascii_numeric`
module. `cargo modules orphans --lib --deny` reports no orphans.

The module-only dependency graph changes from 34 nodes and 152 use edges to 35
nodes and 153 use edges. The new edge is one-way from `header_numeric` to
`ascii_numeric`. The three pre-existing cyclic strongly connected components
are unchanged:

1. `ascii_document`, `binary_document`, `raw_document`;
2. `ascii_group`, `ascii_line`, `binary_wire`, `dialect`, `error`, `source`;
3. `johab`, `text_decoder`.

The tool's unfiltered `--acyclic` check still stops at an existing item-level
type/constructor relationship. This is recorded baseline noise, not evidence
of a new module cycle.

## Mutation evidence

The same 35 mutants were selected before and after extraction for the signed
integer parser and double syntax validator:

| Outcome | Before | After |
| --- | ---: | ---: |
| Caught | 32 | 32 |
| Missed | 0 | 0 |
| Timeout | 3 | 3 |
| Unviable | 0 | 0 |

All three timeouts replace loop progress (`+=`) with multiplication by the
initial zero index (`*=`), creating a non-progressing loop. No surviving mutant
was introduced.

## Public API evidence

The default semver invocation is not a sufficient gate for SeaCad while the
crate version is `0.0.0`: it treats the comparison as a major release and skips
all 253 checks.

The required invocation therefore forces patch compatibility:

```text
cargo semver-checks --package seacad-dxf-core \
  --baseline-rev m6.5o-dimension-formatting-precision-integers \
  --release-type patch
```

It passes 223 checks and skips 30 non-applicable checks. No public item,
visibility, signature, enum variant, trait contract, or re-export is lost.

## Over-engineering review

The post-diff Ponytail review result is `Lean already. Ship.` The private module
has one cohesive responsibility, two parser entry points used by three
production call sites, no trait, no factory, no configuration layer, no
dependency, and no speculative extension point. No further deletion or generic
abstraction was justified.

## Verification coverage

The focused unit tests and the existing nine HEADER numeric integration tests
pass. They preserve signed boundaries, exact IEEE-754 bits including negative
zero, invalid syntax offsets, overflow/underflow rejection, ASCII/Binary
version parity, provenance, structural conflicts, and cancellation.

No external implementation, fixture, or source code was copied, translated, or
ported.

## Verification result

Rust 1.97.1 passed:

- `cargo +1.97.1 fmt --all -- --check`;
- deterministic schema generation in `--check` mode;
- Clippy for the workspace and all targets with warnings denied;
- 233 tests: 202 core unit tests, 9 numeric integration tests, 12 CLI tests,
  and 10 schema-generator tests;
- `git diff --check`.

No dependency, schema row, fixture, public API, unsafe block, source-sized
allocation, parser behavior, or support claim was added.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `crates/seacad-dxf-core/src/ascii_numeric.rs` | `5f00452efe6db4e96c4a35ea58aaedd11aafe905ddfe5d5ff51f4788f81078ef` |
| `crates/seacad-dxf-core/src/header_numeric.rs` | `f992e88b87557a34298566efb7603e5fd69469b6a108df02c5091cea7ec5df80` |
| `crates/seacad-dxf-core/src/lib.rs` | `3bc3651888e8e2fb8d5ce4741207ff20d845a43e60870c1dd87ef16ab4fef97c` |
| `docs/IMPLEMENTATION_PLAN.md` | `3543ca04eb975b1c27af7bb9710452c732bafed8bbac44cdc949e016a46d6bb5` |
