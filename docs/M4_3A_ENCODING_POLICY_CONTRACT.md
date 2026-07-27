# M4.3a text-encoding policy contract

Status: frozen 2026-07-27

This contract covers source-anchored `$DWGCODEPAGE` discovery and selection of
a text-storage policy. It deliberately does not expose decoded Unicode text.

## Public model

`DxfAsciiRawDocument::text_encoding_report()` returns one immutable
`DxfTextEncodingReport` tied to the same `DxfSourceId` as the raw document.

The declaration state is:

- `Absent`: no exact HEADER `$DWGCODEPAGE`;
- `Declared`: exactly one occurrence followed by non-empty group code 3;
- `Invalid`: exactly one occurrence followed by an empty value, wrong group
  code, or no value;
- `Ambiguous`: more than one exact occurrence, even when values agree.

The effective policy is:

- `Utf8(version)` for exactly one supported AC1021-or-later `$ACADVER`;
- `LegacyDeclared(version)` for exactly one supported pre-AC1021 `$ACADVER`
  plus exactly one structurally valid codepage declaration;
- `Indeterminate` for every other evidence combination.

`LegacyDeclared` means only that a raw declaration exists. It does not mean its
token has been mapped to a decoder or that all bytes are valid.

## Provenance and bounds

An occurrence stores group occurrence numbers and content byte spans for the
exact variable and value candidate. Callers recover the original codepage
token through `read_span`; the report never copies it.

Discovery observes borrowed groups during the existing framing/SHA-256 pass.
It performs no second scan, random I/O, decoding, or payload allocation. It
retains only the first occurrence, the second conflicting occurrence, and the
total count, so hostile duplicate input adds constant report memory.

## Precedence and no-fallback rules

The documented version format outranks `$DWGCODEPAGE` for AC1021 and later.
Malformed codepage metadata remains diagnostic, but cannot change a modern
file from UTF-8 to a legacy encoding.

For legacy versions, no declaration means no decoding policy. SeaCad never
uses the current Windows ANSI page, a macOS/Linux locale, UTF-8 guessing,
windows-1252, or any other implicit fallback.

Compatible framing recovery does not relax exact HEADER-variable semantics.
Raw framing conformance and text-policy diagnostics remain separate.

## Stable diagnostics

| Code | Severity | Meaning |
| --- | --- | --- |
| `DXF-E0420` | Error | supported pre-2007 file lacks exact `$DWGCODEPAGE` |
| `DXF-E0421` | Error | declaration value is missing, empty, or not group 3 |
| `DXF-E0422` | Error | multiple exact declarations are ambiguous |

Missing legacy declarations are anchored to the first exact HEADER name span.
Malformed occurrences are anchored to their value candidate or variable span.
These diagnostics do not make immutable raw open or Verbatim output fail.

## Deliberate M4.3a boundaries

- No codepage-token registry or decoder selection.
- No UTF-8 validation of arbitrary string groups.
- No CIF, MIF, `\U+xxxx`, surrogate, or MTEXT escape processing.
- No lossy replacement character API.
- No Binary DXF integration.
- No CLI/JSON v1 expansion.
- No new dependency.
