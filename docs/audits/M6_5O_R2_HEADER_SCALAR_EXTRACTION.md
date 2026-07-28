# M6.5o-r2 HEADER Scalar Extraction Audit

## Scope

M6.5o-r2 is a behavior-preserving cleanup checkpoint. It moves the four exact
numeric scalar representations used by typed HEADER semantics out of
`header_numeric.rs` and into the private `header_scalar.rs` module:

- `DxfDouble`;
- `DxfDayParts`;
- `DxfJulianDate`;
- `DxfElapsedDays`.

It does not add or reinterpret any HEADER variable, default, range, unit,
encoding, version applicability, support claim, dependency, allocation policy,
streaming policy, provenance rule, or writer behavior. It is not part of
M6.5p.

## Boundary decision

The extracted module owns exact IEEE-754 bit preservation and the finite,
`i64`-bounded split of date-like scalars into whole and fractional days.
Calendar conversion, timezone interpretation, and HEADER document reads remain
outside this module.

`header_numeric.rs` continues to own the schema-driven numeric directory,
semantic states, raw provenance, document reads, cancellation, bounded
stack/heap selection, ASCII/Binary decoding, and structural validation.

The four public types remain available at the same crate-root paths through
direct re-exports from `lib.rs`. No public module or new public item is
introduced.

## Test-first boundary evidence

The existing integration test already covered exact IEEE-754 bits, negative
zero, NaN/non-finite rejection, positive and negative fractional-day splitting,
the exclusive upper `i64` boundary, and the public `Copy` contracts.

A pre-refactor mutation run found three surviving mutants:

- `DxfDouble::is_finite` always returning `false`;
- the lower-bound comparison changing from `<` to `==`;
- the lower-bound comparison changing from `<` to `<=`.

The test was strengthened before moving production code to require a finite
value to return `true` and to accept exactly `i64::MIN` with zero fractional
day. The strengthened pre-refactor run caught every viable mutant.

## Size and complexity evidence

Before extraction, `header_numeric.rs` contained 1,185 physical production
lines. After extraction it contains 1,078 lines; `header_scalar.rs` contains
109 lines and one cohesive responsibility.

The production diff adds 114 lines and removes 110 lines, for 224 reviewed
production additions/deletions. Tests add eight lines. Generated code and
fixtures are unchanged.

The moved function bodies are unchanged. Across the two resulting files, the
target's maximum cognitive complexity remains 11, maximum cyclomatic
complexity remains 31, and maximum function SLOC remains 106.
`DxfDayParts::from_raw` is the most complex function in the new scalar module,
with cognitive complexity 2 and cyclomatic complexity 4.

## Module evidence

`cargo modules structure --lib --package seacad-dxf-core` includes the new
private `header_scalar` module. `cargo modules orphans --lib --deny` reports no
orphans.

The new dependency is one-way from `header_numeric` to `header_scalar`; the
scalar module has no dependency back into HEADER decoding. The pre-existing
unfiltered `--acyclic` item-level cycle between `DxfSourceId` and its
`from_sha256` constructor remains baseline tool noise and is unrelated to this
diff. No new module cycle is introduced.

## Mutation evidence

The same 32 mutants were selected after strengthening the pre-refactor tests
and after extraction:

| Outcome | Before extraction | After extraction |
| --- | ---: | ---: |
| Caught | 22 | 22 |
| Missed | 0 | 0 |
| Timeout | 0 | 0 |
| Unviable | 10 | 10 |

The unviable mutants attempted to construct `Default` values for types that
deliberately do not implement `Default`. No viable mutant survives.

## Public API evidence

The semver gate forces patch compatibility because the crate version remains
`0.0.0`:

```text
cargo semver-checks --package seacad-dxf-core \
  --baseline-rev m6.5o-r1-ascii-numeric-parser-extraction \
  --release-type patch
```

It passes 223 checks and skips 30 non-applicable checks. No public item,
visibility, signature, enum variant, trait contract, or crate-root re-export is
lost.

## Over-engineering review

The post-diff Ponytail review result is `Lean already. Ship.` The private module
contains one established responsibility, no trait, factory, configuration
layer, dependency, wrapper, or speculative extension point.

## Verification result

Rust 1.97.1 verification:

- `cargo +1.97.1 fmt --all -- --check`;
- deterministic schema generation in `--check` mode;
- Clippy for the workspace and all targets with warnings denied;
- 233 workspace tests;
- `git diff --check`.

No external implementation, fixture, or source code was copied, translated, or
ported.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `crates/seacad-dxf-core/src/header_scalar.rs` | `2c585e06b8ce7566adb9df7153389ee7c3da91fbf0c4210f8fd7a975c990dedd` |
| `crates/seacad-dxf-core/src/header_numeric.rs` | `0beac9c34643b36b89bbbf84f37c596e76d8ba97fd53dd15b9fe5d0863985d92` |
| `crates/seacad-dxf-core/src/lib.rs` | `fb6027e7eb81a302ddeabbf9ca8fcc96c34697bfc92c2cb2f748d9aa17587f5f` |
| `crates/seacad-dxf-core/tests/header_numeric_tests.rs` | `6797e85a7b898baca2f79583600ea3d3455e43becf02efa59c3b21e922c9c368` |
| `docs/IMPLEMENTATION_PLAN.md` | `41a244a3283832f9e0976b2a0f2ff59e608d4398eedc9f4a6d7083ea4701be4a` |
