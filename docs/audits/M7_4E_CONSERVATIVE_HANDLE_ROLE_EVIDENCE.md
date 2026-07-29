# M7.4e conservative handle-role evidence

M7.4e overlays conservative semantic-role evidence on every M7.4d contextual
handle reference. It preserves the exact raw occurrence, numeric reference
class, application-group evidence, and target-resolution state rather than
turning a candidate shape into a validity or topology claim.

## Normative boundary

Autodesk's common entity and common object contracts place a `330` persistent
reactor handle inside `{ACAD_REACTORS`, a `360` extension-dictionary handle
inside `{ACAD_XDICTIONARY`, and an ordinary `330` owner pointer outside those
application groups:

<https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm>

<https://help.autodesk.com/cloudhelp/2025/ENU/AutoCAD-DXF/files/GUID-6939D69E-04CB-4F4C-87B2-67BC540FCF58.htm>

The numeric group-code classes remain independent evidence:

<https://help.autodesk.com/cloudhelp/2018/ENU/OARX-RefGuide/files/OREF-DXF_Group_Codes.html>

M7.4e therefore emits candidate labels only for exact documented shapes. It
does not infer applicability to an uninterpreted record type or convert
malformed application groups into specialized roles.

## Public contract

`DxfRawDocumentView`, `DxfAsciiRawDocument`, and `DxfBinaryRawDocument` expose
`handle_role_directory(cancellation)`. The immutable directory owns one shared
M7.4d contextual-reference directory and one source-order
`DxfHandleRoleEntry` per contextual reference.

Each entry retains the complete `DxfContextualHandleReferenceEntry` and one
`DxfHandleRoleEvidence`:

- `GenericPointer`;
- `GenericOwnership`;
- `PersistentReactorCandidate`;
- `ExtensionDictionaryCandidate`;
- `CommonOwnerPointerCandidate`.

Specialized candidates require one of the complete common record-bearing
sections `TABLES`, `BLOCKS`, `ENTITIES`, or `OBJECTS`. Reactor candidates
require exact code `330`, soft-pointer class, `{ACAD_REACTORS` context, and a
closed application group. Extension-dictionary candidates require exact code
`360`, hard-owner class, `{ACAD_XDICTIONARY` context, and a closed application
group. Common owner-pointer candidates require exact code `330`, soft-pointer
class, and no application-group context. All other pointer and owner classes
fall back to their generic evidence.

`entry`, `entry_for_group`, and `entries_for_record` are bounded lookups.
Construction scans the shared contextual entries once, checks cancellation and
source identity, reports allocation/compact-index failures, and retains linear
storage. Target resolution is not recomputed or folded into the role.

## Explicit non-claims

Candidate evidence is not record-type validation and is not proof that the
target exists or has the required object type. M7.4e does not reconcile common
owner pointers with incoming ownership-class references, select an
authoritative owner, enforce the one-owner rule, validate dictionary or reactor
payloads, traverse graph edges, diagnose cycles, apply hard/soft lifecycle or
purge behavior, clone, edit, or write DXF.

## Antigravity evidence

Antigravity task 29 performed a read-only mechanical substrate inventory at
base HEAD `65981760c258704d858152968030751fb55360f9`. Its six commands exited
zero, the combined application-group/context suites passed 6/6, its reported
anchors covered numeric class, lexical context, group state, and raw record
section kind, and Agent Hub reported no written paths.

There was one documented coordination deviation: the claim event captured a
clean worktree, but raw command evidence later observed Codex-owned concurrent
M7.4e changes to `lib.rs` and `handle_role.rs`. Codex therefore rejected the
worker's high-level clean-before/after statement, accepted only its mechanical
anchors and test result, reproduced the assigned 6/6 tests and Git checks
independently, and kept every semantic, API, support, commit, and tag decision.

## Verification scope

Public integration tests cover ASCII/Binary parity across all nine supported
dialects, exact role/code/class/context/closure matching, fallback for
interrupted and unclosed groups, all four common record-bearing sections,
non-applicability in `CLASSES`, independent invalid/null/missing/unique/
ambiguous target states, source-order/per-record/per-occurrence lookup,
cancellation, bounded linear source reads, bounds, compact/copy/thread-safe
values, and debug redaction.

## Reviewed artifact receipt

The reviewed production delta is 264 added lines: 262 lines in
`handle_role.rs` and two module/export lines in `lib.rs`. The public integration
test artifact adds 456 lines. There are no manifest, dependency, lockfile,
schema, generated-file, parser-framing, writer, or corpus changes.

SHA-256 at review time:

- `lib.rs`: `9A0C19545CF1D6151CD11A7FD73E298696429B757028F75D20554225D2D1C3A7`
- `handle_role.rs`: `43AB08F3C25423E7EA697DA63909E73444795259281C660E207148D9592F40AD`
- `handle_role_tests.rs`: `317A8455151C80F5695BDFFD8908C6BEFF044C5F564445D794DE923711FB1B8B`
- `IMPLEMENTATION_PLAN.md`: `3FB7C758B1D95432BFF65D3554692D2C62C2B4C67BDF731317F60EA8733A7029`
- `SUPPORT_MATRIX.md`: `280F52199EDDE6B3F4C7B64D58CF9B7FA3B2923B6D92043EAC0F766CC0B8469C`

The reviewed production and integration-test artifacts add no `unsafe`, panic
path, `unwrap`, `expect`, `todo`, or `unimplemented` use.

## Required gates

- focused M7.4e handle-role suite: 6/6 passed;
- Antigravity substrate suites: 6/6 reproduced independently with the
  documented worktree-status deviation;
- `cargo deny --locked check`: passed advisories, bans, licenses, and sources;
- `cargo +1.97.1 fmt --all -- --check`: passed;
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`: passed;
- `cargo +1.97.1 clippy --workspace --all-targets -- -D warnings`: passed;
- `cargo +1.97.1 test --workspace`: 291 tests passed (16 CLI, 6 corpus,
  204 core unit, 3 application-group, 3 contextual-reference,
  3 handle-identity, 3 handle-reference, 3 handle-resolution, 6 handle-role,
  4 HEADER-handle, 12 numeric, 4 text, 4 ownership-evidence, 4 raw-handle,
  3 raw-record, and 13 schema-generator);
- `git diff --check`: passed.
