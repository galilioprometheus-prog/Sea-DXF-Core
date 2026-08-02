# M14.3bi AutoCAD 2007 Entity Alias Receipt

## Scope

M14.3bi replaces behavioral-only alias provenance for `SECTIONOBJECT` and five
surface specializations with Autodesk evidence, then admits those six exact
draft names only for AC1021 and later supported dialects.

## Normative evidence

Autodesk's valid DXF object-name reference maps the concrete names
`SECTIONOBJECT`, `EXTRUDEDSURFACE`, `LOFTEDSURFACE`, `PLANESURFACE`,
`REVOLVEDSURFACE`, and `SWEPTSURFACE`. The generated alias source receipt
records:

- source ID `autodesk.entity_dxf_names.2024`;
- Autodesk topic `GUID-ECB6F2FF-6680-4514-86A7-7AD5551E378D`;
- evidence kind `alias_list`;
- normalized six-row SHA-256
  `35319d08d265d9cbb3aed819d012f15b4bf19249bb39df04c557b461645645ef`.

Autodesk's AutoCAD 2007 API History separately marks `IAcadSection`,
`IAcadSurface`, `IAcadPlaneSurface`, `IAcadExtrudedSurface`,
`IAcadRevolvedSurface`, `IAcadSweptSurface`, and `IAcadLoftedSurface` as new.
The existing Autodesk-backed `$ACADVER` registry maps AutoCAD 2007 to AC1021.
The generated applicability source receipt records:

- source ID `autodesk.autocad2007.entities.compatibility.2024`;
- Autodesk topic `GUID-CC6BE90C-5ABE-4DE5-9390-B36FDCFF798B`;
- evidence kind `applicability_list`;
- normalized six-row SHA-256
  `4bc579a7aa48678e7588de1b98e9c3eb3383461ab14d78a7a0334d5a8f167eb9`.

Moving six aliases away from the AutoCAD inventory oracle leaves that source
with only `ARC_DIMENSION`, `LARGE_RADIAL_DIMENSION`, and `MULTILEADER`; its new
normalized receipt is
`377c7af5f38531e128f5f89e4d3442614199cdcb4b8696bd18b809e9b3d4f3f2`.
All three receipts fail closed when source identity or normalized facts drift.

The user-authorized legacy trees under `D:\SeaCad\tham khảo\New folder` were
searched read-only for behavioral risks. Their AutoCAD-derived inventory and
public-repository intake observe all six concrete names. They also record no
generic group-0 `SECTION` or `SURFACE`, only `SECTIONOBJECT` and concrete
surface records containing base-class subclasses. This supports keeping the
two canonical topic labels fail-closed but is not used to establish the
minimum. No external or legacy code, fixture, data, dependency, or unsupported
fact was copied, translated, vendored, or linked.

## Contract

- The six exact aliases are `NotApplicable` for AC1009 through AC1018.
- The six exact aliases are `Applicable` for AC1021, AC1024, AC1027, and
  AC1032, with no reviewed maximum version.
- Alias descriptors expose `Normative` evidence and the exact Autodesk
  valid-DXF-name GUID.
- Applicability descriptors expose `AutodeskCompatibility` evidence and the
  exact AutoCAD 2007 API-history GUID.
- Canonical `SECTION` and `SURFACE` remain `NotYetReviewed` for all supported
  dialects; topic labels are not treated as interchangeable wire names.
- The independent draft-admission matrix tests all 59 names over all nine
  ASCII and Binary dialect pairs.
- Fifteen names now have reviewed ranges. The other 44 remain
  `NotYetReviewed`.

## Nonclaims

This checkpoint does not decode Section or proprietary surface/modeler
payloads, synthesize surfaces, encode records, insert, update, clone, delete,
or claim `Complete` support. It changes no production dependency and does not
infer applicability for generic `SECTION`, generic `SURFACE`, or other
AutoCAD 2007-era classes.

## Verification

Focused schema tests passed 9/9 and focused draft-applicability tests passed
4/4. The full workspace passed 898/898 tests. Schema `--check`, release-evidence
`--check`, `cargo deny --locked check`, formatting, workspace clippy with
warnings denied, forbidden generated-production macro scan, and
`git diff --check` all passed. This
evidence/generated-only checkpoint adds no handwritten production code; the
generated production changes are provenance and applicability receipts, so the
usual 200-500 handwritten production-line target does not apply. No production
dependency changed. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 253 | `873d5d97f00407f40eb7baad32901f513721c8246aeb55c0d415b645b4c53744` |
| `schema/dxf/v1/sources.json` | 101 | `1aef144d4fbce5dd5ece1a4c9388ae5246d860683cb6735106a4ee293b5f7555` |
| `schema/dxf/v1/entity_aliases.json` | 20 | `5172bd1753c700a7841507c60d052b97404be0743505962a24b71d149915ea95` |
| `schema/dxf/v1/entity_applicability.json` | 65 | `2d082c02c8243a42ca35a81604a4b57b371e7e65d5dc18a4aa0f3a7637b082ef` |
| `crates/seacad-dxf-core/src/generated/entity_schema.rs` | 1576 | `d8d1c001174e2dd2e82e0f7abeab9855920bc14388bf0fcb1a4a2452c1d8e4e2` |
| `crates/seacad-dxf-core/src/generated/header_schema.rs` | 2392 | `59b2d552f9d453ee0b75343295f83889b3dd7f874ff3331ccffdee73b37b9177` |
| `crates/seacad-dxf-core/tests/entity_schema_tests.rs` | 627 | `eed17df5f557830bbe9519ddbf3b8f0d1d1e2627520f98d72599941551cecf51` |
| `crates/seacad-dxf-core/tests/entity_draft_applicability_tests.rs` | 467 | `1a43ffb7f2d39dd80c5ed6ff168b5972c9d00a592191c37f089cc88d160d0d99` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 986 | `ec2bdff62d1cc6548d0563c7320a1d492f63ffd32362802266d5e2125def93da` |
| `docs/IMPLEMENTATION_PLAN.md` | 2420 | `07d4da10526f1718780707e42ceace69fb67fc8b8dcf2b5f1890e39db0e8e9a1` |
| `docs/SUPPORT_MATRIX.md` | 2071 | `d2c5c26510e924e7b0f477eba61b695e5080b419150a5fc7644f5862611cc241` |
