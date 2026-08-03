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

M14.3p adds typed SPLINE topology readiness. Degree evidence remains absent,
duplicate, invalid, nonpositive, or explicit. Knot sequences become empty,
nondecreasing, decreasing at an exact first index, or unavailable with an
invalid-member count. Explicit degree is compared with the observed control
point count and the defining NURBS knot-count relation; arithmetic overflow is
typed. Closed/periodic flag combinations are reported without asserting an
undocumented equivalence. Declared count cards remain available through the
existing count directory. Point values, analytic NURBS projection, HELIX,
CRUD, and writes remain later checkpoints.

M14.3q1 materializes source-backed analytic values without claiming a usable
curve. Knot values retain their exact evidence. Control and fit tuples require
valid X/Y, default omitted Z to zero while retaining an explicit-Z marker, and
link back to the tuple ordinal. Control weights are implicit unit values or a
fully matched positive explicit sequence. Knot, control, fit, and weight
failures accumulate in one typed mask and publish no partial ranges. Full
degree/count/order/multiplicity/domain and vector readiness composition remains
M14.3q2; HELIX, CRUD, and writes remain later checkpoints.

M14.3q2 composes the SPLINE value, topology, relation, and declared-count
directories into one analytic-readiness entry. A curve is available only when
degree and flags are unique and valid, declared knot/control counts match,
optional fit count is absent only for an empty fit sequence, knot order and the
NURBS count equation hold, multiplicity does not exceed degree plus one, and
the active knot domain is positive. Malformed optional tangents and invalid
planar-normal relations fail closed. Closed/periodic flags remain observations;
no evaluator, tessellator, HELIX support, CRUD, or write claim is added.

M14.3r adds exact source evidence for the 16 documented `AcDbHelix` roles:
major/maintenance versions, three WCS triples, radius, turns, turn height,
handedness, and constraint type. The projection consumes the unified entity
index, admits canonical HELIX only from `BLOCKS` or `ENTITIES`, and changes
scope only on exact subclass markers. Colliding SPLINE codes, application
groups, unknown subclasses, duplicates, and invalid numerics remain explicit
or excluded from the typed projection without changing raw data. Cardinality,
defaults, domain/relationship validation, embedded SPLINE composition,
geometry, CRUD, and writes remain later HELIX checkpoints.

M14.3u projects axis base, start point, and axis vector as three stable WCS
semantic entries per HELIX. Each X/Y/Z component uses the common provenance
model and remains explicit, absent, or typed invalid for duplicate, invalid
ASCII, or non-finite Binary evidence. A vector value is published only when
all three components are usable; no undocumented Y/Z zero default is applied.
Zero axis vectors remain exact observations for the later relationship/domain
validator. Nine-dialect ASCII/Binary parity, out-of-order components,
partial/duplicate/invalid/non-finite fixtures, cancellation, lookup, and bounds
are covered. Axis/parameter relations, embedded SPLINE composition, analytic
HELIX geometry, CRUD, and writes remain later checkpoints.

M14.3t projects seven HELIX scalar roles through the shared
`DxfSemanticValue<T, I>` contract. Major/maintenance versions remain exact
Int32 values; finite radius, turns, and turn height remain exact doubles;
handedness maps documented 0/1 to left/right; constraint type maps documented
0/1/2 to turn-height/turns/height. Absence stays distinct from invalidity,
duplicates fail without occurrence selection, invalid ASCII and Binary
non-finite doubles retain provenance, and undocumented positivity/version
domains are not inferred. Nine-dialect ASCII/Binary parity, invalid domains,
non-finite Binary values, cancellation, lookup, and bounds are covered.
Coordinate grouping, parameter relations, embedded SPLINE composition,
geometry, CRUD, and writes remain later HELIX checkpoints.

M14.3s adds exactly 16 stable cardinality cards per HELIX record over M14.3r
evidence. Each role is absent, unique, or multiple with a checked compact range
of source-ordered member references; invalid numeric evidence remains present
for cardinality purposes. No duplicate singleton is selected, and the card
layer makes no unsupported required/optional or default claim. Nine-dialect
ASCII/Binary parity, out-of-order groups, empty and duplicate records,
cancellation, lookup, and public bounds are covered. Typed field semantics,
coordinate grouping, parameter relations, embedded SPLINE composition,
geometry, CRUD, and writes remain later HELIX checkpoints.

M14.3v adds one typed relation entry per HELIX by composing the existing scalar
and WCS-vector projections. Usable nonzero axes are normalized; the
start-minus-axis-base vector supplies a derived base radius and exact
orthogonality residual. No tolerance is invented. Stored radius and turns
receive observational domain states, including Autodesk's 500-turn command
limit, while axial height is derived as turns times turn height with explicit
non-finite and flat-height states. Height zero is retained because Autodesk
documents flat 2D helices; the legacy zero-turn-height rejection is not
adopted. Nine-dialect ASCII/Binary parity, missing/invalid input, zero axes,
non-perpendicular vectors, negative radius, nonpositive/over-limit turns,
derived overflow, cancellation, lookup, and bounds are covered. Embedded
SPLINE composition, analytic HELIX geometry, CRUD, and writes remain later
checkpoints.

M14.3w composes HELIX metadata with its embedded spline representation. The
shared SPLINE directory now distinguishes exact `SPLINE` and `HELIX` records;
for HELIX it collects curve fields only inside the exact `AcDbSpline`
subclass. Readiness joins both projections by raw-record ordinal and requires
one ordered `AcDbSpline` then `AcDbHelix` pair, valid spline analytic data,
exactly perpendicular usable axis data, nonnegative stored radius, positive
turns, and finite derived height. Over-500 turns remain readable observations
and flat zero height remains valid. Typed failures accumulate without partial
analytic output. Autodesk warns that inherited NURBS operations on HELIX are
not a defined evaluation contract, so sampling/tessellation, CRUD, writes,
applicability, and support completion remain later checkpoints.

