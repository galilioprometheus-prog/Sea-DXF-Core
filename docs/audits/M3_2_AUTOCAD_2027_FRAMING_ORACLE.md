# M3.2 AutoCAD 2027 ASCII framing oracle

Status: completed 2026-07-27

This receipt records isolated behavioral evidence for physical ASCII cases
that Autodesk's public DXF reference does not fully specify. It is not a claim
that AutoCAD defines SeaCad semantics, and AutoCAD output is not used to prove
byte-identical preservation.

## Oracle identity and isolation

| Item | Receipt |
| --- | --- |
| Product | AutoCAD Core Console 2027 |
| File/product version | `26.0.60.0.0` |
| Executable SHA-256 | `fc6b59c29fea759d556083cf43d9de3657974ef0bf57962d49a19892f729fa77` |
| Script payload | `_.STATUS<CR><LF>` |
| Script SHA-256 | `0a813ebdcc75697143a0c87d87750b9a806fa780074e3420712724b091dfab87` |
| Locale | `en-US` |

Every input was a generated copy in a new temporary directory outside the
repository. The invocation used:

```text
accoreconsole /i <generated-copy> /s <reviewed-script> /l en-US
  /isolate SeaCadM32 <isolated-user-data> /readonly /safemode
```

`/isolate` selected a separate registry/user-data identity, `/readonly`
prevented input writes, and `/safemode` disabled executable code and drawing
directory autoload. No AutoCAD component is linked or distributed with SeaCad.
Committed evidence uses case IDs and SHA-256 rather than local paths.

An accepted case required process exit code `0` and successful execution of
`STATUS`. A rejected case returned `53`, emitted `Invalid or incomplete DXF
input -- drawing discarded`, and never ran `STATUS`.

## Synthetic baseline

The baseline is a minimal AC1032 document containing these logical pairs:

```text
0 SECTION
2 HEADER
9 $ACADVER
1 AC1032
0 ENDSEC
0 SECTION
2 ENTITIES
0 ENDSEC
0 EOF
```

Normal baseline group-code lines use conventional left padding (`"  0"`,
`"  2"`, and so on). Each case changes only the physical feature named below.
Inputs contain no user drawing data and are not committed to the source repo.

## Results

| Case ID | Bytes | SHA-256 | Exit | Result |
| --- | ---: | --- | ---: | --- |
| `crlf` | 120 | `86f50c6bc9f161b4acd947aac6ee5e21f8744573ff32d2325206ef3721a8e33c` | 0 | Accepted |
| `lf` | 102 | `e27c0d3940d5fee0968357e8e1c23c14fe8baedad2c8f34e8204533d7bb45d79` | 0 | Accepted |
| `cr` | 102 | `3ff24b47ddd436bf0b57575609074f7f9b3b4cb45247a0bbaba12a115900aed7` | 0 | Accepted |
| `mixed_eol` | 108 | `a783e4c242a78a50c6dc9542d2b52cb0320a65b59156b27247ce834f8c673b32` | 0 | Accepted |
| `no_final_terminator` | 118 | `6d31a1acc2bda704abbec72091a10346ae225579acda5445a9ed756203568ff9` | 0 | Accepted |
| `utf8_bom` | 123 | `2d0c86805c4426d15de2299276ef3dd8ee4ff326d551ac018a8f7df9cb018420` | 53 | Rejected at line 1 |
| `leading_blank_line` | 122 | `ca14bc9cd989a9693522b0fa42c891e27be7607644afc7e19cdaad41bd23c86b` | 53 | Rejected at line 1 |
| `single_blank_between_records` | 122 | `850494efbc7492f55d33a3ad09599c3f170c1f89bd2cbdc8c6572aaa582da8be` | 53 | Rejected at line 3 |
| `blank_pair_between_records` | 124 | `9b1f97d3cdfdb45452afd0676ffead9153ac30abb34fdc18e1061240eedb6b2c` | 53 | Rejected at line 3 |
| `tab_padded_code` | 120 | `1bc351dadf0469a513c2f55bdcfb6109e9b1c60ce9a7fd8d740cb10d9fed5ecd` | 0 | Accepted |
| `plus_signed_code` | 119 | `d17728e1c7a93abe7dcc630ad2ffcba8ed91dad19ebfe30827f0ec0cf3d76c5e` | 0 | Accepted |
| `zero_padded_code` | 120 | `3459879d052300f9c565809b81118af4b3c011f63d7b20a8c8d4b590e06ada04` | 0 | Accepted |
| `non_numeric_code` | 118 | `d672d628adeee05a26b4f7b697207b4af6bba445e69d63c8a909660f8c313b4e` | 53 | Rejected at line 1 |
| `empty_code` | 117 | `fe4afaf7faea009badad943a57e0b9ca9f94590bfe70a78703180875e7167cd3` | 53 | Rejected at line 1 |
| `missing_final_value` | 115 | `3c163afe9fb68c843e500af87c2436a4772d76473d7f364e399d02b234faee2b` | 53 | Rejected at line 18 |
| `missing_eof_pair` | 110 | `6fa03844a56fcdcf9dd8afc7ea80fdb01625ccfd9f363452d7bf6ab39f5285a3` | 53 | Rejected at line 17 |

All five accepted newline cases reported the same 120-object empty drawing.
Group-code tabs, an explicit plus sign, and leading zeroes also produced that
same inventory. This supports treating those spellings as strict framing, not
as recoveries.

The BOM result supports Strict rejection. SeaCad Compatible mode deliberately
permits exactly one UTF-8 BOM at absolute byte zero, emits `DXF-W0201`, and
keeps the original bytes. Blank lines and missing values remain fatal because
skipping or synthesizing them could shift or invent subsequent pairs.

Missing terminal EOF is recorded here but belongs to the M3.3 document
envelope. M3.2 neither accepts nor rejects a document as a whole.

## Normative references

- [Autodesk general DXF structure](https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-D939EA11-0CEC-4636-91A8-756640A031D3.htm)
- [Autodesk group codes in numerical order](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3F0380A5-1C15-464D-BC66-2C5F094BCFB9.htm)
- [Autodesk SAFEMODE](https://help.autodesk.com/cloudhelp/2026/ENU/AutoCAD-Core/files/GUID-23D32BD8-2F94-4463-9618-055CA798D465.htm)
