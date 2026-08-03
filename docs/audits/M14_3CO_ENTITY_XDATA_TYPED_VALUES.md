# M14.3co Entity XDATA Typed Values

Retrieved: 2026-08-04

## Normative basis

Autodesk's *About Extended Data (DXF)* assigns the generic XDATA wire domains:
group 1000 strings of at most 255 bytes, group 1002 list controls, group 1003
layer names, group 1004 binary chunks of at most 127 decoded bytes, group 1005
database handles, the point/vector and scalar doubles in groups 1010 through
1042, group 1070 signed 16-bit integers, and group 1071 signed 32-bit integers:

<https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-A2A628B0-3699-4740-A215-C560E7242F63.htm>

## Contract

`DxfEntityXDataTypedDirectory` is a format-neutral, source-bound projection of
every occurrence retained by `DxfEntityXDataDirectory`. Application markers,
application values, and orphans remain one-to-one and source ordered. Each
entry exposes exact text/control provenance, a validated binary chunk, a
handle, one of 15 distinct double roles, a signed integer, or an explicit typed
invalidity. No malformed or unsupported occurrence is dropped or normalized.

Group-1000 payloads above 255 bytes fail typed. ASCII group-1004 payloads must
contain an even number of hexadecimal digits; both formats enforce the
127-decoded-byte ceiling. Valid chunks decode only into an exactly sized
caller-owned buffer. Numeric projection reuses the reviewed raw decoders and
rejects non-finite doubles. Every lookup validates source identity, and all
source scans and caller-buffer reads remain cancellable and bounded.

## Verification boundary

The focused suite pairs ASCII and Binary fixtures for AC1009, AC1012, AC1014,
AC1015, AC1018, AC1021, AC1024, AC1027, and AC1032. It covers every documented
value role, uppercase/lowercase hexadecimal decoding, malformed control,
overlong string/chunk, odd and invalid hexadecimal data, invalid handle,
non-finite number, unsupported code, stable lookup, application slicing,
source mismatch, cancellation, metadata bounds, and non-disclosing debug
output.

This checkpoint does not claim layer-name resolution, point/vector tuple
grouping or coordinate transforms, the total 16-KiB XDATA application policy,
application-specific payload semantics, group-1005 target resolution or remap,
clone/write behavior, or any entity's `Complete` status.

## Gate receipts

The focused typed-value suite passed 3/3 tests and the full workspace passed
all 980 listed tests. Generated schema and release-evidence checks,
`cargo deny --locked check`, formatting, workspace Clippy with warnings denied,
the production forbidden-construct scan, and `git diff --check` all passed.
Production changes are 511 insertions; the focused test target adds 399 lines.
No manifest, lockfile, dependency, generated schema, locale source, committed
fixture, or external corpus changed. This audit intentionally omits its own
hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 424 | `619fb33eb6b0fd225758d5c55fa19dd9ae2b3fef85962ff960d3f3d5427ad84e` |
| `crates/seacad-dxf-core/src/entity_xdata_value.rs` | 505 | `02dd65e640b56cadd2e8a6480a64c3c76f366acad4cb45369f6cb7529290abaf` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,090 | `4e902fd2dc037392c3d1a2d5d84b5603d0004b256985d9426285be560e28dbed` |
| `crates/seacad-dxf-core/tests/entity_xdata_value_tests.rs` | 399 | `4f0702944529d0e56e082f7d68d08dd04c365d102e160ee5e7293910d02d5be3` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,374 | `82adb81d017c73d77bc5b10777f42a7cf34766a13b648688a05fb7b112ec4b8d` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,796 | `34fcd1ad02c27d150be44de6e8ae70fb2fa797e7466c814992d9a238634d580b` |
| `docs/SUPPORT_MATRIX.md` | 2,440 | `c40c385c0065c327a3a566a5bf4195f8adc73474c7d95b51f80b542ac36f6a17` |
