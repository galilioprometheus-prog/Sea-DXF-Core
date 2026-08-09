# M14.4p HATCH Polyline Segment Shapes

M14.4p classifies each M14.4o segment from its start vertex's effective bulge.
Signed zero is Straight, finite nonzero is Arc with exact bits, and unusable
bulge semantics remain Indeterminate with their typed issue. Coordinate failure
does not erase shape, and exact start-bulge provenance remains resolvable. No
line or arc geometry is inferred.

Autodesk documents group 42 as the Polyline vertex bulge:
`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm`

The focused suite passes 4/4 tests and workspace passes 1,153/1,153 tests. All
gates pass. No dependency/license/schema/release/corpus/CI change. Production
adds 205 lines: 200 module and five exports. Arc center/radius/angle
construction, line geometry, OCS/WCS transformation, applicability, CRUD/write,
and `Complete` remain open. This audit omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 227 | `e2c8b0ed3716231a518f1f1c17310c5cb653a318da333dd42e8de938e4ba6c4f` |
| `README.vi.md` | 225 | `86e5c828532d75bdf3f0d543f2af70eeb87f1f5aa4cd3ab7802e79a4b0c8b336` |
| `crates/seacad-dxf-core/src/hatch_polyline_segment_shape.rs` | 200 | `b3b99e6cbc1358ad7c7465060201420f97859b665faef6b9fb7a56fdce41cbcd` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,363 | `cec27cfc227bbb8c44feeeaf603c54100e60cbf49158070f2c071c5c090bdadf` |
| `crates/seacad-dxf-core/tests/hatch_polyline_segment_shape_tests.rs` | 343 | `d1afcf5105e1f59a0cd5199ace5706e9e532a343b7cff3c9bc57bea808f35e96` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `032841ed67695a786a086c1f70b72964c15ec03d53e41c30c25e531a20302979` |
| `docs/SUPPORT_MATRIX.md` | 3,169 | `918f5a4c84c79ed495612faea68c29ad4efd6bae4360b66f93f30a4ae0485798` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,081 | `5de796bc3d64db867b7005e9c6cf1d9414a76302ef75b4617b08836cd5715f1c` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,499 | `0fc1212164addfa855540eb24d0d990a4f68b39c93c67fbc0acb964cafcaccc9` |
