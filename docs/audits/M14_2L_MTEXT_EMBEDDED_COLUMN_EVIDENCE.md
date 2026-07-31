# M14.2l MTEXT Embedded-Column Evidence

## Scope

M14.2l isolates modern MTEXT column data from the enclosing MTEXT entity at the
exact group-101 `Embedded Object` boundary. It adds an immutable source-evidence
directory for observed column fields and prevents those fields from
contaminating the existing main MTEXT role cards.

This is an evidence checkpoint, not full column semantics.

## Normative and behavioral evidence

- Autodesk documents group 101 with the exact value `Embedded Object` as the
  separator between an encapsulating object and its embedded object:
  <https://help.autodesk.com/cloudhelp/2022/ENU/OARX-DevGuide/files/GUID-C953866F-A335-4FFD-AE8C-256A76065552.htm>
- Autodesk's MTEXT DXF reference documents the legacy/flat column-related
  groups 75, 76, 78, 79, 48, 49, and repeated 50:
  <https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-5E5DB93B-F8D3-4433-ADF7-E92E250D2BAB.htm>
- An AutoCAD 2027 `-MTEXT`/`SAVEAS` behavioral oracle placed the main MTEXT
  fields before the exact marker and emitted embedded groups 70, 41, 71, 72,
  44, 45, 73, and 74 after it. The generated oracle SHA-256 was
  `E38C11B0CA9EE40F4E4E206F1BD752BC9FCA1F109B1D72316FB12673958741CF`.
  The temporary file is not part of the repository.
- Read-only inspection of the owner's older
  `D:\SeaCad\cad_2026-07-23_source` project found R2007 examples using
  `ACAD_MTEXT_COLUMN_INFO_BEGIN` XDATA and R2018 examples using the embedded
  object boundary, including repeated group 46 individual heights. No source
  code or fixture was copied.

The Autodesk references remain normative. The AutoCAD output and older project
are behavioral evidence used to bound this checkpoint.

## Implementation

- `text_symbol_evidence` stops main MTEXT field classification at the first
  exact group-101 `Embedded Object` marker.
- `DxfMTextEmbeddedColumnDirectory` indexes each exact embedded marker and
  retains recognized values after it in source order.
- Recognized roles are version (70), shared height (41), individual height
  (46), type (71), count (72), width (44), gutter (45), automatic height (73),
  and reversed flow (74).
- Every retained member points back to the original raw group and preserves the
  document source identity.
- Numeric decoding uses the existing bounded raw integer/double decoders,
  propagates cancellation, and preserves invalid ASCII-number evidence.

## Verification

- Synthetic ASCII/Binary parity covers all nine supported dialects, AC1009
  through AC1032, including one-byte AC1009 Binary group codes.
- A boundary regression proves embedded groups 71, 72, and 44 no longer alter
  the enclosing MTEXT attachment, direction, or line-spacing cards.
- Exact marker matching, cancellation, missing lookup behavior, source
  identity, entry slicing, and `Send`/`Sync` traits are covered.
- Full workspace test, Clippy, formatting, dependency-policy, schema,
  release-evidence, forbidden-macro, and diff-integrity gates are recorded in
  the checkpoint receipt below.

## Explicit nonclaims

M14.2l does not:

- validate column type/count/width/gutter/height domains or relationships;
- decide whether group 41 is semantically a common height in every producer;
- unify legacy flat groups, modern embedded fields, or R2007 XDATA columns;
- resolve group-50 rotation/column-height ambiguity;
- derive column layout or geometry; or
- edit or write MTEXT column objects.

## Checkpoint receipt

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 660 workspace tests with zero failures or ignored tests, changed
production forbidden-macro scanning, and `git diff --check`. The three focused
embedded-column tests also passed independently. No dependency manifest or
lockfile changed.

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 79 | `ba0ebe9f8a654e56b3fe515e681f337025ff13c108d586792f9332df387d90b7` |
| `crates/seacad-dxf-core/src/lib.rs` | 708 | `9c2b0cb43b5bcd7fbaf55dc692de6dbbdf99e94e021b7d7316abea2c3e934c20` |
| `crates/seacad-dxf-core/src/text_symbol_evidence.rs` | 402 | `c188f87f314019d21130ebb44419e14561996861c1a14ee4e02512da7fb7cc42` |
| `crates/seacad-dxf-core/src/mtext_embedded_column_evidence.rs` | 304 | `3fe920886090214f798cb686bf251b57b9d3195701f7620f7fbeaa369a071676` |
| `crates/seacad-dxf-core/tests/mtext_embedded_column_evidence_tests.rs` | 282 | `9de2702ed27629e6dbe658350df7cd14e0623152ab34d7750774dc2f1f9aba85` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 166 | `8bb89f43df7cf88676903dceb447b3c70691acfc9a838c72570669cba601b7d1` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,502 | `afadd6c532e1e3c20bd9a382271ee6496bf8e4328f0d923f968a04a0aab3b1b1` |
| `docs/SUPPORT_MATRIX.md` | 1,195 | `92404f75606b6e1f5342464301562225a13d8310f8a27d944b4a23cbc9e1bcda` |
