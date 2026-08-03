# M14.3cd Multi-POINT Delete

Retrieved: 2026-08-03

## Scope

M14.3cd replaces the single pending delete slot with a bounded delete-only
batch for multiple distinct canonical POINT records. Every member preserves
the existing identity and incoming-reference rules, and all raw removals commit
as one transaction. It does not mix deletion with updates or inserts, admit a
reference-closure cascade, delete another entity family, or advance POINT to
`Complete`.

## Prior evidence

- M14.3aw establishes deterministic source-bound transaction composition and
  exact executable inverses, including adjacent deletion coalescing.
- M14.3ca establishes reference-safe handle-backed canonical POINT deletion.
- M14.3cc adds distinct handleless admission and final entity-count semantic
  verification.

No legacy or external parser code, fixture, data, dependency, generated
artifact, or private corpus content was copied, translated, vendored, linked,
or used at runtime.

## Contract

- A delete-only `DxfEntityEditSession` may queue more than one distinct
  canonical POINT through repeated `delete` calls.
- Each request independently validates classification, record-local identity,
  document uniqueness when handled, and absence of uniquely resolved incoming
  pointers or owners from another record.
- The shared edit limit is applied before each admission, and fallible storage
  reservation occurs before the session is changed.
- Repeating a queued key returns `DxfEntityDeleteIssue::DuplicateDelete` and
  leaves every accepted operation intact.
- Finalization composes every exact record-span transaction in source order.
  Handle-backed expectations require every identity to disappear. Every
  handleless member retains the same exact final entity count after the entire
  batch, so verification cannot use an intermediate count.
- Strict post-image reparse, transaction-byte verification, semantic
  postconditions, and exact inverse materialization all complete before a
  verified journal escapes.

## Verification boundary

Two adjacent handle-backed POINT records followed by a LINE exercise all 18
ASCII/Binary AC1009-through-AC1032 pairs. They prove two admissions, bounded
queue accounting, source-order transaction composition, neighboring LINE
retention, both handle-absence postconditions, strict verification, and byte-
identical inverse restoration. A separate handled/handleless batch proves the
combined identity expectation path and exact final entity-count postcondition.
Duplicate-key coverage proves typed rejection without losing the existing
batch.

## Nonclaims

This checkpoint does not admit incoming references from another member of the
requested delete set, compute graph closure, cascade owner graphs, mix deletion
with update/insert/clone work, delete non-POINT families, or claim complete
entity editing.

## Verification

The focused POINT edit suite passed 42/42 tests. The full workspace passed
957/957 tests, including all 17 locale tests. Generated-schema and release-
evidence checks, `cargo deny check`, formatting, workspace Clippy with warnings
denied, the production forbidden-construct scan, and `git diff --check` all
passed. Production changes are 69 insertions and 41 deletions; focused test
changes are 201 insertions. No manifest, lockfile, dependency, committed
fixture, generated schema, locale source, or external corpus changed. This
audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 365 | `033eba816943d01aa411743dca4d134d591cce621e7195e09fa1b1f8891db7dc` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 2,477 | `b454bbab2c309cc901cc588ec58461d087a2bb64c0e47e4963a71888ab26429d` |
| `crates/seacad-dxf-core/src/entity_edit_verification.rs` | 1,240 | `78dc656b957db7379761bfe4b4a2512628c7e8e393c234e67f4f1667751c172c` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,068 | `64aca4f5d173a42a8ad0116e2bfe93340012ca49e5531418d8a984a55eea5536` |
| `crates/seacad-dxf-core/src/point_edit.rs` | 1,084 | `bcef8c022f2c17e5bf672a06692a9c28787ef7e56fbc24194c0ab44a33c46e51` |
| `crates/seacad-dxf-core/tests/entity_insert_session_tests.rs` | 1,027 | `a58c0c0b979a176093ec5f15716c0d512d54d9f83c9592d60c2235328bfe69d8` |
| `crates/seacad-dxf-core/tests/point_edit_session_tests.rs` | 3,260 | `7229f4713c7fe2be222c7726ef54c1227f28e2ae6919a4b3371137663b7f7999` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,258 | `131af5040e1bb2a18c6ea9c24d733e493bb71a7ba8e2598dd5e3f58efd8b6d61` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,685 | `0c9f402286b8fbdd0cfb521fb6504f1a0f7943c4cf130233dfa7eb74878998c6` |
| `docs/SUPPORT_MATRIX.md` | 2,336 | `8d4225b56d01212a390abeb6d8772f67f9a6bb756a2b0ca7f2ead88e9a16044e` |
