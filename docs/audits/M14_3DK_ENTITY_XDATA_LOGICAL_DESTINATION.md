# M14.3dk Entity XDATA Logical Destination Projection

Retrieved: 2026-08-08

## Contract

`DxfEntityXDataLogicalDestinationDirectory` owns M14.3dj payload-envelope
readiness and publishes one compact source/destination-bound logical state for
every retained typed XDATA occurrence. Source string, control, binary chunk,
non-coordinate double, and integer values remain exact source-referenced
values. Group-1001 and group-1003 values bind exact destination APPID/LAYER
records, group-1005 binds only a uniquely validated remapped handle, and each
complete coordinate component binds its transformed binary64 value.

Unavailable entries retain distinct orphan, invalid-source, APPID, LAYER,
handle, and coordinate issues. The directory revalidates typed-entry ownership,
supports exact per-entity slicing, and resolves each logical entry back to its
M14.3dj payload entry. Handle destination evidence adds exact typed-entry lookup
through its owned typed/resolution/remap chain.

## Verification boundary

Three focused tests cover all four ASCII/Binary source-destination pairings for
all nine Core dialects plus AC1009/AC1032 cross-dialect pairs. Coverage includes
string, list control, binary chunk, non-coordinate double, int16, int32,
destination APPID/LAYER targets, a remapped handle, all three transformed tuple
components, and independent missing application/layer/handle, partial tuple,
orphan, and invalid-source states. Source/destination foreign rejection,
per-entity slicing, cancellation, compact metadata, bounds, and debug redaction
remain explicit.

This checkpoint does not interpret application-specific meaning, transcode
text for a destination storage policy, emit destination bytes, insert XDATA,
mutate either document, complete a cross-container clone, or advance POINT or
another entity to `Complete`.

## Gate receipts

Focused logical-projection tests passed 3/3; APPID/LAYER/coordinate/handle/
logical regression suites passed 15/15. The workspace passed exactly 1,034
tests. Cargo-deny, formatting, schema and release-evidence checks, workspace
Clippy with warnings denied, production safety scan, protected-surface diff,
and `git diff --check` passed. The complete gate run finished in about 275
seconds. No manifest, dependency, lockfile, schema, corpus, legal, or release
surface changed.

Production adds 516 lines across logical projection, typed-handle lookup,
payload typed exposure, and public exports; focused tests add 597 lines. This
audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 155 | `d8219e54a0f4ae727f5d300c72b67f925a43eaf576e965aa3b7d2ee7db84a7f7` |
| `README.vi.md` | 154 | `d8a7179e993c4f06d669bdf9b0de9fc7a2e1d12ae9b9d079b4bcce758e5f488e` |
| `crates/seacad-dxf-core/src/entity_xdata_handle_destination.rs` | 327 | `3e2ba2bf2f4c482fc9fcb81042d106567afee216537ee338a7d5069080660ef0` |
| `crates/seacad-dxf-core/src/entity_xdata_payload_destination.rs` | 339 | `47c7950a29441a9d990aa0a80541e0f10c177f4431cc6c4853f9fca1df89bfd5` |
| `crates/seacad-dxf-core/src/entity_xdata_logical_destination.rs` | 487 | `192eacdf21fb2cd027132002033ac88429b946ded381c9bb10a8977a15a35025` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,204 | `5ccea8aff5374ec856181c1f2ef4d9704bad843bf7261321d5c2f90342b2ffdf` |
| `crates/seacad-dxf-core/tests/entity_xdata_logical_destination_tests.rs` | 597 | `12936341bf11638e310610cdc0179fff4b02ce51a60d7fb4a3d0c28780616986` |
| `docs/IMPLEMENTATION_PLAN.md` | 423 | `c674079e616666a52e4ba18c5a101ea68b67dff56bc915049e36c3fd0a15b9e5` |
| `docs/SUPPORT_MATRIX.md` | 2,716 | `fea82d6289230808cddb09c75cc7983188f1a63c9a07b9f21bfc07229d05401f` |
| `docs/plans/dxf-core-1.0/IMPLEMENTATION_PLAN.md` | 3,083 | `01d8e873b345fc378ad21294cbb03431c4c86846168727b45f91303c33aaa2c9` |
| `docs/plans/dxf-core-1.0/DXF_ENTITY_COMPLETION_PLAN.md` | 1,660 | `1d35b8baa82e3eaf277b530db8ad91f6250e439be21dedb4cf1b8c3a39c180fb` |
