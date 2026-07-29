# M7.4a ownership evidence aggregation

M7.4a filters M7.3b document-local resolution to source-anchored soft-owner and
hard-owner occurrences, while preserving every unresolved outcome. Unique
links are grouped by target record so downstream work can observe zero, one,
or multiple incoming ownership-class links without constructing a graph or
guessing malformed and duplicate handles.

## Normative boundary

Autodesk distinguishes pointers, which express usage, from ownership
references, under which the owner is responsible for another object. It also
states that an object can have only one owner:

<https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-704F5152-B2A4-4DAC-A0FD-03D8ABFC0A4F.htm>

The documented numeric ranges classify `350..359` as soft ownership IDs and
`360..369` as hard ownership IDs:

<https://help.autodesk.com/cloudhelp/2018/ENU/AutoCAD-DXF/files/GUID-3F0380A5-1C15-464D-BC66-2C5F094BCFB9.htm>

Hard references protect their targets from purge while soft references do not:

<https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-28FADE88-9BEA-48DE-8146-43F9514679B8.htm>

Common entity and object records also use group code `330`, classified as a
soft pointer, to identify an owner object. Therefore owner-class incoming links
alone are not sufficient to claim a complete owner relation:

<https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm>

<https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-6939D69E-04CB-4F4C-87B2-67BC540FCF58.htm>

M7.4a consequently exposes ownership-class evidence and defers the normative
one-owner diagnostic until record context and both directions are available.

## Public contract

`DxfRawDocumentView`, `DxfAsciiRawDocument`, and `DxfBinaryRawDocument` expose
`ownership_evidence_directory(cancellation)`. The directory owns the shared
M7.3b `DxfHandleResolutionDirectory`, one source-order
`DxfOwnershipEvidenceEntry` for every soft-owner or hard-owner occurrence, and
one `DxfResolvedOwnershipLink` for every uniquely resolved occurrence.

Each evidence entry retains its M7.3b resolution ordinal, exact reference,
soft/hard class, source record, raw spelling access, and one of `Invalid`,
`Null`, `Missing`, `Unique`, or `Ambiguous`. Pointer occurrences remain in the
owned resolution directory but are excluded from the ownership-evidence slice.

Resolved links are sorted by target record ordinal and then evidence ordinal.
`incoming_links_for_target_record` returns the exact slice for any known raw
record, including an empty slice when no unique ownership-class link targets
that record. `evidence_for_source_record` preserves the outgoing source-order
slice. Out-of-range record and entry ordinals return `None`.

Storage remains linear: target identity arrays are not copied, unresolved
entries do not enter the target index, and every unique ownership occurrence
contributes exactly one compact resolved-link value.

## Explicit non-claims

An incoming link is evidence from a numeric ownership-class group only. M7.4a
does not decide whether the source record is legally allowed to own the target,
combine common `330` owner pointers, enforce the one-owner rule, select one of
multiple links, apply purge behavior, diagnose cycles, traverse reachability,
interpret dictionaries/reactors/extension dictionaries, or mutate, clone, or
write records.

No incoming ownership-class link does not mean that a record is ownerless.
Likewise, multiple incoming links are retained evidence, not yet a conformance
diagnostic.

## Antigravity evidence

Task 25 was issued to Antigravity for a strictly read-only mechanical inventory
at M7.3b HEAD, with explicit exclusion of API, normative, support, and Git
decisions. At review time the task remained open and unclaimed, so it
contributes no evidence to this checkpoint. Codex independently inspected the
substrate, checked the official Autodesk boundary, implemented the milestone,
and ran every verification gate. Stale task 24 also remained explicitly
superseded and contributes no evidence.

## Verification scope

Public integration tests cover ASCII/Binary parity across all nine supported
dialects, the pre-R13 absence of physically unavailable owner group codes,
soft-owner versus hard-owner preservation, pointer exclusion, source-record
slices, zero/one/multiple incoming unique links, missing/null/invalid/ambiguous
non-indexing, duplicate target identities, raw spelling, cancellation, bounded
lookups, linear source reads, compact public values, and thread safety.

## Reviewed artifact receipt

The reviewed production delta is 221 added lines: 217 lines in
`ownership_evidence.rs` and four module/export lines in `lib.rs`. The public
integration test artifact adds 342 lines. Documentation changes update the
implementation plan and support matrix; there are no manifest, dependency,
lockfile, schema, generated-file, parser-framing, writer, or corpus changes.

SHA-256 at review time:

- `lib.rs`: `886001CBB5F417EB0E78A2160119D43BEBA9CE582F37B7FD78E268421B0AFF85`
- `ownership_evidence.rs`: `B5AC795E89F735392D1C468B93D4C73C35B961B41DA146A9EC8A11F2C5562E81`
- `ownership_evidence_tests.rs`: `1AA6321DEA18A18EA8CC9183C8E768F8E8DF9106980FF3F3A9546BA0F5B6F115`
- `IMPLEMENTATION_PLAN.md`: `705A558837CDD6050F8DF4421D3501C70B41E4B47900FD9CAFB5BA5F4CA639E0`
- `SUPPORT_MATRIX.md`: `7FAEEC3F5DF3EAB248F72202CEB766628F612CE11341731BB01B35AB4719A8D4`

The reviewed production and integration-test artifacts add no `unsafe`, panic
path, `unwrap`, `expect`, `todo`, or `unimplemented` use.

## Required gates

- focused M7.4a ownership-evidence suite: 3/3 passed;
- focused M7.3a/M7.3b regression suites: 6/6 passed;
- `cargo deny --locked check`: passed advisories, bans, licenses, and sources;
- `cargo +1.97.1 fmt --all -- --check`: passed;
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`: passed;
- `cargo +1.97.1 clippy --workspace --all-targets -- -D warnings`: passed;
- `cargo +1.97.1 test --workspace`: 278 tests passed (16 CLI, 6 corpus,
  204 core unit, 3 handle-identity, 3 handle-reference, 3 handle-resolution,
  4 HEADER-handle, 12 numeric, 4 text, 3 ownership-evidence, 4 raw-handle,
  3 raw-record, and 13 schema-generator);
- `git diff --check`: passed.

The first sandboxed `cargo deny` attempt could not acquire its advisory
database lock because the external cache path was read-only. The identical
command passed after the required scoped approval; no repository or dependency
state changed.
