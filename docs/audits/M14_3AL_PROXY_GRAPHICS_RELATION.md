# M14.3al Proxy Graphics Relation

## Scope

M14.3al compares the reviewed common group-92 byte count with every exact
group-310 occurrence belonging to the same semantic entity. Proxy graphics
remain an opaque envelope.

## Sources

- Autodesk common entity codes define group 92 as proxy-graphics data size in
  bytes and group 310 as proxy-graphics data split into chunks:
  `https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm`
- SeaCad's reviewed raw wire contracts already establish that an ASCII group
  310 value is hexadecimal text and a Binary group 310 payload excludes its
  one-byte wire length prefix.

The user-authorized legacy trees remained read-only behavioral oracles. No
external or legacy code, fixture, data, dependency, or support claim was
copied, translated, vendored, or linked.

## Contract

- `DxfEntityProxyGraphicsDirectory` owns the existing common-field domain
  directory and publishes one source-bound relation per semantic entity.
- ASCII data must have even length and contain only hexadecimal digits. Its
  observed byte count is half the encoded length. Binary data contributes its
  validated raw payload-span length.
- Scanning uses a fixed 256-byte buffer with cancellation checks. Chunks are
  never concatenated, decoded as graphics, logged, or exposed as owned bytes.
- The states are `Absent`, `MissingSize`, `Matched`, `CountMismatch`,
  `InvalidSize`, and `InvalidChunk`. The first malformed chunk is reported in
  source order with exact group and value-span provenance.
- Invalid chunk evidence takes precedence over the size relation. A missing
  size is never inferred and a mismatch is never repaired.
- Construction and lookup are source-bound, allocation-fallible, group-order
  independent, and payload-redacted.

## Nonclaims

This checkpoint does not decode, render, tessellate, synthesize, normalize, or
edit proxy graphics. It does not validate version applicability, define
sequence replacement, or add clone/delete closure. No entity topic advances to
`Complete`.

## Verification

Paired ASCII/Binary fixtures cover AC1009 through AC1032, with the legacy
dialect exercising its representable zero-size/no-data relation. Negative
fixtures cover missing size, mismatched counts, negative size, odd ASCII hex,
invalid hex digits, cancellation, source mismatch, bounded lookup, and public
traits. Final gate counts, production diff, and artifact hashes are recorded
below. The focused suite passed 4/4 tests and the full workspace passed 850/850
tests. Generated schema and release-evidence checks, `cargo deny --locked
check`, formatting, workspace Clippy with warnings denied, workspace tests, and
`git diff --check` all passed. One initial workspace-test wrapper invocation
timed out and closed its output pipe; the independent full rerun completed with
exit 0. The production diff is 347 added lines: 342 in the relation projection
and 5 module/exports. This audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 194 | `cfdbd126888f839a6688b5a2240a903acb1ba7904ab6e9100929b52d5f139d02` |
| `crates/seacad-dxf-core/src/entity_proxy_graphics_relation.rs` | 342 | `2bbc1a1c9fcaab1d37dc1602d01e393ae8eeaf0f428ac84dc00f8db731b75350` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,001 | `80aa3a429a027a413ca5c82e0ae41a7422d7b72da87d41eb10a2a2eaa127b2ce` |
| `crates/seacad-dxf-core/tests/entity_proxy_graphics_relation_tests.rs` | 318 | `6ea0e765894c72c33b92cf1084a140898ede12df8d90c987cd11d99b2a85fa77` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 716 | `070ba3254b0fc3486c6c95266ca484576ba716f4c27fc21c6e7cf1c98c633bce` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,144 | `c12dc6871ac6337855f1c111fd46090308b79d618efcddb2e32cb527d99964b9` |
| `docs/SUPPORT_MATRIX.md` | 1,804 | `435ea44aba90488cfca49493cdc223d7d4dd2fdca70f524468c4cd4897190d9b` |
