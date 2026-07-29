# M7.4b typed incoming ownership cardinality

M7.4b materializes a compact target entry for every M7.2a raw record over the
M7.4a target-sorted ownership-class links. Downstream work can inspect exact
zero, one, or multiple incoming uniquely resolved links without repeating
partition searches or conflating absence with invalid owner occurrences.

## Normative boundary

Autodesk states that an object can have only one owner and distinguishes owner
references from pointer references:

<https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-704F5152-B2A4-4DAC-A0FD-03D8ABFC0A4F.htm>

However, common entity and object records may identify their owner through
group code `330`, a soft pointer, while M7.4a indexes incoming ownership-class
codes `350..369`:

<https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm>

<https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-6939D69E-04CB-4F4C-87B2-67BC540FCF58.htm>

The new states are therefore explicitly named incoming-link cardinality. They
do not assert that a unique link is the complete legal owner or that multiple
links already constitute a conformance failure.

## Public contract

`DxfOwnershipEvidenceDirectory` now exposes one source-order
`DxfOwnershipTargetEntry` per raw record. Each entry contains the exact
`DxfRawRecord`, a `DxfOwnershipLinkRange` into the shared resolved-link array,
and one `DxfIncomingOwnershipState`:

- `NoIncomingLink` for an empty unique-link slice;
- `UniqueIncomingLink` for exactly one uniquely resolved incoming owner-class
  occurrence;
- `MultipleIncomingLinks { link_count }` for two or more.

`target_entry(record_ordinal)` is exact and bounded.
`incoming_links_for_target_record` now slices through the stored half-open
range rather than repeating two partition searches. The entries include every
raw record, even when its own identity is absent, invalid, duplicate, or null.

Construction performs one linear pass over record identity entries and one
monotonic cursor over target-sorted resolved links. It fails closed if any
resolved link is not consumed. The range and count use bounded `u32` storage;
public accessors return widened `u64` ordinals and lengths.

## Explicit non-claims

M7.4b does not interpret record types, application control groups, dictionaries,
reactors, extension dictionaries, or outgoing group-code `330` owner pointers.
It does not select an owner, enforce the one-owner rule, classify orphaned
records, apply hard/soft purge behavior, diagnose cycles, traverse a graph, or
mutate and write records.

## Antigravity evidence

Task 26 was issued for a read-only mechanical substrate check at exact M7.4a
HEAD, with Gemini 3.5 Flash (High) recommended from fresh sanitized quota.
Antigravity claimed the task against exact HEAD
`207cb1d937be473924ca43c2c406b740ac0a73d6` while Codex's M7.4b diff was
present, reported identical before/after status, wrote no paths, and returned
all six assigned commands at exit code zero. Its focused suite reported 4/4
tests passed and its exact anchors covered the sorted-link array, bounded target
lookup, and zero/one/multiple cases. Codex independently reproduced the focused
result and every required workspace gate before accepting task 26 as done.

The worker evidence is mechanical only. API, normative, support, audit, commit,
and tag decisions remained with Codex.

## Verification scope

The ownership integration suite covers one target entry per raw record,
ASCII/Binary parity across all nine supported dialects, pre-R13 empty ranges,
exact record ordinals and half-open slices, zero/one/multiple states, bounds,
compact/copy/thread-safe public values, pointer exclusion, and all unresolved
owner-occurrence behavior already covered by M7.4a.

## Reviewed artifact receipt

The reviewed production delta adds 130 lines and removes 11: 128 additions and
10 removals in `ownership_evidence.rs`, plus two additions and one removal in
`lib.rs`. The integration test delta adds 51 lines and removes two. There are
no manifest, dependency, lockfile, schema, generated-file, parser-framing,
writer, or corpus changes.

SHA-256 at review time:

- `lib.rs`: `056BB1C1A823AFF1EA73A621392E092324E1FE141FF699B085AF08F4ED30C398`
- `ownership_evidence.rs`: `81F9EF0ACCACA4C6DAC6897EB693ABF048EC78E02AE0051538065C6E47995BD2`
- `ownership_evidence_tests.rs`: `8D6F518D962A315999F0D13DB74C45558AF45842165812377B698D2B566FA0FA`
- `IMPLEMENTATION_PLAN.md`: `EDB3C7136349F173A50F1A88E4253AE01C9852A435B548AD3CBD792E6D9D437E`
- `SUPPORT_MATRIX.md`: `3DB4AF55B27C3E6A5ABA9D2BBAFE9A2BC7F06F47F48702663ACF62EDF79A9110`

The reviewed production and integration-test delta adds no `unsafe`, panic
path, `unwrap`, `expect`, `todo`, or `unimplemented` use.

## Required gates

- focused M7.4 ownership-evidence suite: 4/4 passed;
- `cargo deny --locked check`: passed advisories, bans, licenses, and sources;
- `cargo +1.97.1 fmt --all -- --check`: passed;
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`: passed;
- `cargo +1.97.1 clippy --workspace --all-targets -- -D warnings`: passed;
- `cargo +1.97.1 test --workspace`: 279 tests passed (16 CLI, 6 corpus,
  204 core unit, 3 handle-identity, 3 handle-reference, 3 handle-resolution,
  4 HEADER-handle, 12 numeric, 4 text, 4 ownership-evidence, 4 raw-handle,
  3 raw-record, and 13 schema-generator);
- `git diff --check`: passed.
