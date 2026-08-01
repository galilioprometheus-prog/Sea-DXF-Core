# DXF Entity Semantic Completion Plan

## Goal

DXF Core completion requires more than lossless framing. For every documented
graphical entity in the AC1009--AC1032 scope, SeaCad must retain all public
group-code evidence, expose typed cardinality and semantic states, and provide
exact geometry where the public DXF contract defines enough information.
Proprietary payloads remain bounded opaque bytes with a typed public envelope.

The existing raw record layer already preserves unknown, custom, proxy, and
future records. This plan closes documented entity semantics without weakening
that forward-compatible boundary.

## Normative inventory

The Autodesk DXF entity reference lists 45 public entity topics:

`3DFACE`, `3DSOLID`, `ACAD_PROXY_ENTITY`, `ARC`, `ATTDEF`, `ATTRIB`,
`BODY`, `CIRCLE`, `DIMENSION`, `ELLIPSE`, `HATCH`, `HELIX`, `IMAGE`,
`INSERT`, `LEADER`, `LIGHT`, `LINE`, `LWPOLYLINE`, `MESH`, `MLINE`,
`MLEADERSTYLE`, `MLEADER`, `MTEXT`, `OLEFRAME`, `OLE2FRAME`, `POINT`,
`POLYLINE`, `RAY`, `REGION`, `SECTION`, `SEQEND`, `SHAPE`, `SOLID`,
`SPLINE`, `SUN`, `SURFACE`, `TABLE`, `TEXT`, `TOLERANCE`, `TRACE`,
`UNDERLAY`, `VERTEX`, `VIEWPORT`, `WIPEOUT`, and `XLINE`.

Normative references:

- Autodesk, About the DXF ENTITIES Section:
  <https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-7D07C886-FD1D-4A0C-A7AB-B4D21F18E484.htm>
- Autodesk, About Object and Entity Codes:
  <https://help.autodesk.com/cloudhelp/2019/ENU/AutoCAD-DXF/files/GUID-A35B8C2A-1885-4A8E-8533-E61D8A423D62.htm>

On-wire aliases and specializations written by supported AutoCAD releases are
part of the same inventory. The reviewed AutoCAD 2027 oracle observed examples
including `ACAD_TABLE`, `ARC_DIMENSION`, `LARGE_RADIAL_DIMENSION`,
`MULTILEADER`, `PDFUNDERLAY`, `SECTIONOBJECT`, `PLANESURFACE`,
`EXTRUDEDSURFACE`, `LOFTEDSURFACE`, `REVOLVEDSURFACE`, and
`SWEPTSURFACE`. Every alias requires an explicit public-topic mapping or an
explicit opaque classification; filename or record position is never semantic
evidence.

## Completion levels

Each entity family advances independently through:

1. exact marker and section-scoped source evidence;
2. fixed per-role cardinality without selecting duplicates;
3. typed values, documented defaults, flags, and structural validation;
4. exact coordinate-system and geometry semantics where publicly specified;
5. source-bound edit planning and verified ASCII/Binary materialization;
6. dialect fixtures, malformed fixtures, private-corpus accounting, and all
   six native CI targets.

An entity is not called semantically complete at levels 1 or 2. Rendering,
tessellation, external file resolution, fonts, raster decoding, and proprietary
modeler interpretation remain separate capabilities.

## Current baseline

