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

M14.3cb adds canonical semantic POINT clone through
`DxfEntityEditSession::clone_entity`. The source must be a canonical POINT in
the requested existing container, modern owner identity must remain unchanged,
and every raw group must belong to the currently modeled canonical envelope.
Layer, applicable layout and lineweight, location, and the explicit/defaulted
state of thickness, extrusion, and UCS X-axis angle are reconstructed into a
typed draft; insertion then allocates a fresh handle and reuses the existing
owner/placement, `$HANDSEED`, strict semantic verification, and exact inverse
pipeline. Unsupported common properties, XDATA/application groups, partial or
invalid family semantics, pending source updates, placement drift, and owner
drift fail typed without queueing. Minimal/defaulted clones pass all nine
ASCII/Binary dialect pairs, and an all-explicit payload clone retains every
POINT family value. Cross-container/owner clone, broader common-property
clone, handleless/mixed deletion, and POINT `Complete` remain open.

M14.3cc closes handleless deletion for standalone canonical POINT records.
`DxfHandleIdentityState::Absent` is admitted through a distinct
`HandlelessApplied` outcome, while invalid, null, multiple, and ambiguous
identity remains rejected. Because no handle exists to use as a semantic
postcondition, the plan retains the exact expected post-image entity count;
strict reparse must prove that one entity disappeared before the executable
inverse is released. Exact raw-record removal, retained neighboring LINE,
semantic count verification, and byte-identical restoration pass every nine-
dialect ASCII/Binary pair. Mixed/multi-delete sessions and POINT `Complete`
remain open.

M14.3cd admits multiple distinct canonical POINT deletions in one delete-only
session. Every request independently retains the M14.3ca reference-safety and
M14.3cc identity rules, consumes the shared edit limit, and contributes its
exact raw-record transaction to one source-order composition. Handle-backed
postconditions require every deleted handle to disappear; all handleless
members share the exact final entity-count postcondition after the complete
batch. Duplicate keys fail typed without discarding accepted deletes. Two
handle-backed POINTs pass every nine-dialect ASCII/Binary pair, while a mixed
handle-backed/handleless batch independently proves combined verification and
byte-identical inverse restoration. Delete/update/insert mixing and POINT
`Complete` remain open.

M14.3ce unifies deletion with unrelated update and insertion work in one
session. Admission rejects only an update or clone whose source key is already
selected for deletion, and rejects deleting a key with pending updates; other
records may compose in either API order. Existing-record verification ordinals
add insertions placed before the source marker and subtract queued deletions
with earlier raw-record ordinals. Finalization composes common/POINT patches,
handle reservation, `$HANDSEED`, inserted records, and exact delete spans into
one transaction. Delete/update and delete/insert both pass every nine-dialect
ASCII/Binary pair in both API orders with strict semantic verification and
byte-identical inverse restoration. Broader clone and POINT `Complete` remain
open.

M14.3cf broadens canonical POINT clone to the reviewed singleton scalar common
fields. Exact explicit group 67 space, group 62 indexed color, group 48
linetype scale, group 60 visibility, group 420 true color, group 440
transparency, and group 284 shadow mode are classified into typed domains,
canonically re-encoded, and included in semantic post-image verification.
Defaulted or absent values remain omitted; invalid, duplicate, or
version-inapplicable values fail typed without queueing. All nine supported
versions pass ASCII/Binary clone and inverse verification. Linetype, material,
color-book, plot-style, proxy-graphics, XDATA/application groups, extension
dictionaries, ownership graphs, and POINT `Complete` remain open.

M14.3cg preserves an explicit common linetype name during canonical POINT
clone. The exact group 6 bytes must resolve to one same-document LTYPE table
entry, are canonically emitted through the typed POINT draft, and are checked
again by semantic post-image verification. Missing, ambiguous, invalid-scope,
or unsupported names queue nothing. All nine supported versions pass both
ASCII and Binary clone/inverse coverage. Material, plot-style, color-book, and
graph common properties plus POINT `Complete` remain open.

M14.3ch preserves AC1012+ material and plot-style common references during
canonical POINT clone. Group 347 and 390 handles must be non-null, unique in the
same document, and resolve respectively to MATERIAL and ACDBPLACEHOLDER records
in OBJECTS. The typed draft encoder validates targets before admission and the
post-image verifier requires both explicit handles. All applicable modern
ASCII/Binary dialect pairs pass strict reparse and exact inverse restoration.
Color-book and graph common properties plus POINT `Complete` remain open.

M14.3ci preserves the AC1012+ common color-book tuple during canonical POINT
clone. An explicit group 430 must have exactly one `$` separator with non-empty
book and color names, and requires valid explicit indexed and true colors in
groups 62 and 420. The typed draft rejects incomplete or malformed tuples before
admission; strict post-image verification requires all three exact values.
Every applicable ASCII/Binary dialect pair passes clone and inverse restoration.
Graph common properties plus POINT `Complete` remain open.

M14.3cj preserves matched AC1012+ proxy graphics during canonical POINT clone.
The source group-92 declaration must equal the decoded length of the complete
group-310 sequence. ASCII hexadecimal chunks are validated and decoded, Binary
chunks retain their payload bytes, and the typed draft emits one canonical size
plus bounded canonical chunks. Post-image verification compares both the size
relation and exact decoded payload before releasing the inverse. Mismatched or
malformed relations queue nothing. Graph-scoped handles, application groups,
XDATA, and POINT `Complete` remain open.

M14.3ck makes the existing standalone POINT delete fail closed around attached
record-local graphs. Before identity or incoming-reference admission, the
complete source record is scanned for group-102 application controls and
group-360 hard-owner occurrences. Either returns a typed rejection containing
the exact source occurrence and leaves the edit queue empty, including reactor,
extension-dictionary, custom/malformed application, and unscoped hard-owner
shapes. Every applicable AC1012+ ASCII/Binary dialect covers the three reviewed
graph shapes. This prevents orphan creation but does not claim graph-aware
cascade/remap, application-group/XDATA clone, or POINT `Complete`.

M14.3cl introduces generic entity XDATA evidence before any clone policy. For
every entity in the unified directory, one exact group 1001 opens a distinct
registered-application list and following groups in the inclusive 1000..=1071
range retain source order through the next 1001 or raw-record boundary. A
normal entity group interrupts an open list; later XDATA codes without a new
1001 remain explicit orphans. Duplicate application names remain separate,
and XDATA-shaped codes inside group-102 application controls are excluded from
this directory. All nine ASCII/Binary dialect pairs share the same index.
APPID resolution, name and group-1002 brace validation, typed XDATA values,
16-KiB enforcement, application payload meaning, clone/write, and POINT
`Complete` remain open.