M14.3x completes the next unified-platform layer for all 19 common entity
properties. Singleton fields expose separate typed double, Int16, Int32,
handle, source-backed exact-text, schema-text-default, and material-BYLAYER
values through `DxfSemanticValue`. Omitted values use only reviewed generated
defaults; missing required fields, duplicate singletons, malformed ASCII
numbers/handles, and Binary non-finite doubles remain typed without occurrence
selection. Proxy group-310 members stay an exact opaque sequence linked to the
evidence directory. Nine-dialect ASCII/Binary parity does not settle unreviewed
field applicability. Domain/reference validation, proxy size reconciliation,
application-group closure, CRUD, writes, and support completion remain later
checkpoints.

M14.3y adds the first create-new primitive for the unified CRUD kernel: a typed
common-field group encoder for ASCII and Binary DXF. Generated descriptors
must match borrowed exact-text, handle, finite-double, Int16, Int32, or bounded
binary-chunk edit values. Output is canonical and dialect-aware, including the
AC1009 one-byte group-code boundary and R13-and-later two-byte little-endian
codes. Invalid wire/value pairs, text framing bytes, non-finite numbers,
oversized chunks, resource limits, cancellation, and unrepresentable AC1009
codes fail closed. Strict nine-dialect reparse proves emitted framing, but no
Unicode transcoding, insertion/update operation, sequence replacement, handle
or owner assignment, transaction plan, source mutation, or CRUD claim is made.

M14.3z adds stable source-bound entity keys and the first common-field update
plan. A caller may explicitly replace one already-present unique singleton;
the planner resolves the exact M14.3k occurrence, encodes its new value through
M14.3y, and returns one immutable M11 transaction plan with inverse evidence.
Absent fields are not silently inserted, duplicate singletons are never
selected, proxy-graphics sequences require a specialized operation, and
wrong-section/source/dialect/value failures stay typed. Strict ASCII/Binary
reparse across all nine dialects plus byte-identical inverse restoration prove
the replacement boundary. Canonical insertion anchors, reset-to-default,
multi-patch edit sessions, domain/reference validation, verified destination
writes, and full update/CRUD support remain later checkpoints.

M14.3aa adds the matching reset-to-default operation for one common-field
optional singleton. An already-absent optional field returns `AlreadyImplicit`;
an existing unique value becomes one source-bound deletion over its exact raw
group span. Required fields and duplicates fail typed, proxy group-310 data
requires a sequence operation even when absent, and group 360 inside
`ACAD_XDICTIONARY` requires a nested-structure operation rather than leaving an
empty group-102 envelope. Strict ASCII/Binary reparse across all nine dialects
proves generated omitted defaults and byte-identical inverse restoration.
Canonical insertion anchors, whole nested-group edits, multi-patch sessions,
domain/reference validation, verified destination writes, and full CRUD remain
later checkpoints.

M14.3ab adds a distinct canonical writer-order ordinal to every generated
common-field descriptor. It follows the usual Autodesk common-code table
presentation while deliberately remaining independent from stable registry
ordinal: extension dictionary writes before owner even though their registry
entries are reversed. Schema generation fails closed on missing, duplicate, or
incorrect order values. Parsing remains order-independent and unknown groups
remain lossless. Record-specific insertion anchors, subclass-envelope edits,
field insertion, and broader CRUD support remain later checkpoints.

M14.3ac adds a source-bound insertion-anchor planner for one absent common
singleton. It returns a byte offset between complete raw groups together with
the immediate preceding/following group occurrences. Handle and owner use the
entity preamble boundary; AcDbEntity fields use the generated writer order
inside an exact subclass range. Modern records require exactly one
`AcDbEntity`; AC1009 uses a bounded legacy preamble and rejects incomplete
application groups. Existing fields, sequences, extension dictionaries, and
inconsistent earlier/later field order fail typed without selecting or moving
anything. Unknown groups retain their exact position. Encoding, insertion,
transactions, multi-field sessions, and broader CRUD support remain later
checkpoints.

M14.3ad turns one successful M14.3ac anchor into an immutable insertion
transaction. The existing typed encoder supplies one complete dialect-correct
group; its ASCII separators inherit the preceding group's LF, CR, or CRLF
ending, while Binary bytes remain unchanged. One empty source span owns the
inserted bytes and captures an empty inverse fragment. Strict ASCII/Binary
post-images across all nine dialects publish the requested semantic, and the
materialized inverse restores byte-identical input. Anchor and encoding
failures stay separately typed. Sequence/nested insertion, multi-field edit
sessions, domain/reference validation, destination writes, and broader CRUD
support remain later checkpoints.

M14.3ae adds the first unified source-bound edit session. Its typed common-
field patch either sets an explicit value or resets an optional singleton to
its schema default. Set chooses replacement versus canonical insertion from
the original evidence; reset keeps an already-implicit value as a no-op.
Queued duplicate entity/field targets fail without choosing an occurrence.
At finish, non-overlapping replacements, deletions, and insertions become one
transaction; multiple insertions sharing one entity anchor are concatenated in
generated writer order, independent of caller order. Paired strict
ASCII/Binary tests across all nine dialects prove multi-record semantics and
byte-identical inverse restoration. Family-specific patches, nested/sequence
operations, domain/reference validation, insert/clone/delete, and destination
verification remain later checkpoints.

M14.3af adds a verifiable finish path without weakening the fixed
`finish() -> DxfTransactionPlan` API. `finish_verifiable` returns one
`DxfEntityEditPlan` whose payload-redacted expectations identify every queued
record ordinal/field and its explicit typed value or implicit reset. A fresh
post-image must have the projected envelope, matching unique/absent
cardinality, correct explicit/defaulted/absent state, and exact typed value.
After those semantic postconditions pass, the existing raw verifier must match
every transaction byte before an executable inverse journal escapes. Strict
ASCII/Binary fixtures across all nine dialects distinguish field-value
mismatch from unrelated raw-byte mismatch and restore byte-identical source.
Create-new writer/cleanup integration, domain/reference validation, family
patches, insert/clone/delete, and complete CRUD remain later checkpoints.

