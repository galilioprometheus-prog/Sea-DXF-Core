# SeaCad

SeaCad is a proprietary, clean-room CAD kernel project. Its first release target
is a lossless and resource-bounded DXF core for ASCII and Binary DXF AC1009
through AC1032.

## Current status

Release-evidence implementation is complete through M13.2g and entity-semantic
expansion is complete through M14.3co. SeaCad opens bounded lossless ASCII
and Binary DXF AC1009 through AC1032, preserves exact source identity and raw
evidence, exposes the reviewed HEADER/record/entity semantics and geometry,
plans reversible handle and unique common-field edits, writes verified
preserve-patch outputs, and
emits strictly reparsed canonical ASCII or Binary framing. Dependency policy,
deterministic CycloneDX inventory, distributable legal files, six-native CI
staging, a redacted 1,000-file/10-GiB corpus release gate, and a manual
six-target native artifact assembly workflow, exact six-artifact receipt
aggregation, a host-independent six-platform SBOM dependency union, actionable
stale-evidence diagnostics, canonical-LF Cargo.lock identity, and one
successful six-native artifact/aggregate workflow receipt are present. Private
corpus achievement, twenty consecutive six-native nightly receipts,
signatures, and final Core 1.0 authorization remain open M13 evidence.
The generated schema freezes the complete reviewed HEADER inventory, all 45
canonical Autodesk entity topics, and 14 reviewed alias/specialization wire
names. A generated 59-name by nine-dialect applicability matrix keeps
unreviewed ranges explicit; 31 Autodesk-sourced names now have reviewed ranges.
That set includes the 16 exact pre-Release-13 entity names, ACAD_TABLE, HELIX,
LIGHT, LWPOLYLINE, MESH, MLEADER, SECTIONOBJECT, five surface specializations,
and three underlays. None of this inventory metadata implies semantic support.
The unified entity directory indexes exact canonical, alias, unknown, and
reviewed wrong-section markers in source order across `BLOCKS` and `ENTITIES`,
while retaining exact subclass paths and all untouched raw groups.
A generated common-field registry now exposes 19 Autodesk-backed property
roles with wire type, cardinality, default, subclass/application scope,
coordinate-space classification, version-review state, provenance, and a
separate canonical writer-order ordinal. The writer metadata follows the
usual Autodesk common-code presentation while readers remain completely
order-independent.
A generic common-field evidence directory retains exact source occurrences and
fixed per-entity cardinality cards without decoding or selecting duplicates.
Typed common-field semantics now project all 19 roles through the shared
four-state value model: exact numbers and handles are decoded fail-closed,
exact text remains source-backed, reviewed omitted defaults remain explicit,
and proxy graphics stay an opaque occurrence sequence. Duplicate singletons
are never selected.
The CRUD foundation now includes a typed common-field value encoder. It emits
canonical ASCII groups or dialect-correct Binary groups for exact raw text,
handles, finite binary64, Int16, Int32, and bounded opaque chunks; invalid
wire/value combinations and AC1009-inexpressible group codes fail typed before
any document edit is planned.
Source-bound entity keys now drive explicit replacement planning for an
existing unique common-field singleton. The planner replaces exactly one raw
group through the immutable transaction/inverse kernel; absent fields require
a later insertion-anchor operation, duplicates are never selected, and opaque
sequences require specialized edits.
Optional singleton reset planning can also delete one exact unique raw group
to restore its generated implicit default. Required fields, duplicates,
sequences, and nested extension-dictionary members fail typed. Canonical
common-field insertion-anchor planning now locates one exact between-group
byte offset for an absent singleton. It respects preamble/application-group
envelopes, exact `AcDbEntity` scope, writer order, unknown groups, and legacy
AC1009 records without editing the source.
An absent singleton can now be encoded at that anchor into one immutable
zero-length-source transaction patch. ASCII insertion preserves the preceding
group's LF, CR, or CRLF ending; Binary insertion uses the declared dialect.
Strict post-images and executable inverse plans prove byte-identical source
restoration. A source-bound `DxfEntityEditSession` now batches explicit common-
field set/reset requests across entities into one immutable transaction. It
rejects a second queued edit for the same field and merges same-anchor
insertions in generated writer order. Sequence/nested operations, family
patches, insert/clone/delete, and complete CRUD
remain later checkpoints. `finish_verifiable` can now retain payload-redacted
semantic postconditions beside the transaction. An independently opened post-
image must match each edited field's cardinality, state, and typed value before
the exact raw verifier releases an executable inverse journal. The same plan
can now stream to a new path, strictly reparse the independently opened output,
verify those semantic postconditions, and return the write/semantic receipts
with an executable inverse. Any strict, semantic, raw, or cancellation failure
after creation removes the destination; an existing path is never modified.
Before an explicit common-field edit is planned, a pure typed classifier now
validates the reviewed scalar domains for model/paper space, indexed color,
lineweight, nonnegative linetype scale, visibility, nonnegative proxy byte
count, 24-bit true color, and shadow mode. Invalid values remain typed and do
not enter the edit queue; fields whose domain is not yet reviewed remain
explicitly distinguishable and continue through the existing wire validator.
Those eight domains now project over already-open raw documents without
discarding source evidence. Explicit and defaulted valid values become typed
domain values; raw decode, duplicate, missing-required, and domain failures
remain distinct typed issues with provenance. The other common fields pass
through their exact existing semantic state and are not claimed as reviewed.
The five common handle-valued fields now have a source-bound projection. Group
5 remains lexical object identity, while owner, extension dictionary, material,
and plot-style references retain absent/defaulted/invalid state and distinguish
null, missing, unique, and ambiguous document-local targets. Unique lookup is
not promoted to target-kind, ownership, dictionary, or lifecycle validity.
The four common exact-text fields now have one source-bound projection. Layer
and linetype perform duplicate-preserving exact lookup against completely
closed `LAYER` and `LTYPE` tables; missing and ambiguous names are typed, while
linetype omission remains the schema `BYLAYER` default. Layout group 410 now
resolves exactly against group 1 in the `AcDbLayout` subclass of closed
`OBJECTS`/`LAYOUT` records; missing, duplicate, and ambiguous target names stay
typed. Color-name group 430 now parses the documented exact
`colorbook$colorname` envelope into two source spans, rejects missing, empty,
or multiple separators without guessing, and composes with the reviewed group
420 true-color and group 62 indexed-color semantics.
Proxy graphics now have a source-bound size relation. Group 310 stays opaque;
ASCII hex is validated and counted as decoded bytes, while Binary uses its
already-framed raw payload length. The projection distinguishes absence,
missing size, exact match, count mismatch, invalid size, and the first malformed
ASCII chunk without concatenating or interpreting proprietary data.
Common transparency group 440 is now a ninth reviewed scalar domain. It
projects exact ByLayer, ByBlock, and ByAlpha encodings, retains alpha 0–255,
and rejects unsupported method bytes or reserved payload bits. Valid explicit
edits use the same verified transaction and byte-identical inverse pipeline.
The three common references with Autodesk-defined public target kinds now have
a second source-bound projection. Extension dictionary, material, and plot
style require exact `OBJECTS` records marked `DICTIONARY`, `MATERIAL`, and
`ACDBPLACEHOLDER`; unique targets of another marker or section fail typed.
Owner remains explicitly unreviewed because its valid target kind depends on
entity family and placement.
Generic singleton edits can no longer mutate handle identity or owner links;
those require handle-remap and placement operations. Explicit extension-
dictionary, material, and plot-style edits enter a session only when their
non-null handle resolves uniquely to the reviewed target kind. Rejected
references never enter the transaction queue.
Layer and linetype edits now require one exact same-document name in a closed
matching symbol table. Missing or duplicate names fail before planning.
Layout and color-book names remain locked behind dedicated resolvers instead
of being accepted as unchecked raw text.
The layout resolver ignores plot-settings group 1,
application groups, wrong subclasses, wrong sections, and malformed layout-name
cardinality while retaining their evidence. Layout edits now enter the session
only after one exact same-document layout object resolves; rejected edits queue
nothing, while accepted edits use strict semantic verification and exact inverse
restoration. Color-book edits now require the documented one-separator envelope
and usable existing group-420/group-62 semantics on the exact target entity.
Accepted modern edits use the same strict semantic verification and exact
inverse pipeline. The resolver does not load or validate an external `.acb`
file. A typed composite patch can now insert or replace the complete
62/420/430 tuple atomically; failure rolls the session queue back to its exact
prior state, and accepted modern tuples pass strict semantic verification and
byte-identical inverse restoration. A matching composite reset removes every
explicit tuple member as one logical request, restores indexed color to its
reviewed BYLAYER default, leaves true color and color name absent, and rolls
back partial planning without disturbing earlier unrelated edits.
Independent source-bound transaction plans can now be composed into one
resource-bounded, conflict-checked, source-ordered transaction. Verifiable
entity edit plans retain their semantic postconditions while absorbing raw
handle, owner, or placement work; paired coverage proves existing M11 handle
assignment and `$HANDSEED` replacement can commit atomically with an M14 field
edit and one executable inverse.
Entity-bearing containers now expose source-bound insertion placements. Each
closed `ENTITIES` section anchors before its first content group, and each
closed BLOCK definition anchors before its first member or exact `ENDBLK`.
Interrupted/unclosed containers and orphan nonzero content fail typed instead
of attaching unknown groups to a newly inserted entity. The placement contract
selects no owner and emits no draft bytes yet.
A ready handle policy can now reserve a bounded consecutive range for records
that have not been inserted yet and pair it with the exact successor
`$HANDSEED` replacement. The reservation remains a source-bound transaction,
not a global lock; composing it with an entity placement proves one new
identified record, strict reparse, and byte-identical inverse across all
dialects and both physical formats.
The first typed family draft now emits a complete canonical `POINT` record for
ASCII and Binary AC1009 through AC1032. It composes the reserved handle,
caller-bound owner, placement-conditioned modern layout, exact existing layer,
explicit AC1015+ lineweight, reviewed subclass envelope, and finite WCS
location. `ENTITIES` layout and symbol references must resolve uniquely in the
same document; BLOCK-local records omit layout. Successful record plans compose
with insertion and `$HANDSEED`, strict-reparse in both physical formats, expose
the expected POINT semantics, and restore the source byte-identically through
their inverse. This proves POINT record creation, not the unified insert API or
full POINT CRUD/`Complete` status.
That record can now be consumed by `plan_entity_draft_insert`, which atomically
composes its reserved `$HANDSEED` update with the exact container insertion and
returns the existing verified `DxfEntityEditPlan`. Post-image verification
resolves the allocated handle uniquely, checks canonical POINT classification
and exact ENTITIES/BLOCK placement, then proves owner and explicit common
fields plus the WCS location before releasing an inverse. The same plan uses
the create-new write, strict-reparse, cleanup, and journal pipeline for all
nine ASCII/Binary dialect pairs. Session-level insert batching and the rest of
POINT CRUD remain later checkpoints.
`DxfEntityEditSession::insert(placement, draft)` now exposes that POINT path
through the unified CRUD surface. The typed draft carries the caller-selected
BLOCK_RECORD owner, and the session atomically performs owner binding, one
handle reservation, dialect applicability, canonical record encoding, and the
verified insertion plan. `finish` returns the raw transaction while
`finish_verifiable` retains the POINT postcondition. That checkpoint admitted
one new record and rejected update/insert mixing pending multi-record
allocation and ordinal-independent mixed verification.
An edit session now admits multiple typed POINT drafts. Each successful call
owns its encoded record and semantic expectation, while `finish` creates one
shared consecutive handle reservation for the whole batch. Records targeting
the same zero-width anchor are emitted in caller/handle order as one insertion
patch, and `$HANDSEED` advances once to the exact successor. Rejected drafts do
not consume a handle. Insert/update mixing remains fail-closed.
POINT drafts now cover the complete public family payload: optional thickness,
extrusion direction, and UCS X-axis angle join the required WCS location.
Readers retain out-of-order and duplicate evidence, apply only Autodesk's
documented zero and `(0, 0, 1)` defaults, and keep invalid optional values
typed. Writers omit unspecified defaults, emit explicitly requested values in
canonical subclass order, reject zero extrusion, and verify explicit/defaulted
state plus exact binary64 values after strict reparse. POINT update, clone, and
delete remain open.
Canonical POINT locations can now be replaced atomically through
`DxfEntityEditSession::update`. The family patch requires one unique existing
group `10/20/30` tuple, encodes all three finite WCS components for the source
dialect and format, and composes with independent common-property edits.
Post-image verification resolves the same raw-record ordinal, checks the exact
typed location, and releases the existing byte-identical inverse journal.
Duplicate family patches, incomplete/duplicate tuples, wrong families,
non-finite values, and cancellation fail closed. Mixed insert/update support is
described below. Other POINT fields, clone, and delete remain open.
POINT thickness can now be set atomically through a distinct
`DxfPointPatch::SetThickness` request whether group `39` is uniquely explicit
or absent under its documented zero default. A unique occurrence is replaced;
an absent occurrence is inserted immediately after the last unique source
location component while preserving the source ASCII line ending or Binary
wire. Duplicate thickness and missing/duplicate insertion-anchor location
evidence fail typed without changing the session. Location and thickness remain
independent logical edits, and strict verification requires the exact value in
the `Explicit` state before releasing the byte-identical inverse. The same
logical patch kind can reset a unique explicit group `39` by deleting its exact
source span; an absent group is an `AlreadyImplicit` no-op. Reset verification
requires the documented zero value in the `Defaulted` state before releasing
the byte-identical inverse. Duplicate thickness remains unselectable.
Extrusion/angle updates, clone, and delete remain open.
POINT extrusion can now be set atomically for every absent/unique component
mask. A complete explicit tuple replaces its three source spans; a completely
absent tuple inserts one canonical `210/220/230` sequence; and a partial tuple
replaces each explicit component while inserting each consecutive missing run
at its canonical gap. Partial completion reports `Composite`, preserves local
ASCII endings or Binary framing, and rejects duplicate or source-reordered
partial evidence without guessing. Strict verification requires the exact
nonzero tuple with all three components in the `Explicit` state before exposing
the byte-identical inverse. The same logical patch identity can now reset any
complete or partial explicit tuple by deleting every unique present component;
a fully absent tuple is an `AlreadyImplicit` no-op. Reset verification requires
the documented `(0,0,1)` value with all three components `Defaulted`.
POINT UCS X-axis angle group `50` can now be replaced when unique or inserted
after the last unambiguous extrusion, thickness, or location anchor. It has a
distinct logical patch identity, preserves ASCII endings or Binary framing,
and verifies the exact value in the `Explicit` state. The same patch identity
can reset a unique explicit angle by deleting group `50`; an absent angle is an
`AlreadyImplicit` no-op that does not reserve the patch identity. Reset
verification requires the documented zero value in the `Defaulted` state, and
the inverse restores every original byte. One session can now mix existing-
entity common/POINT updates with one or more POINT insertions in either API
order. Handle reservation, `$HANDSEED`, source patches, record insertions, and
all semantic postconditions commit as one reversible transaction; insertions
before an updated record adjust its verification ordinal deterministically.
POINT now has a first reference-safe whole-record delete operation. The session
admits one standalone canonical POINT only when its non-null handle identity is
unique and no uniquely resolved pointer or owner from another record targets
it. Admission deletes the exact group-zero-delimited raw record; strict
post-image verification requires the handle to disappear before returning the
byte-identical inverse. Missing, invalid, null, or ambiguous identity,
incoming references, wrong families, cancellation, and operation mixing all
fail closed. POINT can also be cloned into a fresh reserved handle through the
same verified insertion path when the source uses only the canonical groups
currently modeled by the POINT draft. The clone preserves layer, applicable
layout/lineweight, exact location, and explicit-versus-defaulted optional
payload. It stays in the source container; modern owner identity must match.
Unknown common properties, XDATA/application groups, partial or invalid POINT
semantics, a pending source update, placement drift, and owner drift fail
closed rather than losing data. Handleless canonical POINT records can now be
deleted through a distinct typed outcome: exact raw removal is paired with a
strict post-image entity-count postcondition and byte-identical inverse across
all supported ASCII/Binary dialects. Invalid, null, multiple, or ambiguous
identity still fails closed. Delete-only sessions can now compose multiple
distinct reference-safe POINT removals, including mixed handle-backed and
handleless identities, into one bounded transaction with strict per-handle and
final entity-count postconditions. Duplicate keys fail without discarding the
accepted batch. Deletes now compose with unrelated POINT/common updates and new
POINT inserts in either API order. Existing-record semantic ordinals account
for both prior insertions and removed records, while handleless deletion
verifies the final source-minus-deletes-plus-inserts entity count. Updating or
cloning a selected delete target remains typed fail-closed. POINT clone now
also preserves explicit paper/model-space, indexed color, linetype scale,
visibility, true color, transparency, and shadow-mode scalar properties through
typed draft encoding and semantic post-image verification across every Core
dialect. Version-invalid scalars, duplicate/invalid values, proxy graphics,
XDATA, extension dictionaries, and ownership graphs still fail closed rather
than being copied incompletely.
POINT clone also preserves an explicit same-document linetype name only when it
resolves to one exact LTYPE table entry; canonical encoding and post-image
verification retain the exact source bytes across every Core dialect. On
AC1012+, clone also preserves same-document material and plot-style handles only
when they resolve uniquely to the required MATERIAL and ACDBPLACEHOLDER object
kinds. Both references are revalidated after strict reparse. M14.3ci also
preserves the AC1012+ indexed/true/color-book tuple as one validated relation:
group 430 must have exactly one non-edge `$` separator and requires explicit,
valid groups 62 and 420. Canonical encoding and post-image verification retain
all three values; graph common-property clone plus POINT `Complete` remain open.
M14.3cj preserves matched AC1012+ proxy graphics as opaque decoded bytes:
group 92 must equal the combined group-310 payload length, ASCII hex is
validated and decoded, Binary payloads remain byte-exact, and clone output is
canonically rechunked and semantically compared. Graph-scoped handles,
application groups/XDATA, and POINT `Complete` remain open.
M14.3ck hardens POINT deletion against attached record-local graphs. Any group
102 application control or group 360 hard-owner occurrence now rejects the
delete with its exact source occurrence before a transaction is queued, so
persistent-reactor, extension-dictionary, custom, malformed, and unscoped
ownership payload cannot be orphaned by the standalone-record delete path.
Graph-aware cascade/remap, graph clone, XDATA clone, and POINT `Complete`
remain open.
M14.3cl adds one format-neutral, source-anchored entity XDATA directory.
Every group 1001 starts a distinct registered-application list, following
groups 1000 through 1071 retain exact source order, duplicate application names
remain separate, and values without a current group 1001 remain typed orphans.
A normal entity group interrupts the active list, while XDATA-shaped codes
inside group-102 application controls remain outside XDATA. APPID resolution,
brace/value validation, payload interpretation, clone/write, and POINT
`Complete` remain open.
M14.3cm resolves every entity XDATA group-1001 name against exact group-2
names admitted from completely closed APPID tables. A collision-safe digest
index narrows candidates, then authoritative source spans are compared
byte-for-byte. Missing, unique, and ambiguous targets remain distinct; near
case, wrong-table, malformed, and unclosed evidence fail closed. Application
name syntax, brace/value validation, the 16-KiB policy, payload meaning,
clone/write, and POINT `Complete` remain open.
M14.3cn validates the documented XDATA application-name byte ceiling and
group-1002 list structure. Only exact `{` and `}` controls are accepted;
nested lists must balance, premature closes, leftover opens, invalid controls,
and normal-group interruptions remain separate source-anchored issues. Symbol
name character policy, typed values, 16-KiB enforcement, payload meaning,
clone/write, and POINT `Complete` remain open.
M14.3co projects every documented generic XDATA value code into a typed,
source-anchored result. Exact strings and controls, caller-buffer binary chunks,
handles, all 15 double roles, signed 16-bit integers, and signed 32-bit integers
have ASCII/Binary parity across all nine Core dialects; malformed or unsupported
values remain explicit without normalization. Layer-name resolution, point and
vector tuple grouping, the 16-KiB application policy, handle remap, payload
meaning, clone/write, and POINT `Complete` remain open.
SPLINE now exposes an analytic-readiness projection that composes exact knots,
weighted WCS control/fit points, degree, declared counts, knot order and
multiplicity, active parameter domain, flags, optional tangents, and planar
normal requirements. Failures accumulate as typed issues and never publish a
partial curve; evaluation and tessellation remain out of scope.
HELIX now has a source-anchored subclass evidence directory for all 16 public
`AcDbHelix` roles. It preserves duplicates and invalid numbers without
selection and prevents colliding SPLINE, application-group, unknown-subclass,
or wrong-section values from entering HELIX evidence.
HELIX cardinality publishes exactly 16 stable cards per record with absent,
unique, or multiple states and exact member-to-value links. Invalid numeric
syntax remains a unique occurrence rather than being mistaken for absence.
Seven HELIX scalar roles now use the shared provenance-bearing semantic value
model. Public handedness and constraint domains are typed, non-finite doubles
fail closed, and undocumented positivity/default policies are not inferred.
HELIX axis base, start point, and axis vector now expose three exact WCS
component semantics each. A tuple is available only when all three components
are usable; missing Y/Z is not silently defaulted.
HELIX relation semantics now normalize a usable nonzero axis, retain the exact
orthogonality residual, derive the base radius from axis-base/start geometry,
classify the stored radius and turns domains, and derive axial height from
turns and turn height. Zero-height flat helices remain observable.
HELIX analytic readiness now composes that metadata with the exact embedded
`AcDbSpline` projection on the same raw record. It requires one ordered
`AcDbSpline`/`AcDbHelix` subclass pair, a valid embedded curve, perpendicular
axis geometry, nonnegative radius, positive turns, and finite derived height;
failures accumulate without publishing partial analytic data. Evaluation and
tessellation remain out of scope.
Budget-aware development runs required quality/dependency gates locally and
does not allocate hosted runners for pushes or pull requests. GitHub Actions
retains a manual Windows x64 self-hosted diagnostic and the manual six-native
hosted release-artifact workflow only.
The remaining documented entity families are tracked in
`docs/DXF_ENTITY_COMPLETION_PLAN.md`; raw preservation does not imply complete
typed semantics for every entity.