M14.3cm resolves each generic entity XDATA application name against the APPID
symbol table without normalizing source bytes. The named-symbol scanner admits
APPID records only from exact, completely closed APPID tables and requires one
outside-application-control group-2 name per record. A sorted digest index
narrows lookup candidates, but exact source-span equality decides matches and
therefore remains collision safe. Missing, unique, and ambiguous results keep
their source application; unique results retain the exact APPID record. All
nine ASCII/Binary dialect pairs cover exact, duplicate, missing, case-near,
wrong-table, malformed, and unclosed evidence plus source identity,
cancellation, lookup limits, and metadata bounds. Name syntax and length,
group-1002 brace validation, typed XDATA values, 16-KiB enforcement, payload
meaning, group-1005 remap, clone/write, and POINT `Complete` remain open.

M14.3cn validates generic entity XDATA list structure without interpreting
application payloads. Each group-1001 name retains its exact source bytes and
fails typed above Autodesk's 31-byte ceiling. Exact group-1002 `{` and `}`
values maintain a per-application nesting counter; malformed controls,
premature closing braces, remaining open lists, and an application interrupted
by a normal group are separately observable. All nine ASCII/Binary dialect
pairs cover balanced nesting and every reviewed failure plus source identity,
cancellation, lookup limits, metadata bounds, and redacted debug output.
Complete symbol-name character policy, typed value domains, 16-KiB
enforcement, application semantics, group-1005 remap, clone/write, and POINT
`Complete` remain open.

M14.3co projects the documented generic XDATA wire domains without assigning
application-specific meaning. Every retained occurrence has one stable typed
entry: exact text/control provenance, bounded caller-buffer binary chunks,
handles, 15 distinct double roles, signed 16-bit integers, signed 32-bit
integers, or a source-anchored invalidity. The 255-byte group-1000 and 127-byte
decoded group-1004 ceilings are enforced, non-finite doubles and invalid ASCII
numbers fail typed, and unknown XDATA codes are never discarded. All nine
ASCII/Binary dialect pairs cover valid parity, malformed domains, source
identity, cancellation, lookup bounds, metadata bounds, and non-disclosing
debug output. Layer-name resolution, point/vector tuple grouping and
transforms, the per-entity 16-KiB policy, application semantics, group-1005
target resolution/remap, clone/write, and POINT `Complete` remain open.

M14.3cp groups the four documented 3D XDATA families before size accounting or
transform semantics. A tuple candidate contains only immediately adjacent,
same-context components of one 101x/102x/103x suffix. Complete X/Y/Z candidates
and every partial axis set remain separately visible; numeric invalidity stays
on the generic typed member. Compact ordinal links avoid copying typed entry
metadata, while tuple and member lookup reject foreign sources and forged
relationships. All nine ASCII/Binary dialect pairs cover the four families,
partial, reordered, duplicated, interrupted, application-boundary, entity-
boundary, orphan, invalid-number, source-identity, cancellation, lookup-bound,
metadata-bound, and non-disclosing-debug cases. Coordinate transforms,
layer-name resolution, per-entity 16-KiB accounting, application semantics,
group-1005 target resolution/remap, clone/write, and POINT `Complete` remain
open.

M14.3cq resolves generic XDATA group-1003 values against the exact LAYER symbol
table before capacity accounting. Every layer occurrence receives one stable
source-bound result: unique target, missing, or ambiguous target count. A
sorted SHA-256 index narrows candidates, followed by authoritative byte-exact
source-span comparison. Only matching records with exactly one group-2 name in
a completely closed LAYER table are admitted; near-case, wrong-table,
malformed-record, and unclosed-table evidence fails closed. All nine
ASCII/Binary dialect pairs cover application values and orphans, exact and
near-case matches, duplicate targets, source identity, cancellation, lookup
bounds, compact metadata, and non-disclosing debug output. Per-entity 16-KiB
accounting, coordinate transforms, application semantics, group-1005 target
resolution/remap, clone/write, and POINT `Complete` remain open.

M14.3cr accounts AutoCAD's logical XDATA storage per indexed entity. A
source-bound directory publishes exact within-limit or exceeded totals against
the observed 16,383-byte `xdroom` ceiling; inputs that cannot produce a valid
AutoCAD XDATA list publish only an accounted lower bound plus typed issues.
The formula is verified independently with AutoCAD 2027 `xdsize`: a nonempty
application costs three bytes, strings cost three plus twice their decoded
Unicode-scalar count, controls cost two, resolved layers three, chunks two plus
decoded length, handles nine, complete point/vector tuples twenty-five,
remaining doubles nine, int16 values three, and int32 values five. Empty
applications cost zero. APPID/layer resolution, structure, typed values, tuple
completeness, storage decoding, and CIF/MIF decoding compose fail-closed.
All nine ASCII/Binary dialect pairs cover every logical value family, Unicode,
multiple/empty applications, exact 16,383 and exceeded 16,384 totals, source
identity, cancellation, metadata bounds, and non-disclosing debug output.
Coordinate transforms, application semantics, group-1005 target
resolution/remap, clone/write, and POINT `Complete` remain open.

M14.3cs transforms the four generic XDATA tuple roles through separately
retained position, displacement, and direction affine channels. AutoCAD 2027
behavior establishes that group 1010 is invariant, group 1011 receives move/
uniform-scale/rotation/mirror effects, group 1012 omits movement, and group 1013
also omits scale. Validated translation, base-point scale, axis rotation, and
mirror-plane factories compose in source-independent order while rejecting
non-finite inputs, zero scale, zero axes/normals, and derived matrix overflow.
A source-bound directory emits one entry per M14.3cp tuple and publishes both
original and transformed values only for complete finite typed tuples; partial,
invalid, and derived-overflow inputs remain explicit. All nine ASCII/Binary
dialect pairs cover the four roles, composed oracle sequence, invalid factories,
source identity, cancellation, lookup and metadata bounds, and non-disclosing
debug output. Application semantics, group-1005 target resolution/remap,
clone/write, and POINT `Complete` remain open.

M14.3ct resolves every generic XDATA group-1005 occurrence through the shared
document-local handle identity/reference directories. One compact source-bound
entry links back to its exact typed occurrence and generic resolution. Invalid,
null, missing, ambiguous, and unique states remain distinct; a unique state
alone retains the exact target record and identity candidate. Application
values and orphan occurrences stay separately visible, while group-1005-shaped
data inside group-102 application controls never enters the projection. All
nine ASCII/Binary dialect pairs cover every state, duplicate identities,
application/orphan context, group-102 exclusion, foreign-source rejection,
cancellation, lookup and metadata bounds, and non-disclosing debug output.
Cross-document handle remap, application semantics, clone/write, and POINT
`Complete` remain open.

M14.3cu maps uniquely resolved XDATA group-1005 sources to caller-supplied
destination handles. Non-null source/target candidates are fallibly copied and
sorted. Invalid, null, missing, and ambiguous source resolution remains typed;
a unique source becomes unmapped, mapped only by exactly one candidate, or
ambiguous with the exact duplicate-candidate count. Only the mapped state
publishes a destination handle, while source target evidence remains reachable.
Entries carry exact source identity so a structurally identical foreign ordinal
cannot pass lookup. All nine ASCII/Binary dialect pairs cover every source and
mapping state, shuffled/duplicate candidates, null rejection, foreign-source
rejection, cancellation, lookup and metadata bounds, and non-disclosing debug
output. Destination identity validation, application semantics, clone/write,
and POINT `Complete` remain open.