Deep typed semantics already cover `POINT`, `LINE`, `CIRCLE`, `ARC`,
`ELLIPSE`, `RAY`, `XLINE`, `LWPOLYLINE`, classic
`POLYLINE`/`VERTEX`/`SEQEND`, `INSERT`, `ATTRIB`, and `ATTDEF`, plus
`BLOCK`/`ENDBLK` topology. M14 starts the missing public entity inventory.
M14.1a retains numeric occurrence evidence for `3DFACE`, `SOLID`, and `TRACE`;
M14.1b adds fixed family-specific cardinality without selecting values; and
M14.1c selects unique typed values, applies only documented defaults, and keeps
missing, invalid, multiple, partial, and unavailable-default states explicit.
M14.1d emits finite WCS corner geometry, preserving 3DFACE order and reordering
the trailing SOLID/TRACE corners into perimeter order before OCS projection.
M14.1e interprets the four documented 3DFACE invisible-edge bits in
source-corner order while retaining the exact signed flags and every unknown
bit.
M14.2a retains exact source-order TEXT, MTEXT, SHAPE, and TOLERANCE field
evidence, including repeated text chunks, ambiguous MTEXT group 50 values, and
the documented text, double, signed-16-bit, and signed-32-bit wire domains.
MTEXT group 420/430 ranges remain explicitly ambiguous with common entity
color fields until M14.11 adds subclass-aware common-property ownership.
M14.2b adds fixed per-record cardinality cards: 19 for TEXT, 33 for MTEXT, 12
for SHAPE, and 11 for TOLERANCE. Repeated MTEXT chunks and group 50 values
remain neutral `Multiple` evidence rather than being prematurely rejected.
M14.2c selects every unambiguous numeric TEXT and SHAPE scalar, applies only
Autodesk-documented defaults, preserves optional second-alignment absence, and
keeps missing required values, invalid ASCII numbers, and duplicates typed
with exact field/raw provenance. Text decoding, enum/range validation, layout,
coordinates, glyphs, and MTEXT/TOLERANCE semantics remain later checkpoints.
M14.2d extends the same four-state scalar contract to unambiguous MTEXT and
TOLERANCE numeric fields. MTEXT group 50 and group 420/430 remain evidence-only
because rotation/column precedence and common-entity color ownership are not
yet resolved. Optional/read-only/layout fields remain `Absent` without
invented defaults; only the documented extrusion default is applied.
M14.2e selects source-anchored TEXT content, SHAPE name, and TOLERANCE
dimension-style/content fields, and applies Autodesk's `STANDARD` style default
only to TEXT and MTEXT. MTEXT group 3 chunks and terminal group 1 values remain
separate, ordered source evidence; missing or repeated terminals and group 3
values after a terminal are retained with typed structural accounting. String
decoding, style-table resolution, inline-format interpretation, and glyph
geometry remain later checkpoints.
M14.2f classifies all documented TEXT horizontal and vertical justification
codes, preserves exact signed generation flags with backward/upside-down and
unknown-bit helpers, and reports whether the first or second alignment point
controls placement. Unsupported justification and unavailable source scalars
remain typed with original provenance; coordinate selection and transformation
remain later checkpoints.
M14.2g classifies all Autodesk-enumerated MTEXT attachment, drawing-direction,
and line-spacing-style codes. Required attachment/direction failures, optional
line-spacing absence, unsupported codes, invalid ASCII values, and duplicates
remain separate typed states with exact provenance.
M14.2h validates the independently documented optional MTEXT line-spacing
factor over the inclusive `0.25..=4.00` domain. Exact boundary values remain
explicit, out-of-range values retain their IEEE-754 payload and raw provenance,
and absence remains `Absent` without an invented default.
M14.2i classifies Autodesk's three published MTEXT background-fill settings:
off, explicit fill color, and drawing-window color. Other signed group-90
values remain unsupported with exact raw provenance rather than being treated
as undocumented flags.
M14.2j enforces Autodesk's read-only MTEXT actual-width relationship: a usable
group 42 must not exceed the required group-41 reference width. Equality is
valid; violations retain both exact IEEE-754 values and source provenance;
unavailable inputs preserve the originating scalar failure. Other numeric
ranges, background-color ownership, column semantics, rotation precedence,
and geometry remain later checkpoints.
M14.2k resolves Autodesk's source-order precedence between MTEXT group-50
rotation input and the group-11/21/31 X-axis input when group 50 is
unambiguously rotational. The last input form wins; invalid or duplicate
rotation evidence remains typed when it is effective, while a later X-axis
input remains selectable. Any coexistence of group 50 with column fields stays
fail-closed as an ambiguous rotation-or-column-height role until column
structure is validated. Vector assembly, normalization, coordinate
transforms, and column-height interpretation remain later checkpoints.
M14.2l recognizes the exact group-101 `Embedded Object` boundary used by modern
MTEXT column storage. The main MTEXT evidence directory stops at that boundary,
while a separate immutable directory retains the observed embedded version,
shared/individual heights, type, count, width, gutter, automatic-height, and
flow-reversal values in source order with raw provenance. This checkpoint does
not validate column relationships, merge legacy flat or R2007 XDATA storage,
or claim geometry, edit, or write support.
M14.2m projects that embedded evidence into typed scalar semantics. It
classifies no/static/dynamic column types, retains a nonnegative count,
requires positive usable width, retains nonnegative height and gutter
measurements, and accepts only exact Boolean 0/1 flags. Missing required type,
duplicate singletons, invalid numbers, and out-of-domain values remain typed
with raw provenance.
M14.2n validates the cross-field no/static/dynamic-auto/dynamic-manual modes.
Static columns require positive count and shared height; dynamic automatic
columns reject individual heights; and dynamic manual columns require positive
count plus either a positive shared height or exactly one source-order height
per column. A zero terminal individual height observed in R2018 behavioral
evidence remains usable rather than being silently rewritten. Legacy-storage
unification, geometry, edit, and write remain later checkpoints.
M14.2o recognizes the R2007-era `ACAD` XDATA column-info envelope with exact
begin/end matching. It retains source-order field identifiers 75/79/76/78/48/49
and the field-50 declared height count plus its 1040 height sequence, including
ASCII/Binary extended XDATA group codes. Incomplete, wrong-app, or inexact
blocks produce no partial evidence. Mapping this evidence into the M14.2m/n
semantics remains the next checkpoint.
M14.2p maps modern Embedded and R2007 `ACAD` XDATA values through one shared
scalar/mode implementation. Public source entries identify their physical
storage, while type/count/width/gutter/flags/heights use the same issues,
provenance, source-order height slices, and relation checks. XDATA
dynamic-manual columns now reach the same usable mode as their modern
equivalent.
M14.2q imports the separate exact R2007
`ACAD_MTEXT_DEFINED_HEIGHT_BEGIN`/`END` block only when selector 46, its 1040
value, the closing marker, and a preceding complete same-record column-info
entry are all present. The result reuses the shared-height scalar and completes
static and dynamic-automatic mode evidence without a parallel semantic path.
Linked-column handles and direct flat group-50 framing remain later
checkpoints.
M14.2r isolates the exact R2007 `ACAD_MTEXT_COLUMNS_BEGIN`/`END` envelope,
selector 47, typed declared count, and contiguous source-order group-1005
handle evidence. It reuses the format-neutral raw-handle parser, so invalid
hex remains typed and no second handle implementation is introduced. Linking
the envelope to column-info, validating the declared-count relationship, and
resolving targets remain separate from this evidence checkpoint.
M14.2s associates that envelope with only the latest complete preceding
column-info block in the same MTEXT record and reuses the generic handle
resolution directory for every group-1005 occurrence. Target states preserve
invalid, null, missing, ambiguous, unique MTEXT, and unique non-MTEXT outcomes.
This does not validate graph membership or a declared-count formula.
M14.2t adds a separate direct/flat evidence directory for Autodesk groups
75/76/78/79/48/49 and retains every coexisting group 50 as explicitly
rotation-or-column-height ambiguous. A lone group 50 remains orientation
evidence, and no field after an exact embedded-object boundary leaks into the
flat slice. Scalar unification and height disambiguation remain later work.
M14.2u adds the flat source to the shared scalar and mode projector. Its six
unambiguous fields use identical domains, provenance, diagnostics, and
cross-field relationships as Embedded/XDATA storage. Ambiguous group 50 maps
to no scalar role, so flat dynamic-auto mode is usable while height-bearing
modes remain fail-closed pending stronger framing evidence.
M14.2v makes that fail-closed boundary a public typed contract. Unified column
semantics distinguish unambiguous Embedded/XDATA height framing, flat storage
with no height evidence, and ambiguous direct group-50 sets with exact count
and first-value provenance. Static and dynamic-manual flat modes report a
dedicated unsupported ambiguity rather than a generic missing-height result.
No source-order heuristic claims rotation, shared height, or individual
heights without a reproducible normative or behavioral discriminator.
M14.2w constructs the effective MTEXT WCS X-axis direction selected by
M14.2k. Rotation input maps to cosine/sine/zero; explicit group 11/21/31 input
retains its exact vector and exposes a separately normalized unit direction.
Partial, invalid, zero-length, non-finite-length, and group-50-ambiguous inputs
remain typed. This does not yet combine insertion, extrusion, or text metrics
into placement or glyph geometry.
M14.2x selects the exact TEXT placement point in OCS. Left/baseline layout
uses group 10/20/30; every other supported horizontal or vertical
justification uses group 11/21/31. Missing required components, absent optional
components, invalid numerics, duplicates, and unavailable justification remain
typed with exact provenance. OCS-to-WCS transformation, style resolution,
metrics, glyph geometry, edit, and write remain later work.
M14.2y transforms that selected TEXT OCS point into WCS with its normalized
extrusion and the shared Autodesk arbitrary-axis basis. It retains the
first/second anchor kind, finite transformed point, normalized normal,
canonical positive zero, and typed provenance. Invalid anchor/extrusion
evidence, non-finite Binary inputs, zero extrusion, basis failure, and derived
overflow remain explicit. Rotation, style metrics, glyph geometry, edit, and
write remain later work.
M14.2z introduced a shared text-symbol extrusion/OCS projection helper and the
first SHAPE placement projection. M14.2aa corrects the coordinate-system
contract: Autodesk defines SHAPE groups 10/20/30 directly in WCS, so the exact
finite point is retained unchanged and only extrusion is normalized into the
WCS normal. Missing, invalid, duplicate, non-finite, zero-extrusion, and basis
failures remain explicit; maximum finite WCS coordinates are not rejected by
an inapplicable transform. SHAPE definition resolution, rotation, metrics,
glyph geometry, edit, and write remain later work.
M14.2ab projects optional SHAPE group 50 rotation, in degrees, onto the
normalized extrusion plane and exposes finite orthonormal WCS x/y axes plus
the normal. Zero degrees preserves the arbitrary-axis basis and the exact WCS
insertion remains untouched. Invalid or duplicate rotation/extrusion evidence,
non-finite Binary values, zero extrusion, and basis failure remain explicit.
SHAPE definition resolution, metrics, glyph geometry, edit, and write remain
later work.
M14.2ac projects TEXT group 50 rotation and documented group 71 backward and
upside-down bits into WCS glyph axes on the normalized extrusion plane. Unknown
generation bits remain preserved, and the selected WCS placement plus all lower
evidence remains reachable. Invalid or duplicate rotation/flags/extrusion,
non-finite Binary values, zero extrusion, and basis failure remain explicit.
Style metrics, oblique/width/height geometry, glyph outlines, edit, and write
remain later work.
M14.2ad composes TOLERANCE's required WCS insertion and WCS x-axis direction
with its normalized extrusion normal. Both WCS vectors remain bit-exact; no
OCS transform, x-axis normalization, or invented y-axis is applied. Missing,
invalid, duplicate, non-finite, zero x-axis/extrusion, and basis failures
remain explicit. Dimension-style resolution, tolerance-string interpretation,
glyph geometry, edit, and write remain later work.
M14.2ae indexes exact group-2 names only from DIMSTYLE records inside complete
exact table envelopes and resolves TOLERANCE group-3 names through bounded
byte-exact source comparison. Missing, unique, ambiguous, and unusable names
remain distinct; duplicate targets preserve source order. Interrupted,
unclosed, wrong-case, malformed, and application-group-decoy evidence is not
admitted. DIMSTYLE field semantics, tolerance-string interpretation, glyph
geometry, edit, and write remain later work.
M14.2af retains all 68 documented DIMSTYLE-specific fields through one sorted
registry and typed source-order text, binary64, signed-16-bit, or handle
evidence. Every admitted exact named DIMSTYLE record receives 68 stable
absent/unique/multiple cards, preserving duplicates and lexical failures while
excluding application-group decoys. No occurrence is selected and no default,
domain rule, name normalization, handle resolution, or downstream text/glyph
meaning is inferred. AC1009 Binary coverage leaves codes above 255 absent
because that dialect's one-byte group-code header cannot encode them.
M14.2ag adds lazy unique-occurrence semantics for every registered DIMSTYLE
field. Explicit values, absence, lexical failure, and duplicates retain typed
state plus field/raw provenance; absence is never converted into an
undocumented default. Group-70 helpers expose documented bits 16, 32, and 64
without discarding unknown bits. Field domains, handle target resolution,
tolerance-string interpretation, glyph geometry, edit, and write remain later
work.
M14.2ah resolves the five DIMSTYLE handle roles through the existing exact
document-local identity graph. Absent or duplicate field occurrences remain
separate from invalid, null, missing, unique, and ambiguous handle targets;
generic duplicate targets preserve source order and no target is selected.
This does not yet validate that a unique record has the expected STYLE or
BLOCK_RECORD table type, resolve names, interpret tolerance strings, construct
glyphs, edit, or write.
M14.2ai adds exact target-table validation for each uniquely resolved role.
Only named records from completely closed, matching DIMSTYLE, STYLE, and
BLOCK_RECORD table envelopes are admitted by one shared scanner. A unique
target is classified as expected STYLE/BLOCK_RECORD membership, another
reviewed named-symbol kind, or another raw record; all absent, duplicate,
invalid, null, missing, and ambiguous states remain unchanged. Target-name
resolution, record-content semantics, defaults, glyphs, edit, and write remain
later work.
M14.3a begins the curve family with exact SPLINE numeric evidence. It retains
five signed-16-bit roles and nineteen binary64 roles from exact records in
complete BLOCKS or ENTITIES sections, including repeated knot/control/fit
values, duplicate singleton candidates, invalid ASCII numbers, raw spans, and
group-102 exclusion. It does not select values, apply documented defaults,
interpret flags, reconcile counts, group coordinates, validate a curve, admit
HELIX subclass fields, construct geometry, edit, or write.
M14.3b gives every retained SPLINE record 24 fixed role cards in stable order.
Each card reports absent, unique, or duplicate-preserving multiple cardinality
and retains compact references to the original M14.3a values independently of
lexical validity. It does not select values, apply defaults, interpret flags or
counts, group points, validate or construct a curve, process HELIX, edit, or
write.
M14.3c adds unique-occurrence semantics for SPLINE group-70 flags. It exposes
the five documented closed, periodic, rational, planar, and linear bits while
retaining the original signed value and every unknown bit. Absence, lexical
failure, and duplicates remain distinct. Other scalar semantics, count
relations, point grouping, curve validation/geometry, HELIX, edit, and write
remain later work.
M14.3d selects unique degree, knot/control/fit counts, and tolerance scalars.
Absent counts remain absent, while only the three documented tolerance
defaults are materialized. Explicit/defaulted/absent/invalid/duplicate states
retain their evidence without domain or range guesses. Count reconciliation,
point grouping, curve validity/geometry, HELIX, edit, and write remain later
work.
M14.3e compares unique nonnegative declared knot/control/fit counts with
retained group-40/10/11 anchor occurrences. It distinguishes matched,
mismatched, negative, absent, invalid, and duplicate declarations without
inventing values or discarding source evidence. Coordinate tuple grouping,
member validity, knots/topology, geometry, HELIX, edit, and write remain later
work.
M14.3f freezes this document's 45 canonical Autodesk topics into the reviewed
schema registry. Generated public descriptors retain stable ordinals, schema
ids, exact canonical group-zero names, and the normalized normative-source
receipt. Byte lookup is exact and case-sensitive; aliases and unknown markers
remain deliberately unclassified. Generator validation fails closed on topic
count, duplicate id/name, namespace, or source-kind drift. This is inventory
infrastructure only: it does not scan a DXF record, classify an alias, define
version applicability or fields, expose common properties, parse any new
entity, construct geometry, edit, or write.

