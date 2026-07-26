# M3 ASCII framing contract

Status: M3.2 group-code/value framing complete

M3.1 establishes the byte-preserving physical layer. M3.2 adds bounded ASCII
group framing and freezes the first Strict/Compatible recovery rule. It still
does not claim that a framed stream is a valid DXF document or provide a
Verbatim writer.

## Normative basis

The Autodesk DXF reference defines an ASCII DXF group as a group-code line
followed immediately by a value line, with each item on its own line:

- [General DXF file structure](https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-D939EA11-0CEC-4636-91A8-756640A031D3.htm)
- [Reading group codes](https://help.autodesk.com/cloudhelp/2015/ENU/AutoCAD-DXF/files/GUID-89CB823D-614D-4D1E-8204-568EC72DF869.htm)
- [DXF group codes in numerical order](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3F0380A5-1C15-464D-BC66-2C5F094BCFB9.htm)

Autodesk specifies the binary representation by its exact 22-byte sentinel:

- [Binary DXF format](https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-FC1C3C69-DBC2-49E4-893A-000D6538C0FE.htm)

The Autodesk pages do not define every physical newline and BOM acceptance
case. Those cases were measured with AutoCAD Core Console 2027; the isolated
receipt is in `docs/audits/M3_2_AUTOCAD_2027_FRAMING_ORACLE.md`.

## M3.1 physical representation probe

`DXF_BINARY_SENTINEL` is exactly
`AutoCAD Binary DXF<CR><LF><SUB><NUL>`. An exact match classifies the source as
Binary. An empty source, a truncated sentinel prefix, or a NUL byte in the
inspected prefix is Unknown. Every other non-empty prefix is only an
AsciiCandidate.

AsciiCandidate is deliberately not a validity result. Arbitrary text, a BOM,
non-ASCII bytes, malformed group codes, and incomplete group pairs may reach
the ASCII framing layer and be rejected or diagnosed later. The probe reads no
more than 22 bytes and rechecks the selected source-byte limit before reading.

## M3.1 lossless physical lines

`DxfAsciiLineCursor` exposes each physical line as borrowed raw bytes and
`DxfAsciiLineMetadata`. Metadata contains a zero-based line index, half-open
content/terminator/full byte spans, and an explicit `CrLf`, `Lf`, `Cr`, or
`None` ending.

Content excludes its terminator. `None` represents a final unterminated line;
there is no synthetic empty line after a terminal newline. Framing performs no
trimming, decoding, Unicode conversion, BOM removal, or NUL filtering. Debug
output reports byte counts and spans rather than payload bytes.

The cursor uses one fixed 8 KiB read buffer and one reusable allocation for the
current line. It accepts legal partial reads and CRLF split across read
boundaries. Source/value limits, checked offsets, allocation failure,
premature EOF, impossible source behavior, and cancellation use the stable M2
error model. Callers discard a cursor after any error.

## M3.2 lossless ASCII groups

`DxfAsciiGroupCursor` consumes exactly two physical lines per successful group:
the group-code line followed by its value line. It exposes:

- a zero-based group occurrence;
- a validated `DxfGroupCode`;
- exact raw group-code and value bytes;
- metadata for both physical lines; and
- the half-open span of the complete pair.

Raw bytes remain authoritative. Parsing a numeric code does not normalize the
stored code line or decode the value. Custom and unknown groups inside the
normative numeric domain remain frameable for later semantic layers.

The group-code grammar is deliberately byte-based:

- leading and trailing ASCII space or horizontal tab are accepted;
- one optional `+` or `-` sign is accepted;
- at least one ASCII decimal digit is required;
- internal whitespace and every non-ASCII byte are rejected; and
- the parsed value must be in Autodesk's `-5..=1071` group-code domain.

No integer overflow, Unicode whitespace normalization, locale-dependent
number parsing, or semantic inference is permitted.

## Strict and Compatible policy

The isolated AutoCAD 2027 matrix produced this policy:

| Physical case | Strict | Compatible | Reason |
| --- | --- | --- | --- |
| CRLF, LF, CR, or mixed endings | Accept | Accept | AutoCAD accepted all four |
| Final value without terminator | Accept | Accept | Complete pair; AutoCAD accepted it |
| Space/tab padding, `+`, leading zeroes | Accept | Accept | Valid integer spellings accepted by AutoCAD |
| UTF-8 BOM at absolute byte 0 | Reject | Accept with `DXF-W0201` | Explicit compatibility recovery; raw BOM remains preserved |
| BOM anywhere else | Reject | Reject | No global stripping or resynchronization |
| Blank group-code line | Reject | Reject | AutoCAD rejected it; skipping could shift every later pair |
| Non-numeric, overflow, or out-of-domain code | Reject | Reject | No semantic guessing |
| Group-code line without a value line | Reject | Reject | A value cannot be synthesized safely |

AutoCAD itself rejects the BOM case. SeaCad Compatible mode accepts only a
single BOM at absolute byte zero because BOM recovery was explicitly approved
for Compatible framing. The recovery is observable, byte-anchored, and never
changes raw source bytes. Strict mirrors the oracle rejection.

Missing terminal `0/EOF` is not a group-pair error when all physical lines
still form complete pairs. AutoCAD rejected the synthetic missing-EOF document;
SeaCad will enforce the document envelope at M3.3 rather than smuggle document
semantics into this cursor.

## Stable M3.2 codes

Fatal errors:

| Code | Meaning |
| --- | --- |
| `DXF-E0201` | Invalid ASCII group-code bytes or numeric domain |
| `DXF-E0202` | Group-code line has no following value line |

Non-fatal diagnostics:

| Code | Meaning |
| --- | --- |
| `DXF-W0201` | Compatible mode ignored a UTF-8 BOM before the first group code |

Errors and diagnostics contain byte spans, never source payloads or paths.
Custom Debug implementations also omit raw group-code and value content.

## Resource behavior

The selected record limit is checked before consuming the first byte of an
excess group and reports observed count `limit + 1`. Source and value limits
remain enforced by the physical-line cursor. Cancellation remains cooperative
before and after potentially blocking reads.

Retained diagnostics never exceed the selected profile limit. On the first
excess diagnostic, the final retained entry becomes `DXF-W0001`; subsequent
diagnostics are suppressed. M3.2 currently has only one possible recovery per
source, but the cap behavior is implemented and tested for future framing
rules.

## Dependency and support boundary

M3.2 adds no dependency, imports no legacy code or fixture bytes, and does not
change the support matrix. It validates physical group pairs only. M3.3 is the
next boundary: terminal EOF/document envelope and an immutable raw ASCII
document. Verbatim round-trip and initial CLI commands follow in later M3
checkpoints.