M14.3cv composes remap planning with a separately supplied destination
document. Unusable source/remap states propagate unchanged. An exactly mapped
destination handle resolves as missing, unique, or ambiguous against the
destination identity index; only a unique result makes its exact destination
record and identity candidate available. Entries carry both source and
destination identities, and unique target evidence is derived through the
owned destination directory instead of copied into every entry. ASCII-to-ASCII,
ASCII-to-Binary, Binary-to-ASCII, and Binary-to-Binary source/destination pairs
cover all nine Core dialects plus every remap/destination state, duplicate
identities, dual-source foreign rejection, cancellation, lookup and metadata
bounds, and non-disclosing debug output. Application semantics, replacement
encoding, clone/write, and POINT `Complete` remain open.

M14.3cw composes destination validation with the canonical entity group
encoder. Only `Unique` destination identity emits one complete group 1005 in
the destination's supported dialect; all other states propagate without bytes.
Ready output uses uppercase hexadecimal through the full 64-bit handle domain,
exact ASCII/Binary framing, and AC1009 Binary extended-data group-code escape.
Entries retain both document identities, foreign lookup fails closed, and byte
storage is bounded and redacted from debug output. All four source/destination
format pairs cover all nine Core dialects plus cancellation and metadata bounds.
Application semantics, transaction composition, clone/write, and POINT
`Complete` remain open.

M14.3cx publishes one compact replacement patch only for each M14.3cw ready
entry. It traces the owned destination/remap/resolution/typed chain back to the
exact source group-1005 occurrence and complete raw span, then binds that group,
target, replacement ordinal, source identity, and destination identity. Byte
lookup revalidates the complete patch; foreign or unavailable entries cannot
release bytes. The binding passes all source/destination format pairs and nine
Core dialects with cancellation, lookup bounds, and debug redaction. Application
semantics, transaction composition, clone/write, and POINT `Complete` remain
open.

M14.3cy groups source-ordered M14.3cx replacement candidates by exact source
entity. A compact set becomes ready only if every retained group-1005 member has
a ready patch; mixed or wholly unavailable sets retain total member count,
unavailable count, first unavailable ordinal, and their original per-member
states. Slice and patch lookup revalidate set identity and membership before
exposing evidence. Ready, mixed, and unavailable sets cover all four
source/destination format pairs and all nine Core dialects with cancellation,
metadata bounds, and foreign-entry rejection. Application semantics,
transaction composition, clone/write, and POINT `Complete` remain open.

M14.3cz creates an all-or-nothing staging transaction for one M14.3cy ready set
when source and destination have identical format and supported dialect state.
Every member's exact complete group span is replaced with its destination-
validated canonical bytes; inverse capture retains each original group. Typed
unavailable outcomes cover incomplete sets and format/dialect mismatch before a
builder escapes. Same-format ASCII/Binary plans and every cross-format rejection
pass all nine Core dialects with source precondition and exact inverse checks.
Destination clone/write verification, application semantics, and POINT
`Complete` remain open.

M14.3da strictly verifies the complete staged post-image before exposing an
inverse journal. The shared transaction verifier proves unchanged regions and
replacement bytes; format-neutral handle projection then proves each original
group occurrence reparses to its expected destination target. The receipt binds
source, destination, post-image, and exact replacement count. Same-format
ASCII/Binary post-images across all nine Core dialects cover exact inverse
restoration, cancellation, and tamper rejection. Destination clone/write
integration, application semantics, and POINT `Complete` remain open.

M14.3db writes one M14.3da-verifiable staging plan to a create-new path through
the shared bounded transaction writer, then strictly reparses the exact ASCII or
Binary output and reruns replacement verification before releasing a combined
write/verification journal and executable inverse. Source, destination, set,
post-image, patch-count, and replacement-count bindings remain exact. Foreign
evidence, source mismatch, existing paths, cancellation, strict-reparse failure,
and raw tampering fail closed; every post-creation failure removes the output.
Same-format ASCII/Binary writes and byte-identical inverse restoration cover all
nine Core dialects. This is a verified staged source clone, not insertion into
the separately parsed destination document. Cross-container clone, application
semantics, and POINT `Complete` remain open.

M14.3dc adds destination APPID validation without mutating either document.
`DxfEntityXDataAppIdDestinationDirectory` owns the M14.3cm source-resolution
directory and the independently parsed destination named-symbol directory.
Source-missing and source-ambiguous states take precedence. Each source-unique
group-1001 name is narrowed through a sorted SHA-256 destination index and then
compared byte-for-byte against exact group-2 names from completely closed APPID
tables. Results publish destination missing, unique, or ambiguous state; only a
unique result derives the exact owned destination APPID record. Four
ASCII/Binary source-destination pairings span all nine Core dialects, plus
cross-dialect boundary cases, long chunked names, malformed and wrong-table
evidence, case-near rejection, dual-source identity, cancellation, bounds, and
debug redaction. This does not validate APPID syntax or the 31-byte limit,
create or edit APPID records, interpret group-1002 or application payloads,
compose coordinate/layer/handle transforms, encode or insert XDATA, mutate a
destination, implement cross-container clone, or advance POINT to `Complete`.

M14.3dd adds destination LAYER validation without mutating either document.
`DxfEntityXDataLayerDestinationDirectory` owns the M14.3cq source-resolution
directory and the independently parsed destination named-symbol directory.
Source-missing and source-ambiguous states take precedence. Each source-unique
group-1003 name is narrowed through a shared sorted SHA-256 destination index
and then compared byte-for-byte against exact group-2 names from completely
closed LAYER tables. Results publish destination missing, unique, or ambiguous
state; only a unique result derives the exact owned destination LAYER record.
Four ASCII/Binary source-destination pairings span all nine Core dialects plus
cross-dialect boundary cases, long chunked names, orphan and application
occurrences, malformed and wrong-table evidence, case-near rejection, dual-
source identity, cancellation, bounds, and debug redaction. This does not
validate layer-name syntax or length, create or edit LAYER records, interpret
application payloads, compose coordinate or handle transforms, encode or insert
XDATA, mutate a destination, implement cross-container clone, or advance POINT
to `Complete`.

M14.3de adds per-application destination symbol readiness without mutating
either document. `DxfEntityXDataSymbolDestinationDirectory` owns the M14.3dc
APPID and M14.3dd LAYER destination directories. Each source application is
ready only when its APPID and every application-bound group-1003 occurrence
have unique destination records. All exact APPID and LAYER blockers accumulate
in one deterministic issue range, preserving source/destination missing and
ambiguity states; only ready entries derive their owned destination APPID and
unique LAYER occurrences. Orphan layer values remain visible through the owned
LAYER directory and are not guessed into an application. Four ASCII/Binary
source-destination pairings span all nine Core dialects plus cross-dialect
boundaries, accumulated failures, malformed evidence, case-near rejection,
dual-source identity, cancellation, bounds, and debug redaction. This does not
compose structure, capacity, transformed coordinates, handle remaps, payload
semantics, encoding/insertion, destination mutation, cross-container clone, or
POINT `Complete`.