M14.3g freezes 14 reviewed on-wire alias/specialization names. Autodesk topic
pages normatively anchor `MPOLYGON` to HATCH, `ACAD_TABLE` to TABLE, and
`DGNUNDERLAY`/`DWFUNDERLAY`/`PDFUNDERLAY` to UNDERLAY. A separate AutoCAD 2027
oracle receipt anchors observed concrete dimension, MLEADER, SECTION, and five
SURFACE spellings; these remain visibly `BehavioralOracle`, not normative.
Generated exact-byte classification distinguishes canonical, alias, and
unknown names and preserves the exact marker through the alias descriptor.
This checkpoint does not decide whether a name is legal in its source section
or dialect and adds no semantic, geometry, CRUD, or support claim.

M14.3h gives all 45 canonical topics and 14 reviewed aliases one generated
nine-dialect applicability descriptor. Reviewed inclusive ranges return typed
`Applicable`/`NotApplicable`; insufficient evidence returns
`NotYetReviewed` rather than a guessed floor. Autodesk compatibility guidance
anchors DWF/DGN underlays at AC1021 and PDF underlays at AC1024. The other 56
names stay explicitly unreviewed. The registry is not yet a parser or writer
gate and does not decide section legality, fields, semantics, geometry, CRUD,
or support completion.

