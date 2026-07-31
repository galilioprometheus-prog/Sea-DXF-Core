# M14.2x TEXT OCS Placement Anchor

## Scope

M14.2x converts the layout applicability reported by M14.2f into one typed
TEXT placement anchor in object coordinates without changing authoritative
source bytes.

## Normative basis

Autodesk's TEXT DXF reference defines groups 10/20/30 as the first alignment
point in OCS and groups 11/21/31 as the second alignment point in OCS. It also
states that the second point is meaningful only when horizontal group 72 or
vertical group 73 is nonzero:
<https://help.autodesk.com/cloudhelp/2016/ENU/AutoCAD-DXF/files/GUID-62E5383D-8A14-47B4-BFC4-35824CAE8363.htm>.

No parser, type, test, fixture, or algorithm text was copied from the legacy
SeaCad workspace.

## Contract

- Left/baseline layout selects the required group-10/20/30 point.
- Any other supported horizontal or vertical justification selects the
  optional group-11/21/31 point.
- The result names whether the first or second alignment point was selected
  and retains the three exact `DxfDouble` components.
- Missing required components, absent optional components, invalid numerics,
  duplicates, and unavailable justification have distinct typed results.
- Successful and invalid projections retain the best available raw
  provenance, while the complete M14.2f layout/scalar evidence remains
  reachable from the projected semantics.

## Coverage and size

All nine supported dialects have ASCII/Binary parity for left/baseline,
horizontal-only, vertical-only, and combined nonzero justification. Focused
cases cover missing first and second components, a partial second point,
invalid ASCII numeric evidence, duplicate values, unsupported horizontal and
vertical codes, cancellation, source identity, family scope, raw-record
lookup, and public trait bounds.

The new production module is 320 lines and its integration test is 354 lines;
the existing `lib.rs` receives only module and re-export glue. Both behavior
files stay below the repository's 500-line review threshold.

## Explicit nonclaims

M14.2x does not transform the selected point from OCS to WCS, construct a text
plane, resolve text styles, measure content, derive glyph geometry, edit, or
write.

## Verification

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 687 workspace tests with zero failures or ignored tests, changed
production forbidden-macro scanning, and `git diff --check`. The focused
placement-anchor suite passed 3/3 tests. No dependency manifest or lockfile
changed.

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 79 | `e24a3c7867f277489aaee053e2d39347cb3d9e467e64ae9cc77a8639e6747f77` |
| `crates/seacad-dxf-core/src/lib.rs` | 753 | `b0ffbd86ecb4c0f41e7fb333c1a74f7e0d9fd58111da606c4e79cfe4b9096abe` |
| `crates/seacad-dxf-core/src/text_placement_anchor.rs` | 320 | `1a1736b807845653506ca6e1cbdc6ffd427c9af56756f340eb64a71bb107a199` |
| `crates/seacad-dxf-core/tests/text_placement_anchor_tests.rs` | 354 | `cbb6e6161bbc490c0e91ca696de0f51d765892d93c5d7ea4c0a0bd9c13261a3c` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 238 | `08f534275acef3ca016e9162eb69ab332e8ae26bc8e24bc11e9c76ebd73825ef` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,587 | `1b767021fcfdb3353628e2fc23c148f947c18ecba6c0c640f8db65b464d1e0cc` |
| `docs/SUPPORT_MATRIX.md` | 1,283 | `1e90c4d742939b2725487e36c8914c8bff6c44fb514eeccdb59576de02d7310c` |