M14.3ag makes the verifiable entity plan executable through the existing M12
create-new writer. The destination must not exist. Its complete bytes are
streamed, hashed, flushed, synced, reopened, and strictly reparsed using the
transaction's exact ASCII/Binary format. Semantic postconditions are checked
before the exact raw transaction verifier releases an inverse, so a modified
edited field remains a typed semantic issue while an unrelated modification
remains a raw mismatch. The returned journal pairs the M12 write receipt with
the semantic receipt and byte-exact inverse. Any failure or semantic
unavailability after creation removes the output; cleanup failure replaces the
primary result. Tests cover ASCII and Binary AC1009 through AC1032, exact
inverse restoration, pre-existing/source mismatch rejection, cancellation,
strict-reparse cleanup, semantic cleanup, and raw-mismatch cleanup. This does
not add field-domain/reference validators, family patches, sequence/nested
operations, insert/clone/delete, handle/owner assignment, or complete CRUD.

M14.3ah classifies explicit common-field edit values before any raw planner is
called. The reviewed scalar set is group 67 model/paper space, group 62 indexed
color including BYBLOCK/BYLAYER and negative layer-off ACI, the public group
370 lineweight enumeration, nonnegative group 48 linetype scale, group 60
visibility, nonnegative group 92 proxy graphics size, group 420 RGB with a zero
high byte, and group 284 shadow mode. Public domain types round-trip their wire
values, invalid value kinds/ranges return typed issues, and unreviewed fields
remain distinguishable rather than being called valid. The edit session checks
this classifier after duplicate-target detection and before insertion or
replacement, so rejected values leave the queue and source unchanged. Strict
ASCII/Binary AC1009-through-AC1032 tests prove accepted edits, semantic
postconditions, and exact inverse restoration. Existing raw-value domain
projection, references/names, transparency, proxy size/data reconciliation,
cross-field relations, version applicability, and family patches remain later
work.

M14.3ai projects those same reviewed domains from existing raw documents. It
composes the generic common-field semantic directory instead of rescanning raw
groups: usable explicit/defaulted scalars become typed domain values, while raw
decode errors, duplicates, missing required fields, and out-of-domain values
remain separate typed invalid states with exact evidence. Unreviewed singleton
and opaque-sequence fields pass through unchanged. Paired ASCII/Binary fixtures
cover all nine dialects, invalid raw values, defaults, absence, cancellation,
source-bound lookup, and bounded public types. Names/references, transparency,
proxy size/data reconciliation, cross-field relations, applicability, family
patches, and complete CRUD remain later work.

M14.3aj joins the five handle-valued common fields to existing M7 evidence.
Object handle group 5 stays lexical identity. Owner 330, extension-dictionary
360, material 347, and plot-style 390 project exact field failure/absence and
typed null, missing, unique, or ambiguous document-local targets; omitted
material remains the schema-backed `ByLayer` default. The join reuses shared
identity/target slices and never selects one ambiguous occurrence or target.
Paired ASCII/Binary fixtures cover AC1009 through AC1032 plus malformed,
duplicate, null, dangling, duplicate-target, cancellation, source identity,
and public-bound cases. Unique lookup is not target-kind compatibility,
authoritative ownership, dictionary membership, pointer-lifecycle validity,
applicability, or reference-safe CRUD.

M14.3ak adds `LAYER` and `LTYPE` to the completely closed named-symbol table
inventory, then projects all four exact-text common fields. Layer 8 and
linetype 6 use a digest-bounded lookup with exact source-byte collision checks;
zero, one, or multiple exact table-name matches remain typed. Linetype omission
keeps the schema `BYLAYER` default without inventing a source span or target.
Layout 410 and color-name 430 remain exact `Unreviewed` semantics. ASCII/Binary
fixtures span AC1009 through AC1032 plus missing/duplicate names, duplicate
fields, defaults, cancellation, source mismatch, wrong-section inventory, and
public bounds. Case-insensitive policy, legal symbol characters, XREF naming,
layout-object/color-book resolution, applicability, and edit validation remain
later work.

M14.3al adds a source-bound relation between common group 92 and the opaque
group-310 sequence. ASCII hexadecimal chunks are checked and counted as
decoded bytes; Binary chunks use their validated payload spans. No payload is
joined, decoded, rendered, or synthesized. Absence, missing size, exact match,
count mismatch, invalid size, and malformed ASCII chunk remain typed and keep
raw evidence. Fixtures cover all nine dialects in ASCII/Binary plus malformed
hex, negative size, cancellation, source mismatch, and public bounds. Proxy
payload semantics, applicability, sequence CRUD, clone/delete, and `Complete`
support remain later work.

M14.3am adds typed common transparency semantics for group 440. The public
domain preserves ByLayer, ByBlock, and ByAlpha with ObjectARX alpha 0–255,
rejects reserved payload bits and unknown method bytes, and composes with raw
projection plus verified common-field edits. Autodesk documentation is joined
with an AutoCAD 2027 CHPROP native receipt for exact method-byte placement.
ASCII/Binary fixtures span all nine dialects; AC1009 retains exact absence and
the existing wrong-dialect edit rejection. Effective inherited transparency,
UI percentage conversion, rendering, applicability, family CRUD, and
`Complete` support remain later work.

M14.3an validates the public target kind of the three reviewed common object
references after document-local handle resolution. Extension dictionary 360
requires an exact `OBJECTS`/`DICTIONARY` target, material 347 requires
`OBJECTS`/`MATERIAL`, and plot style 390 requires
`OBJECTS`/`ACDBPLACEHOLDER`. Wrong-section and wrong-marker unique targets are
typed incompatibilities; earlier null, missing, ambiguous, raw-field, absent,
and default outcomes remain intact. Owner 330 remains explicitly unreviewed
because entity family and placement determine its valid kind. Paired
ASCII/Binary fixtures span all nine dialects plus cancellation, source
identity, and bounds. Ownership topology, dictionary membership, lifecycle,
reference-safe CRUD, applicability, family graphs, and `Complete` support
remain later work.

