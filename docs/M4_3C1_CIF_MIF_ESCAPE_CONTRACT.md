# M4.3c1 CIF decode and MIF recognition contract

Status: frozen 2026-07-27

This contract adds a bounded control-sequence layer after replacement-free
DXF byte-to-Unicode decoding. Raw source bytes remain authoritative. The layer
does not classify entity semantics or interpret MTEXT formatting.

## Pipeline boundary

`decode_dxf_text_escapes_to_utf8_without_replacement()` accepts a valid Rust
`str` produced by the M4.3b decoder and caller-owned UTF-8 destination storage.
It does not accept raw DXF bytes, select a codepage, allocate a `String`, or
change the source-anchored M4.3b2 receipt.

The two layers compose explicitly:

1. select one group occurrence and decode its storage bytes;
2. retain that receipt's `SourceId`, occurrence, and raw value span;
3. pass only the defined decoded UTF-8 prefix to this escape layer.

Escape offsets and `read` counts are relative to the decoded UTF-8 input, not
absolute source-file bytes. Raw provenance remains in the first-layer receipt.

## CIF controls

An exact uppercase prefix `\U+` followed by exactly four hexadecimal digits is
one CIF UTF-16 code unit. Hexadecimal digits `A-F` and `a-f` have identical
numeric meaning. Non-surrogate values become their corresponding Unicode
scalar.

An adjacent high-surrogate CIF token followed immediately by a low-surrogate
CIF token becomes one supplementary Unicode scalar. A high surrogate without
that immediate partner, a standalone low surrogate, an incomplete token, or a
non-hexadecimal digit terminates with typed `Malformed`. No replacement scalar
is inserted and no bytes after the bad control are interpreted.

Only the exact uppercase `\U+` prefix belongs to this layer. Lowercase `\u+`
and unrelated backslash sequences remain literal. Backslash quoting and MTEXT
format controls are deliberately not inferred here.

## MIF controls

An exact uppercase `\M+`, one ASCII decimal selector digit, and four
hexadecimal payload digits is recognized as the documented MIF shape. A valid
token terminates with typed `UnsupportedMif { source_offset, selector, code }`.
It is not copied, normalized, or mapped to Unicode.

This fail-closed boundary is intentional. Autodesk's public references name
the `M+nxxxx`/`M+nxxyy` form but do not publish a complete selector-to-encoding
mapping in the reviewed DXF documentation. SeaCad will not infer that mapping
from another parser. Truncated, non-decimal-selector, and non-hex MIF controls
terminate with their own typed malformed issue.

## Bounded output and retry

The implementation uses fixed four-byte scalar storage and does not allocate.
Plain UTF-8 and decoded CIF output are copied only as complete Unicode scalars.
The result reports:

- `Complete` with all decoded UTF-8 input consumed;
- `OutputFull` before the first scalar that cannot fit;
- `Malformed { source_offset, issue }` at a recognized bad control;
- `UnsupportedMif { source_offset, selector, code }` at a valid MIF token.

Only `destination[..written]` is defined output. After `OutputFull`, the caller
retries the complete decoded source with a larger destination. `read` is
evidence, not a resume cursor.

## Deliberate M4.3c1 boundaries

- No MIF selector mapping or MIF-to-Unicode conversion.
- No MTEXT, stacked text, percent controls, or formatting tokenization.
- No group-code text classification or entity-specific semantics.
- No allocating convenience API, normalization, interning, or semantic cache.
- No Binary DXF integration, encode path, or canonical writer.
- No CLI or JSON v1 expansion.
