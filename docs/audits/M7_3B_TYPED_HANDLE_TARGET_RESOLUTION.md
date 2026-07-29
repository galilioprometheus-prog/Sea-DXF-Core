# M7.3b typed handle target resolution

M7.3b joins M7.3a pointer/owner occurrences to the M7.2b record identity index.
It reports exact document-local lookup evidence without turning lookup into a
topology, ownership-validity, or editing policy.

## Normative boundary

Autodesk defines pointers as usage references and ownership references as
responsibility for another object:

<https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-704F5152-B2A4-4DAC-A0FD-03D8ABFC0A4F.htm>

The numeric reference assigns soft/hard pointer and owner roles to the group
code ranges indexed by M7.3a:

<https://help.autodesk.com/cloudhelp/2018/ENU/AutoCAD-DXF/files/GUID-3F0380A5-1C15-464D-BC66-2C5F094BCFB9.htm>

These sources establish that the parsed values are references. They do not
authorize guessing malformed values, selecting one duplicate target, treating
zero as a record, or accepting a record with multiple identity candidates.
SeaCad therefore preserves all such outcomes as typed evidence.

## Public contract

`DxfRawDocumentView`, `DxfAsciiRawDocument`, and `DxfBinaryRawDocument` expose
`handle_resolution_directory(cancellation)`. The directory owns one shared
`DxfHandleIdentityDirectory`, one shared `DxfHandleReferenceDirectory`, and one
`DxfHandleResolutionEntry` per reference occurrence.

Each entry retains the exact M7.3a reference and one state:

- `Invalid(DxfHandleParseIssue)` when the reference spelling is not an exact
  hexadecimal handle;
- `Null` when the parsed handle equals zero;
- `Missing` when no uniquely parsed record identity has that value;
- `Unique` when exactly one record identity matches;
- `Ambiguous { target_count }` when duplicate record identities match.

`targets_for_reference` returns the shared M7.2b match slice. Invalid, null,
and missing entries return an empty slice; unique and ambiguous entries return
one or all exact source-anchored identity matches. Lookup by reference ordinal
and original group occurrence is bounded and source-order stable.

The implementation does not copy target slices per reference. Its owned
storage is the linear identity directory, linear reference directory, and one
compact resolution entry per reference. Cancellation is checked before and
throughout both underlying scans and during the join.

## Explicit non-claims

A `Unique` lookup proves only that one M7.2b uniquely parsed record identity has
the same numeric value. It does not prove record-type compatibility, legal
ownership, one-owner conformance, hard/soft lifecycle behavior, reachability,
purge protection, dictionary membership, reactor or extension-dictionary
context, valid XDATA framing, or cross-document identity.

M7.3b does not build or traverse a graph, diagnose cycles, mutate references,
clone objects, translate INSERT/XREF handles, edit records, or write DXF.

## Antigravity evidence

The accepted Antigravity substrate for this chain is task 23 at M7.2b HEAD. It
mechanically verified the five non-identity classes, representative reference
codes, raw handle projection, record ranges, identity exclusions, and exact
integration targets; Codex independently reproduced every material result.

An optional M7.3a dirty-diff task 24 remained unclaimed and was explicitly
superseded before the M7.3a checkpoint changed HEAD. It contributes no evidence
to M7.3a or M7.3b. Codex did not infer worker execution from the open task.

## Verification scope

Public integration tests cover ASCII/Binary parity across all nine supported
dialects, `1005` pre-R13 escape resolution, all five states, exact group lookup,
shared target slices, duplicate identities, invalid and multiple identity
records, invalid references, explicit null precedence even when identity zero
exists, missing targets, cancellation, linear source reads, compact/thread-safe
public values, and debug redaction.

## Reviewed artifact receipt

The reviewed production delta is 209 added lines: 205 lines in
`handle_resolution.rs` and four module/export lines in `lib.rs`. The public
integration test artifact adds 309 lines. There are no manifest, dependency,
lockfile, schema, generated-file, parser-framing, writer, or corpus changes.

SHA-256 at review time:

- `lib.rs`: `0DF18A6EE6BDB29CA9B1F4A463C3EA20B18AC79F3F851E978C3144EB8BBE1037`
- `handle_resolution.rs`: `9042BC966BB6FFCC3F4B7EC09B7017FB95595655CA821746E83E5EDC5F7A53EE`
- `handle_resolution_tests.rs`: `685AC2CEEDD4A09EB473DF03CE89F9A059DF850BAE8FF401AC5208212CED962F`
- `IMPLEMENTATION_PLAN.md`: `2A7C753062BE94C565E25DE97CD65EEC086FCBB88FB5182219866C5838D779C3`
- `SUPPORT_MATRIX.md`: `A1EC7D6ADED45E29A0AE52330EBA2F76583D2795C036A885562155EF14B2F7EF`

The reviewed production and integration-test artifacts add no `unsafe`, panic
path, `unwrap`, `expect`, `todo`, or `unimplemented` use.

## Required gates

- focused M7.3b resolution suite: 3/3 passed;
- `cargo deny --locked check`: passed advisories, bans, licenses, and sources;
- `cargo +1.97.1 fmt --all -- --check`: passed;
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`: passed;
- `cargo +1.97.1 clippy --workspace --all-targets -- -D warnings`: passed;
- `cargo +1.97.1 test --workspace`: 275 tests passed (16 CLI, 6 corpus,
  204 core unit, 3 handle-identity, 3 handle-reference, 3 handle-resolution,
  4 HEADER-handle, 12 numeric, 4 text, 4 raw-handle, 3 raw-record, and
  13 schema-generator);
- `git diff --check`: passed.
