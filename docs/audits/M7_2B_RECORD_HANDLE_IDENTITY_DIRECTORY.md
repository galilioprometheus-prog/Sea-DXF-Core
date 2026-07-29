# M7.2b record handle identity directory

M7.2b connects the M7.1 handle-code and source-value substrate to M7.2a raw
record ranges. It exposes exact identity evidence without promoting malformed,
multiple, duplicate, or null values into stronger semantic claims.

## Normative boundary

Autodesk's numerical group-code reference defines code `5` as a handle string
of up to 16 hexadecimal digits and code `105` as the object handle used by a
DIMVAR/DIMSTYLE symbol-table entry:

<https://help.autodesk.com/cloudhelp/2018/ENU/AutoCAD-DXF/files/GUID-3F0380A5-1C15-464D-BC66-2C5F094BCFB9.htm>

The common entity and object tables list code `5` as their handle, while the
common symbol-table-entry table lists code `5` for entries other than DIMSTYLE
and code `105` for DIMSTYLE:

<https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm>

<https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-6939D69E-04CB-4F4C-87B2-67BC540FCF58.htm>

<https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-5926A569-3E40-4ED2-AE06-6ACCE0EFC813.htm>

Autodesk warns that programs must not rely on the displayed order of entity
group codes. The implementation therefore scans each complete raw record range
rather than assuming the identity group immediately follows its group-zero
marker.

These sources support treating codes `5` and `105` as identity candidates in
the already documented record boundary. They do not specify SeaCad behavior
for malformed spelling, multiple candidates, duplicate values, or a parsed null
handle, so all four remain explicit evidence states rather than guessed repair.

## Public contract

`DxfRawDocumentView`, `DxfAsciiRawDocument`, and `DxfBinaryRawDocument` expose
`handle_identity_directory(cancellation)`. There is one
`DxfHandleIdentityEntry` for every M7.2a raw record, retaining the exact
`DxfRawRecord`, its candidate slice, and one state:

- `Absent`;
- `UniqueParsed(DxfHandle)`;
- `UniqueInvalid(DxfHandleParseIssue)`;
- `Multiple { candidate_count }`.

Candidates reuse `DxfRawHandleValue`, preserving source identity, raw group
metadata, exact spelling access, group-code class, and lexical parse result.
The scan reads only payloads for codes classified as object identity and checks
cancellation before and throughout record/group traversal.

Only records with exactly one parsed candidate enter the handle-sorted match
index. `lookup` distinguishes `Missing`, `Unique`, and duplicate-preserving
`Ambiguous` matches. Invalid and multiple-candidate records remain queryable as
evidence but cannot silently become lookup targets.

## Explicit non-claims

`UniqueParsed` means only that one record contains one lexically valid group
`5` or `105`. It does not validate record type, require a non-null value, prove
document-wide uniqueness, establish target existence, or infer a topology edge.
M7.2b does not resolve pointer or owner handles, validate dangling references,
interpret record type spelling, dictionaries, persistent reactors, extension
dictionaries, XDATA, cloning, INSERT, XREF, edits, or writers.

## Antigravity handoff

Antigravity task 21 performed the first read-only inventory at base HEAD
`a8f68b05f3f40e36134a01eeb554267008455c29`. Codex rejected its test counts as
acceptance evidence because Cargo name filtering exercised zero raw-handle and
only one raw-record test, and one broad search included a missing path. The
usable pre/post HEAD, clean status, and API anchors were retained.

Corrective task 22 used exact integration-test targets and existing paths. It
returned 4/4 raw-handle and 3/3 raw-record tests, unchanged HEAD/status, no
writes, and the fixture anchors needed to establish four record-level gaps:
code `105`, invalid spelling, multiple candidates, and duplicate parsed values.
Codex independently reproduced the commands and derived the five-section
coverage matrix before accepting the handoff. M7.2b tests cover every reported
gap; Antigravity made no implementation or policy decision.

## Verification scope

Public integration tests cover ASCII/Binary parity for all nine supported
dialects and all five M7.2a section families. They exercise code `5`, DIMSTYLE
code `105`, records without candidates, exact spelling, null parsing, invalid
spelling, multiple candidates in one record, duplicate values across records,
missing/unique/ambiguous lookup, exclusion of ambiguous record candidates from
lookup, cancellation, bounded source reads, compact public values, thread
safety, and debug redaction.

## Reviewed artifact receipt

The reviewed production delta is 352 added lines: 347 lines in
`handle_identity.rs` and five module/export lines in `lib.rs`. The public
integration test artifact adds 392 lines. There are no manifest, dependency,
lockfile, schema, generated-file, parser-framing, writer, or fixture-corpus
changes.

SHA-256 at review time:

- `lib.rs`: `6ED91A9E40C8EB967B680EA29BFBC8A1B31D747C87AC72F399A406A8C48E5DA6`
- `handle_identity.rs`: `D7EB2B61B385EF3A0AD8CF19EAD77AFFC224FB82B9E9FFD250F7B79E7A520AF4`
- `handle_identity_tests.rs`: `7AEC848769DE419A624097CA2C542C95470614658759D088590C956B69E88232`
- `IMPLEMENTATION_PLAN.md`: `0B8096AC5E15A2271CB56275150DF6E0ADA425127849053B15CFCBAC8122F0A5`
- `SUPPORT_MATRIX.md`: `79844C688F4DC16F282516CEC41F58A747B3292E60F76645AF5FE7FD9F1F8E8B`

The reviewed production and integration-test artifacts add no `unsafe`, panic
path, `unwrap`, `expect`, `todo`, or `unimplemented` use.

## Required gates

- focused M7.2b identity suite: 3/3 passed;
- pre-implementation substrate suites independently reproduced: 4/4 raw
  handle and 3/3 raw record tests;
- `cargo deny --locked check`: passed advisories, bans, licenses, and sources;
- `cargo +1.97.1 fmt --all -- --check`: passed;
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`: passed;
- `cargo +1.97.1 clippy --workspace --all-targets -- -D warnings`: passed;
- `cargo +1.97.1 test --workspace`: 269 tests passed (16 CLI, 6 corpus,
  204 core unit, 3 handle-identity, 4 HEADER-handle, 12 numeric, 4 text,
  4 raw-handle, 3 raw-record, and 13 schema-generator);
- `git diff --check`: passed.