M14.3df adds per-application structure composition without mutating either
document. `DxfEntityXDataApplicationDestinationDirectory` owns the M14.3de
symbol destination directory and M14.3cn source structure directory. An entry
is ready only when every destination APPID/LAYER symbol is unique and the exact
group-1002 list structure is valid. Unavailable entries retain independent
symbol and structure issue counts, and callers derive the exact owned evidence
for each class. Four ASCII/Binary source-destination pairings span all nine Core
dialects plus cross-dialect boundaries, symbol-only, structure-only, and joint
failure, complete and unclosed lists, entity lookup, dual-source identity,
cancellation, bounds, and debug redaction. Orphan values remain outside
application results. This does not compose per-entity capacity, transformed
coordinates, handle remaps, payload semantics, encoding/insertion, destination
mutation, cross-container clone, or POINT `Complete`.

M14.3dg adds per-entity capacity composition without mutating either document.
`DxfEntityXDataEntityDestinationDirectory` owns the M14.3df application
destination directory and M14.3cr capacity directory. An indexed entity is
ready only when all applications are ready and source XDATA is within the exact
16,383-byte limit. Unavailable entries preserve application counts and the full
capacity state, distinguishing destination-only failure, indeterminate source
capacity, and independently exceeded capacity. Zero-XDATA indexed records stay
visible as ready zero-byte entries. Four ASCII/Binary pairings span all nine
Core dialects plus cross-dialect boundaries, dual-source identity,
cancellation, bounds, and debug redaction. Coordinate transforms, handle
remaps, payload semantics, encoding/insertion, destination mutation, cross-
container clone, and POINT `Complete` remain open.

M14.3dh adds per-entity coordinate composition without mutating either
document. `DxfEntityXDataCoordinateDestinationDirectory` owns M14.3dg entity
readiness and M14.3cs transformed tuples. Ready requires both base readiness and
all entity tuples available; zero-tuple entities remain explicit. Unavailable
preserves base state and tuple counts, while exact typed partial, invalid, and
non-finite-derived evidence stays in the owned transform directory. Four format
pairings span all nine dialects plus cross-dialect, role-specific 1010/1011,
destination-only, dual-source, cancellation, bounds, and redaction cases.
Handle remaps, payload semantics, encoding/insertion, mutation, cross-container
clone, and POINT `Complete` remain open.

M14.3di adds per-entity handle composition without mutating either document.
`DxfEntityXDataHandleComposedDestinationDirectory` owns M14.3dh coordinate
readiness and M14.3cv destination-validated group-1005 evidence built from
caller-supplied remaps. Ready requires the coordinate state ready and every
entity handle uniquely present in the destination; zero-handle entities remain
ready. Unavailable preserves the complete coordinate state and exact total and
unavailable handle counts, while the owned handle directory retains every
typed remap/destination result. Four format pairings span all nine dialects plus
AC1009/AC1032 cross-dialect boundaries, independent coordinate and handle
failures, zero handles, dual-source identity, cancellation, bounds, and
redaction. Application payload semantics, encoding/insertion, mutation, cross-
container clone, and POINT `Complete` remain open.

M14.3dj adds per-entity payload-envelope readiness without mutating either
document. `DxfEntityXDataPayloadDestinationDirectory` owns M14.3di and reuses
its exact typed occurrence/application evidence instead of rebuilding a source
directory. Payload counts exclude group-1001 names; orphan values receive a
direct count. Ready requires M14.3di Ready and no orphan values. Unavailable
preserves the complete handle-composed state plus exact payload and orphan
counts, while callers can recover the complete typed and application slices.
Four format pairings span all nine dialects plus AC1009/AC1032 cross-dialect
boundaries, enclosed payloads, orphans, independent handle and coordinate
failures, zero payloads, dual-source identity, cancellation, bounds, and
redaction. Application-specific meaning, destination logical-value projection,
encoding/insertion, mutation, cross-container clone, and POINT `Complete`
remain open.

M14.3dk adds per-occurrence logical destination projection without encoding or
mutating either document. `DxfEntityXDataLogicalDestinationDirectory` owns
M14.3dj and emits exact source-referenced string/control/binary/scalar/integer
values, destination APPID/LAYER targets, remapped destination handles, and
transformed coordinate components. Orphan, invalid-source, APPID, LAYER,
handle, and coordinate failures remain independent typed unavailable states.
Four format pairings span all nine dialects plus AC1009/AC1032 cross-dialect
boundaries, every generic logical family, specialized/source-exact independence,
per-entity slicing, dual-source identity, cancellation, bounds, and redaction.
Application-specific meaning, destination text transcoding, byte encoding/
insertion, mutation, cross-container clone, and POINT `Complete` remain open.

M14.3dl adds canonical destination group encoding without insertion or
mutation. `DxfEntityXDataEncodedDestinationDirectory` owns M14.3dk and stores
bytes only for available logical entries under a supported destination dialect.
It reuses the shared ASCII/Binary encoder, reads APPID/LAYER names from exact
destination spans, and reads source-exact text/control/chunk values through
bounded provenance. ASCII-only text is portable across reviewed encodings;
non-ASCII source text requires matching decoders or remains typed transcoding-
required. Logical, dialect, transcoding, and encoder blockers expose no bytes.
Four format pairings span all nine dialects plus AC1009/AC1032 cross-dialect
boundaries, every generic wire family, AC1009 extended-data escapes, decoder
mismatch, dual-source identity, cancellation, bounds, and redaction.
Application-specific meaning, actual text transcoding, application/entity
grouping, insertion, mutation, cross-container clone, and POINT `Complete`
remain open.

M14.3dm groups M14.3dl encoded occurrences into exact per-application
destination sets without insertion or mutation.
`DxfEntityXDataEncodedApplicationDestinationDirectory` owns M14.3dl and emits
one source-order entry for every exact source application. Ready requires the
owning M14.3dj payload envelope ready and every application occurrence encoded;
orphans are never assigned. Unavailable entries retain the full payload state,
total and unavailable member counts, the first unavailable member ordinal,
exact application evidence, and every underlying encoded result, while
aggregate bytes remain fail-closed. Four format pairings span all nine dialects
plus AC1009/AC1032 cross-dialect boundaries, ready/unavailable siblings,
empty applications, nested controls, source-exact values, transformed tuples,
remapped handles, transcoding blockers, orphans, dual-source identity,
cancellation, bounds, and redaction. Application-specific meaning, actual text
transcoding, per-entity grouping, insertion, mutation, cross-container clone,
and POINT `Complete` remain open.

