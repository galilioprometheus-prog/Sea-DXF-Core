# M7.4g bidirectional owner-evidence comparison

M7.4g compares the two independently retained ownership-evidence directions
for every raw record without selecting or inventing an owner.

## Normative boundary

Autodesk's common entity and common object contracts document an outside-group
`330` soft pointer to an owner, while ownership references separately express
an owner's responsibility for referenced objects:

<https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm>

<https://help.autodesk.com/cloudhelp/2025/ENU/AutoCAD-DXF/files/GUID-6939D69E-04CB-4F4C-87B2-67BC540FCF58.htm>

<https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-704F5152-B2A4-4DAC-A0FD-03D8ABFC0A4F.htm>

M7.4g compares these directions only when each side has one exact record-level
candidate. It does not treat agreement as proof that an uninterpreted record
type legally participates in that ownership relation.

## Public contract

`DxfRawDocumentView`, `DxfAsciiRawDocument`, and `DxfBinaryRawDocument` expose
`owner_evidence_comparison_directory(cancellation)`. The immutable directory
owns one M7.4f common-owner candidate directory, one M7.4b ownership-evidence
directory, and one `DxfOwnerEvidenceComparisonEntry` per raw record.

Each entry retains the exact raw record, its common-owner record card, its
incoming-ownership target card, optional compact candidate/link ordinals, and
one state:

- `NotComparable`;
- `Matched`;
- `Conflicting`.

A record is comparable only when its common-owner card is `UniqueCandidate`,
that candidate's target state is `Unique`, and its incoming card is
`UniqueIncomingLink`. M7.4g then compares the uniquely matched candidate target
record ordinal with the source record ordinal retained by the incoming
ownership evidence. Equal ordinals produce `Matched`; unequal ordinals produce
`Conflicting`. Every other shape remains `NotComparable` and keeps both source
dimensions available for inspection.

`entry`, `candidate_for_entry`, `candidate_target_for_entry`,
`incoming_link_for_entry`, and `incoming_source_for_entry` are bounded exact
lookups. Construction validates both source identities, identical record-card
coverage, exact record agreement, cancellation, allocation failure, compact
ordinals, and single-element invariants. Storage and processing remain linear
in the two evidence directories and raw record count.

## Explicit non-claims

`Matched` is evidence agreement, not a validated or authoritative owner.
`Conflicting` records differing evidence but does not choose a winner.
`NotComparable` does not mean unowned or invalid. M7.4g does not enforce the
one-owner rule, validate record/target types, interpret dictionaries/reactors,
traverse a graph, diagnose cycles, apply lifecycle or purge behavior, clone,
edit, or write DXF.

## Antigravity evidence

Antigravity task 31 performed a read-only mechanical comparison-substrate
inventory at exact clean base `357aad6eee0364e4da5c4855ba293189f5afebd8`
after a fresh quota snapshot recommended Gemini 3.5 Flash (High). All six
commands exited zero, the common-owner and ownership-evidence suites passed
8/8, before/after HEAD and Git status stayed clean and identical, and no paths
were written. Its anchors established candidate target lookup, incoming-link
lookup, evidence ordinals, and incoming source-record access.

Codex independently reproduced the 8/8 focused tests and Git checks before
accepting task 31, then chose the comparison states, fail-closed eligibility,
and public API. Normative, semantic, support, audit, commit, and tag decisions
remained with Codex.

## Verification scope

Public integration tests cover ASCII/Binary parity across all nine supported
dialects; pre-R13 non-comparability where the modern reference codes are not
physically representable; exact matched and conflicting owner record ordinals;
candidate-only and incoming-only evidence; unresolved candidates; multiple
candidates; multiple incoming links; preservation of both source cards and
compact evidence ordinals; source identity; cancellation; bounded linear
reads; compact/copy/thread-safe values; bounds; and debug redaction.

## Reviewed artifact receipt

The reviewed production delta is 309 added lines: 304 lines in
`owner_evidence_comparison.rs` and five module/export lines in `lib.rs`. The
public integration test artifact adds 454 lines. There are no manifest,
dependency, lockfile, schema, generated-file, parser-framing, writer, or corpus
changes.

SHA-256 at review time:

- `lib.rs`: `A46888304BBEDA64C4453C7362CF4AE526059A56EAB37D16B79FBB5B69AF3C0C`
- `owner_evidence_comparison.rs`: `F84CD1CCCD4079D3DFD077A6441AF1C02936B62300FDDD741AAC63EEDB8CEE96`
- `owner_evidence_comparison_tests.rs`: `3A9F196440F1D1FF9BFA789345240AFDBA0EEE5D88B8EB9FC8A5C1E8C81AE445`
- `IMPLEMENTATION_PLAN.md`: `D613C4BC9A8EAE2114E4BEFFED74D7C5105F260AEADCBC94133633B0DCC42FB2`
- `SUPPORT_MATRIX.md`: `5CDD5E4AD384048F4948C5DD6FFED7F7746ED93EC1A996D0128EE6FA83538CC3`

The reviewed production and integration-test artifacts add no `unsafe`, panic
path, `unwrap`, `expect`, `todo`, or `unimplemented` use.

## Required gates

- focused M7.4g owner-evidence comparison suite: 4/4 passed;
- Antigravity substrate suites: 8/8 reproduced independently at the clean
  base;
- `cargo deny --locked check`: passed advisories, bans, licenses, and sources;
- `cargo +1.97.1 fmt --all -- --check`: passed;
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`: passed;
- `cargo +1.97.1 clippy --workspace --all-targets -- -D warnings`: passed;
- `cargo +1.97.1 test --workspace`: 299 tests passed (16 CLI, 6 corpus,
  204 core unit, 3 application-group, 4 common-owner-candidate,
  3 contextual-reference, 3 handle-identity, 3 handle-reference,
  3 handle-resolution, 6 handle-role, 4 HEADER-handle, 12 numeric, 4 text,
  4 owner-evidence-comparison, 4 ownership-evidence, 4 raw-handle,
  3 raw-record, and 13 schema-generator);
- `git diff --check`: passed.
