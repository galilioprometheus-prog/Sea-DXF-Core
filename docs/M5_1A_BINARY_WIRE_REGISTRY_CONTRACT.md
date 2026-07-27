# M5.1a Binary DXF wire-registry contract

Status: frozen 2026-07-27

This checkpoint freezes and implements the two Binary DXF group-code wire
encodings and the complete documented group-code-to-value-family registry. It
does not yet consume a source, frame value bytes, validate the document
envelope, build a raw document/index, expose Binary DXF through the CLI, or
write Binary DXF.

## Explicit group-code encoding

`decode_binary_group_code` receives caller-owned bytes, their absolute source
offset, and one explicit encoding:

- `OneByteWithExtendedDataEscape`: the pre-R13 form. Codes 0 through 254 are
  one byte. Byte 255 is followed by the true 16-bit little-endian extended-data
  code, which must be in 1000 through 1071.
- `TwoByteLittleEndian`: the R13-and-later form. Every group code is a signed
  16-bit little-endian value and must fit SeaCad's -5 through 1071 domain.

The successful header returns the typed group code and exact number of wire
bytes consumed. Extra input bytes remain untouched as the future value. A
truncated or invalid header returns a stable typed error with an exact
half-open source span.

Encoding detection is deliberately deferred. M5.1b will give its streaming
cursor one explicit encoding. A later document layer can inspect the canonical
opening records, read `$ACADVER`, and verify encoding/dialect agreement without
making the low-level decoder guess.

## Value-family registry

The wire family is determined only from the group code. Reserved gaps return
`None` because their value length cannot be inferred safely.

| Group codes | Binary family |
| --- | --- |
| 0-9, 100-102, 105 | NUL-terminated string |
| 10-59, 110-149, 210-239 | IEEE-754 f64, 8-byte little-endian |
| 60-79, 170-179, 270-289 | i16, 2-byte little-endian |
| 90-99 | i32, 4-byte little-endian |
| 160-169 | i64, 8-byte little-endian |
| 290-299 | Boolean wire byte |
| 300-309 | NUL-terminated string |
| 310-319 | u8 length followed by that many payload bytes |
| 320-369, 390-399, 410-419, 430-439 | NUL-terminated string/handle |
| 370-389, 400-409 | i16, 2-byte little-endian |
| 420-429, 440-459 | i32, 4-byte little-endian |
| 460-469 | IEEE-754 f64, 8-byte little-endian |
| 470-481, 999, 1000-1003, 1005-1009 | NUL-terminated string/handle |
| 1004 | u8 length followed by that many payload bytes |
| 1010-1059 | IEEE-754 f64, 8-byte little-endian |
| 1060-1070 | i16, 2-byte little-endian |
| 1071 | i32, 4-byte little-endian |

Group 999 remains classifiable as a string so a future cursor can retain a
malformed file, but Autodesk states it is not used in conforming Binary DXF.
XDATA group 1004 is a binary chunk even though surrounding 1000-series codes
are strings. Its documented 127-byte maximum is semantic validation; the wire
length prefix itself can express 0 through 255.

## Safety and deferred boundary

The decoder is allocation-free, constant-time, locale-independent, and never
reads beyond the supplied slice. Span arithmetic is checked. It does not
decode numeric payloads or strings and introduces no dependency.

M5.1b will add the 8 KiB bounded source cursor, exact raw/value/payload spans,
record/value limits, cancellation, truncated-value handling, and linear-scan
tests. Binary raw document/index, EOF conformance, unchanged replay, CLI,
semantic validation, edit transactions, conversions, and writers remain in
their later milestones.
