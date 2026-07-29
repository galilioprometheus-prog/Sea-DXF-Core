# M7.4f common-owner candidate index

M7.4f groups every conservative M7.4e common-owner pointer candidate by its
source raw record. It exposes typed zero, one, or multiple candidate
cardinality without discarding lexical or target-resolution failures.

## Normative boundary

Autodesk's common entity contract documents an outside-group `330` soft
pointer to the owner `BLOCK_RECORD`, while its common object contract documents
an outside-group `330` soft pointer to the owner object:

<https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm>

<https://help.autodesk.com/cloudhelp/2025/ENU/AutoCAD-DXF/files/GUID-6939D69E-04CB-4F4C-87B2-67BC540FCF58.htm>

Autodesk separately states that an object can have any number of pointer
references but only one owner:

<https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-704F5152-B2A4-4DAC-A0FD-03D8ABFC0A4F.htm>

M7.4f records candidate cardinality as evidence needed for later reconciliation.
It does not apply the one-owner rule to uninterpreted record types.

## Public contract

`DxfRawDocumentView`, `DxfAsciiRawDocument`, and `DxfBinaryRawDocument` expose
`common_owner_candidate_directory(cancellation)`. The immutable directory owns
one shared M7.4e role directory, one filtered source-order
`DxfCommonOwnerCandidateEntry` stream, and one `DxfCommonOwnerRecordEntry` for
every complete raw record.

Each candidate retains its exact M7.4e role entry, its role/reference ordinal,
and the independent M7.3b resolution state. `targets_for_candidate` returns the
existing identity-match slice: empty for invalid, null, or missing values; one
match for a unique target; and every duplicate-preserving match for an
ambiguous target.

Each record entry retains the exact raw record, a half-open
`DxfCommonOwnerCandidateRange`, and one state:

- `NoCandidate`;
- `UniqueCandidate`;
- `MultipleCandidates { candidate_count }`.

`entry`, `record_entry`, `candidates_for_record`, and
`targets_for_candidate` are bounded lookups. Construction filters the shared
role stream once and performs a linear merge with the existing record/identity
entries. It checks source identity, cancellation, allocation failure, compact
ordinals, and complete candidate consumption.

## Explicit non-claims

Candidate count is not legal-owner count. M7.4f does not compare a candidate's
target record with the source records of incoming `350..369` ownership-class
links, select an authoritative owner, enforce one-owner conformance, validate
record types or target types, interpret dictionaries/reactors, traverse a
graph, diagnose cycles, apply lifecycle or purge behavior, clone, edit, or
write DXF.

## Antigravity evidence

Antigravity task 30 performed a read-only mechanical substrate inventory at
exact clean base `f5642740dbbf03408dfaa2c918af0d0b17ed3f5c` after a fresh
quota snapshot recommended Gemini 3.5 Flash (High). All six commands exited
zero, the handle-role and ownership-evidence suites passed 10/10, before/after
HEAD and Git status stayed clean and identical, and no paths were written. Its
anchors covered role filtering, per-record slices, target matches, incoming
ownership cardinality, and raw record ordinals.

Codex independently reproduced the 10/10 focused tests and Git checks before
accepting task 30, then chose and implemented the public contract. Normative,
semantic, support, audit, commit, and tag decisions remained with Codex.

## Verification scope

Public integration tests cover ASCII/Binary parity across all nine supported
dialects; pre-R13 absence of unavailable `330`; candidate filtering that
excludes a reactor-context `330`; every invalid, null, missing, unique, and
ambiguous target state; duplicate-preserving target slices; per-record zero,
one, and multiple cardinality; exact candidate/record ranges and ordinal
lookups; cancellation; bounded linear source reads; compact/copy/thread-safe
values; bounds; and debug redaction.

## Reviewed artifact receipt

The reviewed production delta is 319 added lines: 314 lines in
`common_owner_candidate.rs` and five module/export lines in `lib.rs`. The
public integration test artifact adds 377 lines. There are no manifest,
dependency, lockfile, schema, generated-file, parser-framing, writer, or corpus
changes.

SHA-256 at review time:

- `lib.rs`: `B249C28AAAED600F83205106031C11ACE553C25815D67F3A5D287702A4D078BE`
- `common_owner_candidate.rs`: `EB11A7750B9CB624CFC3390DCAF4A87AFD53A2DD4CDB8EB2866F5E9142D3CC36`
- `common_owner_candidate_tests.rs`: `93A01291113CE9D12ACF73615B4D1E0816CA4C8F4D3A7E0D9BD6D001380D2A96`
- `IMPLEMENTATION_PLAN.md`: `74930EFB11234B8E25EF08645CF8EFB4124E2C6209D4E34C91A63E3C71830510`
- `SUPPORT_MATRIX.md`: `CCC05BB0C19E9806A0DDA98F6A41DDEC7F3A526D82754BD83B9D16F4B92BBAA7`

The reviewed production and integration-test artifacts add no `unsafe`, panic
path, `unwrap`, `expect`, `todo`, or `unimplemented` use.

## Required gates

- focused M7.4f common-owner candidate suite: 4/4 passed;
- Antigravity substrate suites: 10/10 reproduced independently at the clean
  base;
- `cargo deny --locked check`: passed advisories, bans, licenses, and sources;
- `cargo +1.97.1 fmt --all -- --check`: passed;
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`: passed;
- `cargo +1.97.1 clippy --workspace --all-targets -- -D warnings`: passed;
- `cargo +1.97.1 test --workspace`: 295 tests passed (16 CLI, 6 corpus,
  204 core unit, 3 application-group, 4 common-owner-candidate,
  3 contextual-reference, 3 handle-identity, 3 handle-reference,
  3 handle-resolution, 6 handle-role, 4 HEADER-handle, 12 numeric, 4 text,
  4 ownership-evidence, 4 raw-handle, 3 raw-record, and 13 schema-generator);
- `git diff --check`: passed.