M14.3dn groups M14.3dm application sets into exact per-entity encoded payloads
without insertion or mutation. `DxfEntityXDataEncodedEntityDestinationDirectory`
owns M14.3dm and emits one entry per indexed source entity. Ready requires the
M14.3dj payload envelope and every application set ready; zero-XDATA entities
remain explicit with empty bytes. Unavailable entries retain the full payload
state, application/member totals, unavailable counts, and first unavailable
application ordinal, while aggregate bytes remain fail-closed. Four format
pairings span all nine dialects plus AC1009/AC1032 cross-dialect boundaries,
ready, zero-XDATA, failed-handle and orphan entities, empty/nested applications,
dual-source identity, cancellation, bounds, and redaction. Application-specific
meaning, actual text transcoding, insertion, mutation, cross-container clone,
and POINT `Complete` remain open.

M14.3do composes one M14.3dn ready entity payload with a canonical destination-
bound entity draft record without insertion or mutation.
`DxfEntityXDataDraftRecordPlan` requires exact encoded-entry membership and the
draft record's destination source identity, appends complete canonical XDATA
groups after the family record, and retains source-entity/encoded-state
evidence. Zero-XDATA is an exact no-op; unavailable payloads, cancellation,
foreign entries, foreign destination drafts, and bounded-growth failures are
fail-closed. Four format pairings span all nine dialects plus AC1009/AC1032
cross-dialect boundaries, exact prefix/suffix bytes, zero-XDATA, orphan
payloads, dual-source identity, cancellation, bounds, and redaction. Insertion,
post-write XDATA verification, application-specific meaning, actual text
transcoding, cross-container clone completion, and POINT `Complete` remain
open.

M14.3dp composes M14.3do with the existing destination draft-insertion
transaction without writing. `DxfEntityXDataDraftInsertPlan` retains the exact
expected XDATA suffix, source entity, encoded entry/state, destination identity,
and family edit plan. The atomic transaction reserves the new handle and
inserts the complete canonical family-plus-XDATA record; zero-XDATA retains an
explicit empty expectation. Cancellation and foreign destinations fail before
publishing a plan. Four format pairings span all nine dialects plus
AC1009/AC1032 cross-dialect boundaries, transaction shape, exact payload
suffixes, zero-XDATA, dual-source identity, cancellation, bounds, and redaction.
Destination writes, post-write XDATA verification, application-specific
meaning, actual text transcoding, cross-container clone completion, and POINT
`Complete` remain open.

M14.3dr adds create-new writing for M14.3dp. The atomic destination transaction
is streamed only to a nonexistent path, strictly reparsed as ASCII or Binary,
and passed through complete M14.3dq family-plus-XDATA verification. The journal
binds write and verification receipts and retains the executable inverse.
Existing destinations are untouched; cancellation creates nothing; any failure
after creation removes the destination. Four format pairings span all nine
dialects plus AC1009/AC1032 cross-dialect boundaries, non-empty and zero-XDATA
writes, exact output bytes, receipt identities, inverse restoration, existing-
file rejection, cancellation, final-progress tampering, cleanup, bounds, and
redaction. Application-specific meaning, actual text transcoding, generic
source-entity-to-family draft projection, cross-container clone completion, and
POINT `Complete` remain open.

M14.3ds extracts the existing same-document POINT clone semantics into an
immutable source snapshot and projects it into a separately parsed destination
family draft without insertion. Explicit destination-local bindings cover
layer, layout, linetype, material, and plot style; existing destination draft
validation remains authoritative. Reviewed common scalar fields, exact proxy
graphics, and POINT geometry remain intact. Four format pairings span all nine
dialects, with AC1009/AC1032 boundary behavior, same-document rejection,
missing/ambiguous layers, unexpected bindings, cancellation, and redaction.
XDATA composition with this family projection, complete cross-container
insertion/write, application-specific meaning, actual text transcoding, and
POINT `Complete` remain open.

M14.3dt selects one exact encoded-XDATA entity entry, derives its owning source
POINT, performs M14.3ds family projection in an internal XDATA-composition
mode, and immediately appends the M14.3dn payload through M14.3do. The result
retains source semantic provenance, dual-source identities, the exact encoded
entry, and the complete family-plus-XDATA draft. Standalone family projection
still rejects XDATA. Four format pairings span all nine dialects plus AC1009-
to-AC1032, with downstream insertion, strict post-image verification, exact
inverse restoration, cancellation, foreign identity, unavailable/orphan
payloads, bounds, traits, and redaction. A direct insertion/create-new wrapper,
application-specific meaning, actual text transcoding, reverse-boundary field
adaptation, and POINT `Complete` remain open.

M14.3du consumes M14.3dt into an atomic destination insertion plan and retains
compact source POINT identity, dialect, placement, and owner evidence through
dedicated strict-verification and create-new write journals. It delegates to
the existing family-plus-XDATA transaction, receipts, cleanup, and executable
inverse. Four format pairings span all nine dialects plus AC1009-to-AC1032
through exact file creation and strict reparse. Existing destinations remain
unchanged; cancellation creates nothing; final-progress tampering removes the
created file. Foreign destination planning, identity, bounds, traits, and
redaction remain fail-closed. Application-specific meaning, actual text
transcoding, reverse-boundary adaptation, higher entity families, and POINT
`Complete` remain open.

M14.3dv admits AC1032-to-AC1009 POINT cloning only for two reviewed semantic
equivalences: destination placement/owner carries the modern layout role, and
explicit `BY_LAYER` lineweight may be omitted where group 370 is inapplicable.
A compact adaptation mask survives family projection, XDATA composition,
insertion, verification, and write journals. Any other explicit lineweight is
typed `DestinationFieldNotRepresentable`; no approximation occurs. Four format
pairings cover both AC1009/AC1032 directions through strict create-new writing,
verification, and exact inverse restoration. Application-specific XDATA
meaning, actual text transcoding, non-default modern common fields, higher
entity families, and POINT `Complete` remain open.

M14.3dw introduces the missing replacement-free destination encoder and a
dual-document exact-text transcoding primitive. It reads one bounded source
span, resolves source and destination storage independently, rejects malformed,
unsupported, unmappable, output-full, and unavailable cases with typed
evidence, and publishes destination bytes only after decoding them back to the
same byte-exact UTF-8 sequence. The immutable plan binds both document
identities, the source span, resolutions, counts, and encoded bytes; debug
output exposes counts but not text. All four ASCII/Binary format pairings
cover UTF-8 and Windows-1252 in both directions, with same/cross-legacy, empty,
Johab-unavailable, malformed, indeterminate, cancellation, resource-bound,
traits, and redaction cases. Integration into actual POINT/XDATA fields,
application-specific meaning, higher entity families, and POINT `Complete`
remain open; destination-bound symbol names are not implicitly transcoded.

