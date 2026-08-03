# M14.3cj POINT Proxy-Graphics Clone

Retrieved: 2026-08-03

## Scope

M14.3cj extends canonical semantic POINT clone with matched AC1012+ opaque
proxy graphics in group 92 and the group-310 sequence. ASCII hexadecimal and
Binary chunk storage are normalized to the same bounded decoded-byte payload,
then canonically emitted, strictly reparsed, compared byte-for-byte, and covered
by the executable inverse.

## Contract

- Clone admission uses the source-bound proxy relation and accepts only
  `Matched` size/data state or complete absence.
- The group-92 declaration must equal the combined decoded byte count of every
  group-310 member; missing, invalid, malformed, or mismatched relations fail
  typed without queueing an insertion.
- Payload materialization is source-identity checked, cancellation-aware, and
  bounded by the selected value-byte resource limit.
- ASCII chunks require exact hexadecimal pairs and decode case-insensitively;
  Binary chunks retain their validated payload bytes.
- The POINT draft emits one canonical nonnegative group-92 count followed by
  bounded 128-byte group-310 chunks. AC1009 rejects non-empty group 310 as not
  applicable to its one-byte Binary group-code wire.
- Post-image verification reconstructs the inserted payload and compares exact
  bytes in addition to checking the size relation before releasing a journal or
  inverse.

## Verification boundary

The positive fixture carries two source chunks totaling three bytes. Every
supported AC1012+ version in ASCII and Binary clones the POINT to a fresh
handle, canonicalizes the payload to one chunk, reports an exact three-byte
matched relation, verifies the decoded payload, and produces an exact inverse.
A dedicated negative fixture declares four bytes for the same three-byte
payload and proves the typed `CountMismatch` outcome leaves the queue empty.
Existing minimal, scalar-common, linetype, object-reference, color-book,
POINT-payload, scope-decoy, invalid-domain, owner/placement, and mixed-session
tests remain active.

## Nonclaims

The payload remains opaque: this checkpoint does not render, interpret, or
synthesize proxy graphics. It does not clone group-102 application scopes,
persistent reactors, extension dictionaries, XDATA, or arbitrary ownership and
reference graphs, and does not support cross-container or cross-document clone.
POINT is not yet `Complete`.

## Verification

The focused entity insert session suite passed 18/18 tests and the POINT edit
suite passed 43/43 tests. The full workspace passed 967/967 tests, including all
17 locale tests. Generated-schema and release-evidence checks, `cargo deny
check`, formatting, workspace Clippy with warnings denied, the production
forbidden-construct scan, and `git diff --check` all passed. Production changes
are 225 insertions and 18 deletions; focused test changes are 131 insertions and
4 deletions. No manifest, lockfile, dependency, committed fixture, generated
schema, locale source, or external corpus changed. This audit intentionally
omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 389 | `6c04f604713b6b1baf2fb59ec9b1c8adb1e3d03f8872084105baa616067adb8a` |
| `crates/seacad-dxf-core/src/entity_draft_record.rs` | 1,301 | `35ce0fb968aeb7ace44cc217010678e06d49361c1d3e426dab051b54e45a88d0` |
| `crates/seacad-dxf-core/src/entity_edit_session.rs` | 2,805 | `61d3c594a6bc3bf603ff026c6b7588f06f4b97d15f647dc799e26cfdd8408e08` |
| `crates/seacad-dxf-core/src/entity_edit_verification.rs` | 1,337 | `b265960162dfdd51dbd05d29e0124f208eff28fb602a1136c84fc846a2480811` |
| `crates/seacad-dxf-core/src/entity_proxy_graphics_relation.rs` | 424 | `485a30fc0bc361e4a4a46b61398e25825259faff95c3dbea3c5dfd408a79da7b` |
| `crates/seacad-dxf-core/tests/entity_insert_session_tests.rs` | 1,730 | `5d72be26ca7a663289006d0c80fce216a55e77d6a19ac6921bc9ff007dde94a3` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,314 | `3a59e90fda5b196d8aa80a24e2727ef2bb703109cb0658f576ea676419fb1b56` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,738 | `968e6b839fcf8969e21027c799b03c03131aca13244ad0fa4725d12ec6c620d8` |
| `docs/SUPPORT_MATRIX.md` | 2,390 | `717d1cb765a267ae8227aeaf04bace4de332cd825a1a06a6da48a9bdfbdcad33` |
