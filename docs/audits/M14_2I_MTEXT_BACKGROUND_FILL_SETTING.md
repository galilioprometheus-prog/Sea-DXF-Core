# M14.2i MTEXT Background-Fill Setting

## Scope

M14.2i projects the optional MTEXT group-90 signed scalar from M14.2d into the
three settings published by Autodesk:

- `0`: background fill off;
- `1`: use the background fill color;
- `2`: use the drawing-window color.

Unsupported values fail typed with exact field/raw provenance. Absence remains
`Absent`; no default is invented.

## Normative basis

Autodesk's DXF MTEXT entity reference enumerates the three group-90 settings:
<https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-5E5DB93B-F8D3-4433-ADF7-E92E250D2BAB.htm>.
The same three values remain published in Autodesk's 2026 MTEXT reference.

## Clean-room oracle boundary

`D:\SeaCad\cad_2026-07-23_source` was inspected read-only as behavioral
evidence. Its legacy validator treated group 90 as a mask containing bits 1,
2, and 16. That undocumented interpretation was deliberately not adopted:
current SeaCad classifies only Autodesk's published values and retains all
other signed values as source-anchored unsupported evidence. No implementation,
parser, type, fixture bytes, test code, dependency, or source text was copied,
translated, linked, vendored, or imported.

## Behavior locked

- All AC1009-AC1032 dialects have ASCII/Binary parity.
- Published settings `0`, `1`, and `2` map to distinct public enum values.
- Negative, combined, and undocumented values, including `3` and `16`, fail
  typed without discarding their signed value or raw provenance.
- Invalid ASCII and duplicate values retain the underlying scalar issue.
- Missing group 90 remains `Absent`.
- Cancellation, family scope, raw-record bounds, source identity, and public
  `Copy`/`Send`/`Sync` contracts remain covered.

## Explicit nonclaims

M14.2i does not claim undocumented group-90 bit flags, validate background
color/index/transparency applicability, resolve group 420/430 ownership,
validate fill-box scale, interpret columns, resolve rotation precedence,
transform coordinates, decode text, resolve styles, derive glyph geometry,
edit, or write.

## Verification

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 648 workspace tests with zero failures/ignored tests, prohibited
production-macro scanning, and `git diff --check`. No dependency manifest or
lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 78 | `4172099aa994db54da657319f797600390fb2ea378ec92e96b423f96d69e14f7` |
| `crates/seacad-dxf-core/src/lib.rs` | 697 | `68c66b8e582dc3235a92e00f52cab13c8f31f3da8349049bc153e2c476ee34d6` |
| `crates/seacad-dxf-core/src/mtext_numeric_domain.rs` | 257 | `449892f08995bf55ae16409a7a53d536b5f1260b0a0300096f8f39763536222b` |
| `crates/seacad-dxf-core/tests/mtext_numeric_domain_tests.rs` | 451 | `f90dc587717c0f48fc2da6c95498f2867166137362166a7117c875b58f906e17` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 146 | `b1ff92f1d43fb869f47cae60a146ab5cb9e803aefff8e24e3c7109f32b76f7f6` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,470 | `c6aaf17cf92b5340e044aa1edcab396e94fdbb38947539a7414808330d6f0363` |
| `docs/SUPPORT_MATRIX.md` | 1,173 | `0db09235ebc1cfdbe58474ecd10f344d8f744b8fdd16d66393af478edacdba81` |