M14.3ao moves the three reviewed target-kind rules into common-field edit
admission. A generic singleton patch cannot change group-5 identity or group-
330 owner; those now return typed handle-remap or placement-operation
requirements. Extension dictionary, material, and plot-style edits require a
non-null handle resolving to one exact reviewed `OBJECTS` target before they
enter a session. Missing, duplicate, wrong-marker, and wrong-section targets
leave the queue and source unchanged. The public classifier has ASCII/Binary
parity across all nine dialects; modern-session materialization, strict
reparse, semantic target verification, and exact inverse cover AC1012 through
AC1032. AC1009 Binary remains physically unable to encode group 347, while
ASCII applicability is not inferred. Handle remap, owner placement,
dictionary membership, lifecycle, cross-document clone/delete, applicability,
and `Complete` support remain later work.

M14.3ap makes common symbol-name editing source-bound. Explicit layer and
linetype names require exactly one byte-identical entry in a completely closed
matching `LAYER` or `LTYPE` table before a singleton patch enters the session.
Missing and duplicate names remain typed and queue nothing. Layout and color-
book names are now explicitly blocked behind their future dedicated resolvers.
The session lazily builds and reuses one named-symbol directory, while its
exact-text resource ceiling still precedes lookup. Paired ASCII/Binary tests
span AC1009 through AC1032 with verified post-image semantics and byte-identical
inverse restoration. Case folding, legal symbol characters, XREF rules,
layout/color-book resolution, table-record insertion, applicability, family
graphs, and `Complete` support remain later work.

M14.3aq resolves common layout names on the read side. A source-bound directory
indexes only exact `LAYOUT` records from completely closed `OBJECTS` sections
and reads group 1 only while the exact `AcDbLayout` subclass is active. The
inherited plot-settings group 1, application groups, wrong subclasses, wrong
sections, and interrupted/unclosed sections cannot become layout targets.
Every layout object retains missing, unique, or duplicate name cardinality;
common group 410 then projects exact unique, missing, or ambiguous target state
without choosing an occurrence. Paired ASCII/Binary fixtures cover all nine
dialects plus subclass collision and malformed cardinality. Matching remains
byte-exact and case-sensitive. Layout edit admission, reciprocal block-record
ownership, name folding/validity, color-book resolution, applicability,
lifecycle, family graphs, and `Complete` support remain later work.

M14.3ar moves exact layout resolution into common-field edit admission. A
public source-bound classifier returns non-layout, wrong-value, missing,
ambiguous, or one exact target state without mutating the document. The shared
edit session lazily builds one layout directory only for an exact group-410
proposal and keeps the existing value-byte ceiling ahead of lookup. Valid
modern ASCII/Binary edits use canonical insertion/replacement, strict reparse,
target-bearing semantic verification, raw verification, and byte-identical
inverse restoration; rejected edits queue nothing. Classifier parity covers
all nine dialects. AC1009 Binary cannot physically encode group 410 and fails
typed at the existing wire gate. Matching remains byte-exact and case-
sensitive. Reciprocal block-record ownership, layout lifecycle, legal-name/
case policy, color books, applicability, family graphs, and `Complete` support
remain later work.

M14.3as reviews common color-book names on the read side. Explicit group 430
must contain exactly one `$` with nonempty book and color components; both are
retained as exact source spans and no separator occurrence is guessed. A
structured value is published only when the same entity also has usable
reviewed group-420 true color and group-62 indexed color semantics. Missing or
invalid related scalars remain embedded typed relation failures with group-430
provenance. Paired ASCII/Binary fixtures cover all nine dialects, including
absence in AC1009, out-of-order groups, every delimiter failure, duplicate and
out-of-domain related scalars, cancellation, source identity, and bounds. The
core does not open `.acb` files, verify external color names/RGB mappings,
admit color-book edits, infer applicability, or advance an entity to
`Complete`.

M14.3at moves color-book syntax and same-entity color relations into edit
admission. The public source-bound classifier distinguishes non-color fields,
wrong value kinds, every documented-envelope delimiter failure, unusable
group-420/group-62 semantics, and a valid proposed book/color pair without
copying its bytes. The shared session builds one common-domain directory lazily
for group 430 and rejects invalid proposals without queueing. Valid modern
ASCII/Binary edits use canonical replacement/insertion, strict reparse,
structured color-book semantics, generic semantic/raw verification, and exact
inverse restoration. AC1009 retains typed rejection because true color and
group 430 are absent/unencodable in its Binary wire. External `.acb`
resolution, name-to-RGB verification, pending multi-field tuple composition,
applicability, family graphs, and `Complete` support remain later work.

M14.3au adds the first correlated common-property CRUD operation. A typed
`CommonColorBook` patch carries exact raw name input plus already validated
indexed and true-color values, then inserts or replaces groups 62, 420, and 430
as one logical session update. Syntax and duplicate-pending fields fail before
mutation; any insertion/replacement failure while materializing the tuple
truncates the pending queue to its exact checkpoint. AC1009 is rejected for
both physical formats because its Binary wire cannot represent the tuple;
modern ASCII/Binary insert and replace paths strict-reparse to structured
color-book semantics and inverse to the byte-identical source. Patch debug is
payload-redacted. External `.acb` validation, name/RGB mapping, reset of the
whole tuple, applicability beyond the parity restriction, family graphs, and
`Complete` support remain later work.

