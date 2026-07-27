# M5.2a AutoCAD Binary raw-document oracle

Status: completed 2026-07-27

No external parser source was inspected, copied, translated, or ported. The
checker depends only on the current local `seacad-dxf-core` path and Rust's
standard build graph.

## Inputs

The three files were created by the isolated AutoCAD Core Console 2027 process
recorded in the M5.1a audit. They remain outside the repository.

| Dialect | Bytes | SHA-256 |
| --- | ---: | --- |
| AC1009/R12 | 3,926 | `76f9f56bf9a4395c7bc324cf831c5e473cad68a5ee0a442975cc7647c08f5a04` |
| AC1015/AutoCAD 2000 | 112,153 | `cd059a69f0c238f28807b2079b53aef6d04c68d0b486416ac2bede3df59de2c3` |
| AC1032/AutoCAD 2018 | 42,235 | `4b5584619ef9ba4665f46caf268891b7dd54ad527955348325484d6f7c80b0ef` |

AutoCAD executable version `26.0.60.0.0` and SHA-256
`fc6b59c29fea759d556083cf43d9de3657974ef0bf57962d49a19892f729fa77`
are inherited from the committed M5.1a generation receipt.

## Independent checker assertions

For each source, the checker calls `DxfBinaryRawDocument::open` without an
encoding hint and requires:

- exact expected physical encoding and supported `$ACADVER` state;
- expected group count and whole-source SHA-256 `SourceId`;
- contiguous group full spans from byte 22 through physical source end;
- exact first `0/SECTION` and last `0/EOF` payloads read back through the
  source-backed document.

Result:

```text
AC1009 version=AC1009 encoding=OneByteWithExtendedDataEscape groups=535 bytes=3926 source_id=76f9f56bf9a4395c7bc324cf831c5e473cad68a5ee0a442975cc7647c08f5a04 first=SECTION last=EOF
AC1015 version=AC1015 encoding=TwoByteLittleEndian groups=10281 bytes=112153 source_id=cd059a69f0c238f28807b2079b53aef6d04c68d0b486416ac2bede3df59de2c3 first=SECTION last=EOF
AC1032 version=AC1032 encoding=TwoByteLittleEndian groups=5898 bytes=42235 source_id=4b5584619ef9ba4665f46caf268891b7dd54ad527955348325484d6f7c80b0ef first=SECTION last=EOF
total_groups=16714 total_bytes=158314 status=ok
```

## External checker receipt

| Artifact | SHA-256 |
| --- | --- |
| `Cargo.toml` | `988a42e177dfefc1f5a363ebc4a710b2daacd20cb94206fd1c45d5481d7d2c26` |
| `Cargo.lock` | `1b7fb6b2513bbdc308bf37dd521b2a0c9aa8f5e6aa4755a706362ae13bdf9ca2` |
| `src/main.rs` | `f0e88e11a31f2148e220fbced904611bb25b68393a6b07f79398c9e93defd308` |
| `oracle-results.txt` | `b262af86569951ad2d1b2d05a26326427a24de1afdc5fd8031ea60bd5376b5d8` |
| checker executable | `984553d9c15d4e31de153153c8a565eb7b3e3227ddb22f33a5c13caf6ba7f259` |

Command: `cargo run --quiet` from the external checker directory. The checker,
build output, logs, and CAD fixtures are not committed.

This proves automatic opening/dialect agreement and physical raw accounting
for these three oracle dialects. It does not prove M5.2b section/EOF
conformance, M5.2c unchanged replay/CLI, semantic decoding, edits, or writers.
