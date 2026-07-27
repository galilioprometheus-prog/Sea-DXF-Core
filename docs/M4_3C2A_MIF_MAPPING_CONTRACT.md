# M4.3c2a evidence-backed MIF mapping contract

Status: frozen 2026-07-27

This contract extends the bounded M4.3c1 escape layer with independently
verified MIF selector meanings. Raw DXF bytes and the source-anchored storage
decode receipt remain authoritative.

## Recognized controls

SeaCad recognizes `\M+nxxxx` and `\m+nxxxx` when `n` is exactly `1` through
`5` and every `x` is an ASCII hexadecimal digit. AutoCAD 2027 and the reviewed
Autodesk-authored historical header both accept upper- and lowercase `M`.

Selectors outside `1..=5`, a missing selector, and a non-decimal selector are
not MIF controls. They remain literal decoded UTF-8, matching the AutoCAD 2027
oracle. Once a valid selector is present, a short token or non-hexadecimal
payload is a typed malformed control.

## Selector registry

| Selector | Windows codepage | SeaCad M4.3c2a result |
| ---: | ---: | --- |
| 1 | 932, Shift-JIS | decoded without replacement |
| 2 | 950, Big5 | decoded without replacement |
| 3 | 949, Wansung | decoded without replacement |
| 4 | 1361, Johab | typed `UnsupportedMifCodePage` |
| 5 | 936, Simplified Chinese | decoded without replacement |

The mapping is exposed as `DxfMifCodePage`. Each value returns its exact
selector and Windows numeric codepage identifier.

## Payload and decoding

The four hexadecimal digits are one 16-bit payload. If its high byte is zero,
only the low byte is decoded. Otherwise the high byte precedes the low byte.
Selectors 1, 2, 3, and 5 use the already reviewed replacement-free
`encoding_rs` decoder for CP932, CP950, CP949, and CP936.

An invalid byte sequence terminates with typed `MifInvalidCode`. No U+FFFD,
fallback codepage, normalization, or lossy conversion is allowed. A two-byte
payload must decode to exactly one Unicode scalar; two independent single-byte
characters are invalid and remain fail-closed. Selector 4 returns:

`UnsupportedMifCodePage { source_offset, code_page: Windows1361, code }`

The token is not consumed or copied. M4.3c2a does not derive Johab behavior
from an obsolete mapping file or a Windows-only runtime API.

## Bounded output

MIF decoding uses fixed four-byte scalar storage and caller-owned output. A
decoded scalar is copied atomically. If it cannot fit, `OutputFull` reports
the input offset before the MIF token and the caller retries the complete
source with a larger destination.

All M4.3c1 CIF behavior and provenance boundaries remain unchanged. In
particular, lowercase `\u+` remains literal because CIF case expansion is not
part of this checkpoint.

## Deliberate M4.3c2a boundaries

- No CP1361/Johab Unicode decoder; that is M4.3c2b.
- No MTEXT formatting, percent controls, or entity-specific text semantics.
- No allocating convenience API, semantic cache, Binary DXF, or writer path.
- No CLI or JSON v1 expansion.
