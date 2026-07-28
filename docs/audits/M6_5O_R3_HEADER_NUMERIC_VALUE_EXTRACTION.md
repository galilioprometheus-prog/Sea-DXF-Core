# M6.5o-r3 HEADER Numeric Value Extraction Audit

## Scope

M6.5o-r3 is a behavior-preserving cleanup checkpoint. It moves the typed result
contract used by numeric HEADER semantics out of `header_numeric.rs` and into
the private `header_numeric_value.rs` module:

- `DxfHeaderNumericIssue`;
- `DxfHeaderNumericValue`;
- tuple state projection for `Double2` and `Double3`;
- first source-ordered raw-provenance projection.

It does not add or reinterpret any HEADER variable, default, range, unit,
encoding, version applicability, support claim, dependency, allocation policy,
streaming policy, provenance rule, or writer behavior. It is not part of
M6.5p.

## Boundary decision

The extracted module owns the public invalidity/result enums and the pure
projection of their contained semantic values into state, field provenance,
raw provenance, and typed accessors.

`header_numeric.rs` continues to own the schema-driven directory and view,
document reads, cancellation, bounded stack/heap selection, ASCII/Binary
decoding, structural validation, and construction of semantic values.

The two public types remain available at the same crate-root paths through
direct re-exports from `lib.rs`. No public module or new public item is
introduced.

## Test-first boundary evidence

The initial pre-refactor mutation run selected 33 mutants and exposed ten
survivors. Existing integration tests did not directly prove:

- tuple-wide `Defaulted` and `Absent` state projection;
- raw provenance when the first evidence appears in the second or third tuple
  component;
- `None` when every tuple component lacks raw provenance.

A focused public-contract test was added before moving production code. The
strengthened baseline catches all viable semantic mutations. One timeout
replaces loop progress with multiplication by the initial zero index, creating
a deliberately non-progressing loop.

## Size and complexity evidence

Before extraction, `header_numeric.rs` contained 1,078 physical production
lines. After extraction it contains 848 lines;
`header_numeric_value.rs` contains 241 lines and one cohesive responsibility.

The production diff adds 248 lines and removes 238 lines, for 486 reviewed
production additions/deletions. Tests add one focused test. Generated code and
fixtures are unchanged.

The set and bodies of production functions are unchanged. Re-running
`rust-code-analysis` over the two resulting files reports 82 functions,
maximum cognitive complexity 11, maximum cyclomatic complexity 35, and maximum
function SLOC 106.

Re-reading the archived r1 JSON report showed that the r2 audit's recorded
cyclomatic maximum of 31 was a transcription error: both archived and current
reports contain 35 for `double_tuple_field`. The r2 audit is corrected in this
checkpoint; no production metric changed.

## Module evidence

`cargo modules structure --lib --package seacad-dxf-core` includes the new
private `header_numeric_value` module. `cargo modules orphans --lib --deny`
reports no orphans.

The new dependency is one-way from `header_numeric` to
`header_numeric_value`; the result-contract module has no dependency back into
document decoding. The unfiltered `--acyclic` command continues to stop at
pre-existing item/constructor self-cycles such as `ByteSpan` and
`from_validated_bounds`. This baseline tool noise is unrelated to the new
module, which introduces no cycle.

## Mutation evidence

The same 33 mutants were selected after strengthening the pre-refactor tests
and after extraction:

| Outcome | Initial audit | Before extraction | After extraction |
| --- | ---: | ---: | ---: |
| Caught | 10 | 19 | 19 |
| Missed | 10 | 0 | 0 |
| Timeout | 0 | 1 | 1 |
| Unviable | 13 | 13 | 13 |

The single timeout is the same non-progress mutation before and after the move.
No viable mutant survives and mutation quality does not regress.

## Public API evidence

The semver gate forces patch compatibility because the crate version remains
`0.0.0`:

```text
cargo semver-checks --package seacad-dxf-core \
  --baseline-rev m6.5o-r2-header-scalar-extraction \
  --release-type patch
```

It passes 223 checks and skips 30 non-applicable checks. No public item,
visibility, signature, enum variant, trait contract, or crate-root re-export is
lost.

## Over-engineering review

The post-diff Ponytail review result is `Lean already. Ship.` The private module
contains one established result-contract responsibility, no trait, factory,
configuration layer, dependency, wrapper, or speculative extension point.

## Verification result

Rust 1.97.1 verification:

- `cargo +1.97.1 fmt --all -- --check`;
- deterministic schema generation in `--check` mode;
- Clippy for the workspace and all targets with warnings denied;
- 234 workspace tests;
- `git diff --check`.

No external implementation, fixture, or source code was copied, translated, or
ported.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `crates/seacad-dxf-core/src/header_numeric_value.rs` | `bb6e12887033d49fee8ecbd7ac9ffb9edb0639367e70611cef00f92cda69d86e` |
| `crates/seacad-dxf-core/src/header_numeric.rs` | `f9b60e68c9c7001f9dbfbcfc92925482f3072d4dd20a220ad079d0db00b45ba2` |
| `crates/seacad-dxf-core/src/lib.rs` | `dec463e2aebc686a2973090e60edc9e3f398c0b5a76e81640f55f833021f8c03` |
| `crates/seacad-dxf-core/tests/header_numeric_tests.rs` | `3bdd1db297a03c83fe340c02bcb0cde838b3a24b7f5732ccc058305505cb5c7c` |
| `docs/IMPLEMENTATION_PLAN.md` | `55f247018eadfe0a065870c04273c9c163315cf18c1c5cf27b2507a46e32410b` |
| `docs/audits/M6_5O_R2_HEADER_SCALAR_EXTRACTION.md` | `cfb219818b6d7ae4ff82df40724b8166b3e3cd971b437aabc3d24836833c89d6` |
