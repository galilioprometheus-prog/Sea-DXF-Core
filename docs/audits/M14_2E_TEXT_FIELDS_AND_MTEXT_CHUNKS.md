# M14.2e Text Fields and MTEXT Chunks

## Scope

M14.2e adds immutable, source-anchored semantic selection for:

- TEXT group 1 content and group 7 style name;
- MTEXT group 7 style name and ordered group 3/group 1 content chunks;
- SHAPE group 2 shape name;
- TOLERANCE group 3 dimension-style name and group 1 content.

The selector distinguishes explicit, documented-default, missing-required, and
multiple-value states without copying source strings. MTEXT chunk entries keep
their original group occurrences and order. Structural accounting reports the
terminal group 1 count and the number of group 3 values observed after the
first terminal.

## Normative basis

- Autodesk DXF TEXT entity reference:
  <https://help.autodesk.com/cloudhelp/2016/ENU/AutoCAD-DXF/files/GUID-62E5383D-8A14-47B4-BFC4-35824CAE8363.htm>
- Autodesk DXF MTEXT entity reference:
  <https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-5E5DB93B-F8D3-4433-ADF7-E92E250D2BAB.htm>
- Autodesk DXF SHAPE entity reference:
  <https://help.autodesk.com/cloudhelp/2025/ENU/AutoCAD-DXF/files/GUID-0988D755-9AAB-4D6C-8E26-EC636F507F2C.htm>
- Autodesk DXF TOLERANCE entity reference:
  <https://help.autodesk.com/cloudhelp/2018/ENU/AutoCAD-DXF/files/GUID-ADFCED35-B312-4996-B4C1-61C53757B3FD.htm>

The Autodesk references define group 1/2/3/7 roles, the TEXT/MTEXT
`STANDARD` style default, and MTEXT's group 3 continuation chunks followed by
the final group 1 chunk.

## Clean-room oracle boundary

`D:\SeaCad\cad_2026-07-23_source` was inspected read-only as a behavioral
oracle. Its legacy MTEXT tests confirmed useful independent failure cases:
multiple continuation chunks, a continuation after the terminal chunk,
ASCII/Binary parity, and source preservation under malformed input. No
implementation, parser, type, fixture bytes, or test code was copied,
translated, linked, vendored, or imported. Production behavior and public API
were derived from the Autodesk references and the current SeaCad evidence/card
contracts.

## Behavior locked

- Every supported AC1009-AC1032 dialect has ASCII/Binary parity.
- Required text/name fields remain typed-invalid when absent.
- Duplicate scalar text/name/style fields retain exact first-occurrence
  diagnostic provenance.
- Missing TEXT/MTEXT style group 7 defaults to `STANDARD`; SHAPE and TOLERANCE
  receive no invented style default.
- MTEXT chunks remain separate source-anchored entries.
- A valid chunk sequence has exactly one terminal group 1 and no group 3 after
  the first terminal.
- Missing terminals, multiple terminals, and late group 3 values are retained
  and counted rather than repaired or discarded.
- Cross-document chunk tokens are rejected by source identity.
- Cancelled builds fail before publishing a partial directory.

## Explicit nonclaims

M14.2e does not concatenate or decode text, validate MTEXT chunk byte lengths,
interpret inline formatting, resolve STYLE or DIMSTYLE table references,
validate enum/range values, choose alignment applicability, transform
coordinates, derive glyph geometry, edit, or write entities.

## Verification

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 633 workspace tests with zero failures/ignored tests, prohibited
production-macro scan, and `git diff --check`. No dependency manifest or
lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 63 | `7e0e53268d337f91795161dbdd6cd3edeef14a781c80b6508a13fd6a4b908763` |
| `crates/seacad-dxf-core/src/lib.rs` | 674 | `2a064c5bd12e53d3446edb088860e594dbb1222b63272d3255d43d656898c744` |
| `crates/seacad-dxf-core/src/text_symbol_text.rs` | 417 | `a67d718516d6b9e7ed004ea84032552ca96a43ef97102654e8c3f19289e4ec4d` |
| `crates/seacad-dxf-core/src/text_symbol_text_chunk.rs` | 64 | `dea3f61f88dcd13917e2b668619b90b771bb4def539c69194f55a2fe0677d9f0` |
| `crates/seacad-dxf-core/src/text_symbol_text_select.rs` | 131 | `e247f76882fde29d15371154cd5ed6bcafbf5726a3e6b7181ec3fb508c58d693` |
| `crates/seacad-dxf-core/tests/text_symbol_text_tests.rs` | 356 | `99fe4d349374749dbb9d7471c492e8c03fdf4f9e6cf175be83ad17a856cb194b` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 107 | `657644b85220215e3d2dd12147f05831eac1133c5725c1651319e14d5adf0e6e` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,436 | `70bb0232edd62d119525203c851bb9c119a8ac88ccc6a90952ad6ff58853e1c7` |
| `docs/SUPPORT_MATRIX.md` | 1,136 | `a0b60595ca0626b4bcaf75d12f919eacd4de9d795930b3d3e404bbdbb0d40d5f` |
