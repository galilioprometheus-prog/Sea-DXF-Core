# M4.3b1 exact codepage registry and decoder contract

Status: frozen 2026-07-27

This contract adds a reviewed legacy-codepage registry and a bounded Unicode
decoder. Raw DXF bytes remain authoritative; decoded UTF-8 is a derived view.

## Exact registry

`DxfLegacyCodePage::from_dwg_codepage_token()` recognizes only these exact,
case-sensitive byte strings:

| `$DWGCODEPAGE` bytes | Windows identifier | Decoder mapping |
| --- | ---: | --- |
| `ANSI_874` | 874 | WHATWG `windows-874` |
| `ANSI_932` | 932 | WHATWG `Shift_JIS` |
| `ANSI_936` | 936 | WHATWG `GBK` |
| `ANSI_949` | 949 | WHATWG `EUC-KR` |
| `ANSI_950` | 950 | WHATWG `Big5` |
| `ANSI_1250` | 1250 | WHATWG `windows-1250` |
| `ANSI_1251` | 1251 | WHATWG `windows-1251` |
| `ANSI_1252` | 1252 | WHATWG `windows-1252` |
| `ANSI_1253` | 1253 | WHATWG `windows-1253` |
| `ANSI_1254` | 1254 | WHATWG `windows-1254` |
| `ANSI_1255` | 1255 | WHATWG `windows-1255` |
| `ANSI_1256` | 1256 | WHATWG `windows-1256` |
| `ANSI_1257` | 1257 | WHATWG `windows-1257` |
| `ANSI_1258` | 1258 | WHATWG `windows-1258` |

The numeric identifiers and ANSI-family boundary follow Microsoft's Windows
codepage registry. The byte-to-Unicode algorithms are the named WHATWG
compatibility encodings implemented by the pinned `encoding_rs 0.8.35`.
SeaCad does not call the current operating system codepage API.

Whitespace, case variants, web labels, `ANSI_1361`, DOS/OEM declarations, and
empty or unknown tokens return `None`. There is no alias normalization and no
fallback to windows-1252, UTF-8, or the host locale.

## Replacement-free decode

`DxfTextDecoder` has two explicit sources:

- `Utf8` for a version-backed AC1021-or-later storage decision;
- `Legacy(DxfLegacyCodePage)` after exact legacy-token resolution.

`decode_complete_to_utf8_without_replacement(source, destination)` processes
one complete DXF value into caller-owned bytes. It does not allocate, strip a
BOM, insert U+FFFD, skip malformed bytes, or continue after a decoding error.
It returns:

- `Complete` when all source bytes were consumed;
- `OutputFull` when the destination cannot accept the next complete scalar;
- `Malformed { malformed_len, bytes_after_malformed }` at invalid input.

The receipt also reports source bytes read and destination bytes written. Only
`destination[..written]` is defined output. A non-empty input and destination
smaller than four bytes returns `OutputFull` with zero bytes read or written;
an empty input completes even with an empty destination.

The one-shot API does not return decoder state. After `OutputFull`, the caller
must retry the entire source value with a larger destination; `read` is an
evidence count and not a cursor for resuming with a fresh decoder.

Each invocation constructs a fresh decoder and passes `last = true`, so state
cannot leak between DXF group values. Streaming across physical chunks belongs
in the future source-anchored Unicode view, not in this one-shot primitive.

## Evidence and support boundary

Tests cover registry round-trip for all 15 entries, one reviewed decode vector
per entry, UTF-8 validation, incomplete multibyte input, destination
exhaustion, and the absence of replacement output. AutoCAD 2027 separately
confirmed representative bytes for 874, 932, 936, 949, 950, 1252, and 1258.

This checkpoint does not claim every historical DXF codepage. In particular,
DOS/OEM pages and Johab remain explicit unsupported results until a dependency
or independent implementation has its own provenance, license, conformance,
and hostile-input evidence.

## Deliberate M4.3b1 boundaries

- No automatic resolution from `DxfTextEncodingReport` and its source span.
- No allocating `String` convenience API or semantic LRU cache.
- No CIF, MIF, `\U+xxxx`, surrogate, MTEXT, or formatting escape processing.
- No Binary DXF text integration.
- No encode path and no canonical writer integration.
- No new support claim for unknown or proprietary payload bytes.
