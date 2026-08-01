# M14.3i Unified Entity Directory

## Scope

M14.3i introduces one format-neutral, immutable entity index over the existing
lossless raw document and completely closed raw-record sections. It is the
common discovery boundary for later schema evidence, family projections, and
CRUD work.

## Contract

- `DxfEntityRef` retains the source identity, complete raw record, exact
  group-zero marker, placement-aware classification, and compact subclass
  range.
- `DxfEntityClassification` distinguishes canonical topic, reviewed alias,
  unknown `BLOCKS`/`ENTITIES` marker, and a reviewed name in a wrong section.
- Entity placement is restricted to `BLOCKS` and `ENTITIES`; `BLOCK` and
  `ENDBLK` remain structural records rather than unknown entities.
- Classification compares exact case-sensitive bytes with a fixed 64-byte
  stack buffer. A longer marker is safely unknown. A registry test requires
  every reviewed name to fit that audited bound.
- Every group-code 100 occurrence outside group-code 102 application content
  remains in source order, including duplicate and out-of-order subclass
  markers. The public markers retain exact raw spans rather than decoded text.
- Interrupted and unclosed sections inherit the raw-record layer's fail-closed
  rule and contribute no partial entity records.
- Lookup by raw-record ordinal or contained group is allocation-free after
  construction. Cross-source subclass lookup fails with the existing typed
  source-identity error.
- Construction checks cancellation before scanning, during records and group
  sequences, after source reads, and before publication. Allocation failure is
  converted to the existing typed read error.

## Evidence and nonclaims

The 45 canonical topics and 14 aliases come only from the reviewed M14.3f--h
generated registry. Section placement follows the fixed Core 1.0 rule that
entity occurrences are accepted from `BLOCKS` or `ENTITIES`. Existing raw
record and application-group indexes provide the structural boundaries; no
legacy or third-party implementation was copied.

The directory is metadata. It does not interpret common properties, validate
subclass order or cardinality, project family semantics or geometry, enforce
dialect applicability, edit records, or write output. Unknown/custom groups
remain byte-exact in the authoritative raw document, and no entity support
state advances to `Complete`.

## Verification

Six integration tests exercise all nine AC1009--AC1032 dialects in both ASCII
and Binary, canonical/alias/unknown/wrong-section parity, exact case
sensitivity, BLOCK controls, duplicate/out-of-order subclass paths, exclusion
of application content, lookup boundaries, source mismatch, pre-scan and
mid-read cancellation, interrupted/unclosed sections, debug redaction, and
public metadata size/thread-safety bounds.

The focused integration target passed 6/6 tests. The complete workspace passed
746 tests. Schema and release-evidence freshness, dependency policy,
formatting, workspace Clippy with warnings denied, and `git diff --check` all
passed. The first workspace-test attempt reached the command time limit after
compilation and its output pipe closed; a clean rerun with an adequate bound
passed in full. Production added 366 lines in the new module plus five module
and export lines, within the checkpoint's 200--500-line review target.

This audit intentionally omits its own hash so the receipt is not
self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 77 | `b26c0b9f14d1d93b26c400f82b1a0fb422302e8315c450a4cb60b1cdb83adb1b` |
| `crates/seacad-dxf-core/src/entity_directory.rs` | 366 | `f76e0d270bc2027892ad00752e9b10d9c6bed3b129dca1725bd1213b38454c58` |
| `crates/seacad-dxf-core/src/lib.rs` | 847 | `06694157abfe43c367a33cb95f6ab50e67979c0aead225402331068b5e74191b` |
| `crates/seacad-dxf-core/tests/entity_directory_tests.rs` | 454 | `7ab966325a29577b6bc6d5b31a251726dad71da2e58988664d5e4b4ed74f051a` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 356 | `0b24af5c442a4a1d883598265fe9f64a03843d975bd155a2d7fbb8d8bcb232e5` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,760 | `7018542c17de8096fddff3331c120238343788df90e7b9f918ebed77fd61beef` |
| `docs/SUPPORT_MATRIX.md` | 1,393 | `36581bb287b56e71119eb48a86709e892eb544a39d9a7779e9fa80db9615e562` |
