# M6.3b Binary `$DWGCODEPAGE` and typed HEADER view

M6.3b extends the reviewed M4.3a text-storage policy to Binary DXF. Autodesk's
HEADER variable semantics are shared by ASCII and Binary; M5.1/M5.2 already
established exact Binary group-code and NUL-string framing.

`DxfTextEncodingTracker` now accepts one raw occurrence, group code, payload,
and byte span. Its ASCII adapter and Binary document call the same state
machine. The Binary parse loop reuses the payload already obtained for dialect
and structure tracking, so this adds no source read or second scan.

`DxfBinaryRawDocument` stores and exposes the same `DxfTextEncodingReport` as
ASCII. `DxfHeaderView` projects `$DWGCODEPAGE` as:

- `Explicit(Reviewed)` for a token with a reviewed decoder identity;
- `Explicit(Unrecognized)` for a structurally valid non-empty token that has no
  reviewed decoder;
- `Absent` when no exact declaration exists;
- `Invalid` for empty, wrong-group, missing, or duplicate declarations.

Decoder selection remains separate. A pre-2007 unrecognized token still yields
`UnsupportedLegacy`; AC1021 and later still select UTF-8 by version. No host
locale, replacement text, or fallback codepage is introduced.

Synthetic parity tests cover all nine AC1009-AC1032 versions in ASCII and
Binary with `ANSI_1252`. Binary boundary vectors cover absent, unrecognized,
empty, wrong group, duplicate, and compatible missing-value input, including
exact raw-span readback and stable diagnostics.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `crates/seacad-dxf-core/src/encoding.rs` | `fc2cda9510d4f1c3a2a46566c85249cbb62eb51580766dee526a8870c5b9e0ef` |
| `crates/seacad-dxf-core/src/binary_document.rs` | `367251c49ae86ea62b3ed953fce97e4c99dbaf39fc63eb2b245b1457e934432c` |
| `crates/seacad-dxf-core/src/header_view.rs` | `3ea86c03c73fcb22c5e77e44370e36340a9d653e99a4cc5587e9e4ec842da7f1` |
| `crates/seacad-dxf-core/src/lib.rs` | `5ea23a3593f42743adf6ee1f99ddff1ca3ac32421f3fcb367be91eda855fe12d` |

This checkpoint adds no dependency, allocation proportional to file size,
writer behavior, default value, or `$HANDSEED` semantics. Its support claim is
limited to the reviewed `$DWGCODEPAGE` structure and storage-policy resolution.