M14.3i adds the common entity index required before more family-specific
directories migrate. It classifies exact canonical, alias, unknown, and
wrong-section occurrences without relying on group order; `BLOCK`/`ENDBLK`
remain structural controls. Entity references preserve the source identity,
raw record and exact marker plus a source-order group-100 subclass path.
Application-control content is excluded from that path but never removed from
the raw document. The index has ASCII/Binary parity for all nine dialects and
does not yet expose common fields, family semantics, geometry, CRUD, or writer
applicability gates.

M14.3j freezes the common-property schema before the generic evidence scan.
Nineteen generated field descriptors cover handle/owner, extension dictionary,
paper/layout placement, layer, linetype/material/color/lineweight, linetype
scale, visibility, proxy graphics, true color/name, transparency, plot style,
and shadow. Each retains wire type, cardinality, Autodesk default, structural
scope, coordinate-space classification, explicit unreviewed field-version
applicability, and normalized provenance. Application-control handles remain
scope-separated. This checkpoint is registry metadata only and does not yet
parse or edit these fields.

M14.3k projects the generated common-field schema over the unified entity
directory. Each semantic entity receives 19 descriptor-order cards backed by
exact source-order occurrences. Required/optional absence, unique or duplicate
singletons, and proxy-data sequences stay typed without choosing an occurrence.
Subclass and group-102 scopes prevent reactor, extension-dictionary, and
family-specific code collisions. Unknown entities retain common evidence;
wrong-section records do not become semantic entities. Value decoding,
defaults, domain/reference checks, CRUD, and writes remain later checkpoints.