M14.3dx integrates M14.3dw into exact group-1000 XDATA string encoding.
Different reviewed source/destination storage decisions now use replacement-
free, round-trip-verified destination bytes; portable ASCII and matching
available decoders preserve exact source bytes. Compact receipts bind the two
documents, source span, resolutions, and byte counts without expanding each
hot entry or disclosing text. APPID/LAYER occurrences remain exact destination
symbol targets and list controls remain source-exact. A converted destination
string above 255 bytes is typed unavailable before group encoding. All four
ASCII/Binary pairs cover UTF-8/Windows-1252 both ways through application and
entity grouping, while one Binary AC1018-to-ASCII AC1021 POINT clone passes
create-new writing, strict reparse, exact XDATA verification, and inverse
restoration. Unmappable, malformed, unsupported, indeterminate, unavailable,
over-limit, cancellation, identity, metadata-bound, trait, and redaction cases
fail closed. Application-specific meaning, automatic symbol creation, other
entity text fields, higher families, and POINT `Complete` remain open.

M14.3dy integrates verified storage transcoding for the reviewed POINT
group-430 color-book name. Clone snapshots retain exact field provenance and
bytes for common text, while layer/layout/linetype remain explicit destination
bindings and only color name may convert storage. Portable ASCII and matching
available decoders stay byte-exact. The successful receipt binds field span,
dual identities/resolutions, and counts through family/XDATA draft, insertion,
strict verification, and create-new write journals. UTF-8/Windows-1252 both
ways span all four ASCII/Binary pairs; one Binary AC1018-to-ASCII AC1021 clone
passes write/reparse, color and XDATA postconditions, and exact inverse.
Unmappable text is typed and AC1009 rejects group 430 as non-representable. A
four-byte bounded verifier slack fixes exact-length legacy round trips without
raising payload ceilings. Other free-text common fields, application-specific
XDATA meaning, automatic symbol creation, higher families, and POINT
`Complete` remain open.

M14.3dz audits the six-level completion contract against the accumulated POINT
evidence and publishes the first typed completion-ledger entry. POINT reaches
`VerifiedMutation` (level 5): exact record/field evidence, fixed cardinality,
typed/defaulted semantics, exact WCS geometry, and source-bound create/update/
clone/delete with strict create-new verification and executable inverse are
all evidenced. It does not reach `ReleaseQualified`: no passing private
1,000-file/10-GiB corpus receipt exists, and six-native run `30557566354`
belongs to commit `222eec2c9d9b18fbb7ff1b8d5f0120ba30633365` rather than the current
checkpoint. Those gaps remain typed as `PrivateCorpusQualification` and
`CurrentCheckpointSixNativeCi`. The other 44 public topics intentionally have
no ledger entry until independently audited. Rendering, POINT display
behavior, application-specific XDATA interpretation, and automatic symbol
creation remain separate capabilities and do not prevent the evidenced level-5
assessment. POINT is not called `Complete`.

M14.3ea consumes only M14.3q2 `Available` SPLINE analytic data and evaluates
one requested point through a bounded homogeneous De Boor recurrence. The
source-bound lookup accepts the closed active knot interval, selects the
deterministic active span, preserves positive rational weights, and
canonicalizes derived zero without changing source evidence. Degree 64 is the
public hard ceiling, bounding scratch space
to 65 four-component points and recurrence work to 4,096 interpolation steps.
Unavailable analytic records, non-finite or out-of-domain parameters,
non-finite inputs, degenerate denominators, arithmetic overflow, cancellation,
allocation failure, and missing record ordinals remain distinct outcomes. All
nine dialects have ASCII/Binary endpoint/midpoint parity, and an independent
rational fixture proves homogeneous division. Sampling, derivatives,
tessellation, rendering, HELIX evaluation, SPLINE CRUD/write, and SPLINE
`Complete` remain open.

M14.3eb refactors M14.3ea's validated homogeneous-control preparation and De
Boor recurrence into one internal math path, then adds a source-bound rational
first derivative. Degree-scaled differences form the local derivative control
polygon over the trimmed knot vector; a degree-minus-one recurrence produces
the homogeneous derivative, and the rational quotient rule projects it into
the WCS Cartesian derivative while returning the evaluated point beside it.
The degree-64 ceiling bounds simultaneous scratch storage to 129
four-component points and both triangular recurrences to 4,096 combined steps.
All nine dialects have ASCII/Binary point-and-derivative bit parity at both
endpoints and the midpoint; a nonconstant-weight fixture independently proves
the quotient rule. The complete M14.3ea failure/cancellation contract is reused,
and a zero derivative remains valid evidence rather than being normalized or
rejected. Tangent normalization/frame policy, higher derivatives, curvature,
adaptive sampling, tessellation, rendering, HELIX evaluation, SPLINE CRUD/
write, and SPLINE `Complete` remain open.

M14.3ec audits the curve-family boundary before advancing to M14.4. SPLINE is
recorded at `Geometry`, level 4: M14.3a/q2 prove levels 1-3, while M14.3ea/eb
prove bounded rational WCS point and first-derivative evaluation. Its typed
blockers are verified mutation plus the private-corpus and current-checkpoint
six-native release gates. HELIX is recorded at `TypedSemantics`, level 3:
M14.3r-w prove exact subclass evidence, cardinality, scalar/vector semantics,
cross-field relations, and embedded-SPLINE readiness. Autodesk's explicit
warning that inherited NURBS operations on HELIX have unknown behavior and are
not recommended prevents a public stored-curve evaluation claim; this remains
`PublicGeometryQualification`, followed by verified mutation and release
evidence blockers. POINT remains level 5, the three assessments are sorted by
topic ordinal, and the other 42 topics remain unaudited. This checkpoint does
not call either curve `Complete`; family mutation closure remains M14.11.

M14.4a introduces an exact, fail-closed evidence foundation shared by HATCH
and modern MESH without interpreting their nested structures. Canonical exact
uppercase records are admitted only from complete BLOCKS and ENTITIES ranges.
Each exact `AcDbHatch` or `AcDbSubDMesh` marker creates a separate subclass
scope, so missing, reversed, duplicate, case-variant, and near markers remain
observable without leaking fields across scope boundaries. Every ordinary raw
group in a matching scope retains its source occurrence, code, payload span,
family, subclass ordinal, record boundary, and source identity; application
groups and XDATA remain available through the existing raw layers rather than
being mislabeled as family fields. All nine Core dialects have ASCII/Binary
shape parity. This evidence recognition does not imply dialect applicability,
semantic validity, boundary/path roles, mesh topology, geometry, subdivision,
CRUD/write, or `Complete` support. HATCH applicability remains unreviewed and
the existing AC1024 MESH boundary remains unchanged.

