# M14.2j MTEXT Actual-Width Relation

## Scope

M14.2j projects the M14.2d MTEXT group-41 reference width and optional,
read-only group-42 actual width into a typed relationship. A usable actual
width is valid when it is equal to or less than the reference width. An excess
value fails typed while retaining both exact IEEE-754 inputs and group-42 raw
provenance.

## Normative basis

Autodesk's DXF MTEXT entity reference states that group 42 is the horizontal
width of the characters, is read-only, and will always be equal to or less than
group 41:
<https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-5E5DB93B-F8D3-4433-ADF7-E92E250D2BAB.htm>.

## Clean-room oracle boundary

`D:\SeaCad\cad_2026-07-23_source` was inspected read-only as behavioral
evidence. Its legacy validator checked the same width relationship and its
historical corpus notes reported MTEXT geometry mismatches, making exact
boundary and malformed-input coverage high-value cases. No implementation,
parser, type, fixture bytes, test code, dependency, or source text was copied,
translated, linked, vendored, or imported. Autodesk documentation and current
SeaCad scalar provenance remain normative.

## Behavior locked

- All AC1009-AC1032 dialects have ASCII/Binary parity.
- Actual width below or exactly equal to reference width is usable.
- The adjacent representable value above a reference width fails typed.
- Excess failures preserve both exact IEEE-754 inputs and group-42 provenance.
- Missing optional group 42 remains `Absent`.
- Invalid or duplicate group 41/group 42 values preserve the originating
  scalar issue and provenance.
- Cancellation, family scope, raw-record bounds, source identity, and public
  `Copy`/`Send`/`Sync` contracts are covered.

## Explicit nonclaims

M14.2j does not assign semantic validity to negative widths because the cited
entity reference states only the cross-field relationship. It does not compute
actual extents, validate height or other numeric ranges, interpret columns,
resolve group-50 rotation precedence, transform coordinates, decode text,
resolve styles, derive glyph geometry, edit, or write.

## Verification

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 652 workspace tests with zero failures/ignored tests, prohibited
production-macro scanning, and `git diff --check`. No dependency manifest or
lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 78 | `997d17170d7f588ef51c61d68bf41488031a7a1a628f1365def46f028ae34dab` |
| `crates/seacad-dxf-core/src/lib.rs` | 698 | `6a9dc570962cb0472f736abf0a31f89ef0fae3695c1dd8784cb7297f1e111359` |
| `crates/seacad-dxf-core/src/mtext_numeric_domain.rs` | 359 | `f319b507e6e4078a74c457f259cbd1b97d5ce55f210df847b8d6bb4b8e211c59` |
| `crates/seacad-dxf-core/tests/mtext_width_relation_tests.rs` | 333 | `d01b777fc66d4e0d28460ad22765efa1d685899a93c354c9c8962891b8231aee` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 151 | `18840b05ac6f07149652cdfa7ed0edb187ae2d99f1957e610e1e6ccec27d8683` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,477 | `53e3620fea27f20975487b6f02b65fda0ef22c2f658b1b850b53f40062e1b3f1` |
| `docs/SUPPORT_MATRIX.md` | 1,179 | `395ee480848271421b6cc22452de427ffb7e1fa28e80b29d2b72d94dd98bcbb5` |