## Workspace

- `seacad-dxf-core`: bounded lossless ASCII/Binary framing, source identity,
  reviewed lazy semantics/topology/geometry, reversible transaction planning,
  and verified preserve-patch/canonical create-new writers.
- `seacad-cli`: operational `inspect` and `verify` commands with English or
  Vietnamese human output sourced from per-locale compile-time catalogs and
  stable JSON v1, plus the separate aggregate-only `seacad-corpus-receipt`
  offline evidence harness.
- `seacad-schema-gen`: deterministic provenance-backed schema generation,
  CycloneDX inventory, legal-bundle generation, cross-platform freshness
  verification, fail-closed native artifact assembly, and six-target receipt
  verification.

The previous CAD workspaces under `D:\Backups` are immutable research inputs.
Production code is not copied from them. Tests, fixtures, and knowledge may be
transferred only after the M1 provenance audit.

## Build

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

The repository pins Rust 1.97.1 with rustfmt, Clippy, and LLVM tools.

## Current CLI

```text
seacad inspect drawing.dxf
seacad verify drawing.dxf
seacad inspect drawing.dxf --lang vi
seacad verify drawing.dxf --json
```

English is the default. `--lang vi` localizes human help and reports. JSON
keys, statuses, and codes remain stable English identifiers. Paths are hidden
unless `--show-path` is explicit. See `docs/CLI_JSON_V1.md` for the complete
contract and `docs/SUPPORT_MATRIX.md` for the exact support boundary.

Private corpus bytes stay outside the repository. The Q2.2 harness reads the
public aggregate-only policy in `corpus/offline-manifest.json` and emits no
paths, filenames, source IDs, or per-file hashes. See
`docs/Q2_2_OFFLINE_CORPUS_RECEIPT_CONTRACT.md`.
M13 release qualification uses the separate threshold policy in
`corpus/release-manifest.json`; policy presence is not evidence that a private
corpus has passed.

## License

Copyright (c) 2026 SeaCad. All rights reserved. See `LICENSE`.
