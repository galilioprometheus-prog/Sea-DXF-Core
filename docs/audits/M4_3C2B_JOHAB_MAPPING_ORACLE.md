# M4.3c2b CP1361/Johab mapping and oracle audit

Status: completed 2026-07-27

This audit proves the frozen CP1361 decode map from licensed primary data,
current Windows strict behavior, AutoCAD 2027 semantics, and exhaustive
in-repository tests. No external parser implementation was copied or ported.

## Microsoft/Unicode source data

Unicode hosts Microsoft's Windows best-fit data at:

<https://www.unicode.org/Public/MAPPINGS/VENDORS/MICSFT/WindowsBestFit/bestfit1361.txt>

Reviewed source artifact:

- byte length: 1,223,827;
- SHA-256:
  `7dcda2d5d2cfc5ddf43757d589a0106e020ef0d9b84de47f08e18297ae0fe1ec`;
- `CODEPAGE`: 1361;
- `CPINFO`: double-byte page, default `?`;
- decode records: 143 single and 17,252 double.

The adjacent Microsoft/Unicode `readme.txt` defines `MBTABLE` and
`DBCSTABLE` as codepage-to-Unicode mappings, identifies high-byte/low-byte
order, and distinguishes the Unicode-to-codepage `WCTABLE` best-fit section.
SeaCad consumes only decode records before `WCTABLE`.

## Exhaustive Windows NLS comparison

`tools/audit-johab-table.ps1` calls
`MultiByteToWideChar(1361, MB_ERR_INVALID_CHARS, ...)` for every `u16`
payload. Values `00xx` are presented as one byte; all others are presented
high byte then low byte. Only results containing exactly one UTF-16 code unit
enter the strict MIF map.

| Evidence | Count/hash |
| --- | --- |
| source decode records | 17,395 |
| explicitly undefined single-byte EUDC records | 11 |
| strict source records | 17,384 |
| Windows NLS strict records | 17,384 |
| extra Windows records | 0 |
| differing Unicode values | 0 |
| two-scalar byte sequences excluded from MIF | 17,292 |
| strict source canonical SHA-256 | `5f038ab2832fc3b597115b3138480142d5dccc97a39edcf61567b0dff8385a84` |
| Windows NLS canonical SHA-256 | `5f038ab2832fc3b597115b3138480142d5dccc97a39edcf61567b0dff8385a84` |

The eleven excluded source records are single bytes D4-D7, DF, and FA-FF.
The source comments label each `Undefined -> EUDC`; Windows strict rejects
them. Valid double-byte EUDC mappings, such as D8 31 to U+E000, remain intact.

## Generated lookup receipt

The audit tool writes the direct plus-one little-endian table and refuses any
source, canonical map, NLS result, count, or output hash drift.

- output bytes: 131,072;
- nonzero slots: 17,384;
- SHA-256:
  `d04a1a13d5f4706df6fa46394cda98a817570e774d601acd042e0fa57249f7ea`.

A second generation run produced a byte-identical artifact. The repository
test rehashes the table, reconstructs all canonical records, validates every
stored scalar, and verifies the same counts and canonical hash on all
platforms.

## AutoCAD 2027 oracles

Oracle executable:

- product/file version: `26.0.60.0.0`;
- SHA-256:
  `fc6b59c29fea759d556083cf43d9de3657974ef0bf57962d49a19892f729fa77`.

Each fixture ran outside the repository with `/readonly`, `/safemode`, and an
isolated registry/data profile. The complete `LIST` result contained no
`ERROR` line. This Core Console build returns process code 1 when the script
ends after producing the result, so semantic text and hashes are the evidence.

### MIF selector 4 families

- fixture SHA-256:
  `a12d7912f72b820d949e7cc1623b02139553bc1a13d8eaa5546e6d122d4d2172`;
- cases: ASCII, Jamo, Hangul start/mid, EUDC, Kana, and Hanja;
- extracted scalar sequence: `AA|ᆨ|가|한|<U+E000>|ア|伽|禍Z`, where
  `<U+E000>` denotes the single private-use scalar rather than literal label
  characters;
- exact UTF-8 result SHA-256:
  `ba8027f27edb11c7d24211a9600daac5338682ea93e0df09643c530ddfcb5cbb`.

### Raw `ANSI_1361` document storage

- fixture bytes: 165;
- fixture SHA-256:
  `2f33c8769cf1143a258764f41690a082a2e2d9c699b4f4932d5a3b4a084210ad`;
- raw text bytes include 88 61, D0 65, DE 32, and E0 31;
- extracted text: `A가|한|ア|伽Z`;
- exact UTF-8 result SHA-256:
  `b909b0fe7efa01a5cfe6a640390d289f84edcaecacaee9f38b0c4375d2ed0912`.

Fixtures, logs, profiles, and the downloaded source data remain outside the
repository. Only hashes, observations, the audited generator, and the derived
table are retained.
