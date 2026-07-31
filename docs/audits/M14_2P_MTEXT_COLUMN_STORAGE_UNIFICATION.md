# M14.2p MTEXT Column Storage Unification

## Scope

M14.2p projects modern Embedded and R2007-era `ACAD` XDATA column evidence
through the same scalar and mode-relation implementation. It removes the need
for a second semantic parser while retaining the physical storage identity.

## Design

- `DxfMTextColumnSourceEntry` distinguishes Embedded and `AcadXData` envelopes
  and exposes their common raw record and marker.
- `DxfMTextColumnSemanticDirectory` owns both evidence directories, orders
  source entries by marker occurrence, and publishes one semantics slice.
- One private `ColumnEvidenceValue` adapter maps either evidence value to the
  shared role/data/raw-group contract.
- The existing scalar projector and relation directory consume that common
  contract; there is no duplicated XDATA semantic switch.
- XDATA's declared field-50 height count remains evidence-only. Its captured
  height slice is checked against the column count by the common manual-mode
  relation.

## Behavior locked

- Modern Embedded behavior and all prior scalar/mode tests remain unchanged.
- Exact `ACAD` XDATA type, count, width, gutter, auto-height, flow-reversal,
  and individual heights produce the same values and issues as Embedded data.
- R2007 dynamic-manual XDATA with count 3 and heights 20, 30, 0 projects to
  `DynamicManual`; the terminal zero remains a source-anchored usable scalar.
- ASCII/Binary parity covers AC1009 through AC1032 for the unified XDATA path.
- Source identity, physical source-entry discrimination, marker lookup,
  cancellation, and bounded height slices remain intact.

## Code-size discipline

The unified directory/public-type module and generic projection-helper module
remain independently below 500 lines. The XDATA evidence test containing the
cross-storage regression also remains below 500 lines.

## Explicit nonclaims

M14.2p does not:

- merge two simultaneous column envelopes into one winner;
- consume `ACAD_MTEXT_DEFINED_HEIGHT_BEGIN` or
  `ACAD_MTEXT_COLUMNS_BEGIN` blocks;
- make R2007 static/shared-height modes usable without defined-height evidence;
- validate linked column handles;
- interpret direct flat group-50 framing; or
- derive geometry, edit, or write columns.

## Verification

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 670 workspace tests with zero failures or ignored tests, changed
production forbidden-macro scanning, and `git diff --check`. Ten focused
column evidence/scalar/relation tests also passed independently. No dependency
manifest or lockfile changed.

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 79 | `762efd482af96db72a251b415fb299960a575d037aaea33e723afc66f9a5b424` |
| `crates/seacad-dxf-core/src/lib.rs` | 725 | `2ad00e0847b8f2ff5b4fe09b31dd2c60949eba87a98c44dba7ac9198db04435d` |
| `crates/seacad-dxf-core/src/mtext_column_semantic.rs` | 327 | `63b3a673e4ce9e652f086290e03944d649a6aa12baa2fe6deb2e1e027e7e3772` |
| `crates/seacad-dxf-core/src/mtext_column_semantic_project.rs` | 344 | `b9f98b6e5d4e218e92b9ee17d44d79111591191a155a8ba8fb539688c646c405` |
| `crates/seacad-dxf-core/tests/mtext_xdata_column_evidence_tests.rs` | 325 | `5bc5eb021270359def3584db812a116b97d318256d3b0617767bf0cc583cca31` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 192 | `a8a9271bae9157ca316f1e87fa4c076622e36cdb8588db567f04fe00d2ec74ff` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,530 | `0e657d65faa677cea1bbdfeeae99b680829993192ebf2a9fb9b8e9091fabbaac` |
| `docs/SUPPORT_MATRIX.md` | 1,224 | `274075927b0351ea14fdfe7c346f89a7afd4da11e431f6b5430ff7f18d8704ad` |