M14.3av adds the inverse correlated operation through
`ResetCommonColorBook`. It attempts groups 62, 420, and 430 in canonical field
order, deletes every unique explicit singleton, and treats an absent optional
member as already implicit. The resulting semantics restore indexed color to
the reviewed BYLAYER value 256 while true color and color name remain absent.
Any duplicate or structural reset failure truncates only components added by
the request; earlier unrelated pending edits remain intact. Because deletion
does not encode a new group, the same operation covers ASCII and Binary across
all nine dialects, including AC1009. External `.acb` access, applicability,
family graphs, entity insertion/clone/delete, and `Complete` support remain
later work.

M14.3aw adds atomic source-bound transaction composition for the unified CRUD
pipeline. Every input plan must match the exact document identity, length, and
physical format; all patches are replayed through the bounded M11 builder so
cross-plan overlaps, duplicate insertion offsets, source limits, value limits,
plan counts, and cancellation remain typed. Source-order is independent of
caller plan order, and the composed plan captures a fresh exact inverse.
`DxfEntityEditPlan::compose_supplemental_transactions` retains existing field
postconditions while attaching handle, owner, or placement byte work. Paired
tests across all nine dialects prove M11 handle assignment plus successor
`$HANDSEED` and M14 common-field reset commit and verify as one transaction.
This checkpoint does not yet encode a new entity draft, choose placement or
owner, reserve handles across sessions, or implement insert/clone/delete.

M14.3ax adds the source-bound `DxfEntityPlacementDirectory`. Every exact
`ENTITIES` section and BLOCK definition receives one typed assessment in
container source order. A completely indexed `ENTITIES` section anchors before
its first content group, including exact `ENDSEC` for an empty section. A
closed BLOCK definition anchors before its first member record or exact
`ENDBLK` when empty. This container-start policy cannot split an existing
POLYLINE or INSERT/ATTRIB sequence. Interrupted/unclosed sections, interrupted
or unclosed BLOCK definitions, and orphan nonzero content groups remain typed
and produce no placement. All anchors retain source identity, container target,
section kind, following group occurrence, and a zero-width raw span. Paired
fixtures for all nine dialects execute each ASCII/Binary anchor through the M11
transaction builder and strict reparse. This checkpoint does not encode entity
drafts, infer or validate group-330 owner, allocate a handle, or implement
insert/clone/delete.

M14.3ay adds `DxfHandleReservationPlan` for records that do not yet exist in
the raw directory. It consumes the existing fail-closed allocation policy,
retains the consecutive allocation proposal, and replaces the exact
`$HANDSEED` payload with its uppercase-hex successor in one immutable
transaction. Zero-count requests remain explicit empty transactions;
unavailable policy, numeric exhaustion, source mismatch, record limits, and
cancellation remain typed. The uppercase handle encoder is shared with M11.2b
so assignment and reservation cannot drift. Paired AC1009-through-AC1032
ASCII/Binary tests compose one reserved handle with each M14.3ax placement,
insert an identified record, strict-reparse the new identity and successor
seed, and restore the byte-identical source through one inverse. Reservation is
source-bound optimistic planning rather than a cross-session lock. This
checkpoint does not encode typed drafts, choose group-330 owner, or complete
insert/clone/delete.

M14.3az adds the source-bound `DxfEntityPlacementOwnerDirectory`. A requested
non-null handle must resolve uniquely to an exact named record admitted from a
completely closed `BLOCK_RECORD` table. `ENTITIES` placements retain the
caller's explicit owner choice. A placement inside a BLOCK additionally
requires exactly one common owner candidate on its `BLOCK` marker, a unique
target, and equality with the requested BLOCK_RECORD; missing, duplicate,
invalid, null, dangling, ambiguous, wrong-kind, and mismatched states remain
typed. The validator never chooses model/paper space from group 67 or layout
410 and never treats a uniquely resolved non-BLOCK_RECORD as compatible.
Paired fixtures cover all nine dialects; AC1009 retains owner absence in both
physical formats because its Binary wire cannot represent group 330. This
checkpoint binds placement preparation only. Draft encoding, applicability,
new-record insertion, clone/delete closure, and `Complete` support remain open.

M14.3ba adds typed draft-identity preparation without exposing arbitrary
group-zero input. `DxfEntityDraftName` admits exactly the generated 45
canonical topics and 14 reviewed aliases, preserving each alias's canonical
topic mapping and exact wire spelling. A preparation consumes one M14.3ay
reservation and one M14.3az placement-owner binding from the same exact source;
zero or multi-handle reservations fail with typed cardinality before encoding.
The prepared plan retains the exact name, reserved handle, placement, owner
BLOCK_RECORD, allocation, and `$HANDSEED` transaction. Tests exercise all 59
names for every AC1009-through-AC1032 ASCII/Binary pair and prove strict
HANDSEED reparse plus byte-identical inverse. Name admission does not imply
dialect applicability or family support. Draft fields, record encoding,
insertion, semantic verification, clone/delete, and `Complete` remain open.

M14.3bb consumes that identity only after fail-closed dialect admission. One
supported `$ACADVER` must remain bound to the same exact source and the
generated applicability descriptor must return `Applicable`; absent,
unsupported, invalid, ambiguous, `NotApplicable`, and `NotYetReviewed` states
remain typed. The currently reviewed matrix admits only DGN/DWF underlay from
AC1021 and PDF underlay from AC1024, so the other 56 registered names cannot be
encoded until their ranges have normative evidence. All 59 names are checked
against all nine paired ASCII/Binary dialect fixtures. The admitted plan keeps
the exact descriptor provenance, identity, owner, handle, and reversible seed
transaction. Record bytes, family payload validation, insertion, semantic
postconditions, clone/delete, and `Complete` remain open.

M14.3bc reviews canonical MESH applicability from Autodesk's statement that
the newer MESH object type was implemented for AutoCAD 2010 subdivision
surface workflows. The generated range therefore starts at AC1024 with no
reviewed maximum: earlier supported dialects are `NotApplicable`, while
AC1024/AC1027/AC1032 are `Applicable`. The source GUID and normalized facts
receipt are generated into the descriptor, and the 59-name draft admission
matrix now has four reviewed names and 55 `NotYetReviewed` names. This does not
claim MESH payload/topology validity, CRUD, geometry, or `Complete` support.