M14.4b consumes M14.4a but assigns roles only to 25 HATCH codes that do not
collide anywhere in Autodesk's boundary-path, pattern-line, seed-point, or
gradient grammar. Pattern/gradient names remain exact source-backed text;
elevation Z, extrusion components, pattern angle/scale, pixel size, and
gradient doubles retain exact binary64 bits; flags/style/type/group 78 decode
as signed Int16; path/seed counts and groups 450-453 decode as signed Int32.
Invalid ASCII numerics and duplicates remain ordered evidence. Every exact
`AcDbHatch` occurrence is independent, including duplicate subclasses, while
MESH and absent/near subclass markers contribute no scalar entries. All nine
Core dialects have paired ASCII/Binary evidence; AC1009 omits group 450-470 in
both formats because its Binary entity group-code grammar cannot encode those
codes. This wire boundary does not establish HATCH applicability. Cardinality,
semantic defaults/domains, elevation/seed tuples, nested-state partitioning,
boundary paths, pattern lines, gradient relations, geometry, CRUD/write, and
`Complete` remain open.

M14.4c builds 25 fixed cards for every exact M14.4b `AcDbHatch` subclass.
Cards remain ordered by the public role inventory and retain the subclass
entry, compact member range, and explicit absent/unique/multiple state. Members
reference global typed-occurrence ordinals, so raw groups and payloads are not
copied. Duplicate subclass markers in one raw record remain independent; a
record-level lookup exposes the contiguous aggregate without merging roles.
Invalid numeric values retain their members and do not change cardinality. All
nine Core dialects have paired ASCII/Binary card/member parity; AC1009 exposes
the nine high-code roles as absent. This checkpoint does not select values or
add defaults, domains, semantic relations, tuples, nested HATCH state,
geometry, applicability, CRUD/write, or `Complete` support.

M14.4d selects every M14.4c card into one source-anchored four-state semantic.
Unique values are explicit; duplicates are invalid without member selection;
malformed ASCII and non-finite Binary doubles retain exact raw provenance.
Absent extrusion components receive the documented `(0,0,1)` defaults, while
all other absent roles remain absent. Reviewed individual domains cover binary
flags, style/type enums, nonnegative counts, gradient reserved values,
color-mode/count, shift, and tint. Gradient defaults remain absent until a
later group-450 envelope relation proves they apply. Every field has stable
`entity.hatch` provenance and all nine Core dialects have ASCII/Binary semantic
parity, including nine AC1009 high-code absences. This checkpoint does not add
tuple assembly, cross-field/gradient relations, nested boundary or pattern
state, geometry, applicability, CRUD/write, or `Complete` support.

M14.4e assembles the extrusion X/Y/Z semantics into one exact tuple per
`AcDbHatch` subclass. Each component preserves explicit or defaulted input
kind; partially explicit tuples use the reviewed independent `0/0/1` defaults.
Invalid, duplicate, malformed, or non-finite evidence returns a compact mask of
unavailable components, while an exact all-zero vector (including signed zero)
returns a distinct issue. Values remain bit-exact and are not normalized.
Duplicate subclasses remain independent and all nine Core dialects retain
ASCII/Binary parity. Elevation assembly is deliberately deferred because
top-level groups 10/20 collide with boundary and seed-point groups until
stateful HATCH partitioning exists. No transform, boundary topology, geometry,
applicability, CRUD/write, or `Complete` support is added.

M14.4f partitions each exact HATCH subclass at the two unambiguous grammar
fences surrounding boundary-path data. A unique group 91 before a unique group
75 yields exact header, opaque boundary payload, and trailing ranges over the
retained M14.4a fields. The two fence fields remain source anchored; missing,
duplicate, or reversed anchors yield typed issues and no ranges. Duplicate
subclasses remain independent and all nine Core dialects retain ASCII/Binary
parity. This isolates pre-boundary elevation groups 10/20 from colliding
boundary payload without yet selecting an elevation tuple. Declared path-count
relations, path/edge decoding, pattern/seed/gradient partitioning, geometry,
applicability, CRUD/write, and `Complete` support remain open.

M14.4g consumes the M14.4f header partition to assemble the required HATCH
elevation point from unique groups 10/20/30. Every component retains exact
binary64 bits and its raw group. X/Y accept positive or negative zero but reject
other finite values; Z accepts any finite value. Missing, duplicate, malformed,
or non-finite components remain typed, and an invalid boundary partition blocks
the tuple. Boundary and seed decoys cannot enter header cardinality. Duplicate
subclasses remain independent and all nine Core dialects retain ASCII/Binary
parity. No defaults, OCS/WCS transform, path relation/topology, later nested
partition, geometry, applicability, CRUD/write, or `Complete` support is added.

M14.4h groups each available boundary span by exact group-92 path anchors.
Every path retains its raw marker and opaque payload range through the next
anchor or group-75 fence. Pre-anchor fields remain a separate orphan range.
The unique group-91 declaration decodes as signed Int32 and compares with the
observed anchor count; matches, mismatches, malformed ASCII, and negative
declarations remain distinct. Invalid M14.4f partitions expose no grouping.
Duplicate subclasses remain independent and all nine Core dialects retain
ASCII/Binary parity. Group-92 flag semantics, polyline/edge branching, path
payload cardinality, source handles, geometry, applicability, CRUD/write, and
`Complete` support remain open.

M14.4i decodes each M14.4h group-92 marker into the documented External,
Polyline, Derived, Textbox, and Outermost bits. Values using only mask `0x1F`
retain exact raw provenance and classify as Polyline when bit 2 is set or Edges
otherwise; zero is valid. Malformed ASCII, negative values, and unsupported
bits remain typed without classification. Duplicate subclasses and all nine
ASCII/Binary dialect pairs remain isolated. Payload grammar/cardinality,
vertices, edges, handles, geometry, applicability, CRUD/write, and `Complete`
support remain open.

M14.4j selects unique group 72/73/93 headers only for M14.4i Polyline paths.
Bulge-presence and closed flags accept 0/1; declared vertex count must be
nonnegative. Edges are NotPolyline, while flag failure, absence, duplicates,
malformed ASCII, and domain failures remain typed. All nine dialects retain
ASCII/Binary parity. Vertex grouping/count relation, geometry, applicability,
CRUD/write, and `Complete` support remain open.

M14.4k groups Polyline vertex fields by exact group-10 anchors. Group 20 and
42 occurrences remain per-vertex Y/bulge members; pre-anchor 20/42 remain
orphans. Fixed cardinality and group-93 count comparison are explicit. Edges
and invalid headers publish no vertices; all nine dialects retain ASCII/Binary
parity. Numeric semantics, bulge defaults, geometry, CRUD/write, and `Complete`
remain open.

M14.4l maps every M14.4k X/Y/bulge component into a source-anchored four-state
numeric semantic. Unique members decode to exact finite binary64 values;
malformed ASCII and non-finite Binary values remain invalid with exact raw
provenance. Absent Y/bulge stays absent, while duplicates are invalid without
member selection. Edges and invalid headers publish no numeric entries, and all
nine dialects retain ASCII/Binary parity. Required-Y and has-bulge relations,
bulge defaults, OCS/WCS geometry, applicability, CRUD/write, and `Complete`
support remain open.

