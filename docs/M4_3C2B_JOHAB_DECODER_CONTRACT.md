# M4.3c2b exact CP1361/Johab decoder contract

Status: frozen 2026-07-27

This contract completes replacement-free Windows CP1361/Johab decoding for
both exact `$DWGCODEPAGE` storage and MIF selector 4. Raw DXF bytes and all
source-anchored receipts remain authoritative.

## Registry and decode scope

`DxfLegacyCodePage::from_dwg_codepage_token()` now recognizes the exact,
case-sensitive token `ANSI_1361` as `Windows1361`. Its numeric identifier is
1361 and its canonical token is `ANSI_1361`. Aliases, whitespace, case
variants, and host-locale fallback remain forbidden.

MIF selector 4 selects the same decoder. All five evidence-backed MIF
selectors therefore decode without replacement.

## Frozen strict mapping

The decoder table derives from Microsoft's CP1361 decode records distributed
by Unicode as `bestfit1361.txt`. The source file contains 17,395 decode
records. Eleven single-byte records are explicitly labelled
`Undefined -> EUDC`; current Windows NLS rejects those bytes with
`MB_ERR_INVALID_CHARS`, so SeaCad excludes them.

The strict mapping contains:

- 132 single-byte mappings;
- 17,252 two-byte mappings;
- 17,384 mappings in total.

The canonical sorted `(code little-endian, Unicode little-endian)` record
hash is
`5f038ab2832fc3b597115b3138480142d5dccc97a39edcf61567b0dff8385a84`.
It matches the exhaustive Windows NLS oracle over all 65,536 MIF payloads.

## Runtime representation

`johab_decode_le.bin` is a fixed 65,536-slot table. Each little-endian `u16`
stores the Unicode BMP scalar plus one; zero means invalid. This preserves
U+0000 while keeping invalid lookup explicit.

- size: 131,072 bytes;
- SHA-256:
  `d04a1a13d5f4706df6fa46394cda98a817570e774d601acd042e0fa57249f7ea`;
- lookup: constant time;
- runtime allocation: none;
- platform API, locale, FFI, build script, and initialization: none.

The 128 KiB table is intentionally preferred over a smaller undocumented
algorithm: it makes every accepted and rejected code auditable and has
negligible cost relative to CAD source limits.

## Streaming and malformed input

The source-backed decoder carries at most one pending lead byte across the
existing 4 KiB input chunks. It copies output only on complete Unicode scalar
boundaries.

- A valid single byte consumes one byte.
- A valid pair consumes lead and trail together.
- A lead at a non-final chunk boundary is retained.
- An incomplete final lead, undefined byte, or invalid pair returns typed
  `Malformed`.
- `OutputFull` never consumes the scalar that does not fit.
- No U+FFFD, default character, EUDC substitution, byte skipping,
  normalization, or retry with another codepage is allowed.

For MIF, a payload must still decode to exactly one scalar. A payload that is
merely two independently valid single bytes remains `MifInvalidCode`.

## Provenance and license

The source data is covered by Unicode License v3. The source text is not
vendored; SeaCad commits only the frozen derived lookup table, its generator
and exhaustive hashes. The required copyright, permission, and disclaimer
appear in `THIRD_PARTY_NOTICES.md`.

## Deliberate M4.3c2b boundaries

- No CP1361 encode path or best-fit Unicode-to-Johab conversion.
- No MTEXT formatting, percent controls, or entity-specific text semantics.
- No allocating convenience API, semantic cache, Binary DXF, or writer path.
- No CLI or JSON v1 expansion.
