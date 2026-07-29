# M7.2a format-neutral raw record directory

M7.2a adds an immutable directory of group-zero-delimited raw chunks inside
documented record-bearing sections. It establishes reliable occurrence ranges
needed by later handle identity and reference work without interpreting a
chunk's type spelling as a specific entity, object, table entry, or separator.

## Normative boundary

Autodesk states that entities, objects, classes, tables, table entries, and
file separators are introduced by group code `0` followed by a name:

<https://help.autodesk.com/cloudhelp/2018/ENU/AutoCAD-DXF/files/GUID-89CB823D-614D-4D1E-8204-568EC72DF869.htm>

For entities, Autodesk states that the next group `0` ends the current entity
and begins the next entity or ends the section:

<https://help.autodesk.com/cloudhelp/2018/ENU/AutoCAD-DXF/files/GUID-A35B8C2A-1885-4A8E-8533-E61D8A423D62.htm>

The published CLASSES, TABLES, ENTITIES, and OBJECTS examples show their
group-zero record and separator structure:

<https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-42E19B4F-61E1-4795-93E7-C8769CE2D7C0.htm>

<https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-A66D0ACA-3F43-4B2E-A0C2-2B490C1E5268.htm>

<https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-995ABB55-571A-4D0F-882E-8A74A738643E.htm>

<https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-1038FDE4-745D-469D-972E-1F977D674882.htm>

These sources support raw boundaries. They do not justify assigning semantic
record variants, resolving handles, or treating every group-zero marker as an
entity or object.

## Public contract

`DxfRawDocumentView`, `DxfAsciiRawDocument`, and `DxfBinaryRawDocument` expose
`raw_record_directory(cancellation)`. The directory recognizes only the five
documented record-bearing section families:

- `Classes`;
- `Tables`;
- `Blocks`;
- `Entities`;
- `Objects`.

Each `DxfRawRecordSection` retains its structure-index ordinal, exact section
and content group ranges, section kind, directory record range, and one state:

- `Indexed` for a completely closed section;
- `Interrupted` when another SECTION marker interrupted it;
- `Unclosed` when it reached EOF without ENDSEC.

Only `Indexed` sections contribute records. `DxfRawRecord` retains global and
section-local ordinals, structure-section provenance, section family, and its
half-open source group range. The first occurrence is always the group-zero
marker; the range ends at the next group-zero marker or section content end.
Binary search supports exact record lookup by any contained group occurrence.

The directory is built only from already validated section and group-zero
metadata. It performs no source I/O, checks cancellation throughout bounded
metadata traversal, allocates no per-group payload, and preserves source bytes
as authoritative.

## Explicit non-claims

HEADER, THUMBNAILIMAGE, unknown or invalidly named sections, section envelope
markers, ENDSEC, and EOF are never record entries. TABLE/ENDTAB, BLOCK/ENDBLK,
class, entity, object, and custom marker spelling remains uninterpreted.
M7.2a does not determine which chunks own handles, build a handle index,
resolve references, validate targets, or implement dictionary, XDATA, reactor,
edit, clone, INSERT, XREF, or writer behavior.

## Antigravity handoff

Before implementation, Antigravity Agent Hub task 20 performed a read-only
inventory at HEAD `67a41d19dbeda542f85265b0792f85483d6a85a3`. It confirmed the
existing section ranges, closure states, global group-zero ranges, shared
Binary aliases/adapters, malformed-section evidence, and absence of a reusable
per-record directory. Six ascii-index-filtered tests passed, Git state stayed
unchanged, and the worker wrote no files. Codex independently reproduced the
material inventory and tests before accepting the task.

## Verification scope

Public integration tests exercise ASCII/Binary parity across all nine supported
dialects, all five recognized section families, TABLE/ENDTAB and BLOCK/ENDBLK
chunks, exact marker spelling, record and section lookup, exclusion of HEADER,
thumbnail, unknown sections and EOF, interrupted/unclosed fail-closed behavior,
empty closed sections, cancellation, metadata-only source access, compactness,
thread safety, and redacted debug output.

## Reviewed artifact receipt

The reviewed production delta is 407 added and 1 removed line: 401 lines in
`raw_record.rs`, 5 export lines in `lib.rs`, and the one-line visibility change
in `ascii_index.rs`. The public integration test artifact adds 418 lines.

SHA-256 at review time:

- `ascii_index.rs`: `5B227E84948483CFCB746F5031A0A55B318389DAC306C034995AD0D288CF6255`
- `lib.rs`: `DA82CB99F75B38764EF8D3926E01BB5CA8F4DD51C64A39E1EF307917D9817E49`
- `raw_record.rs`: `E03A545A61D1D67D23B7D30F837D4A7EB4E685508CDF5C2813BE4A32B1C276CE`
- `raw_record_tests.rs`: `6997C91C7AFE89E8DC353D328BD86C8FB24B8BF8C88850881345BE576655B1E4`
- `IMPLEMENTATION_PLAN.md`: `CF6CE03389864F7CFA8AD740760258A083F3E8ED2BE7D6033A76A82683881801`
- `SUPPORT_MATRIX.md`: `90675D5F0996FBF15DE0446B7AB5118307F26ABECBD3110402E0EE1C2D600A26`

No `unsafe`, panic path, `unwrap`, `expect`, `todo`, or `unimplemented` was
introduced in the M7.2a production or integration-test artifacts.

## Required gates

- focused raw-record integration suite: 3/3 passed;
- `cargo deny --locked check`: passed advisories, bans, licenses, and sources;
- `cargo +1.97.1 fmt --all -- --check`: passed;
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`: passed;
- `cargo +1.97.1 clippy --workspace --all-targets -- -D warnings`: passed;
- `cargo +1.97.1 test --workspace`: 266 tests passed (16 CLI, 6 corpus,
  204 core unit, 4 HEADER-handle, 12 numeric, 4 text, 4 raw-handle,
  3 raw-record, and 13 schema-generator);
- `git diff --check`: passed.
