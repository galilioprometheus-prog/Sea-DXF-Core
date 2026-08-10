# M14.4an HATCH Boundary Edge Payload Partition

M14.4an separates each grouped non-Polyline HATCH edge payload from the
path-level source-boundary-object trailer without decoding either group-97
count or any group-330 handle. Interior edges retain their complete exact
payload. Only the final edge of a path can own the outer trailer.

Autodesk's
[Boundary Path Data table](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-DC5215D6-E73F-4DFF-8BE9-01CA9610FAEE.htm)
places group 97 followed by repeated group 330 references after the complete
non-Polyline edge sequence. The same source separately places group 97 inside
Spline edge data as the fit-data count, followed by fit points and optional
start/end tangents. The two occurrences therefore belong to different grammar
levels and cannot be classified by group code alone.

For every final edge, the last group 97 is the outer source-object count only
when no group 330 occurs before it and every following field is group 330. The
edge-data range ends immediately before that anchor, while the exact
group-97/330 suffix remains available as the path-trailer range. On a Spline
edge, an earlier group 97 and all intervening fit points/tangents remain edge
data. A non-final edge is retained byte-for-byte as edge data and receives no
trailer range.

Missing outer counts, source references before the selected outer count, and
non-330 fields after it remain typed `SourceBoundaryCountAbsent`,
`SourceBoundaryReferenceBeforeCount`, or `UnexpectedTrailerField` results.
Malformed partitions publish no edge-data or trailer slice but retain the
complete lower M14.4t/M14.4u grouping and type evidence. Declared/observed edge
count mismatch does not erase otherwise partitionable entries.

The focused suite passes 4/4 tests and the edge grouping-to-partition chain
passes 12/12. Workspace passes 1,249/1,249 tests and all required dependency,
format, schema, release-evidence, Clippy, exact test-count, forbidden-production-
pattern, protected-surface, link, and whitespace gates pass. Production adds
305 lines: a 300-line module and five module/export lines. Tests add 341 lines.
No dependency, license, schema, release, corpus, CI, fixture, or protected-
surface change occurs. Source-count numeric/cardinality semantics, source-
handle resolution, repeated Spline knot/control-point/weight/fit/tangent
grouping, topology, geometry, applicability, CRUD/write, rendering, and
`Complete` remain open.

The repository batch note and cumulative external `Status: READY` packet are
synchronized through M14.4an at
`D:\SeaCad\AntigravityReports\seacad-m14.3dr-m14.4an-hatch-boundary-edge-payload-partition-2026-08-10.yaml`.
No independent Antigravity PASS is claimed. This audit omits its own hash and
the self-referential active batch-note hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 295 | `9f869578ca3f03bbd3125b5de15e78eea0f772e404df2bb524f151e112b262ba` |
| `README.vi.md` | 294 | `6b2a7f3bd081ffc75d1246033c685a5db49acd63d854d124da80dfe1ef1feea5` |
| `crates/seacad-dxf-core/src/hatch_boundary_edge_payload_partition.rs` | 300 | `10e187bb2f9bb01bc93dab662adc0014be0aa19eb12499d9289989b84130ff3f` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,531 | `a88ea960b8e7fdabb844fe1392119fbee9d7097188ea2d3d0ccafed53b57e2e4` |
| `crates/seacad-dxf-core/tests/hatch_boundary_edge_payload_partition_tests.rs` | 341 | `930caa7a300efd893d7714050c7b8aff59b5bea47a8b0d8da65ddd78d9c5a2fa` |
| `docs/IMPLEMENTATION_PLAN.md` | 424 | `1e1989cbb078b8e2c5191775a796fcbd7c339304e73e582c354fd66b6bdea0b0` |
| `docs/SUPPORT_MATRIX.md` | 3,465 | `bc250c1297a1c62620951cae35a5a3b2bcdd6bfe197c78b48b461cfc753888bb` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,388 | `8d9a8c95363f66feb15e5f57568e74333a9319043d955cd2ba3932554e0c8c64` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,817 | `1b4f8364402d540f06e22c786ea1396bdeb426a3b0617310e07b8c1daa14214e` |
