# M7.3a record pointer and owner occurrences

M7.3a records source-anchored pointer and owner handle occurrences inside the
complete raw record ranges established by M7.2a. It does not resolve those
occurrences against M7.2b identities.

## Normative boundary

Autodesk defines `330..339` as soft pointers, `340..349` as hard pointers,
`350..359` as soft owners, and `360..369` as hard owners. It also defines
`390..399` as plot-style handles that are basically hard pointers and
`480..481` as hard pointers:

<https://help.autodesk.com/cloudhelp/2018/ENU/AutoCAD-DXF/files/GUID-3F0380A5-1C15-464D-BC66-2C5F094BCFB9.htm>

Autodesk distinguishes pointers, which indicate use without possession, from
ownership references, under which an owner is responsible for another object:

<https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-704F5152-B2A4-4DAC-A0FD-03D8ABFC0A4F.htm>

Autodesk states that XDATA code `1005` has soft-pointer behavior:

<https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-8243079C-B44F-493A-BAF7-1D11A6E6C78C.htm>

Arbitrary handles `320..329` are explicitly different: they are taken as-is
and ignored by handle translation during drawing-merge operations. They are
therefore excluded rather than mislabeled as pointer or ownership references:

<https://help.autodesk.com/cloudhelp/2017/ENU/AutoCAD-DXF/files/GUID-4A44BF4D-63C2-4A17-A6FB-58443D63571C.htm>

## Public contract

`DxfRawDocumentView`, `DxfAsciiRawDocument`, and `DxfBinaryRawDocument` expose
`handle_reference_directory(cancellation)`. Each
`DxfHandleReferenceEntry` retains the exact `DxfRawRecord` and
`DxfRawHandleValue` for one source-order occurrence classified as:

- `SoftPointer` (`330..339`, `1005`);
- `HardPointer` (`340..349`, `390..399`, `480..481`);
- `SoftOwner` (`350..359`);
- `HardOwner` (`360..369`).

The directory exposes its source identity, raw-record count, source-order
entries, entry lookup, per-record slices, and exact occurrence lookup. A valid
record with no pointer/owner occurrence returns an empty slice; an out-of-range
record ordinal returns `None`.

Building the directory reuses complete M7.2a record ranges and the fixed-buffer
M7.1b raw handle projection. It reads only selected handle payloads, preserves
typed lexical failures and exact spelling access, checks cancellation during
record/group traversal, and uses only standard-library storage.

## Explicit non-claims

Object identities (`5`, `105`) and arbitrary handles (`320..329`) are not
entries. A parsed value is not proof of target existence or semantic validity.
M7.3a does not resolve a target, report missing or ambiguous identities, enforce
one-owner rules, distinguish reactor/extension-dictionary/application-defined
containers, validate XDATA framing, construct a graph, or implement purge,
clone, INSERT, XREF, edit, or writer behavior.

Code `1005` retains its numeric soft-pointer classification only. Its presence
does not prove the surrounding record contains a valid registered XDATA list.

## Antigravity handoff

After the M7.2b checkpoint, Antigravity task 23 performed a read-only mechanical
inventory at exact HEAD `8d47674abe618e05696b3d39d590c6a10087cd45`.
All nine commands exited zero. The worker returned path/line anchors for all
five non-identity handle classes and representative codes `320`, `330`, `340`,
`350`, `360`, `390`, `480`, and `1005`; exact integration targets passed 4/4
raw-handle and 3/3 handle-identity tests. HEAD and clean ahead-five status were
unchanged and `written_paths` was empty.

Codex independently reproduced the class/code searches, both test targets,
HEAD/status, and diff check before marking the task done. Codex—not the worker—
made the normative decision to exclude `Arbitrary` and retain only the four
pointer/owner classes.

## Verification scope

Public integration tests cover all nine supported dialects in ASCII and Binary,
all four pointer/owner classes, plot-style and `480` hard-pointer codes, XDATA
`1005`, exact source spelling, parsed and invalid values, lookup by entry,
record, and group occurrence, records without references, exclusion of identity
and arbitrary handles, fail-closed interrupted sections, cancellation, bounded
source reads, compact public values, thread safety, and debug redaction.

The AC1009 fixture uses only code `1005` through the documented pre-R13 Binary
extended-code escape. Codes above 255 that cannot use that encoding are absent
from both AC1009 physical fixtures; this is physical parity evidence, not a
general version-applicability claim.

An optional follow-up task 24 was prepared to repeat focused gates on the exact
dirty diff, but the worker did not claim it before Codex completed the full
required gates. Codex explicitly told the worker not to claim the now-stale
hash task. Task 24 contributes no evidence or completion claim.

## Reviewed artifact receipt

The reviewed production delta is 202 added lines: 200 lines in
`handle_reference.rs` and two module/export lines in `lib.rs`. The public
integration test artifact adds 425 lines. There are no manifest, dependency,
lockfile, schema, generated-file, parser-framing, writer, or corpus changes.

SHA-256 at review time:

- `lib.rs`: `D8D860B62F2098E46720601172A5F977B6C8637D4DEF5A140B379DD6EF3263B0`
- `handle_reference.rs`: `93122477C817E89BFAB4D3E6B5BA667B6056A1B6DAC1F2AE4F713CDA650B264D`
- `handle_reference_tests.rs`: `BEA3EDF6F29CBC15DF483E46FF6A63F7E0C7AEB3A3E9C36E56DB8D8063F2E78B`
- `IMPLEMENTATION_PLAN.md`: `8C7E56A11FC8C13894191609B6F1B7B57BCB304D48B122F956C99768C71D93C0`
- `SUPPORT_MATRIX.md`: `A9D243100CD117E4B0279704B5366784B3A6A10470D36B149F3650ED8A02738D`

The reviewed production and integration-test artifacts add no `unsafe`, panic
path, `unwrap`, `expect`, `todo`, or `unimplemented` use.

## Required gates

- focused M7.3a reference suite: 3/3 passed;
- independently reproduced Antigravity substrate suites: 4/4 raw handle and
  3/3 handle identity tests;
- `cargo deny --locked check`: passed advisories, bans, licenses, and sources;
- `cargo +1.97.1 fmt --all -- --check`: passed;
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`: passed;
- `cargo +1.97.1 clippy --workspace --all-targets -- -D warnings`: passed;
- `cargo +1.97.1 test --workspace`: 272 tests passed (16 CLI, 6 corpus,
  204 core unit, 3 handle-identity, 3 handle-reference, 4 HEADER-handle,
  12 numeric, 4 text, 4 raw-handle, 3 raw-record, and 13 schema-generator);
- `git diff --check`: passed.
