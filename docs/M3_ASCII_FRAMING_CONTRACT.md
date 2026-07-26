# M3 ASCII framing contract

Status: M3.1 physical representation and physical-line framing complete

M3.1 establishes the byte-preserving layer beneath ASCII group-record parsing.
It does not claim that an ASCII candidate is a valid DXF document, implement
Strict or Compatible acceptance policy, or provide a Verbatim writer.

## Normative basis

The Autodesk DXF reference defines an ASCII DXF group as a group-code line
followed by a value line, and describes sections beginning with group code `0`
and value `SECTION`:

- [DXF ASCII format](https://help.autodesk.com/cloudhelp/2020/ENU/AutoCAD-DXF/files/GUID-D939EA11-0CEC-4636-91A8-756640A031D3.htm)
- [Reading a DXF file](https://help.autodesk.com/cloudhelp/2015/ENU/AutoCAD-DXF/files/GUID-89CB823D-614D-4D1E-8204-568EC72DF869.htm)

Autodesk specifies the binary representation by its exact 22-byte sentinel:

- [DXF binary format](https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-FC1C3C69-DBC2-49E4-893A-000D6538C0FE.htm)

M3.1 uses these facts only for physical classification and line framing. Group
code interpretation remains the responsibility of M3.2.

## Physical representation probe

`DXF_BINARY_SENTINEL` is exactly
`AutoCAD Binary DXF<CR><LF><SUB><NUL>`. An exact match classifies the source as
Binary. An empty source, a truncated sentinel prefix, or a NUL byte in the
inspected prefix is Unknown. Every other non-empty prefix is only an
AsciiCandidate.

AsciiCandidate is deliberately not a validity result. Arbitrary text, a BOM,
non-ASCII bytes, malformed group codes, and incomplete group pairs may reach
the ASCII framing layer and be rejected or diagnosed later.

The probe reads no more than 22 bytes, accepts legal partial source reads, and
rechecks the selected source-byte limit before its first read.

## Lossless physical lines

`DxfAsciiLineCursor` exposes each physical line as borrowed raw bytes plus:

- a zero-based line index;
- a half-open byte span for content;
- a half-open byte span for its terminator;
- a half-open byte span for the complete physical line; and
- an explicit `CrLf`, `Lf`, `Cr`, or `None` terminator.

Content excludes the line terminator. `None` records a final unterminated line;
it does not by itself mean that Strict or Compatible mode accepts the file.
There is no synthetic empty line after a terminal newline.

The cursor performs no trimming, decoding, Unicode conversion, BOM removal, or
NUL filtering. Whitespace, high bytes, escape syntax, and all other content
remain the raw source truth. Its custom Debug output reports byte counts and
spans rather than exposing source payload bytes.

## Resource and failure behavior

The cursor uses one fixed 8 KiB read buffer and one reusable allocation for the
current line. It does not allocate in proportion to the source. The selected
profile's source-byte and value-byte limits are rechecked at the framing
boundary. A line fails before the byte exceeding the value limit is appended.

All offset arithmetic is checked. Legal partial reads and CRLF split across
read boundaries are supported. Premature EOF, impossible source behavior,
allocation failure, limit violations, and cancellation fail closed through
the stable M2 error model. Cancellation is checked before and after each
potentially blocking source read. After any error, callers discard the cursor.

Consumers can use `consumed_bytes()` for progress accounting. M3.1 deliberately
does not invoke a progress observer between individual lines because the next
group-record layer owns record-level progress and diagnostics.

## Deferred acceptance policy

The public Autodesk pages above do not unambiguously specify every acceptance
case for CRLF, LF, CR, BOM, whitespace, malformed final pairs, or missing EOF
across all target releases. M3.1 therefore records exact physical evidence but
does not guess a Strict/Compatible policy.

M3.2 will add group-code/value pairing, record-count enforcement, stable
framing errors and diagnostics, and a documented recovery allowlist. Ambiguous
cases will be measured with the isolated AutoCAD oracle before policy is
frozen. Compatible mode will recover only evidenced framing defects and will
not infer semantics.

## Dependency and support boundary

M3.1 adds no dependency, imports no legacy code or fixture bytes, and does not
change the support matrix. A validated ASCII read claim requires later M3
checkpoints for record parsing, document framing, and unchanged Verbatim
round-trip evidence.