M14.3l groups retained SPLINE control-point and fit-point coordinate evidence
without depending on field interleaving. The nth X, Y, and Z occurrence within
each role forms the nth compact tuple, and each component remains a reference
to the exact evidence-card member. Component counts, empty sequences,
complete tuples, partial tuples, and invalid numeric evidence stay explicit;
no missing coordinate is defaulted. Weight association, tangent/normal
vectors, point/count validation, analytic NURBS data, HELIX, CRUD, and writes
remain later checkpoints.

M14.3m retains group-41 weights in the SPLINE evidence/card layer and adds a
compact auxiliary projection. Absent weights are represented as an implicit
unit sequence for the observed group-10 control-point count; explicit sequence
counts remain matched or mismatched and every raw numeric failure is retained.
Start tangent, end tangent, and normal expose per-component absent, unique, or
duplicate states plus aggregate absent/present/ambiguous structure. Scanning
is limited to legacy or exact `AcDbSpline` scope so later subclass collisions
are ignored without changing raw bytes. Numeric domain validation, effective
vector defaults, invariants, NURBS geometry, HELIX, CRUD, and writes remain
later checkpoints.

M14.3n adds typed effective values over that auxiliary evidence. Implicit unit
weights, explicit matched sequences, count mismatches, invalid numbers, and
behaviorally nonpositive weights remain distinct. Optional vectors are absent
as a whole or require one unique valid X component; unique missing Y/Z values
default to zero while retaining an explicit-component mask. Duplicate,
invalid, and missing-X component issues remain independently visible in one
unavailable state. Flag relations, zero-normal validation, control/fit point
semantics, degree/knot invariants, NURBS geometry, HELIX, CRUD, and writes
remain later checkpoints.

