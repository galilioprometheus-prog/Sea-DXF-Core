# M9.2g Classic POLYLINE Record Semantics

Retrieved: 2026-07-30

## Normative boundary

- Autodesk [POLYLINE (DXF)](https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-ABF6B778-BE20-4B49-9B58-A94E64CEFFF3.htm)
  says dummy X/Y `10/20` are always zero and group `30` is the polyline
  elevation. M9.2g treats all three components as required but does not enforce
  the dummy-zero invariant.
- The same table documents zero defaults for thickness `39`, default widths
  `40/41`, flags `70`, mesh counts/densities `71`-`74`, and smooth-surface type
  `75`, plus extrusion default `(0, 0, 1)` for `210/220/230`.
- It documents eight independent flag bits: closed/mesh-closed-M, curve-fit
  vertices, spline-fit vertices, 3D polyline, polygon mesh, mesh-closed-N,
  polyface mesh, and continuous linetype generation.
- Group `66` is obsolete, optional, and must be ignored if present. M9.2g does
  not project it into semantic state; its exact evidence remains in M9.2b/f.

## Implementation contract

`crates/seacad-dxf-core/src/polyline_record_semantic.rs` provides:

- one lazy semantic directory owning the complete M9.2f card/evidence graph;
- required source-anchored dummy X/Y/elevation semantics with typed missing,
  invalid-number, and multiple-value failures;
- documented defaults for absent thickness, default widths, flags, mesh
  counts/densities/type, and independent extrusion components;
- exact explicit binary64 bits and signed-i16 values with raw provenance;
- helpers for all eight documented flag bits, returning unavailable when flags
  are invalid or duplicated;
- raw-record, ASCII-document, and Binary-document adapters plus validated
  lookups and bounded cancellation;
- no semantic projection for obsolete entities-follow group `66`.

## Test evidence

`crates/seacad-dxf-core/tests/polyline_record_semantic_tests.rs` covers:

- ASCII/Binary parity across every supported AC1009-AC1032 dialect;
- exact signed-zero bits, all fifteen projected fields, and all eight flag
  helpers;
- missing required components and every documented optional default;
- invalid doubles/integers, duplicates, negative mesh metadata without domain
  guessing, unrecognized surface-type values, and partial extrusion state;
- explicit proof that group `66` remains available in cards but does not affect
  semantics, and that following VERTEX values remain excluded;
- semantic availability for closed, interrupted, and unclosed sequences;
- source/raw provenance, lookup bounds, cancellation, and public traits.

The parity fixtures prove physical decoding only. They do not claim that every
field is semantically applicable to every `$ACADVER` dialect.

## Review-size deviation

The self-contained production module is 503 lines, three lines above the
preferred 500-line upper target. Keeping the shared integer/double projection,
all documented defaults, and flag helpers in one module makes the complete
record policy directly inspectable without hiding behavior behind another
abstraction.

## Non-claims

M9.2g does not enforce dummy X/Y zero; validate widths, counts, densities,
surface type, or extrusion; reject negative or unknown signed values; reconcile
contradictory flags; infer a single polyline family; apply mesh metadata to
VERTEX records; interpret VERTEX integer fields; resolve polyface faces;
transform OCS/WCS; assemble geometry; edit; write; render; or diagnose
conformance.

## Required checkpoint gates

- `cargo deny --locked check`
- `cargo +1.97.1 fmt --all -- --check`
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`
- `cargo +1.97.1 clippy --locked --workspace --all-targets -- -D warnings`
- `cargo +1.97.1 test --locked --workspace`
- `git diff --check`

All checkpoint gates passed on 2026-07-30:

- dependency policy: advisories, bans, licenses, and sources all `ok`;
- formatting and generated-schema drift checks passed;
- workspace Clippy passed with warnings denied;
- workspace tests: 386 passed, 0 failed, 0 ignored, including all three focused
  M9.2g integration tests;
- `git diff --check` passed.

No dependency manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/polyline_record_semantic.rs` | 503 | `90c0e336e9e482282d457a4226ec3c0f627e8850ac2986791a81088857efcc08` |
| `crates/seacad-dxf-core/src/lib.rs` | 347 | `7ed049c0d2f3fca5d688431b285bede78afbd8ad8adfc9fbd743a1eb3cf39c11` |
| `crates/seacad-dxf-core/tests/polyline_record_semantic_tests.rs` | 407 | `02832471a466f34d11a22b188c77cc3ec79c05653a88142974fcd20277952b48` |
| `docs/IMPLEMENTATION_PLAN.md` | 569 | `0552e0ebc856cc5c3f3f54e7b9b0a052bfed258a43a7702522057fd2ad73a605` |
| `docs/SUPPORT_MATRIX.md` | 479 | `2a928ef541a88ba4e8e81388d99160b5afc7aab447dce7677543cdf1312fdb1d` |
