# M7.4c application control group evidence

M7.4c adds a format-neutral, record-scoped directory for every DXF group-code
`102` control occurrence. It retains exact lexical markers and bounded group
ranges needed to distinguish ordinary owner pointers from persistent-reactor
and extension-dictionary context in later milestones.

## Normative boundary

Autodesk documents group code `102` as the start of an application-defined
group (`{application_name`) and `}` as the group end. The common entity and
object contracts specifically identify `{ACAD_REACTORS` and
`{ACAD_XDICTIONARY`, with `330` and `360` handles inside those ranges:

<https://help.autodesk.com/cloudhelp/2017/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm>

<https://help.autodesk.com/cloudhelp/2025/ENU/AutoCAD-DXF/files/GUID-6939D69E-04CB-4F4C-87B2-67BC540FCF58.htm>

Common symbol-table records use the same extension-dictionary control shape:

<https://help.autodesk.com/cloudhelp/2025/ENU/AutoCAD-DXF/files/GUID-8427DD38-7B1F-4B7F-BF66-21ADD1F41295.htm>

Autodesk also warns consumers not to depend on the displayed order of common
group-code tables. M7.4c therefore scans each complete record and classifies
markers by exact bytes rather than by expected position.

## Public contract

`DxfRawDocumentView`, `DxfAsciiRawDocument`, and `DxfBinaryRawDocument` expose
`application_group_directory(cancellation)`. The immutable directory contains:

- one `DxfApplicationControlEntry` per group-code `102` occurrence, retaining
  its exact `DxfRawGroup`, raw record, and `Start`, `Close`, or `Invalid` role;
- one `DxfApplicationGroupEntry` per start marker, retaining the exact record,
  known or other kind, start/end control ordinals, source range, content range,
  and `Closed`, `Interrupted`, or `Unclosed` state.

Known starts require exact case-sensitive raw bytes for `{ACAD_REACTORS` or
`{ACAD_XDICTIONARY`. Any other nonempty payload beginning with `{` is retained
as `Other`; `}` is a close; every remaining spelling is `Invalid`. Other names
remain source-backed and are never decoded, normalized, or placed in debug
output.

A new start interrupts an existing open group at that occurrence. A close with
no open group remains visible as a control entry but invents no group. An open
group ends unclosed at its raw-record boundary and never crosses into the next
record. `group_for_content_occurrence` returns only content membership, never a
start or closing marker.

Exact known-token comparison uses the existing bounded raw-span helper. Other
starts read only their first payload byte. Cancellation is checked throughout
record scanning and around source reads; storage is linear in controls and
starts.

## Explicit non-claims

M7.4c does not validate application-defined payloads, infer nested application
semantics, repair malformed controls, interpret record types, or reclassify any
`330`/`360` handle. It does not yet claim a persistent reactor, extension
dictionary, ordinary owner pointer, legal owner relation, one-owner
conformance, dictionary membership, purge behavior, graph, edit, or writer.

## Antigravity evidence

Antigravity task 27 performed a strictly read-only substrate inventory at exact
clean M7.4b HEAD `2348f20c26746cd2b68bc353c3308adf8f4b13c7` using the
recommended Gemini 3.5 Flash (High) quota class. All six commands exited zero,
the filtered raw-document suite passed 8/8, before/after status remained clean
and identical, and no paths were written. Its anchors covered format-neutral
payload spans, bounded source reads, raw-record iteration, and handle projection.

Codex independently inspected the cited mechanics, chose the normative/API
boundary, implemented the directory, and reproduced the focused and full
workspace verification before accepting task 27. Antigravity made no support,
architecture, audit, commit, or tag decision.

## Verification scope

Public integration tests cover ASCII/Binary parity across all nine supported
dialects, exact known/other/close/invalid controls, closed/interrupted/unclosed
groups, unmatched closes, record-local slices, exact source/content ranges,
known `330` reactor and `360` extension-dictionary membership versus outside
`330`, raw spelling, cancellation, exact source-read count, bounded lookups,
compact/copy/thread-safe public values, and debug redaction.

## Reviewed artifact receipt

The reviewed production delta is 382 added lines: 377 lines in
`application_group.rs` and five module/export lines in `lib.rs`. The public
integration test artifact adds 366 lines. There are no manifest, dependency,
lockfile, schema, generated-file, parser-framing, writer, or corpus changes.

SHA-256 at review time:

- `lib.rs`: `0FA51D8F1EEDDA166B3CB0DD82F14536E2FC75D13E164AFF3687D5C0F78B634E`
- `application_group.rs`: `0B1F2B7B2A026665338742AAB17C0AF7FE3A61320532A44B182854729EEDFC0F`
- `application_group_tests.rs`: `CC5C3D133903F323A3F63C9682D865A4ABC33CAFCD667B0F7BC45C5E04BC8C9C`
- `IMPLEMENTATION_PLAN.md`: `642629518EB3BCD172073C3CFCE878E61DDB9FD8393CEC30A440F2359F4AB22F`
- `SUPPORT_MATRIX.md`: `E944FA95CF72828945E04C90F2AB7F2D5227D18FC57B2D517A7F8B1874D63C13`

The reviewed production and integration-test artifacts add no `unsafe`, panic
path, `unwrap`, `expect`, `todo`, or `unimplemented` use.

## Required gates

- focused M7.4c application-group suite: 3/3 passed;
- Antigravity substrate suite: 8/8 passed read-only at the clean base;
- `cargo deny --locked check`: passed advisories, bans, licenses, and sources;
- `cargo +1.97.1 fmt --all -- --check`: passed;
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`: passed;
- `cargo +1.97.1 clippy --workspace --all-targets -- -D warnings`: passed;
- `cargo +1.97.1 test --workspace`: 282 tests passed (16 CLI, 6 corpus,
  204 core unit, 3 application-group, 3 handle-identity, 3 handle-reference,
  3 handle-resolution, 4 HEADER-handle, 12 numeric, 4 text,
  4 ownership-evidence, 4 raw-handle, 3 raw-record, and 13 schema-generator);
- `git diff --check`: passed.
