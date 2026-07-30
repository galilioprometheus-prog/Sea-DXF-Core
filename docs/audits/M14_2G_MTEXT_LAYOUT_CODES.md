# M14.2g MTEXT Layout Codes

## Scope

M14.2g projects M14.2d MTEXT integer scalars into all code domains enumerated
completely by Autodesk's MTEXT entity reference:

- attachment points `1..=9`;
- drawing directions `1`, `3`, and `5`;
- optional line-spacing styles `1` and `2`.

The projection preserves explicit/default/absent/invalid states and exact raw
provenance. Unsupported values are not normalized or guessed.

## Normative basis

Autodesk's DXF MTEXT entity reference defines group 71 attachment points, group
72 drawing directions, and group 73 line-spacing styles:
<https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-5E5DB93B-F8D3-4433-ADF7-E92E250D2BAB.htm>.

## Clean-room oracle boundary

`D:\SeaCad\cad_2026-07-23_source` was inspected read-only as behavioral
evidence. Its legacy MTEXT diagnostics highlighted invalid attachment,
direction, and spacing-style codes as independent compatibility cases. No
implementation, parser, type, fixture bytes, test code, dependency, or source
text was copied, translated, linked, vendored, or imported. Autodesk
documentation and current SeaCad scalar provenance remain normative.

## Behavior locked

- All AC1009-AC1032 dialects have ASCII/Binary classification parity.
- Attachment codes map to the nine top/middle/bottom and left/center/right
  positions.
- Drawing-direction codes map to left-to-right, top-to-bottom, and by-style.
- Line-spacing codes map to at-least and exact.
- Missing required attachment/direction values retain typed M14.2d failures.
- Missing optional line-spacing style remains `Absent`.
- Unsupported, invalid ASCII, and duplicate values remain independently typed
  and source-anchored.
- Cancellation, family scope, raw-record bounds, source identity, and public
  `Copy`/`Send`/`Sync` contracts are covered.

## Explicit nonclaims

M14.2g does not validate height/width/spacing numeric ranges, classify
background or column fields, resolve group-50 rotation precedence, transform
coordinates, decode text, resolve styles, derive glyph geometry, edit, or
write.

## Verification

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 641 workspace tests with zero failures/ignored tests, prohibited
production-macro scanning, and `git diff --check`. No dependency manifest or
lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 63 | `a7861ddd7ff3211e71596634c9dad7830efc4610364b49cde288e2f638051ad2` |
| `crates/seacad-dxf-core/src/lib.rs` | 686 | `9480a753d0973597236bca60403c91395516b9799e7ee2f77d5fc4bfc49aadb6` |
| `crates/seacad-dxf-core/src/mtext_layout.rs` | 250 | `2bd4fc33a5350dc0348905faa55b4f518191cb2fd6f41d3ac3826ce4e5c625d9` |
| `crates/seacad-dxf-core/tests/mtext_layout_tests.rs` | 327 | `c3a58a625b1766b5d7e13b9bfc1d20499f743ca1cbdcd001e56fc5d834360e5b` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 119 | `fdaf66d501bc1089d49fc95ee42b6e0f6563bcb25e01703a2ff491fc3ca36fdc` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,454 | `7449d5e6124edada991a5b8c073ca5c89ca237c1c96076c7ec9a3805cd44c2c1` |
| `docs/SUPPORT_MATRIX.md` | 1,148 | `c989d8a5f9918ff828177e623502b92ae8d183b76511174e0668d2142d37eadb` |