M14.3o adds one typed flag/auxiliary relation entry per SPLINE. It identifies
Autodesk's required linear-plus-planar combination, records rational versus
implicit/explicit weight form with count and numeric-domain issue totals, and
classifies the planar normal as missing, explicit, exact-zero, or unavailable.
A normal on a nonplanar spline remains an unexpected source observation rather
than being discarded. Missing, duplicate, or invalid flags make all dependent
relations unavailable without selecting a value. Point semantics,
degree/knot/periodic invariants, NURBS geometry, HELIX, CRUD, and writes remain
later checkpoints.

## Milestone queue

- M14.1: planar primitives — `3DFACE`, `SOLID`, `TRACE`.
- M14.2: text and symbols — `TEXT`, `MTEXT`, `SHAPE`, `TOLERANCE`.
- M14.3: curves — `SPLINE`, `HELIX`.
- M14.4: fills and meshes — `HATCH`, `MESH`.
- M14.5: annotation graphs — `DIMENSION` families, `LEADER`, `MLEADER`,
  `MLEADERSTYLE`.
- M14.6: compound linework — `MLINE`.
- M14.7: raster and external graphics — `IMAGE`, `WIPEOUT`, `UNDERLAY`.
- M14.8: view and lighting — `VIEWPORT`, `LIGHT`, `SUN`, `SECTION`.
- M14.9: tables and embedded content — `TABLE`, `OLEFRAME`, `OLE2FRAME`.
- M14.10: public envelopes with opaque proprietary payloads — `3DSOLID`,
  `BODY`, `REGION`, `SURFACE` families, and `ACAD_PROXY_ENTITY`.
- M14.11: version applicability, aliases, common entity properties, edit/write
  closure, corpus accounting, and generated support-matrix completeness.

Every micro-milestone must cover ASCII and Binary, all applicable dialects,
exact source identity, cancellation, duplicate/invalid evidence, and public API
bounds before its support state can advance.

## Legacy evidence boundary

`D:\SeaCad\cad_2026-07-23_source` is a read-only behavioral oracle. Its
inventories, authored fixtures, aggregate counters, and failure notes may guide
tests and risk ordering. Its parser implementation is not copied, translated,
vendored, or used as a runtime dependency. Autodesk documentation remains
normative.