M14.3bd reviews canonical MLEADER applicability from Autodesk's statement that
multileaders display as proxies in versions before AutoCAD 2008. The generated
range starts at AC1021 with no reviewed maximum. The source GUID and normalized
facts receipt remain attached to the descriptor, while the behaviorally
observed exact `MULTILEADER` alias stays `NotYetReviewed` rather than inheriting
an unproved wire-version floor. Five of 59 names now have reviewed ranges and
54 remain fail-closed. Nested contexts, content, lines, breaks, style/reference
resolution, geometry, CRUD, and `Complete` support remain open.

M14.3be reviews canonical LIGHT applicability from Autodesk's documented
conversion boundary: lighting created before AutoCAD 2007 must be converted to
the AutoCAD 2007/2008 lighting format. The generated range starts at AC1021
with no reviewed maximum and carries the source GUID plus normalized facts
receipt. Six of 59 names now have reviewed ranges and 53 remain fail-closed.
LIGHT field semantics, type/vector relations, photometric settings, rendering,
shadows, CRUD, and `Complete` support remain open.

M14.3bf reviews exact ACAD_TABLE alias applicability. Autodesk's AutoCAD 2005
API history identifies the Table entity/API as new, while the normative TABLE
DXF page independently identifies `ACAD_TABLE` as the exact group-0 entity
name. Its generated range starts at AC1018 with no reviewed maximum. Canonical
`TABLE` stays `NotYetReviewed` rather than inheriting an unproved wire identity.
Seven of 59 names now have reviewed ranges and 52 remain fail-closed. Cell
grammar, merges, styles, layout extents, CRUD, and `Complete` remain open.

M14.3bg reviews canonical HELIX applicability. Autodesk's AutoCAD 2007 API
history marks `IAcadHelix` and its public constraint/twist enums as new, while
the normative HELIX DXF page independently defines the exact HELIX entity and
`AcDbHelix` subclass. Its generated range starts at AC1021 with no reviewed
maximum. Eight of 59 names now have reviewed ranges and 51 remain fail-closed.
This receipt does not change existing HELIX field evidence or claim analytic
composition, CRUD, writer support, or `Complete` status.

M14.3bh reviews canonical LWPOLYLINE applicability. Autodesk states that 2D
polylines are created as lightweight polyline entities as of Release 14 and
that earlier-release 2D polylines convert when opened, while the normative
LWPOLYLINE DXF page independently defines the exact entity and `AcDbPolyline`
subclass. Its generated range starts at AC1014 with no reviewed maximum. Nine
of 59 names now have reviewed ranges and 50 remain fail-closed. Existing
LWPOLYLINE semantics and geometry do not imply insert/CRUD/writer closure or
`Complete` status.

M14.3bi replaces behavioral-only provenance for `SECTIONOBJECT` and the five
surface-specialization aliases with Autodesk's valid-DXF-name inventory, then
reviews their shared AutoCAD 2007 introduction boundary. The generated ranges
start at AC1021 with no reviewed maximum and retain the API-history GUID plus
normalized six-row receipt. Canonical `SECTION` and `SURFACE` remain
`NotYetReviewed`: observed files contain the concrete aliases, not generic
group-0 records. Fifteen of 59 names now have reviewed ranges and 44 remain
fail-closed. Payload semantics, proprietary modeler decoding, CRUD, and
`Complete` remain open.

M14.3bj reviews the 16 exact canonical names Autodesk lists as introduced
before AutoCAD Release 13: `3DFACE`, `ARC`, `ATTDEF`, `ATTRIB`, `CIRCLE`,
`DIMENSION`, `INSERT`, `LINE`, `POINT`, `POLYLINE`, `SEQEND`, `SHAPE`, `SOLID`,
`TEXT`, `VERTEX`, and `VIEWPORT`. AC1009 is the shared Release 11/12 dialect
and the lower bound of Core 1.0, so every supported dialect is applicable with
no earlier-version claim. The generated receipt binds the exact 16 rows to the
Autodesk page GUID. Thirty-one of 59 names now have reviewed ranges and 28
remain fail-closed. The page's separate `entmake` restriction for VIEWPORT is
preserved as a nonclaim: applicability does not establish insertion, payload
semantics, family CRUD, or `Complete` support.

M14.3bk proves the first typed complete-record draft for canonical `POINT`.
The encoder reuses the dialect-correct raw group encoder and composes exact
group 0/name, reserved handle, caller-bound owner, reviewed subclass markers,
placement-conditioned common properties, and finite WCS 10/20/30 values in
canonical subclass order. Existing layer and modern `ENTITIES` layout names
must resolve uniquely; AC1015+ lineweight is explicit because Autodesk gives
group 370 no omission default. Layout is required for modern `ENTITIES` and
rejected for BLOCK-local records, matching the distinct placement envelopes.
All nine ASCII/Binary dialect pairs compose with handle reservation and a real
insertion transaction, strict-reparse, publish POINT semantics, and invert to
the byte-identical source. Typed negative coverage includes name mismatch,
missing/inapplicable layout or lineweight, missing references, malformed exact
text, non-finite values, cancellation, and foreign source identity. Unified
session insertion, POINT update/clone/delete, optional POINT fields, common
field applicability generation, and `Complete` remain open.

M14.3bl turns that source-bound record into one atomic, semantically verifiable
insert plan. `plan_entity_draft_insert` places the exact encoded bytes at the
admitted zero-width anchor and composes that patch with the retained successor
`$HANDSEED` transaction. Its `DxfEntityEditPlan` expectation resolves the new
handle uniquely, requires canonical POINT classification, checks the exact
ENTITIES section or BLOCK definition, verifies the explicit owner where the
dialect represents it, verifies layer/layout/lineweight fields, and compares
the analytic WCS location. The standard create-new writer now strict-reparses,
semantically verifies, cleans up on failure, and returns an executable inverse
for POINT insertion across every ASCII/Binary Core dialect. Missing/ambiguous
identity, wrong family or placement, common-field mismatch, and geometry
mismatch remain typed. This does not yet expose session-level `insert`, batch
multiple draft records, add optional POINT fields, or complete update/clone/
delete and the POINT support ledger.

