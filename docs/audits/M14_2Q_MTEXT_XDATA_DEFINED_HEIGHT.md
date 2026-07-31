# M14.2q MTEXT XDATA Defined Height

## Scope

M14.2q imports the R2007-era `ACAD_MTEXT_DEFINED_HEIGHT` XDATA envelope into
the unified MTEXT column scalar and mode-relation path established by M14.2p.
The old project was consulted only as a read-only behavioral oracle; no parser
source or fixture bytes were copied.

## Fail-closed framing

- Recognition occurs only inside an exact `ACAD` XDATA application scope.
- A defined-height candidate must contain the exact begin marker, a 1070
  selector with value 46, one 1040 double, and the exact end marker.
- The candidate must follow a complete column-info entry in the same MTEXT
  record, and that entry's value range must still end at the evidence tail.
- Any wrong marker, selector, wire type, missing group, or absent preceding
  column-info entry publishes no partial value.
- Cancellation and allocation failures remain explicit.

## Shared semantics

`DxfMTextXDataColumnRole::DefinedHeight` retains both the selector and value
group provenance. The existing generic XDATA adapter maps that role to
`SharedHeight`; scalar numeric validation and mode relationships therefore
remain storage-independent. R2007 static values can satisfy the positive
shared-height requirement, dynamic-automatic values remain source-visible,
and dynamic-manual values coexist with per-column height arrays.

## Coverage and size discipline

ASCII/Binary parity covers all nine supported dialects from AC1009 through
AC1032. Negative fixtures cover wrong begin/end markers, selector, and wire
type. Focused semantic fixtures demonstrate usable R2007 static and
dynamic-automatic modes. The defined-height reader, XDATA evidence reader,
shared projector, and integration test each remain below 500 lines.

## Explicit nonclaims

M14.2q does not:

- interpret `ACAD_MTEXT_COLUMNS_BEGIN` linked-column handles;
- choose or validate linked MTEXT ownership;
- interpret direct flat group-50 framing;
- derive MTEXT column layout geometry; or
- edit or write MTEXT columns.

## Verification

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 671 workspace tests with zero failures or ignored tests, changed
production forbidden-macro scanning, and `git diff --check`. Eleven focused
column evidence/scalar/relation tests also passed independently. No dependency
manifest or lockfile changed.

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 79 | `d1f3485cbc9b8ab410f78f158956a0e6a8c6f5c392a8dd782e5741846ba92b4e` |
| `crates/seacad-dxf-core/src/lib.rs` | 726 | `f8c137e2663a968daed30b078df5fd75adeac71a07d683fc3c18231d342c17ab` |
| `crates/seacad-dxf-core/src/mtext_column_semantic_project.rs` | 345 | `8f3225c94ec2ee933b8190df6be3f17cb6c7a21826e0e2b0aa9727d63e73fb60` |
| `crates/seacad-dxf-core/src/mtext_xdata_column_evidence.rs` | 438 | `2fe9308967a03c82270d367447c4f32126b0b96a09645d6f302298837213626d` |
| `crates/seacad-dxf-core/src/mtext_xdata_defined_height.rs` | 113 | `503f42bb4dd75aa857aaad051d0afd81faa238a5c8b7ac72ac417ef56810750c` |
| `crates/seacad-dxf-core/tests/mtext_xdata_column_evidence_tests.rs` | 419 | `c4cf7e1741c2096643274d8e9e5eeef2e390f6080fce3814e097fd35efb58b48` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 198 | `8a6a9b8c670818421485e3561c2d61f6bb724ac1a0fc300033893ec4eacbe014` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,537 | `ca51b3da6d3f8ccd4192311b67bfb07c52809fd8ebcfd032c2edf1042831f976` |
| `docs/SUPPORT_MATRIX.md` | 1,230 | `2c557014ae573d7490ff7d7d13991077b5bc79acd343e06ca34b246733e8c0dc` |
