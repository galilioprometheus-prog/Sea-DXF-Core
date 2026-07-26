# M3 ASCII framing contract

Status: M3.4 verified Verbatim writer complete

M3.1 establishes the byte-preserving physical layer. M3.2 adds bounded ASCII
group framing and freezes the first Strict/Compatible recovery rule. M3.3 adds
the terminal EOF envelope, a source-backed immutable raw document, and a
one-pass SHA-256 identity. It still does not validate sections or `$ACADVER`
and therefore does not claim versioned semantic support. M3.4 adds a
create-new-only Verbatim writer with source preconditions and output
verification.

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
physical receipt is in
`docs/audits/M3_2_AUTOCAD_2027_FRAMING_ORACLE.md`, and the EOF-envelope receipt
is in `docs/audits/M3_3_AUTOCAD_2027_EOF_ORACLE.md`.

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

M3.3 applies this document-envelope policy after group framing:

| Envelope case | Strict | Compatible |
| --- | --- | --- |
| Exact group code `0` and exact value bytes `EOF` | Accept | Accept |
| EOF value without a final line terminator | Accept | Accept |
| Space/tab around the EOF value | Reject as missing EOF | Recover with `DXF-W0202` |
| Lowercase or mixed-case EOF value | Not an EOF marker | Not an EOF marker |
| Completely paired stream without EOF | Reject with `DXF-E0203` | Open recovered with `DXF-W0203` |
| Any source bytes after the first recognized EOF group | Reject with `DXF-E0204` | Preserve as opaque tail with `DXF-W0204` |

Compatible whitespace and missing-EOF recovery are explicit SeaCad recovery
rules. AutoCAD rejected those inputs. Compatible trailing-tail recovery is
supported by AutoCAD evidence: duplicate EOF, a group after EOF, blank/space
lines, and SUB bytes were ignored after the first EOF. SeaCad does not discard
that tail; it records one exact source span and excludes it from group or
semantic interpretation. A lowercase `eof` may therefore occur in a recovered
missing-EOF document, but it is never reclassified as the EOF marker.

The envelope deliberately does not require HEADER, section ordering, or a
recognized version. AutoCAD's rejection of a file containing only `0/EOF` is
a document-structure result owned by M4, not a reason to mix semantics into
the M3 framing layer.

## Immutable raw ASCII document

`DxfAsciiRawDocument` owns only immutable framing metadata and keeps a borrowed
reference to the bounded `DxfByteSource`. Every `DxfAsciiRawGroup` carries its
occurrence, validated code, exact content/full spans, and both line endings.
Payload bytes remain in the source and are available through bounded
`read_span`; Debug output never includes payload or paths.

Opening computes `DxfSourceId` over every observed byte while the line cursor
parses it. Bytes already read ahead by the fixed 8 KiB parser buffer are not
read again. If Compatible stops at EOF before physical source end, hashing
continues forward in fixed 64 KiB chunks. Thus successful open reads every
source byte exactly once, retains no full-file copy, and reports monotonic
progress/cancellation. The stored source ID is the identity observed during
open and will become the transaction precondition in M11.

Raw group metadata is capped at 56 bytes on supported 64-bit targets. The Safe
five-million-record ceiling therefore has a metadata upper bound of about 267
MiB before allocator overhead, independent of value payload size. This is a
deliberate large-file tradeoff: random group access without loading a 300 MiB
DXF payload into a second in-memory copy.

## M3.4 verified Verbatim writer

`DxfAsciiRawDocument::write_verbatim_to_new_file` accepts a destination path
that must not exist. It uses `create_new`; an existing file, including the
source path, returns an `AlreadyExists` create error without modifying that
file. The API exposes no overwrite flag and creates no parent directories.

Copying uses one fixed 64 KiB buffer and writes every source byte in original
order. A fresh SHA-256 is computed during the copy and must equal the
document's opening `DxfSourceId`. This detects same-length source mutation
between open and write and returns `DXF-E0301` instead of publishing changed
bytes.

After flush and `sync_all`, the writer closes and reopens the output. It checks
the exact length, reads the complete output in bounded chunks, probes for an
unexpected extra byte, and computes an independent output SHA-256. Length or
identity mismatches return `DXF-E0302` or `DXF-E0303`. Success returns
`DxfVerbatimWriteReceipt` containing source ID, output ID, and byte count; both
IDs are equal by construction and verification.

Progress is one monotonic range of `2 * source_len`: the first half is source
copy and the second is output verification. Cancellation is checked around
blocking reads and after progress callbacks. Any failure after destination
creation attempts to remove that exact incomplete file. If cleanup itself
fails, its path-redacted IO error replaces the primary error so callers know a
partial destination may remain.

Verbatim is permitted for both Strict and Compatible raw documents. It copies
BOM, mixed line endings, missing EOF, padded EOF, opaque tail bytes, and every
unknown payload without normalization. This does not promote a recovered
document to semantic-edit or canonical-write eligibility.

## Stable M3.4 codes

Fatal errors:

| Code | Meaning |
| --- | --- |
| `DXF-E0201` | Invalid ASCII group-code bytes or numeric domain |
| `DXF-E0202` | Group-code line has no following value line |
| `DXF-E0203` | Strict document has no exact terminal `0/EOF` marker |
| `DXF-E0204` | Strict document has source bytes after its EOF group |
| `DXF-E0301` | Source bytes no longer match the document's opening SHA-256 |
| `DXF-E0302` | Reopened Verbatim output length differs from the source length |
| `DXF-E0303` | Reopened Verbatim output SHA-256 differs from the source SHA-256 |

Non-fatal diagnostics:

| Code | Meaning |
| --- | --- |
| `DXF-W0201` | Compatible mode ignored a UTF-8 BOM before the first group code |
| `DXF-W0202` | Compatible mode recognized EOF after trimming ASCII space/tab |
| `DXF-W0203` | Compatible mode opened a paired stream without EOF |
| `DXF-W0204` | Compatible mode preserved opaque bytes after EOF |

Errors and diagnostics contain byte spans, never source payloads or paths.
Custom Debug implementations also omit raw group-code and value content.

## Resource behavior

The selected record limit is checked before consuming the first byte of an
excess group and reports observed count `limit + 1`. Source and value limits
remain enforced by the physical-line cursor. Cancellation remains cooperative
before and after potentially blocking reads.

Retained diagnostics never exceed the selected profile limit. On the first
excess diagnostic, the final retained entry becomes `DXF-W0001`; subsequent
diagnostics are suppressed. Document-level diagnostics share the same cap as
group framing diagnostics.

## Dependency and support boundary

M3.4 adds no dependency, imports no legacy code or fixture bytes, and does not
claim version/section support. It reuses the reviewed `sha2` dependency added
at M2.3. Deterministic writer vectors, implementation hashes, and failure-mode
evidence are in `docs/audits/M3_4_VERBATIM_WRITER_RECEIPT.md`. Initial
`inspect` and `verify` CLI commands remain the next M3 checkpoint.
