# M14.3cq Entity XDATA Layer Resolution

Retrieved: 2026-08-04

## Normative basis

Autodesk's *About Extended Data (DXF)* defines group 1003 as the name of the
layer associated with the XDATA value:

<https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-A2A628B0-3699-4740-A215-C560E7242F63.htm>

Autodesk's LAYER reference identifies group 2 as the LAYER symbol-table entry
name, and its symbol-table framing reference requires a matching record kind
inside TABLE/ENDTAB framing:

<https://help.autodesk.com/cloudhelp/2015/ENU/AutoCAD-DXF/files/GUID-D94802B0-8BE8-4AC9-8054-17197688AFDB.htm>

<https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-5AB9300F-F0AC-4ADE-89EA-A9D1D152D8B8.htm>

## Contract

`DxfEntityXDataLayerResolutionDirectory` owns the generic typed XDATA and named
symbol-table directories. It emits one stable entry for every group-1003 typed
value, including application values and orphans. Each entry retains its source
identity and compact typed-entry/entity ordinals, then publishes Missing,
Unique, or Ambiguous resolution.

Only exact group-2 names admitted from completely closed LAYER tables are
candidates. A sorted SHA-256 index bounds the candidate range; authoritative
source spans are still compared byte-for-byte, so digest collisions cannot
create a false match. Wrong table kinds, near-case names, records with multiple
group-2 values, and unclosed tables are never admitted. Foreign entries and
source values fail closed during reverse lookup.

## Verification boundary

The focused suite pairs ASCII and Binary fixtures for AC1009, AC1012, AC1014,
AC1015, AC1018, AC1021, AC1024, AC1027, and AC1032. It covers exact unique,
duplicate ambiguous, missing, near-case, wrong-table, malformed-record,
unclosed-table, application, orphan, multi-entity, source-identity,
cancellation, lookup-bound, metadata-bound, and non-disclosing-debug cases.

This checkpoint does not validate the full layer symbol-name character policy,
calculate Autodesk `xdsize`, enforce the per-entity 16-KiB limit, apply XDATA
coordinate transformations, assign application-specific payload meaning,
resolve or remap group-1005 targets, clone/write XDATA, or advance any entity
to `Complete`.

## Gate receipts

The focused layer-resolution suite passed 3/3 tests and the full workspace
passed all 986 listed tests across 185 targets. Generated schema and release-
evidence checks, `cargo deny --locked check`, formatting, workspace Clippy with
warnings denied, the production forbidden-construct scan, and `git diff
--check` all passed. Production changes are 348 insertions; the focused test
target adds 304 lines. No manifest, lockfile, dependency, generated schema,
locale source, committed fixture, or external corpus changed. This audit
intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 439 | `ae7e77804021f5f7bffbf8fada932766244fcd89174f7a4f79238ebb1bdd6618` |
| `crates/seacad-dxf-core/src/entity_xdata_layer_resolution.rs` | 343 | `1710feb68e853b9f755cf5af560b20776e1c09db16bf4e63943e807ff60306ed` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,100 | `48f85d5dd2d2f20de557265e0a34b3d2834967b9c57cfdd073f158a188554afb` |
| `crates/seacad-dxf-core/tests/entity_xdata_layer_resolution_tests.rs` | 304 | `39f6c396138782be29ac18622abdba208a29e7a825d38c1d61d2096b053646c8` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,401 | `f1528347fe6f24618ab38fc3d0da03566bf01c0faee0737c629e2163e444c44b` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,820 | `e2419260e262091ba21123da027b9bbd090ef35f00aa45996a8978e0b209a754` |
| `docs/SUPPORT_MATRIX.md` | 2,461 | `21678b91a5d1797be7ec22d7d8cd127cd700b91bc951f89c019afca0e533a86e` |