M14.4m resolves every M14.4l bulge against its unique group-72 has-bulge flag.
Absent group 42 receives the documented exact `+0.0` default without invented
raw provenance. If has-bulge is true, explicit values and typed numeric failures
retain their M14.4l source evidence. If false, any unique or duplicate group 42
becomes a typed relation failure while the complete underlying numeric state is
still retained. Edges and invalid headers publish no bulge entries; all nine
dialects retain ASCII/Binary parity. Required-Y semantics, closed-path topology,
OCS/WCS geometry, applicability, CRUD/write, and `Complete` support remain open.

M14.4n converts both Polyline vertex coordinates into required source-anchored
semantics. Usable X/Y assemble one exact two-component OCS position with
signed-zero fidelity. Missing Y becomes a typed required-value failure without
raw provenance; duplicate, malformed ASCII, and non-finite Binary states retain
their M14.4l issue and available provenance. An unusable tuple returns an exact
X/Y component mask. The full M14.4m bulge/header directory remains retained;
Edges and invalid headers publish no coordinate entries, and all nine dialects
retain ASCII/Binary parity. OCS/WCS transformation, closed-path topology,
segment geometry, applicability, CRUD/write, and `Complete` support remain open.

M14.4o publishes Polyline segment topology only when M14.4k's group-93 relation
is matched. Open paths produce consecutive adjacent-vertex segments; closed
paths add last-to-first closure, including one self-loop for a single vertex,
while empty paths remain empty. Count mismatch, Edges, and invalid headers yield
typed path states with no segments. Coordinate failures do not erase known
topology; compact endpoint ordinals resolve back to the exact M14.4n entries.
All nine dialects retain ASCII/Binary parity. Segment shape, bulge arc
construction, OCS/WCS transformation, applicability, CRUD/write, and `Complete`
support remain open.

M14.4p classifies each M14.4o segment solely from its start vertex's effective
bulge. Defaulted or explicit signed zero yields Straight; finite nonzero bulge
yields Arc with bit-exact value; numeric and group-72 relation failures yield
Indeterminate with the retained typed issue. Coordinate failure does not change
known shape, and the exact start-bulge provenance plus complete topology remain
resolvable. All nine dialects retain ASCII/Binary parity. Arc center/radius/
angle construction, line geometry, OCS/WCS transformation, applicability,
CRUD/write, and `Complete` support remain open.

M14.4q exposes exact-endpoint OCS line geometry for each M14.4p Straight
segment only when both endpoint tuples are usable. Start and end coordinate
failures remain distinct typed issues. Arc segments retain their exact bulge,
Indeterminate segments retain the original shape issue, and shape has
precedence over coordinate availability. Signed-zero endpoint bits remain
exact and zero-length straight lines remain valid. All nine dialects retain
ASCII/Binary parity. Arc center/radius/sweep construction, OCS/WCS
transformation, applicability, CRUD/write, and `Complete` support remain open.

M14.4r publishes finite OCS geometry for every M14.4q segment result. Straight
segments retain exact source endpoints. Arc segments with usable endpoints
derive a circular center, positive radius, and signed included-angle sweep
while retaining exact endpoints and authoritative bulge orientation. A
nonzero-bulge zero chord and non-finite intermediate or derived arithmetic fail
with distinct typed issues; prior shape and endpoint failures remain typed.
All nine dialects retain ASCII/Binary parity. Derived binary64 geometry is not
raw evidence or guaranteed cross-platform canonical bits. OCS/WCS
transformation, HATCH elevation/extrusion application, applicability,
CRUD/write, and `Complete` support remain open.

M14.4s joins every M14.4r segment to its exact HATCH subclass elevation and
explicit/defaulted extrusion, normalizes the nonzero extrusion, and applies the
documented arbitrary-axis basis with the exact `1/64` branch. Straight and Arc
endpoints plus Arc centers become finite WCS triples with normalized WCS
normals; radius, signed sweep, and exact bulge are preserved. Failure precedence
is source geometry, elevation, extrusion, then derived transform. All nine
dialects retain ASCII/Binary parity, including non-axis-aligned and negative
normals. Derived WCS binary64 values are not raw evidence or guaranteed
cross-platform canonical bits. Applicability, boundary Edges geometry,
CRUD/write, rendering, and `Complete` support remain open.

M14.4t selects one required non-negative group-93 edge count for each HATCH
boundary path classified as Edges and groups every subsequent group-72 field
as an exact source-anchored edge marker. Each marker retains a conservative raw
slice through the next marker or path end; the final slice may retain the path
source-boundary trailer for later typed partitioning. Declared and observed
counts remain Matched or Mismatched, and zero edges are valid. Polyline paths,
unavailable flags, invalid count cardinality/value, and marker-before-count
states publish no edge entries. All nine dialects retain ASCII/Binary parity.
Edge-type semantics, line/circular/elliptic/spline fields and geometry, HATCH
applicability, CRUD/write, rendering, and `Complete` support remain open.

M14.4u decodes every grouped M14.4t group-72 marker as a signed 16-bit edge
type: values 1, 2, 3, and 4 publish Line, CircularArc, EllipticArc, and Spline.
Malformed ASCII and every other signed 16-bit value remain exact
source-anchored InvalidAsciiNumber or ValueOutOfDomain issues. Count mismatches
retain independently known types; an empty grouped path exposes an empty typed
slice, while Polyline or unavailable paths expose no typed entries. All nine
dialects retain ASCII/Binary parity. Edge payload-field selection and geometry,
HATCH applicability, CRUD/write, rendering, and `Complete` support remain open.

M14.4v publishes four stable cardinality cards for every M14.4u Line edge:
StartX group 10, StartY group 20, EndX group 11, and EndY group 21. Each card
retains every exact source field and independently reports Absent, Unique, or
Multiple; count mismatch does not erase cards. Non-Line and invalid typed edges
publish no Line cards while the complete M14.4u directory remains available.
All nine dialects retain ASCII/Binary parity. Numeric selection,
required-coordinate semantics, OCS/WCS line geometry, the other edge payload
families, HATCH applicability, CRUD/write, rendering, and `Complete` support
remain open.

M14.3dq adds strict post-image verification for M14.3dp. Family verification
must first prove exact transaction bytes, typed POINT postconditions, and the
inverse. The XDATA verifier then resolves the inserted handle uniquely in the
strict-reparsed document, locates its indexed entity, rejects orphan values,
reads the complete contiguous raw XDATA span under the resource profile, and
compares it byte-for-byte with the retained expectation. The receipt binds
source/destination/post-image identities, inserted handle, application count,
and byte count; the journal retains the family verification and inverse. Four
format pairings span all nine dialects plus AC1009/AC1032 cross-dialect
boundaries, non-empty and zero-XDATA payloads, inverse restoration,
cancellation, foreign pre-images, tamper rejection, bounds, and redaction.
Create-new write cleanup under this XDATA wrapper, application-specific
meaning, actual text transcoding, cross-container clone completion, and POINT
`Complete` remain open.

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