M14.3bm exposes the first exact roadmap signature on the unified CRUD surface:
`DxfEntityEditSession::insert(DxfEntityPlacement, DxfEntityDraft)`. The typed
draft retains the caller's explicit BLOCK_RECORD owner instead of deriving
model/paper space. Without mutating the source, the session validates the
placement-owner pair, reserves one handle, admits generated applicability,
encodes the canonical POINT record, and queues the M14.3bl verified insertion.
`finish` returns the atomic insertion/`$HANDSEED` transaction and
`finish_verifiable` retains its family postcondition. Receipt and compact typed
issues expose the allocated handle, name, placement, missing/invalid owner,
allocation policy, applicability, and record failures without retaining raw
payload. All nine ASCII/Binary dialect pairs pass strict semantic/inverse
verification. This first session checkpoint admits one insert and rejects
insert/update mixing; multi-record reservation, mixed verification, optional
POINT fields, and the remaining update/clone/delete ladder stay open.

M14.3bn removes the one-record session limit without cloning reservation
transactions or retaining caller borrows. Each successful POINT insert uses
the source policy to propose its deterministic handle ordinal, validates and
copies the complete encoded record plus semantic expectation into the session,
and mutates the queue only after every check passes. At `finish`, one bounded
reservation covers the entire consecutive handle range and advances
`$HANDSEED` exactly once. Same-anchor records are grouped into one patch in
caller/handle order; distinct anchors remain transaction source-ordered. The
verifiable plan retains one family expectation per inserted handle. Three-
record batches pass all nine ASCII/Binary dialect pairs with two raw patches,
successor-seed `0x43`, strict semantic verification, and byte-identical inverse
restoration. Invalid drafts do not consume handle ordinals; exhaustion remains
typed. Mixed insert/update verification, optional POINT payload, other draft
families, and POINT update/clone/delete remain open.

M14.3bo closes the public POINT payload gap identified by Autodesk. The shared
POINT/LINE evidence platform now retains POINT thickness `39` and UCS X-axis
angle `50` alongside location and extrusion, gives POINT eight stable role
cards, and projects the two optional scalars with documented zero defaults.
Invalid and duplicate explicit values never fall back to defaults. The typed
POINT draft adds optional thickness, complete extrusion, and angle values.
Unspecified fields stay omitted; supplied fields are emitted after location in
canonical `39`, `210/220/230`, `50` order, and a zero extrusion vector is
rejected. Insert verification checks both exact binary64 values and whether
each optional value remained explicit or defaulted. Minimal and all-explicit
records pass all nine ASCII/Binary dialect pairs, while tampering of each new
field fails typed and the inverse remains byte-identical. POINT update,
clone/delete closure, mixed insert/update sessions, and `Complete` remain open.

M14.3bp introduces the first typed existing-record POINT update. One
`DxfPointPatch::SetLocation` replaces the unique source-backed WCS `10/20/30`
tuple as a single logical session edit without selecting duplicate
occurrences. All three components are encoded for the original physical
format and dialect and compose atomically with independent common-property
updates. The verifiable plan checks the same raw-record ordinal and exact typed
location after strict reparse before exposing its byte-identical inverse.
Missing or duplicate tuple members, wrong classification, non-finite input,
duplicate patch admission, cancellation, tampering, and insert/update mixing
remain typed fail-closed outcomes across all nine ASCII/Binary dialect pairs.
Thickness, extrusion, angle, reset, clone/delete, and POINT `Complete` remain
open.

M14.3bq adds the second typed existing-record POINT update.
`DxfPointPatch::SetThickness` replaces one unique explicit group `39` through
the original ASCII/Binary dialect encoder. Missing or duplicate thickness is
not selected or silently defaulted. Location and thickness have distinct patch
identities, may be queued together for the same source-bound POINT, and remain
separate logical expectations when composed with common-property edits. Strict
post-image verification requires the exact binary64 value and `Explicit`
semantic state before exposing the byte-identical inverse. Wrong families,
non-finite values, duplicate patches, cancellation, tampering, and resource
limits remain typed fail-closed. Inserting an absent group `39`, reset to zero,
extrusion/angle updates, mixed insert/update sessions, clone/delete, and POINT
`Complete` remain open.

M14.3br makes the existing `SetThickness` operation total over the two usable
POINT thickness states. A unique explicit group `39` is still replaced in
place; documented defaulted absence inserts one encoded group immediately
after the last unique source location component. The insertion path requires
all three location cards to be unique, preserves the preceding ASCII line
ending or exact Binary framing, and enters the session as the same logical
thickness patch identity. Duplicate thickness and missing/duplicate location
anchors remain typed failures with no queued transaction. Strict post-image
verification requires the exact value and `Explicit` state, and the inverse
removes the inserted bytes exactly across all nine ASCII/Binary dialect pairs.
Reset to implicit zero, extrusion/angle updates, mixed insert/update sessions,
clone/delete, and POINT `Complete` remain open.

M14.3bs adds reset-to-default behavior for the existing POINT thickness patch
identity. `DxfPointPatch::ResetThickness` deletes one unique explicit group
`39` and retains a postcondition requiring the documented zero value in the
`Defaulted` semantic state. An already absent group returns an
`AlreadyImplicit` receipt without entering the transaction or reserving the
logical patch kind; duplicate explicit groups still fail typed. A queued set or
reset rejects another thickness request, and POINT receipts now report whether
the accepted operation inserted, replaced, reset, or changed nothing. All nine
ASCII/Binary dialect pairs prove strict reparse, semantic verification, and
byte-identical inverse restoration. Extrusion/angle updates, mixed
insert/update sessions, clone/delete, and POINT `Complete` remain open.

