# M14.3bb Entity Draft Applicability Admission

## Scope

M14.3bb admits one M14.3ba draft identity to the future record encoder only
when the document has one supported dialect and the generated applicability
registry marks that exact canonical or alias name as applicable.

## Evidence boundary

- The generated M14.3f/M14.3g 59-name registry and applicability descriptors
  remain the sole source of admitted names and version ranges.
- The reviewed Autodesk compatibility receipt already present in
  `schema/dxf/v1/entity_applicability.json` covers DGN/DWF underlays from
  AC1021 and PDF underlays from AC1024.
- M14.3ba remains the reviewed source of exact name, handle reservation,
  placement, owner binding, and reversible `$HANDSEED` transaction.
- This checkpoint introduces no new Autodesk wire fact and changes no schema
  source receipt.

The user-authorized legacy trees under `D:\SeaCad\tham khảo\New folder` were
searched read-only. They contain entity-specific readers and topology gates but
no generic source-bound editable draft applicability admission primitive. No
external or legacy code, fixture, data, or dependency was copied, translated,
vendored, or linked.

## Contract

- `prepare_entity_draft_applicability` consumes one prepared identity and
  checks cancellation before any admission work.
- The identity and its reservation transaction must retain the exact raw
  document source identity and pass the transaction source precondition.
- `$ACADVER` must be one unambiguous supported dialect. Absent, unsupported,
  invalid, and ambiguous states return `VersionUnavailable` without lookup or
  mutation.
- The exact generated name classification must resolve to its matching
  generated applicability descriptor. Registry mismatch is an internal-data
  error rather than a fallback.
- `Applicable` produces a plan retaining the identity, dialect, descriptor,
  provenance, owner, handle, and reversible transaction.
- `NotApplicable` retains the exact classification, dialect, and reviewed
  minimum/maximum range. `NotYetReviewed` remains a separate typed issue.
- No unknown/custom name or guessed version range can reach the admitted plan.
- ASCII, Binary, and format-neutral raw document views expose the same API;
  preparation performs no untrusted-size allocation and writes no bytes.

## Reviewed matrix boundary

All 59 generated names are evaluated on every nine-dialect ASCII/Binary pair.
DGNUNDERLAY and DWFUNDERLAY are admitted from AC1021; PDFUNDERLAY is admitted
from AC1024. Earlier versions return `NotApplicable`. The other 56 names return
`NotYetReviewed` for every dialect. This deliberately blocks future encoding
until normative applicability evidence is added to the generated registry.

## Nonclaims

This checkpoint does not add applicability evidence, encode group 0, handle 5,
owner 330, subclass markers, or family fields; validate an underlay payload;
insert a record; verify inserted semantics; update, clone, or delete an entity;
or advance any entity topic to `Complete`.

## Verification

Focused tests cover all 59 names over all 18 dialect/format pairs, the three
reviewed underlay ranges and 56 unreviewed names, every unavailable `$ACADVER`
state, source mismatch, cancellation, descriptor provenance, public bounds,
strict Binary reparse, and byte-identical inverse materialization. The focused
suite passed 4/4 tests and the full workspace passed 898/898 tests. Generated
schema and release-evidence checks, `cargo deny --locked check`, formatting,
workspace Clippy with warnings denied, workspace tests, the forbidden
production-macro scan, and `git diff --check` passed. Production adds 192
lines: 188 in the new module and four module/export lines, near the usual
200-500 line checkpoint target. No production dependency changed. This audit
intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 252 | `6289ec0e0c9967ec6765d019607ffc603a43af66b3a69061db09d2533da04a5a` |
| `crates/seacad-dxf-core/src/entity_draft_applicability.rs` | 188 | `7d28b4b0109201a3a2b9f83692515f5e135adc42444c8dcd7af243676a0f4ed1` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,058 | `79810798f831356f996e1740ecd9f003ae49c429f33fc2f6a7ac3e29c6e2be55` |
| `crates/seacad-dxf-core/tests/entity_draft_applicability_tests.rs` | 439 | `9118fe959d9cf6d570dd672552405b787818deeb422097b0c71f0fd58c146f02` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 925 | `22500d522acc936614de0e59a10f5b79fd361f00b5675cc751852da30f500675` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,355 | `39f527dabdd6d7bdfde91dd517c32a68e2a5cd4aa0eb67900d1aa632d92c5fce` |
| `docs/SUPPORT_MATRIX.md` | 2,007 | `562503a5f61a6647a33426a67671a5096eb7430fb7533b1e1bdda1640738641f` |
