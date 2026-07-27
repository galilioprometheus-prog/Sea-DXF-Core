# M4.1 DXF dialect discovery contract

Status: frozen 2026-07-27

This contract covers only typed `$ACADVER` discovery for lossless ASCII raw
documents. It does not claim section accounting, encoding interpretation,
Binary DXF, entity semantics, or version conversion.

## Public model

`DxfAcadVersion` contains the nine exact supported codes AC1009 through AC1032.
`DxfAsciiRawDocument::acad_version_report()` returns one immutable
`DxfAcadVersionReport` tied to the document's `DxfSourceId`.

The report state is exactly one of:

- `Absent`: no exact `$ACADVER` occurrence was found in an exact HEADER.
- `Supported(version)`: exactly one occurrence has group code 1 and a registry
  value.
- `Unsupported`: exactly one occurrence has group code 1 but its exact value is
  outside the registry.
- `Invalid`: exactly one occurrence has no following value or the following
  group is not code 1.
- `Ambiguous`: more than one exact occurrence exists, regardless of whether the
  individual values agree.

An occurrence records the group occurrence number and content byte span for the
variable name and, when present, its immediate value candidate. Callers can use
the span with the raw document to read the exact original bytes.

## One-pass bounded discovery

Discovery observes borrowed groups during the existing ASCII framing pass,
before compact raw group metadata is stored. It does not rescan the source,
perform random reads, decode text, or copy version payload bytes.

The tracker retains only the first occurrence, the second occurrence as
conflicting evidence, and the total count. Memory use is therefore constant
even if a hostile file repeats `$ACADVER` millions of times. Later occurrences
remain available in the raw group stream but are not duplicated in the report.

The source SHA-256 is finalized before the report is created, so report and raw
document always carry the same `DxfSourceId`. The immutable report is
`Send + Sync`.

## Exact structural state machine

1. Exact `0/SECTION` arms the next group as the section name.
2. Exact `2/HEADER` enters HEADER; any other candidate does not.
3. Exact `0/ENDSEC` leaves HEADER.
4. Exact `9/$ACADVER` inside HEADER arms the immediate next group as its value.
5. Exact group code 1 is classified by the registry; any other code is invalid.
6. End of input while a value is armed produces `MissingValue`.

Compatible framing recovery does not loosen any semantic match. For example, a
missing terminal EOF may produce raw conformance `Recovered` while an otherwise
exact `$ACADVER` remains supported. Conversely, whitespace or case differences
never become valid dialect evidence.

## Stable diagnostics

| Code | Severity | Meaning |
| --- | --- | --- |
| `DXF-E0401` | Error | exact HEADER `$ACADVER` is missing |
| `DXF-E0402` | Error | value group is missing or is not group code 1 |
| `DXF-E0403` | Error | a second exact occurrence makes the dialect ambiguous |
| `DXF-W0401` | Warning | exact group-code 1 value is outside the registry |

These are semantic discovery diagnostics, separate from raw ASCII framing
diagnostics and conformance. An `E` diagnostic does not make lossless raw open
fail: malformed or unsupported documents remain inspectable and eligible for
unchanged Verbatim output. Semantic edits and canonical writers are not present
yet and gain their own eligibility gates in later milestones.

## Deliberate M4.1 boundaries

- No codepage, Unicode, or escape interpretation.
- No complete section index or group-0 record index.
- No CLI/JSON v1 schema expansion.
- No HEADER variable semantics beyond `$ACADVER`.
- No inference for undocumented or future version codes.
- No new dependency.
