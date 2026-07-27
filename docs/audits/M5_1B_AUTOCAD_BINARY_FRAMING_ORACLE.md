# M5.1b AutoCAD Binary DXF full-stream framing oracle

Status: completed 2026-07-27

No external parser source was inspected, copied, translated, or ported. The
only parser used in this audit is the current local SeaCad path dependency.

## Inputs and provenance

The three external fixtures were produced by AutoCAD Core Console 2027 under
the isolated procedure frozen in the M5.1a reference audit.

| Dialect | Encoding | Bytes | Groups | Fixture SHA-256 |
| --- | --- | ---: | ---: | --- |
| AC1009/R12 | one byte + XDATA escape | 3,926 | 535 | `76f9f56bf9a4395c7bc324cf831c5e473cad68a5ee0a442975cc7647c08f5a04` |
| AC1015/AutoCAD 2000 | 16-bit little-endian | 112,153 | 10,281 | `cd059a69f0c238f28807b2079b53aef6d04c68d0b486416ac2bede3df59de2c3` |
| AC1032/AutoCAD 2018 | 16-bit little-endian | 42,235 | 5,898 | `4b5584619ef9ba4665f46caf268891b7dd54ad527955348325484d6f7c80b0ef` |

AutoCAD executable version `26.0.60.0.0` has SHA-256
`fc6b59c29fea759d556083cf43d9de3657974ef0bf57962d49a19892f729fa77`.
Fixtures, profiles, scripts, helper build output, and logs remain outside the
repository.

## Independent checker procedure

The external checker opens each fixture with `DxfFileSource` and the explicit
M5.1a encoding, then advances `DxfBinaryGroupCursor` to physical source end.
For every returned pair it proves:

1. `full_span.start` equals the previous pair's `full_span.end`;
2. bytes reread directly from the source over `full_span` equal
   `raw_group_code || raw_value` exactly;
3. the final span ends at the source length and the cursor is complete;
4. the first group-0 marker is `SECTION` and the last is `EOF`.

The checker does not use AutoCAD-save output as evidence for SeaCad unchanged
replay, which remains deferred.

| External checker artifact | SHA-256 |
| --- | --- |
| `Cargo.toml` | `9bce75a50d22dc54974e2059bfde7809a86a63777b7d42dc99ed727cd79e8600` |
| SeaCad-only checker source | `46ef1d79a809c5f16962b36d0613d0db686629975c72a19459550f1b2666c77a` |
| offline runner | `615e3d4ba8a7044118501eac41488a2f7760afdb7e166f5336ca979a00b268db` |
| UTF-8 result log | `ce2873055bbbda41a8e019ac1ca4565c950c7c9bb8598cad64af2822855d003d` |

## Results

Family counts use this fixed order: string, f64, i16, i32, i64, Boolean,
binary chunk.

| Dialect | Family counts | First/last marker | Continuous byte accounting |
| --- | --- | --- | --- |
| AC1009 | `[242, 158, 135, 0, 0, 0, 0]` | `SECTION` / `EOF` | 22 through 3,926 |
| AC1015 | `[5162, 913, 3583, 541, 0, 82, 0]` | `SECTION` / `EOF` | 22 through 112,153 |
| AC1032 | `[1663, 896, 2147, 805, 1, 377, 9]` | `SECTION` / `EOF` | 22 through 42,235 |

All 16,714 groups and all 158,314 fixture bytes were accounted without a gap,
overlap, unsupported group code, value-family guess, or raw-byte mismatch.
This evidence supports physical streaming/framing only. Dialect inference,
section semantics, EOF conformance state, immutable Binary documents, CLI,
and writers are not claimed by M5.1b.
