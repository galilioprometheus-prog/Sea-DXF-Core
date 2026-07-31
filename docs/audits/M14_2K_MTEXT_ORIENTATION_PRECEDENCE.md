# M14.2k MTEXT Orientation Precedence

## Scope

M14.2k projects MTEXT group-50 rotation evidence and group-11/21/31 X-axis
evidence into a typed effective-input selection. When group 50 is
unambiguously rotational, the input form whose final source occurrence is
later wins. The projection retains the complete M14.2d scalar directory and
does not mutate or duplicate source bytes.

## Normative basis

Autodesk's DXF MTEXT entity reference states that group 50 is rotation in
radians, that entering rotation converts it to the X-axis direction vector,
and that when rotation and X-axis direction are both supplied, the last input
takes precedence. The same reference also uses group 50 for column heights and
documents the surrounding column fields:
<https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-5E5DB93B-F8D3-4433-ADF7-E92E250D2BAB.htm>.

## Clean-room oracle boundary

`D:\SeaCad\cad_2026-07-23_source` was inspected read-only as behavioral
evidence. Its legacy validator and historical corpus notes confirmed that
group-50 column coexistence is a real ambiguity and that source-order
precedence needs explicit coverage. No implementation, parser, type, fixture
bytes, test code, dependency, or source text was copied, translated, linked,
vendored, or imported. Autodesk documentation and current SeaCad provenance
remain normative.

## Behavior locked

- All AC1009-AC1032 dialects have ASCII/Binary parity.
- A lone group 50 is exposed as rotation; a lone X-axis input is selected as
  the effective orientation input.
- When both input forms occur, their final raw group occurrences determine the
  winner.
- Invalid or duplicate rotation evidence remains a typed scalar issue when
  rotation is effective. A later X-axis input can still be selected without
  erasing the separately accessible rotation diagnostic.
- Any column field coexisting with one or more group-50 values produces a
  typed `AmbiguousGroup50Role` result with the exact occurrence count and raw
  provenance.
- Cancellation, family scope, raw-record bounds, source identity, and public
  `Copy`/`Send`/`Sync` contracts are covered.

## Explicit nonclaims

M14.2k does not assemble or normalize the three X-axis components, reject a
zero vector, convert rotation through sine/cosine, transform coordinates,
validate column-height count/order, disambiguate group-50 column heights,
resolve styles, derive glyph geometry, edit, or write.

## Verification

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 656 workspace tests with zero failures/ignored tests, prohibited
production-macro scanning, and `git diff --check`. No dependency manifest or
lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 78 | `bc83e2e456bf7a9eb5c927da056be248b4761acb0cf521b358a5dbaea545c09c` |
| `crates/seacad-dxf-core/src/lib.rs` | 703 | `8f9902f82a5132be3fe0e8046afc5ccedaf8dd54a958ebf3567b6336aa406017` |
| `crates/seacad-dxf-core/src/mtext_orientation.rs` | 318 | `2d8f7a4fb3592dcc3ce86c3f18fd81792c68d4e4f2abf48b8ccc2536e0145667` |
| `crates/seacad-dxf-core/tests/mtext_orientation_tests.rs` | 388 | `47d3047602afc6f3d2932df61b64f93fafacb2cf2947b521ee6f8a77b496be34` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 159 | `e741b5ad95b8cd072e48d5265f2f2c75b16e6bcac832a71c51c25d3296f64af4` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,486 | `950ceccdf487724689d1829054df2edbeee2f21367071e1a4157208cd10b2201` |
| `docs/SUPPORT_MATRIX.md` | 1,186 | `b3ac3c51475cbeee62509f428a50188b15c161ed5da934d15baaa7a62a8ce9d0` |
