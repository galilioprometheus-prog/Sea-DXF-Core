# DXF-STRUCT-R1a HATCH Read Support

DXF-STRUCT-R1a removes identical fallible-construction helpers from the seven
M14.4t-M14.4z HATCH Line-edge modules. The crate-private `read_support` module
now owns checked `usize`/`u64` to `u32` conversion, exact source-identity
agreement, cooperative cancellation, and path-redacted `Read` errors for
invalid internal data and allocation failure.

Domain policy does not move. Boundary grouping, edge typing, Line cardinality,
numeric decoding, required coordinates, OCS geometry, and WCS projection still
live in their original modules and retain their immediate lower directories.
No public item is added, removed, renamed, or retyped; raw bytes, provenance,
issue precedence, debug redaction, compact metadata, and cancellation behavior
remain unchanged.

The seven focused suites pass 28/28 tests and workspace passes 1,193/1,193
tests. All gates pass. No dependency, license, schema, release, corpus, CI,
fixture, support-matrix, or support-claim change occurs. Production changes nine
files, adds 75 lines, deletes 264, and therefore removes 189 net lines.

The repository batch note and cumulative external `Status: READY` packet are
synchronized through DXF-STRUCT-R1a at
`D:\SeaCad\AntigravityReports\seacad-m14.3dr-dxf-struct-r1a-hatch-read-support-2026-08-09.yaml`.
No independent Antigravity PASS is claimed. This audit omits its own hash and
the self-referential active batch-note hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/read_support.rs` | 41 | `8bc57927b02ec282b32ede57d0a62cf3e9b7083ffa1ae9fd30fa680930480ed5` |
| `crates/seacad-dxf-core/src/hatch_boundary_edge.rs` | 468 | `b1f3c61d79bb4dbebc70c1e65e2ab03ebd3290ce9eee0bb643d6672342f5543b` |
| `crates/seacad-dxf-core/src/hatch_boundary_edge_type.rs` | 174 | `6e4819d65757e92fd37aa9fc35f1dfa3186f4f31d796b9daccd1d7f0f31aa927` |
| `crates/seacad-dxf-core/src/hatch_boundary_line_edge_card.rs` | 292 | `b012a334c8fce0727afdcf5e37b3b6b577600c4e0dfe21adc285c6666cfd3ab6` |
| `crates/seacad-dxf-core/src/hatch_boundary_line_edge_numeric.rs` | 307 | `4d6f849a4477d109c76d34f27b75a7190570543ab997edad827d1d892fd2e1ec` |
| `crates/seacad-dxf-core/src/hatch_boundary_line_edge_coordinate.rs` | 336 | `3ad0595a8345cffd3a1bef3857e1966fc2ae193a3c6960508a80aa38063ccdd0` |
| `crates/seacad-dxf-core/src/hatch_boundary_line_edge_geometry.rs` | 189 | `af93d35e08088958fd1adbcb15c0b1920d9b07905ce43d4756b65fa87606f2fc` |
| `crates/seacad-dxf-core/src/hatch_boundary_line_edge_wcs_geometry.rs` | 275 | `c83f303ba838986b8f1eadf9177a4e3281759c21b82b33b15bdf404163af4c52` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,423 | `5475c259467f331463b183bb183dffc0ecf3021d0c6f892c4828af50f32b6fc9` |
| `docs/ARCHITECTURE.md` | 197 | `d1b4d59ecc2d17064f3207b78379d0abac28a16c12f16fd7ca299a0ac7835db8` |
| `docs/IMPLEMENTATION_PLAN.md` | 424 | `6901925dbda10cda1276b56cbb9006c430c1f65f8b2964ef49eb16ef45c7fb01` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 2,197 | `de24cf8f1f3fe36ecc84b43c0b44f2b86759c57b118a01118543a0497728c405` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,617 | `aa023bae9e526acb797ae54cf5b318bdc54ef963c5755de72854531fc65537fe` |
