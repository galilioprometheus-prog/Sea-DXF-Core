# M14.3y Entity Value Encoder

## Scope

M14.3y begins the unified CRUD kernel with a create-new encoder for one typed
common-entity field group. It turns a generated common-field descriptor and a
borrowed edit value into one complete canonical ASCII or Binary DXF group. It
does not place that group into a document or mutate source bytes.

## Normative evidence

Autodesk's Binary DXF reference defines the sentinel, pre-R13 one-byte and
R13-and-later two-byte group-code representations, pre-R13 extended-data
escape, little-endian numeric payloads, NUL-terminated strings, and
length-prefixed binary chunks:

`https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-FC1C3C69-DBC2-49E4-893A-000D6538C0FE.htm`

Autodesk's group-code reference provides the wire-type ranges used by the
existing generated descriptor registry:

`https://help.autodesk.com/cloudhelp/2027/ENU/OARX-RefGuide/files/OARX-RefGuide-DXF_Group_Codes.html`

Autodesk's common entity-code table identifies group 310 as hexadecimal binary
data with at most 256 characters per ASCII line. One encoded group is therefore
bounded to 128 payload bytes:

`https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm`

The legacy fixture builder was inspected read-only as behavioral evidence:

`D:\SeaCad\tham khảo\New folder\cad_2026-07-23_source\crates\cad-dxf-lossless\tests\support\binary_fixture_builder.rs`

Its SHA-256 is
`5b7ebcf9e914a1f5394c1278342ef9614cfd34c284747a1df0c5e25570a862a6`.
It escaped every pre-R13 group code at or above 255. That broader behavior was
not imported: SeaCad follows Autodesk and its existing strict parser by
accepting the pre-R13 escape only for extended-data codes 1000 through 1071.
No external or legacy source, fixture, or implementation was copied,
translated, vendored, linked, or used at runtime.

## Contract

- `DxfEntityEditValue` keeps exact raw text, handles, binary64, Int16, Int32,
  and opaque binary chunks in distinct borrowed wire domains. A generated
  descriptor must match the value kind.
- ASCII groups use minimal decimal group-code and integer spelling, Rust's
  shortest round-tripping finite binary64 spelling, uppercase unprefixed handle
  hexadecimal, uppercase chunk hexadecimal, and LF framing.
- Binary AC1009 uses one-byte codes 0 through 254 and the documented XDATA
  escape. AC1012 through AC1032 use little-endian signed 16-bit codes. Numeric
  values are little-endian; text and handles are NUL-terminated; chunks have a
  one-byte length prefix.
- Exact raw text is cross-format-safe and rejects NUL, CR, and LF with the exact
  offending offset. Unicode-to-storage transcoding is deliberately separate.
- Binary64 values must be finite. A single chunk is limited to 128 bytes.
  Resource-profile value limits are checked before output allocation.
- Semantic failures are typed inner results; cancellation, resource limits,
  offset overflow, I/O-shaped internal failures, and allocation failure remain
  fatal outer `DxfError` values.
- The returned group retains descriptor, physical format, dialect, and bytes.
  Its `Debug` output exposes byte count but never payload content.

## Dialect and test boundary

Strict ASCII and Binary documents assembled from encoder output reparse for all
nine supported dialects AC1009 through AC1032. AC1009 fixtures omit common
codes above 254, and a dedicated assertion rejects group 420. Exact-byte tests
cover all six value domains and the modern two-byte group-code form. Typed
validation covers wire mismatch, non-finite binary64, all forbidden framing
bytes, an oversized chunk, the Safe profile's value limit, pre-cancellation,
public trait bounds, and debug redaction.

## Nonclaims

M14.3y does not accept Unicode semantic input or transcode legacy code pages;
construct arbitrary non-common descriptors; split, concatenate, or replace a
group-310 sequence; validate property domains or version applicability; select
duplicate occurrences; choose subclass insertion order or anchors; allocate
handles or owners; insert, update, clone, or delete records; build or verify a
transaction/inverse journal; write a destination; mutate a source; or advance
any entity to `Complete`.

## Verification

The focused encoder suite passes 4/4 tests and the workspace passes all 801
tests. Schema and release-evidence freshness checks, `cargo deny --locked
check`, format, workspace Clippy with warnings denied, workspace tests, the
production forbidden-macro scan, and `git diff --check` all pass. The checkpoint
adds 502 physical production lines: 497 in the encoder and five module/export
lines. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 135 | `8d432a227a45e1748731bb0877e6487081822e0703896f96e5a9670043ea147b` |
| `crates/seacad-dxf-core/src/lib.rs` | 944 | `7d0db291d080b0eb86df14ea1fe9a3c5e79c14020b4d21317b8623ba7c4c4b4a` |
| `crates/seacad-dxf-core/src/entity_value_encoder.rs` | 497 | `b38aa7748ca544f3bda71b689e8df211d96679fdea2367f8037dc43e5325c869` |
| `crates/seacad-dxf-core/tests/entity_value_encoder_tests.rs` | 381 | `cb86d2af431e5f5d0501d35deaa48c672534a87e740851807f22d897f679a11c` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 558 | `44954a237b10df681fe7afe0ce62b02d430855e6d6e5960e0331bc28acdb3557` |
| `docs/IMPLEMENTATION_PLAN.md` | 1983 | `416f3b59c30f5f815237041b57a7b25040f68345b93d0d22b53a9e9abee08781` |
| `docs/SUPPORT_MATRIX.md` | 1624 | `b63584398ea9eac87f80e7887562e44657dd32bf844ede632fa61f4b353fc3e6` |
