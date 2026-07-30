# M14.1e Planar-Face Edge Visibility

Retrieved: 2026-07-31

## Normative boundary

Autodesk's [3DFACE
reference](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-747865D5-51F0-45F2-BEFE-9572DBC5B151.htm)
defines optional group 70 with a default of zero. Bits 1, 2, 4, and 8 make the
edges beginning at the first, second, third, and fourth source-order corners
invisible, respectively.

M14.1e interprets only those four documented bits. The entire signed-16-bit
source value remains available, and the exact unsigned bit pattern is exposed
without treating undocumented bits as invalid.

## Legacy evidence boundary

`D:\SeaCad\cad_2026-07-23_source` remains a read-only behavioral oracle. Its
normalized-view evidence identifies invisible-edge flags and preservation of
unknown bits as compatibility risks. No legacy parser or flag implementation
was copied, translated, vendored, or linked. Autodesk documentation remains
normative.

## Implementation contract

`DxfPlanarFaceSemantics` now exposes:

- the existing signed group-70 semantic value with full field/raw provenance;
- the exact 16-bit reinterpretation of an available signed value;
- one source-corner-indexed invisibility query;
- all four documented invisibility states in source-corner order; and
- every uninterpreted bit outside the documented low-nibble mask.

Defaulted zero produces four visible edges. Explicit negative values retain
their exact two's-complement bits. Invalid, duplicate, or inapplicable flags
make derived edge queries unavailable. An out-of-range corner index is also
unavailable.

## Test evidence

`planar_face_edge_visibility_tests.rs` covers ASCII/Binary parity across all
nine AC1009--AC1032 dialects; defaulted zero; individual documented bit
interpretation; combined bits; exact signed/unsigned preservation; negative
values carrying unknown bits; SOLID inapplicability; out-of-range lookup; and
invalid and duplicate flags.

## Non-claims

M14.1e does not reject, normalize, or assign meaning to unknown bits. It does
not add edge flags to SOLID or TRACE, assemble planar faces into mesh topology,
apply thickness surfaces, expand blocks, edit, write, render, or tessellate.
It does not claim typed semantic completion for all currently recognized DXF
entity families.

## Required checkpoint gates

All required gates passed on 2026-07-31: dependency policy, generated-schema
drift, release-evidence drift, formatting, workspace Clippy with warnings
denied, 617 workspace tests with zero failures/ignored tests, and
`git diff --check`. No dependency manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 78 | `05f1737bb0cec1408a2e76b07d87117feb7ffe45f31cc529cabe55e57b458a2e` |
| `crates/seacad-dxf-core/src/planar_face_geometry_semantic.rs` | 405 | `e4265b565cac14fbceb306c563bcabe13795aabacc42ce8d13e0b7b1752253ec` |
| `crates/seacad-dxf-core/tests/planar_face_edge_visibility_tests.rs` | 247 | `89d6f54954970abb6d6a40e5af4bb5fb663d65065d57958b6f683e7fe0dc8b7c` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 102 | `26598b6d2ddd70a9a6094fc2f681e8b1a97be77e68d30cc7ca56c1a453cb5980` |
| `docs/IMPLEMENTATION_PLAN.md` | 1387 | `5ee8c7f627b47e5d6b9124a19caaa7c0d165d804c28848e9c9e16e2c6ab9cd26` |
| `docs/SUPPORT_MATRIX.md` | 1109 | `3adfc466e47de69baf66ae87d3511a4e5208aff3cc00195cdfd9d78846bc899d` |
