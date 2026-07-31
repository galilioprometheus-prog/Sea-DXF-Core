# M14.2w MTEXT Effective X-Axis Direction

## Scope

M14.2w converts the effective orientation input selected by M14.2k into one
typed MTEXT WCS X-axis direction without changing authoritative source bytes.

## Normative basis

Autodesk's MTEXT DXF reference identifies groups 11/21/31 as the WCS X-axis
direction vector. It also states that a DXF input group-50 rotation in radians
is converted to the equivalent direction vector and that the later rotation
or vector input wins:
<https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-5E5DB93B-F8D3-4433-ADF7-E92E250D2BAB.htm>.

The read-only legacy SeaCad validator independently exercised cosine/sine
conversion, explicit vectors, source-order precedence, and zero-vector
diagnostics. It was used only as behavioral evidence; no source, type, test,
fixture, or algorithm text was copied.

## Contract

- A selected rotation produces `[cos(angle), sin(angle), 0]`.
- A selected group-11/21/31 input retains all three exact components.
- Explicit vectors additionally expose a normalized unit direction while
  retaining the original magnitude-bearing vector.
- Missing components, invalid numerics, zero length, and length overflow have
  distinct typed results with the best available raw provenance.
- Absent orientation remains absent.
- M14.2v direct group-50 ambiguity propagates as an orientation issue.

## Coverage and size

All nine supported dialects have ASCII/Binary parity for rotation-only,
vector-only, both source orders, and absent input. Focused cases cover partial,
invalid, zero-length, non-finite-length, and column-ambiguous inputs plus
cancellation, source identity, family scope, raw-record lookup, and public
trait bounds.

The new production module is 307 lines and its integration test is 362 lines;
the existing `lib.rs` receives only module and re-export glue.

## Explicit nonclaims

M14.2w does not combine the direction with insertion or extrusion, derive an
orthonormal text plane, resolve text styles, measure content, derive glyph or
column geometry, edit, or write.

## Verification

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 684 workspace tests with zero failures or ignored tests, changed
production forbidden-macro scanning, and `git diff --check`. The focused
effective-direction suite passed 3/3 tests. No dependency manifest or lockfile
changed.

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 79 | `6756547948cf5078ba50eeb4af49a15dc26f527e5bc1a08a7cc46f1588a1fe52` |
| `crates/seacad-dxf-core/src/lib.rs` | 746 | `d7f13b36461e591b2a0d230f24cf28eb5ea328f0a9e98dc6f067389413ff02ce` |
| `crates/seacad-dxf-core/src/mtext_x_axis_direction.rs` | 307 | `37235ecf5c66fc83de785a2b5d902d504f906da3310f61c90d5fd49349f73d19` |
| `crates/seacad-dxf-core/tests/mtext_x_axis_direction_tests.rs` | 362 | `ba81857cf80ad5e85d9f082ac67c4b4ffde9194b42cac522dce1c9912947c8fb` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 232 | `f2066909566b7dd54b3f94e5d829923c2c50966e310c42c012555c227c7b91b5` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,580 | `55963a19c005dc9d982a4f64ac80366507c3cdc770281b857b4483c5862a0252` |
| `docs/SUPPORT_MATRIX.md` | 1,275 | `1f3bb30853e9277815fa5c09f198073a076f44330e2b49877c3274fff63d8b56` |