M14.3bt introduces `DxfPointPatch::SetExtrusion` for an already explicit POINT
extrusion tuple. Groups `210`, `220`, and `230` must each be unique; one missing
or duplicated component rejects the whole request. The requested tuple must be
nonzero and every component must pass the existing dialect encoder before the
three exact replacements enter the session as one logical edit. Extrusion has
its own patch identity and composes with location and thickness. Post-image
verification requires the exact tuple and an `Explicit` state for every
component on the same raw-record ordinal before exposing the byte-identical
inverse. All nine ASCII/Binary dialect pairs pass replacement, strict reparse,
tamper rejection, cancellation, and inverse restoration. Absent/default
extrusion insertion or reset, angle updates, mixed insert/update sessions,
clone/delete, and POINT `Complete` remain open.

M14.3bu makes the extrusion patch usable when all three extrusion groups are
absent and therefore defaulted to `(0,0,1)`. The insertion path encodes the
complete requested `210/220/230` tuple in canonical order and emits one
zero-width transaction patch. It anchors after one unique thickness group when
available, otherwise after the last of three unique location components;
duplicate thickness and missing or duplicate fallback location evidence are
typed failures. The shared insertion framing preserves LF, CRLF, CR, and Binary
without changing the replacement path. Verification still requires the exact
tuple with every component `Explicit`, and the inverse removes all inserted
bytes exactly across nine ASCII/Binary dialect pairs. Partial
explicit/default extrusion tuples, extrusion reset, angle updates, mixed
insert/update sessions, clone/delete, and POINT `Complete` remain open.

M14.3bv closes the remaining partial-tuple gap for `SetExtrusion`. Every mask
with one or two unique explicit components replaces those exact source spans
and inserts each consecutive missing component run at the canonical
`210/220/230` gap. The operation remains one logical edit, reports `Composite`,
and produces two or three physical transaction patches. Reordered partial
source evidence and duplicate components fail typed rather than being moved or
guessed. Local ASCII line endings and Binary framing are preserved. All six
partial masks pass every supported ASCII/Binary dialect with exact explicit
post-image semantics, tamper-resistant verification, and byte-identical inverse
restoration. Extrusion reset, angle updates, mixed entity insert/update
sessions, clone/delete, and POINT `Complete` remain open.

M14.3bw adds reset-to-default behavior for the extrusion patch identity.
`DxfPointPatch::ResetExtrusion` deletes every unique present group among
`210/220/230` as one logical edit with one to three physical patches. A fully
absent tuple returns `AlreadyImplicit`, queues nothing, and leaves a later set
admissible. Duplicate component evidence rejects the complete request before a
transaction escapes. Post-image verification requires both the documented
`(0,0,1)` value and `Defaulted` state on all three components; exact inverse
restoration recovers the original complete or partial source. All seven
nonempty masks pass every supported ASCII/Binary dialect. Angle updates, mixed
entity insert/update sessions, clone/delete, and POINT `Complete` remain open.

M14.3bx introduces typed set/update behavior for POINT UCS X-axis angle group
`50`. `DxfPointPatch::SetUcsXAxisAngle` replaces one unique explicit group or
inserts an absent group after the last explicit extrusion component. When the
extrusion tuple is fully absent, insertion falls back to unique thickness and
then three required location components. Duplicate angle, duplicate extrusion,
duplicate thickness, or unusable location anchors fail typed without queueing.
The angle patch identity composes independently with location, thickness, and
extrusion. All supported ASCII/Binary dialects pass explicit replacement and
all eight extrusion-mask insertion states with exact semantic verification and
byte-identical inverse restoration. Angle reset, mixed entity insert/update
sessions, clone/delete, and POINT `Complete` remain open.

M14.3by adds reset-to-default behavior for the POINT UCS X-axis angle patch
identity. `DxfPointPatch::ResetUcsXAxisAngle` deletes one unique explicit group
`50`; absent evidence returns `AlreadyImplicit`, queues nothing, and leaves a
later set admissible. Duplicate evidence rejects the request before a
transaction escapes, and set/reset share duplicate-patch admission. Post-image
verification requires the documented zero value with `Defaulted` state rather
than accepting an explicit zero. All supported ASCII/Binary dialects pass
strict reparse, semantic verification, and byte-identical inverse restoration;
focused coverage also retains CRLF framing. Mixed entity insert/update
sessions, clone/delete, and POINT `Complete` remain open.

M14.3bz admits POINT insertion and existing-entity updates in one unified edit
session regardless of call order. A shared logical-operation limit covers both
queues. Finalization composes the handle reservation and successor `$HANDSEED`,
all new POINT records, generated common-field patches, and typed POINT-family
patches into one immutable source-bound transaction. Same-offset insertion
fragments retain existing-record bytes before new records. Existing-record
postconditions add the exact count of earlier inserted records to their raw
record ordinal, including insertions into a preceding ENTITIES section. Every
supported ASCII/Binary dialect passes both API orders, earlier-section ordinal
shifts, strict semantic verification, and byte-identical inverse restoration.
Clone/delete and POINT `Complete` remain open.

M14.3ca adds the first reference-safe whole-entity delete to the unified CRUD
session. A standalone canonical POINT is selectable only when its record has
one non-null parsed handle, that handle resolves uniquely back to the selected
record, and no uniquely resolved pointer or owner occurrence from another raw
record targets it. The accepted operation deletes the exact complete raw
record span, retains a handle-absence semantic postcondition, and exposes an
inverse only after strict byte and semantic verification. Missing, invalid,
null, multiple, or document-ambiguous identities, incoming references, wrong
families, cancellation, and mixing with queued work remain typed fail-closed.
All nine ASCII/Binary dialect pairs prove exact deletion and byte-identical
restoration. Handleless deletion, mixed/multi-delete sessions, clone, and
POINT `Complete` remain open.

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
