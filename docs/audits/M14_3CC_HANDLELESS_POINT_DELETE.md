# M14.3cc Handleless POINT Delete

Retrieved: 2026-08-03

## Scope

M14.3cc closes standalone whole-record deletion for a canonical POINT whose
record-local handle identity is exactly absent. It preserves the stronger
handle-backed delete receipt and exposes handleless admission as a distinct
typed outcome. It does not admit invalid, null, multiple, or ambiguous
identity, mix delete with other queued operations, batch deletes, or advance
POINT to `Complete`.

## Prior evidence

- M14.3i through M14.3r establish exact record identity, handle candidates,
  document-local handle resolution, and incoming reference evidence.
- M14.3aw establishes source-bound transaction composition and exact inverse
  materialization.
- M14.3ca establishes exact reference-safe canonical POINT deletion when a
  unique non-null handle exists.
- M14.3cb establishes canonical semantic POINT cloning through fresh identity.

No legacy or external parser code, fixture, data, dependency, generated
artifact, or private corpus content was copied, translated, vendored, linked,
or used at runtime.

## Contract

- `DxfEntityEditSession::delete` admits `DxfHandleIdentityState::Absent` only
  for the same standalone canonical POINT boundary established by M14.3ca.
- `DxfEntityDeleteOutcome::HandlelessApplied` carries a dedicated
  `DxfEntityHandlelessDeleteReceipt`; handle-backed admission retains
  `DxfEntityDeleteReceipt` and its non-optional handle.
- Invalid handle syntax, a null parsed handle, multiple candidates, or an
  ambiguous document identity remains unavailable and queues no operation.
- The transaction deletes the exact complete raw record from its group-zero
  marker through its final group and leaves neighboring records untouched.
- With no handle available for absence lookup, the semantic expectation stores
  the exact post-image entity count. Strict reparse must prove that exactly one
  entity disappeared before the executable inverse is released.
- The first handleless checkpoint retains the standalone session boundary:
  updates, inserts, clones, and additional deletes cannot be mixed with it.

## Verification boundary

Paired in-memory fixtures exercise ASCII and Binary AC1009 through AC1032. All
18 dialect/encoding pairs prove handleless admission, exact POINT removal,
retention of the following LINE, the one-entity semantic reduction, strict
post-image verification, and byte-identical inverse restoration. Negative
coverage preserves incoming-reference and ambiguous-identity failures for
handle-backed records and replaces the formerly absent-identity rejection with
an invalid-identity rejection. Existing cancellation, wrong-family, null,
mixing, raw-span, and inverse coverage remains active.

## Nonclaims

This checkpoint does not mix or batch deletes, calculate arbitrary entity graph
closure, cascade ownership graphs, clone broader common properties, render
POINT display behavior, or claim complete entity editing.

## Verification

The focused POINT edit suite passed 39/39 tests. The full workspace passed
954/954 tests, including all 17 locale tests. Generated-schema and release-
evidence checks, `cargo deny check`, formatting, workspace Clippy with warnings
denied, the production forbidden-construct scan, and `git diff --check` all
passed. Production changes are 155 insertions and 67 deletions; focused test
changes are 66 insertions and 12 deletions. No manifest, lockfile, dependency,
committed fixture, generated schema, locale source, or external corpus changed.
This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 361 | `a61430f9768e024027924e2dadc837aa0671988844fa58409d8705803e686935` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 2,449 | `51b8ccfb2f2b8a9824e703a358ac078bd55738e67aba076cd106dd062cc08a36` |
| `crates/seacad-dxf-core/src/entity_edit_verification.rs` | 1,240 | `78dc656b957db7379761bfe4b4a2512628c7e8e393c234e67f4f1667751c172c` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,068 | `64aca4f5d173a42a8ad0116e2bfe93340012ca49e5531418d8a984a55eea5536` |
| `crates/seacad-dxf-core/src/point_edit.rs` | 1,084 | `bcef8c022f2c17e5bf672a06692a9c28787ef7e56fbc24194c0ab44a33c46e51` |
| `crates/seacad-dxf-core/tests/entity_insert_session_tests.rs` | 1,027 | `a58c0c0b979a176093ec5f15716c0d512d54d9f83c9592d60c2235328bfe69d8` |
| `crates/seacad-dxf-core/tests/point_edit_session_tests.rs` | 3,059 | `33e3ca0baf1b4327fe6c9065e1237b079dc890d78181c8c469b0752206149fd5` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,246 | `0299e07fd30ac912492b50e7b303bc1be86266845a872b66f5c7fa2e6b208db9` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,674 | `ec4dd5fe6c25ac4d2b754a754f5e66f995fffd1e165a1f0f3f508406766ad0a4` |
| `docs/SUPPORT_MATRIX.md` | 2,325 | `e72e8f08e104c25ef04bd3d2d7483e2411ed37eaf751e1a49984dfb5e3f9aecf` |
