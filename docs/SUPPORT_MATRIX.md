# Format Support Matrix

SeaCad through M14.4m can open an immutable raw ASCII framing document, enforce
or recover its EOF envelope, attach a one-pass SHA-256 source identity,
discover an exact HEADER `$ACADVER`, account every parsed group inside or
outside non-overlapping sections, index every numeric group code 0, discover
the exact `$DWGCODEPAGE` declaration, derive a fail-closed text-storage policy,
resolve the reviewed 15-token Windows ANSI subset during the original parse,
decode any selected raw group value by document occurrence into caller-owned
UTF-8 without replacement, interpret documented CIF `\U+hhhh` controls
including valid UTF-16 surrogate pairs, map all five evidence-backed MIF
selectors, strictly decode CP932, CP950, CP949, CP1361, and CP936 MIF
payloads, decode exact `ANSI_1361` document storage across source chunks,
tokenize documented MTEXT and context-specific percent controls into exact
UTF-8 byte spans with typed structural failures, and write a separately
verified byte-identical copy. It also opens an immutable Binary raw snapshot:
the canonical opening selects pre-R13 or R13-and-later group-code encoding,
the exact HEADER `$ACADVER` must agree, and an 8 KiB bounded cursor losslessly
accounts every accepted group/value span under a one-pass SHA-256 identity.
Binary Strict/Compatible EOF conformance, section ranges, unknown sections, and
every numeric group-zero occurrence are indexed by the shared state machine.
Binary also has verified byte-identical new-file replay and CLI inspection. A
shared borrowed raw-document adapter and generated HEADER directory resolve all
214 expanded fields from the 206-row reviewed inventory across ASCII and Binary
without a second source scan. Numeric values retain exact signed integers or
IEEE-754 bits, tuple components retain independent provenance, strict Booleans
reject values outside `0/1`, exact text retains source spelling, documented
handle-valued numeric group codes have a context-neutral pointer/owner
classification, and any one raw group occurrence can be projected into an
exact source-anchored handle or typed lexical failure. Complete raw records
also expose identity evidence, document-local pointer/owner target resolution,
incoming soft/hard ownership-class evidence grouped by unique target, exact
application-group context, and conservative reactor, extension-dictionary, or
common-owner candidates grouped per raw record with typed cardinality; these
can be conservatively compared with unique incoming ownership-class evidence
as matched, conflicting, or non-comparable, but remain evidence rather than a
validated topology. Completely closed known record-bearing sections expose
format-neutral group-zero record chunks. Exact uppercase `POINT` and `LINE`
records in complete `BLOCKS` or `ENTITIES` sections additionally expose
source-order WCS and extrusion coordinate-component evidence with exact
ASCII/Binary double bits or typed lexical failure, plus per-role absent, unique,
or multiple cardinality cards. Lazy semantic projections retain required WCS
component failures and apply only reviewed extrusion defaults. Date or elapsed-day values do not infer
calendars or timezones. Exact uppercase `CIRCLE` and `ARC` records separately
expose source-order OCS center, radius, angle, and extrusion evidence without
assembling or transforming circular geometry, plus fixed per-role cardinality
cards that retain every occurrence. Lazy circular semantic projections preserve
required-value failures and apply only reviewed extrusion defaults. Exact
uppercase `ELLIPSE` records separately expose source-order WCS center,
relative major-axis endpoint, ratio, parameter, and extrusion-component
evidence plus fixed per-role cardinality cards without selecting values or
assembling ellipse geometry. Lazy ELLIPSE semantic projections preserve
required-value failures and apply only reviewed extrusion defaults. Exact
uppercase `RAY` and `XLINE` records separately expose source-order WCS
start/first-point and unit-direction evidence without normalization, selection,
or infinite-line assembly, plus fixed per-role cardinality cards. Lazy
RAY/XLINE semantic projections preserve required-component failures without
normalizing direction. Exact uppercase `LWPOLYLINE` records separately expose
source-order floating-point elevation, thickness, constant width, OCS vertex,
per-vertex width, bulge, and extrusion evidence without vertex grouping,
defaults, validation, or polyline assembly. Separate typed integer evidence
retains LWPOLYLINE count, flags, and vertex identifiers in their exact signed
wire domains without selection or interpretation. Exact group-10 anchors now
form conservative vertex slices with six fixed cardinality cards; pre-anchor
vertex evidence remains explicit as orphans. Lazy vertex semantics require OCS
X/Y, apply only documented local-width/bulge zero defaults, and retain optional
identifier state without claiming effective widths or segments. Eight fixed
record-level cards separately retain cardinality for count, flags, elevation,
thickness, constant width, and extrusion without admitting vertex-scoped
fields or selecting values. Lazy record semantics apply only documented
defaults, expose Closed/Plinegen bits, compare declared and observed anchor
counts, and retain constant/variable width coexistence without choosing an
effective width. Consecutive and proven closing segment topology now binds OCS
endpoints plus local width/bulge fields to each segment's start vertex without
assembling transformed geometry. Lazy OCS segment geometry preserves straight
endpoints and derives finite circular center/radius/signed sweep from usable
nonzero bulge, with typed unavailable, degenerate, and overflow states. Classic
POLYLINE records now expose exact section-local consecutive VERTEX
record slices and closed/interrupted/unclosed SEQEND boundary evidence without
classifying polyline families. Their sixteen documented record-level numeric
roles retain exact double/int16 values, duplicates, and typed ASCII failures;
sixteen fixed record cards separately retain per-role cardinality without
admitting VERTEX values or selecting occurrences. Lazy record semantics require
dummy X/Y/elevation, apply the documented zero and extrusion defaults, and
expose each documented flag bit while ignoring obsolete entities-follow `66`;
each retained VERTEX separately exposes thirteen documented double/int16/int32
roles with the same wire-level fidelity and strict record locality. Thirteen
fixed cards per VERTEX separately retain absent, unique, or multiple
cardinality without selecting values. Lazy double semantics require location,
apply only width/bulge zero defaults, and keep optional tangent absence
explicit. Separate lazy integer semantics preserve optional flags, four
polyface indices, and identifier, with helpers for the seven meaningful flag
bits. Fail-closed family evidence classifies only non-conflicting parent and
VERTEX flag combinations and retains unavailable, conflicting, matched, or
mismatched states. Complete family-consistent classic 2D/3D sequences expose
consecutive and proven closing segment topology; unsupported or indeterminate
records expose zero segments with typed state. Lazy segment bindings expose the
2D OCS/3D WCS boundary, endpoint tuples, local widths/bulge/tangent, and parent
default widths without choosing effective width. Classic 2D segments now
retain planar straight or derived circular OCS geometry at the parent
elevation, while classic 3D segments retain exact straight WCS endpoints;
unavailable, contradictory, degenerate, and non-finite inputs remain typed.
Usable classic 2D geometry is also projected into WCS by the documented
arbitrary-axis algorithm with normalized extrusion and explicit transform
failures; native 3D WCS lines remain unchanged.
Classic 2D segments also expose independently selected effective start/end
widths, preserving whether each came from an explicit VERTEX field or its
parent POLYLINE default; explicit zero remains an override.
Complete count-consistent classic polygon meshes expose row-major
quadrilateral cell topology with independent M/N closure and exact VERTEX
references; invalid mesh records retain typed zero-cell states.
Each proven polygon-mesh cell additionally exposes four exact named WCS corner
tuples or a typed corner/component failure.
Polygon-mesh smooth-surface metadata retains exact signed M/N densities and
classifies the four documented type codes without generating a fitted surface.
Complete family-consistent classic polyface meshes expose separate exact
coordinate and face-definition VERTEX ranges, reported and observed counts,
and tolerant odd-ordering evidence without trusting parent count fields.
Usable polyface face records resolve signed 1-based indices to those exact
coordinates, preserve edge visibility, and retain typed per-face failures.
Resolved polyface corners additionally expose exact WCS point tuples with
typed per-component coordinate failures.
Complete `BLOCKS` sections additionally expose exact uppercase
`BLOCK`/`ENDBLK` definition topology with source-order member-record slices and
closed, nested-BLOCK-interrupted, or section-end-unclosed state.
Each retained BLOCK record separately exposes its eight documented defining
roles as source-anchored exact text, double, or signed-16-bit evidence while
excluding group-102 application payloads and definition-member records.
Eight fixed per-role BLOCK cards separately retain absent, unique, or multiple
cardinality and compact references to every source-order value occurrence.
Lazy BLOCK semantics require both names, flags, and all three base-point
components; keep xref path/description optional; expose seven flag helpers; and
assemble the base-point tuple only from usable components without defaults.
Usable primary/secondary BLOCK names are compared as exact bounded raw bytes
with matched, conflicting, or not-comparable evidence.
Matched BLOCK names participate in exact duplicate-preserving
missing/unique/ambiguous lookup with collision-safe raw-byte confirmation.
Exact INSERT records in BLOCKS/ENTITIES retain all documented defining groups
as source-order text, binary64, or signed-16-bit evidence.
Each INSERT has 16 fixed cardinality cards retaining every source-order member
as absent, unique, or multiple evidence.
Typed INSERT semantics require block name/insertion point and distinguish
explicit values from Autodesk-documented optional defaults.
Usable INSERT names resolve by exact bounded source bytes to missing, unique,
or duplicate-preserving ambiguous BLOCK targets.
Unique targets are eligible only when their definitions are closed and their
reachable BLOCK expansion graph is acyclic.
Each eligible INSERT additionally exposes a finite single-instance row-major
3x4 BLOCK-to-WCS affine transform with its normalized extrusion normal.
Usable positive array counts and finite spacing additionally expose a
constant-space rotated rectangular layout with bounded per-index transforms.
Attributes-follow evidence additionally retains exact consecutive
ATTRIB/SEQEND sequence boundaries without decoding attribute payloads.
Sequence-owned classic ATTRIB records additionally retain source-order
defining values while excluding AcDbXrecord/AcDbMText extension payloads.
Each classic ATTRIB additionally exposes 23 fixed per-role cardinality cards.
Classic ATTRIB double semantics additionally distinguish required text-start
and height fields, documented defaults, and optional alignment components.
Classic ATTRIB text semantics additionally retain required source-anchored
value/tag fields and the documented `STANDARD` style default.
Classic ATTRIB integer semantics additionally retain required attribute flags,
documented zero defaults, and exact flag-bit helpers without selecting group
`280`.
Classic ATTRIB justification semantics additionally classify published
horizontal/vertical codes and expose typed alignment-point applicability.
Classic ATTRIB placement semantics additionally select the applicable OCS
text-start or alignment tuple without coupling the ignored tuple's failures.
Usable ATTRIB placement anchors additionally project to finite WCS points and
normalized normals through the shared arbitrary-axis implementation.
Exact uppercase ATTDEF members additionally retain their owning BLOCK
definition, definition state, and source-order member/ATTDEF positions without
decoding definition fields or associating ATTRIB records.
Classic ATTDEF records additionally retain source-order defining values while
excluding application-group and AcDbXrecord/AcDbMText extension payloads.
Each classic ATTDEF additionally exposes 24 fixed per-role cardinality cards.
Classic ATTDEF double semantics additionally distinguish required text-start
and height fields, documented defaults, and optional alignment components.
Classic ATTDEF text semantics additionally retain required source-anchored
default/prompt/tag fields and the documented `STANDARD` style default.
Classic ATTDEF integer semantics additionally retain required attribute flags,
documented zero defaults, and exact flag-bit helpers without selecting group
`280`.
Classic ATTDEF justification semantics additionally classify published
horizontal/vertical codes and expose typed alignment-point applicability.
Classic ATTDEF placement semantics additionally select the applicable stored
text-start or alignment tuple without coupling failures from the ignored tuple.
Usable ATTDEF placement anchors additionally project to finite WCS points and
normalized normals through the shared arbitrary-axis implementation.
Usable ATTDEF tags additionally participate in collision-safe, duplicate-
preserving exact lookup scoped to their owning BLOCK, with unusable-tag counts.
Each storage decode receipt retains
source ID, occurrence, raw span, encoding, and
terminal status. The CLI exposes `inspect` and `verify` with
English/Vietnamese human output, JSON v1, stable exits, and path redaction.
These M4 reports and decode views remain core APIs and are not exposed in CLI
JSON v1 yet.

M14.1a additionally recognizes exact uppercase `3DFACE`, `SOLID`, and `TRACE`
records in complete `BLOCKS`/`ENTITIES` record ranges. It retains twelve
source-order corner components for each family, 3DFACE group-70 invisible-edge
flags in their signed-16-bit wire domain, and SOLID/TRACE thickness plus
extrusion components as exact binary64 evidence. Duplicate occurrences, raw
spans, and typed ASCII lexical failures remain explicit with ASCII/Binary
parity across all nine dialects. No component selection, defaults, validation,
corner reorder, OCS/WCS transform, face geometry, edit, or write support is
claimed.

M14.1b adds fixed family-specific cardinality over that evidence: thirteen
stable role cards for every 3DFACE and sixteen for every SOLID/TRACE. Each card
is explicitly absent, unique, or multiple and points back to all source-order
M14.1a occurrences without copying or selecting them. Numeric lexical validity
remains independent from cardinality; cross-family roles have no card. No
defaults, semantic validation, corner reorder, OCS/WCS transform, face
assembly, edit, or write support is claimed.

M14.1c selects unique planar-face values with typed field/raw provenance and
keeps missing, invalid, multiple, partial, and unavailable-default states
distinct. It applies the documented 3DFACE zero edge-flag default,
SOLID/TRACE zero thickness and (0,0,1) extrusion defaults, and the documented
3DFACE/SOLID third-to-fourth-corner fallback only when the complete fourth
tuple is absent. TRACE receives no undocumented corner fallback. Edge-bit
interpretation, finite geometry/normal validation, SOLID/TRACE corner reorder,
OCS/WCS transforms, face assembly, edit, and write remain unclaimed.

M14.1d emits finite WCS corner geometry: native WCS/source order for 3DFACE,
and perimeter order (first, second, fourth, third) after shared arbitrary-axis
OCS projection for SOLID/TRACE. The latter retains a normalized WCS normal and
finite thickness. Missing semantic inputs, non-finite corners/thickness/
extrusion, zero extrusion, and non-finite derived coordinates fail typed;
positive zero is canonicalized. Edge-bit interpretation, thickness surfaces,
BLOCK expansion, edit, write, rendering, and tessellation remain unclaimed.

M14.1e interprets 3DFACE invisible-edge bits 1, 2, 4, and 8 in source-corner
order. It retains the signed group-70 value, exposes its exact 16-bit pattern,
and reports rather than discards unknown bits. Defaulted zero produces four
visible edges; invalid or duplicate flags produce no derived edge state.
SOLID/TRACE edge flags, topology assembly, editing, and writing remain
unclaimed.

M14.2a indexes exact source-order TEXT, MTEXT, SHAPE, and TOLERANCE fields in
closed BLOCKS/ENTITIES sections. It labels every documented per-entity text,
binary64, signed-16-bit, and signed-32-bit role while retaining raw spans,
duplicates, invalid ASCII numbers, repeated MTEXT chunks, and ambiguous MTEXT
group 50 occurrences. MTEXT group 420/430 ranges remain ambiguous with common
entity color fields until subclass-aware common-property ownership. It does
not yet choose cardinality, apply defaults, decode content/style strings,
validate layout codes, resolve styles, transform coordinates, or produce glyph
geometry.

M14.2b adds fixed cardinality cards for every documented role: 19 per TEXT,
33 per MTEXT, 12 per SHAPE, and 11 per TOLERANCE record. Members point back to
M14.2a source occurrences, and absent/unique/multiple states remain independent
of lexical validity. Multiple MTEXT chunks and group 50 values are not rejected
before later structural semantics. Selection, defaults, decoding, validation,
style resolution, and geometry remain unclaimed.

M14.2c selects all unambiguous numeric TEXT/SHAPE scalars. Required values,
documented defaults, optional TEXT second-alignment absence, invalid ASCII
numbers, and duplicate values retain distinct typed states and exact
provenance. It does not yet decode text/name/style fields, validate numeric or
layout codes, choose TEXT alignment applicability, transform coordinates,
derive glyph geometry, or provide MTEXT/TOLERANCE value semantics.

M14.2d selects unambiguous MTEXT/TOLERANCE numeric scalars with the same
four-state provenance contract. It applies only documented extrusion defaults
and keeps optional/read-only/layout fields independently absent. MTEXT group 50
and group 420/430 remain evidence-only pending precedence/column structure and
subclass-aware common-property ownership. Enum/range validation, chunk
semantics, text decoding, coordinate transforms, styles, and glyphs remain
unclaimed.

M14.2e selects source-anchored TEXT content, SHAPE name, and TOLERANCE
dimension-style/content fields, while applying the documented `STANDARD`
style default only to TEXT and MTEXT. It exposes MTEXT group 3/group 1 chunks
in exact source order and retains missing terminals, repeated terminals, and
group 3 values after a terminal as typed structural evidence. Text decoding,
table-reference resolution, inline formatting, chunk-length validation,
coordinate transforms, and glyph geometry remain unclaimed.

M14.2f classifies every documented TEXT horizontal/vertical justification code
and derives whether the first or second alignment point controls placement.
Generation flags preserve their exact signed 16-bit pattern and expose
backward, upside-down, and unknown bits without normalization. Unsupported
justification codes and unavailable scalar inputs remain typed with raw
provenance. Coordinate availability/selection, OCS/WCS transformation, numeric
range checks, style resolution, and glyph geometry remain unclaimed.

M14.2g classifies every Autodesk-enumerated MTEXT attachment, drawing-direction,
and line-spacing-style code. Required/optional states, unsupported codes,
invalid ASCII values, and duplicates retain distinct typed states and exact
raw provenance.

M14.2h validates optional MTEXT group 44 over Autodesk's inclusive
`0.25..=4.00` line-spacing-factor domain. Accepted and rejected values retain
their exact IEEE-754 payload and source provenance; absence remains `Absent`
without a default.

M14.2i classifies Autodesk's published MTEXT group-90 background settings
`0/1/2`. Other signed settings, invalid ASCII, and duplicates remain typed and
source-anchored; absence receives no default. Background-color ownership,
other numeric ranges, column codes, rotation precedence, coordinate transforms,
styles, and glyphs remain unclaimed.

M14.2j validates Autodesk's read-only MTEXT width relation: group 42 must be
equal to or less than group 41. Excess values retain both exact IEEE-754 inputs
and source provenance; absent, invalid, and duplicate inputs remain
independently typed. Other numeric relationships, background ownership,
columns, rotation precedence, transforms, styles, and glyphs remain unclaimed.

M14.2k selects the effective MTEXT orientation input by Autodesk's source-order
rule: the later unambiguous group-50 rotation or group-11/21/31 X-axis input
wins. Invalid and duplicate rotation evidence remains typed, and any group 50
coexisting with column fields remains fail-closed as
rotation-or-column-height ambiguity. Axis-vector assembly, normalization,
column-height interpretation, transforms, styles, and glyphs remain unclaimed.

M14.2l separates modern MTEXT embedded-column evidence at the exact group-101
`Embedded Object` marker. Main MTEXT cards stop before the marker; a dedicated
immutable directory retains the observed embedded version, common and
individual heights, column type/count/width/gutter, automatic-height, and
flow-reversal groups with exact source identity and raw provenance. The
boundary and ASCII/Binary parity are verified for AC1009 through AC1032.
Semantic column validation, legacy flat/R2007 XDATA unification, geometry,
edit, and write remain unclaimed.

M14.2ah composes the five DIMSTYLE handle roles with exact generic
document-local handle resolution. Absent and duplicate fields remain distinct
from invalid, null, missing, unique, and ambiguous targets; all generic
identity matches remain reachable without choosing a duplicate. AC1009 Binary
keeps these group codes above 255 absent. Expected STYLE/BLOCK_RECORD target
validation, name resolution, tolerance-string interpretation, glyph geometry,
edit, and write remain unclaimed.

M14.2ai validates unique DIMSTYLE handle targets against exact membership in
completely closed, matching STYLE and BLOCK_RECORD tables. One shared scanner
retains named DIMSTYLE, STYLE, and BLOCK_RECORD records and rejects wrong-case
table names, mismatched record markers, duplicate names, application-group
decoys, and interrupted or unclosed envelopes. Unique targets are classified
as expected kind, another reviewed named-symbol kind, or another raw record;
all non-unique resolver states remain unchanged. Target-name resolution,
style/block content semantics, defaults, glyph geometry, edit, and write remain
unclaimed.

M14.3a retains all documented SPLINE numeric fields as exact source-order
signed-16-bit or binary64 evidence from complete BLOCKS and ENTITIES sections.
Duplicates, invalid ASCII numbers, repeated knots/control points/fit points,
raw spans, and float bits remain visible; group-102 application content cannot
impersonate a field. Value selection, defaults, version applicability, flags,
count reconciliation, point grouping, curve validity, geometry, HELIX subclass
data, edit, and write remain unclaimed.

M14.3b adds 24 fixed source-ordered role cards per retained SPLINE record.
Absent, unique, and duplicate-preserving multiple states remain independent of
numeric lexical validity, with compact members resolving back to exact M14.3a
evidence. Value selection, defaults, flags/count semantics, point grouping,
curve validity, geometry, HELIX, edit, and write remain unclaimed.

M14.3c selects group-70 SPLINE flags only for a unique valid occurrence.
Closed, periodic, rational, planar, and linear helpers preserve the exact
signed value and all unknown bits; absent, invalid, and duplicate states stay
typed. Other scalar semantics, count relations, point grouping, curve
validity/geometry, HELIX, edit, and write remain unclaimed.

M14.3d selects unique SPLINE degree, declared knot/control/fit counts, and
tolerances. Counts receive no invented default; absent knot, control-point,
and fit tolerances use only the documented `1e-7`, `1e-7`, and `1e-10`
defaults. Explicit, defaulted, absent, invalid, and duplicate states remain
distinct. Count reconciliation, point grouping, validity/geometry, HELIX,
edit, and write remain unclaimed.

M14.3e reconciles each unique nonnegative declared SPLINE knot, control-point,
and fit-point count with retained group-40, group-10, or group-11 anchor
occurrences. Matched, mismatched, negative, absent, invalid, and duplicate
states remain distinct and source-anchored. Coordinate completeness, numeric
member validity, curve validity/geometry, HELIX, edit, and write remain
unclaimed.

M14.3f adds a deterministic generated registry for the 45 canonical Autodesk
entity topics. Each public descriptor exposes a stable ordinal, schema id,
exact canonical group-zero spelling, normative topic id, and normalized source
receipt. Lookup accepts only exact case-sensitive canonical bytes. The
freshness gate rejects missing/extra topics, duplicate ids or names, namespace
drift, source-kind drift, or changed generated output. Aliases and unknown
markers remain unclassified. This registry does not scan records, establish
version applicability or field schemas, expose common properties, parse any
new entity, construct geometry, edit, or write.

M14.3g adds a generated registry of 14 exact group-zero
alias/specialization spellings. Five mappings carry normative Autodesk source
GUIDs; nine observed concrete names carry a distinct AutoCAD 2027 behavioral
oracle receipt. Classification is byte-exact and returns canonical, alias, or
unknown while alias descriptors expose the canonical topic, exact wire name,
evidence kind, source reference, and normalized facts hash. This is inventory
only: section/dialect legality, record indexing, typed fields, semantic or
geometry projection, CRUD, and support-state advancement remain unclaimed.

M14.3h adds a deterministic applicability descriptor for all 59 reviewed exact
names and evaluates each descriptor against AC1009, AC1012, AC1014, AC1015,
AC1018, AC1021, AC1024, AC1027, and AC1032. Source-backed inclusive ranges
produce typed `Applicable` or `NotApplicable`; rows without sufficient
version evidence produce `NotYetReviewed` for every dialect. Autodesk's
underlay compatibility guidance establishes DWF/DGN at AC1021 and PDF at
AC1024. The remaining 56 rows deliberately stay unreviewed. This matrix is
metadata only: it does not validate record section placement, reject a parsed
record or writer request, add entity fields/semantics/geometry/CRUD, or advance
any entity to `Complete`.

M14.3i adds one format-neutral `DxfEntityDirectory` over completely indexed
record-bearing sections. Records in `BLOCKS` and `ENTITIES` are classified as
canonical topic, exact alias, or unknown; reviewed entity names elsewhere are
retained as typed wrong-section evidence. `BLOCK` and `ENDBLK` remain BLOCKS
structure rather than unknown entities. Each `DxfEntityRef` retains source
identity, raw-record ordinal and section, exact marker span, classification,
and a source-order group-100 subclass range. Group-100 values inside group-102
application envelopes do not enter that semantic path. All other groups and
bytes remain authoritative and untouched. This is indexing infrastructure;
common fields, family semantics, geometry, CRUD, writer gating, and support
completion remain unclaimed.

M14.3j adds a deterministic generated registry for 19 common entity-property
roles from Autodesk's common entity-code table. Each descriptor exposes its
stable field ordinal, exact group code, wire type, cardinality, documented
default, preamble/`AcDbEntity`/extension-dictionary scope, non-coordinate
classification, explicit field-version review state, and source receipt.
Proxy graphics code 310 is the sole common optional sequence; group 360 is
scoped to `ACAD_XDICTIONARY`, while the group-330 BLOCK_RECORD owner is distinct
from reactor content. Field applicability remains `NotYetReviewed` rather than
guessing historical introduction versions. This schema does not yet scan
occurrences, form cards, decode semantics, edit, write, or advance support.

M14.3k adds `DxfEntityFieldEvidenceDirectory`, retaining source-order raw
occurrences and 19 fixed common-field cards for every canonical, alias, or
unknown entity in `BLOCKS`/`ENTITIES`. Cards distinguish required or optional
absence, a unique singleton, typed duplicate singleton counts, and positive
sequence counts. Subclass context separates preamble, `AcDbEntity`, and later
family fields; application context accepts group 360 only from
`ACAD_XDICTIONARY` and keeps its exact closure evidence. Reactor handles,
family-subclass collisions, and reviewed names in wrong sections are excluded.
This layer does not decode values, apply defaults, validate references or
proxy byte counts, edit, write, or advance entity support.

M14.3l adds `DxfSplinePointTupleDirectory` over the existing SPLINE evidence
cards. Control-point group 10/20/30 sequences and fit-point group 11/21/31
sequences are paired by component-local ordinal, so arbitrary interleaving of
different group codes does not change the result. Tuples retain compact member
references, exact component counts, complete/partial shape, and underlying
numeric failures. Missing Z is preserved as absence rather than silently
defaulted. Weight, tangent, normal, relation, NURBS, HELIX, CRUD, and writer
support remain unclaimed.

M14.3m adds group 41 to the SPLINE evidence/cards and introduces
`DxfSplineAuxiliaryDirectory`. One weight entry per SPLINE record distinguishes
Autodesk's implicit unit weights from explicit matched or mismatched counts
against group-10 control-point anchors. Three vector entries retain each
start-tangent, end-tangent, and normal component as absent, unique, or
duplicate, with aggregate present-component or ambiguity masks. Evidence is
limited to legacy pre-subclass and exact `AcDbSpline` scope. This layer does
not validate weight/vector domains, derive effective vectors, evaluate spline
invariants, construct NURBS data, process HELIX, edit, write, or advance
entity support.

M14.3n adds `DxfSplineAuxiliarySemanticDirectory`. It retains implicit unit
weights, explicit matched and mismatched sequences, invalid numeric evidence,
and a visibly behavioral nonpositive classification. Start/end tangent and
normal vectors become absent, effective, or unavailable: only a unique valid
X permits a value, omitted Y/Z components default to zero, and explicit
component provenance stays in a mask. Duplicate, invalid, and missing-X masks
remain typed without selecting evidence. Rational/planar/linear relations,
zero-normal policy, spline invariants, NURBS data, HELIX, CRUD, writer, and
support completion remain unclaimed.

M14.3o adds `DxfSplineRelationDirectory`. It composes group-70 evidence with
effective weights and normals without selecting duplicate or invalid flags.
The documented linear-plus-planar requirement is explicit; rational state is
paired with implicit or explicit weights plus count/domain issue totals but is
not treated as an undocumented validity equivalence. Planar normals are
missing, explicit (including exact zero), or unavailable, while a nonplanar
normal remains a typed unexpected observation. Point semantics,
degree/knot/periodic invariants, NURBS data, HELIX, CRUD, writer, and support
completion remain unclaimed.

M14.3p adds `DxfSplineTopologyDirectory`. It types degree domain, knot ordering
and the first decreasing index, minimum control-point readiness, the defining
NURBS knot-count equation, overflow, and every closed/periodic flag
combination. Invalid sequence members make ordering unavailable rather than
being skipped, and no flag combination is normalized. Declared-versus-observed
group 72/73/74 states remain in `DxfSplineCountDirectory`. Point values,
analytic NURBS data, HELIX, CRUD, writer, and support completion remain
unclaimed.

M14.3q1 adds `DxfSplineAnalyticValueDirectory`. Exact knots, effective WCS
control/fit points, explicit-Z provenance, and positive effective control
weights are exposed through compact ranges only when every member is usable.
Missing/invalid X or Y, invalid knot values, mismatched weights, and
invalid/nonpositive weights accumulate in a typed mask without partial output.
Omitted Z uses a behavioral zero default and remains distinguishable. Full
topology/count/multiplicity/domain and vector readiness remains M14.3q2;
HELIX, CRUD, writer, and support completion remain unclaimed.

M14.3q2 adds `DxfSplineAnalyticDirectory`, the fail-closed composition of the
existing value, topology, count, flag-relation, and vector-semantic layers.
Available entries expose exact compact value ranges, flags, degree, optional
vectors, and the active knot interval. Typed issues cover missing or ambiguous
prerequisites, declared-count mismatch, topology contradictions, excessive
knot multiplicity, a nonpositive active domain, malformed optional tangents,
and invalid planar-normal relations. Closed/periodic bits are retained without
normalization. Nine-dialect ASCII/Binary parity and malformed/cancellation
coverage do not advance SPLINE to `Complete`; evaluation, tessellation, HELIX,
CRUD, and writer support remain unclaimed.

M14.3r adds `DxfHelixDirectory` with exact source evidence for all 16 public
`AcDbHelix` roles. Canonical HELIX records are selected through the unified
entity index in `BLOCKS` and `ENTITIES`; exact subclass scope prevents group
10/40 collisions with embedded `AcDbSpline` data. Int32, Int16, BooleanByte,
and double wire values retain exact groups and invalid ASCII numeric states.
Duplicates remain source ordered, application groups and near-match markers
are excluded, and unknown/raw groups remain lossless in the raw document.
Nine-dialect ASCII/Binary evidence parity does not claim HELIX applicability
in older dialects. Cardinality, semantics, embedded SPLINE composition,
geometry, CRUD, writer support, and `Complete` status remain unclaimed.

M14.3s adds `DxfHelixCardDirectory`. Each indexed HELIX record receives the
same 16 source-role cards in registry order, with absent, unique, or multiple
cardinality and exact source-ordered member references. Numeric validity does
not alter occurrence cardinality, and duplicate singleton evidence is never
selected. Nine-dialect ASCII/Binary parity includes out-of-order fields and
the AC1009 group-code boundary. This does not establish required/default
semantics, version applicability, coordinate or parameter validity, embedded
SPLINE composition, geometry, CRUD, writer support, or `Complete` status.

M14.3t adds `DxfHelixScalarDirectory` with seven entries per HELIX record over
the shared source-provenance semantic value type. Versions and finite scalar
parameters retain exact values; handedness and constraint type expose their
documented enum domains. Absent, duplicate, invalid ASCII, non-finite Binary,
and invalid-domain states remain distinct. No positivity, requiredness,
default, or cross-field relationship is inferred. Nine-dialect ASCII/Binary
parity does not advance applicability or support. WCS tuples, axis/parameter
relations, embedded SPLINE composition, geometry, CRUD, writer support, and
`Complete` status remain unclaimed.

M14.3u adds `DxfHelixVectorDirectory`. Axis base, start point, and axis vector
each expose three WCS `DxfSemanticValue` components with exact provenance.
Complete finite triples are available as exact values; absence, duplicates,
invalid ASCII, or Binary non-finite evidence suppresses the triple without
inventing coordinates. No Y/Z default or zero-axis validity claim is made.
Nine-dialect ASCII/Binary parity does not advance applicability or support.
Axis/parameter relations, embedded SPLINE composition, analytic geometry,
CRUD, writer support, and `Complete` status remain unclaimed.

M14.3v adds `DxfHelixRelationDirectory` with one composed relation entry per
HELIX. Complete nonzero axes expose a normalized direction, derived radial
vector/base radius, and exact orthogonality residual; zero axes and non-finite
derived arithmetic remain typed. The stored radius is classified without
relabeling group 40, turns are compared with zero and Autodesk's 500-turn
command limit, and axial height is derived from turns times turn height with a
distinct flat state. Nine-dialect ASCII/Binary parity does not establish
HELIX applicability in older versions or advance support. Embedded SPLINE
composition, analytic geometry, CRUD, writer support, and `Complete` status
remain unclaimed.

M14.3w adds `DxfHelixAnalyticDirectory` and extends the shared SPLINE
projection with explicit `Spline`/`Helix` record kinds. HELIX curve evidence
is scoped only to the exact `AcDbSpline` subclass and is joined to HELIX
relations by exact raw-record ordinal. Availability requires one ordered
`AcDbSpline` then `AcDbHelix` marker, an available embedded spline, exact axis
perpendicularity, nonnegative stored radius, positive turns, and finite
compared height. Failures accumulate without partial analytic data; turns
above the command's 500 limit remain readable existing-data state and zero
height remains a valid flat state. Nine-dialect ASCII/Binary parity does not
establish HELIX applicability in older versions or advance support. NURBS
evaluation on HELIX, sampling/tessellation, CRUD, writer support, and
`Complete` status remain unclaimed.

M14.3x adds `DxfEntityFieldSemanticDirectory` for every semantic entity and all
19 generated common fields. Singleton fields preserve distinct double, Int16,
Int32, handle, and exact-text wire domains through the shared four-state
semantic value model. Exact text is source-backed and decodes without
replacement; schema defaults are materialized only when the field is omitted,
including a distinct material `ByLayer` value. Missing required fields,
duplicate singletons, invalid ASCII numbers/handles, and Binary non-finite
doubles fail typed without selecting an occurrence. Proxy group-310 data stays
an opaque sequence linked to exact card members. Nine-dialect ASCII/Binary
parity does not establish field applicability in older versions or validate
property domains, references, proxy byte counts, application-group closure,
CRUD, writer support, or any `Complete` status.

M14.3y adds `DxfEntityGroupEncoder`, a create-new group primitive over the 19
generated common-field descriptors. It emits canonical ASCII or Binary bytes
for exact raw text, handles, finite binary64, Int16, Int32, and individual
opaque binary chunks. Binary group-code framing follows the AC1009 one-byte
boundary and the R13-and-later two-byte little-endian contract; numeric payloads
are little-endian, strings are NUL-terminated, and chunks are length-prefixed.
Wire mismatches, non-finite doubles, NUL/CR/LF exact text, chunks above 128
bytes, resource exhaustion, cancellation, and common codes unavailable in
AC1009 fail closed. All nine dialects strictly reparse paired ASCII/Binary
fixtures. This does not transcode Unicode, concatenate group-310 sequences,
insert/update entities, assign handles or owners, build transactions, mutate a
source, advance applicability, or establish CRUD/writer/`Complete` support.

M14.3z adds `DxfEntityKey` and
`DxfEntityFieldReplacementPlan`. A source-bound key plus one generated common
field may replace exactly one existing unique singleton. The plan composes the
generic field card, M14.3y encoder, and M11 raw transaction/inverse machinery;
its only patch covers the original group's exact full span. Missing fields,
duplicate singletons, group-310 sequences, wrong-section records, unavailable
dialects, encoder failures, source mismatches, and cancellation fail closed.
For AC1009 through AC1032, paired ASCII/Binary post-images strictly reparse,
publish the requested typed field value, and materialize an inverse that
restores byte-identical source. This does not insert or reset fields, validate
property domains/references, batch patches in `DxfEntityEditSession`, write a
destination, advance applicability, or establish full update/CRUD/`Complete`
support.

M14.3aa adds `DxfEntityFieldResetPlan`. An optional singleton already absent
returns `AlreadyImplicit` without a patch; an existing unique optional
singleton produces one deletion patch over the exact full raw group span.
Required fields, duplicate singletons, proxy group-310 sequences, and the
nested group-360 extension-dictionary member fail with distinct typed states;
the planner never selects a duplicate or leaves an empty `ACAD_XDICTIONARY`
wrapper. For AC1009 through AC1032, paired ASCII/Binary post-images strictly
reparse, publish the generated omitted default or absence, and materialize an
inverse restoring byte-identical source. This does not insert fields, delete a
whole application group, batch edits, validate domains/references, write a
destination, advance applicability, or establish full update/CRUD/`Complete`
support.

M14.3ab adds `DxfEntityFieldWriteOrder` to the generated 19-field common
property registry. Its exact ordinals freeze a canonical writer policy derived
from the usual Autodesk table presentation, independently of registry order;
schema validation rejects omissions, duplicates, and reordered values. This
metadata does not constrain parsing, which remains group-order independent and
unknown-group preserving. It does not select a record-specific insertion
anchor, insert or move a group, resolve subclass envelopes, advance
applicability, or establish full update/CRUD/`Complete` support.

M14.3ac adds `DxfEntityFieldInsertionAnchor`. For one absent singleton it
locates an exact byte boundary between complete source groups and publishes the
immediate neighbor occurrences. Preamble handle/owner anchors remain outside
closed group-102 envelopes; modern common properties require exactly one
`AcDbEntity` subclass, while AC1009 uses a bounded legacy preamble. Generated
writer order constrains the target between all earlier/later known common
fields, and contradictory existing order fails typed. Existing singletons,
sequences, extension dictionaries, malformed application groups, wrong
sections, unavailable dialects, source mismatches, and cancellation remain
explicit. Nine-dialect ASCII/Binary fixtures have anchor parity, including
BLOCKS/unknown records and untouched unknown groups. This checkpoint does not
encode or insert bytes, create a transaction or destination, validate domains
or references, advance applicability, or establish full CRUD/`Complete`
support.

M14.3ad adds `DxfEntityFieldInsertionPlan`. A successful M14.3ac absent-
singleton anchor and M14.3y value encoding become one M11 transaction patch
whose source span is empty at the exact byte offset. ASCII insertion preserves
the preceding raw group's LF, CR, or CRLF ending; Binary insertion retains the
declared dialect's group-code and payload encoding. For AC1009 through AC1032,
paired strict ASCII/Binary post-images publish the inserted semantic and an
executable inverse restores byte-identical source. Existing/duplicate fields,
sequences, nested extension dictionaries, structural anchor failures, wire
mismatches, non-finite/invalid values, unavailable group codes, source
mismatches, resource limits, and cancellation prevent a plan from escaping.
This does not batch edits, validate property domains/references, allocate
handles/owners, write a destination, advance applicability, or establish full
CRUD/`Complete` support.

M14.3ae adds `DxfEntityEditSession` and the first typed
`DxfEntityPatch::CommonField` variant. One session admits explicit singleton
set/reset requests against its original source/evidence pair, rejects a second
queued edit for the same entity/field, and retains already-implicit reset as a
zero-patch receipt. Set chooses exact replacement or canonical insertion;
reset chooses exact deletion. `finish` orders source spans and merges multiple
same-entity insertions at one empty source span by generated writer order, so
caller order cannot reorder common fields. Five edits across two entities
produce one three-patch transaction with strict ASCII/Binary semantic parity
for AC1009 through AC1032 and an executable byte-identical inverse. Structural,
encoding, source, resource, conflict, and cancellation failures escape no
ambiguous partial plan. Topic-family patches, sequence/nested operations,
domain/reference validation, entity insertion, handle/owner allocation,
closed-set clone/delete, create-new verified writes, applicability, and full
CRUD/`Complete` support remain open.

M14.3af adds `DxfEntityEditPlan` and `finish_verifiable`. Every queued common-
field singleton retains a payload-redacted postcondition keyed by raw record
ordinal and field. Explicit expectations own exact text or retain typed handle,
finite-double, Int16, and Int32 values; reset expects the optional card to be
absent and its generated semantic state to be defaulted or absent. Verification
first enforces the source and post-image envelope, then rebuilds common-field
semantics and checks cardinality, semantic shape, state, and value. M11.1b raw
verification and inverse materialization run only after the semantic checks.
Strict ASCII/Binary AC1009-through-AC1032 tests prove typed semantic mismatch,
unrelated raw-byte mismatch, source/length/cancellation failures, payload
redaction, and byte-identical inverse restoration. This checkpoint does not
invoke M12 create-new writing, clean a filesystem destination after semantic
failure, validate field domains/references, add family patches or sequence/
nested edits, insert entities, clone/delete closed sets, advance applicability,
or establish full CRUD/`Complete` support.

M14.3ag adds the create-new execution path for `DxfEntityEditPlan`. It composes
the existing streaming hash/flush/sync verifier with a strict reparse in the
transaction's physical format, the M14.3af semantic postconditions, the exact
raw post-image verifier, and inverse materialization. A successful journal
exposes the write and semantic receipts plus the executable inverse without
duplicating source/output identities internally. Any strict parse, semantic,
raw, cancellation, or identity failure after creation removes the destination;
semantic unavailability also removes it before returning its typed issue, and
cleanup failure replaces the primary result. A pre-existing destination is
never changed. Paired ASCII/Binary tests across AC1009 through AC1032 prove
successful output and byte-identical inverse restoration. Controlled post-hash
tampering separately proves strict-reparse cleanup, edited-field semantic
cleanup, and unrelated raw-mismatch cleanup. Domain/reference validation,
family patches, sequences/nested edits, entity insert/clone/delete, handle/
owner assignment, applicability, and full CRUD/`Complete` support remain open.

M14.3ah adds `classify_entity_common_field_edit_domain` and typed representations
for eight reviewed scalar domains. Group 67 accepts model/paper values 0/1;
group 62 accepts BYBLOCK, ACI 1-255, BYLAYER, and negative layer-off ACI values;
group 370 accepts the public `AcDb::LineWeight` set from -3 through the discrete
2.11 mm value; group 48 requires a finite nonnegative scale; group 60 accepts
visible/invisible; group 92 requires a nonnegative byte count; group 420
requires a zero high byte and exposes RGB channels; group 284 accepts its four
documented shadow modes. Wrong value kinds and unsupported values are typed;
all other common fields return `Unreviewed` rather than a false validity claim.
`DxfEntityEditSession` rejects invalid reviewed domains before a planner runs,
without queue growth or source mutation. Boundary tests and paired strict
ASCII/Binary edits for AC1009 through AC1032 verify semantic postconditions and
byte-identical inverse restoration. Existing raw-value domain projection,
text/name and handle/reference validation, transparency, proxy byte-count/data
agreement, cross-field relations, applicability, family patches, and full
CRUD/`Complete` support remain open.

M14.3ai adds an existing-document common-field domain directory. It projects
the eight M14.3ah reviewed scalar domains over the generic semantic directory,
retaining exact explicit/defaulted/absent/invalid state and provenance. Valid
usable values become typed domain values; scalar decode failures, duplicate or
missing-required cardinality, and out-of-domain values remain distinguishable
typed issues. The remaining common fields pass through their exact unreviewed
singleton or opaque-sequence semantics. Paired ASCII/Binary tests cover AC1009
through AC1032, invalid scalars, defaults, absence, source identity,
cancellation, lookup, and public bounds. This is domain projection, not name or
reference validation, transparency support, proxy count/data reconciliation,
cross-field validation, applicability, family patches, full CRUD, or a
`Complete` support claim.

M14.3aj adds a common-handle directory for the five generated handle-valued
fields. Group 5 remains exact lexical object identity. Owner 330, closed
extension-dictionary 360, material 347, and plot-style 390 references compose
the generic field semantics with shared document-local handle resolution.
Absent/defaulted/invalid states remain exact; explicit references distinguish
null, missing, one unique target, and ambiguous duplicate targets. Omitted
material stays `ByLayer`, not a fabricated handle. Paired ASCII/Binary tests
cover all nine dialects and every resolution/cardinality failure. A unique
numeric target is lookup evidence only: target-kind compatibility,
authoritative ownership, one-owner conformance, dictionary membership,
lifecycle behavior, applicability, edit validation, full CRUD, and `Complete`
support remain open.

M14.3ak extends closed named-symbol membership to `LAYER` and `LTYPE` and adds
a four-field common-text directory. Layer group 8 and linetype group 6 retain
raw field states and perform duplicate-preserving exact source-byte lookup
against their matching closed tables. A unique exact name carries its target;
missing and multiple exact names are typed failures. Omitted linetype remains
the schema-backed `BYLAYER` default. Layout 410 and color-name 430 remain exact
unreviewed text. Paired ASCII/Binary tests cover all nine dialects, wrong-
section table markers, missing/duplicate names, duplicate fields, defaults,
source identity, cancellation, and public bounds. This does not claim AutoCAD
case-folding, symbol-character validity, XREF name rules, layout or color-book
resolution, applicability, name-safe CRUD, or `Complete` support.

M14.3al adds an exact proxy-graphics size relation for every semantic entity.
It compares reviewed group 92 with the complete opaque group-310 sequence.
ASCII chunks must contain an even number of hexadecimal digits and contribute
half their encoded length; Binary chunks contribute the validated payload span
after their wire length prefix. The implementation scans bounded chunks,
retains exact failure provenance, and never concatenates or interprets payload
bytes. Typed states preserve absence, missing size, match, mismatch, invalid
size, and first invalid chunk. Paired fixtures cover AC1009 through AC1032;
AC1009 covers the representable zero-size/no-data relation. This does not claim
payload decoding, rendering, inferred repair, applicability, sequence editing,
clone/delete closure, or `Complete` support.

M14.3am reviews common transparency group 440 as an exact Int32 domain.
ByLayer is method byte 0 with no payload, ByBlock is method byte 1 with no
payload, and ByAlpha is method byte 2 with alpha in the low byte; ObjectARX
defines alpha 0 as clear and 255 as opaque. Reserved middle/payload bits and
unknown method bytes remain typed invalid states with raw provenance. Valid
edits pass through wire validation, strict semantic post-image verification,
and byte-identical inverse restoration. Paired ASCII/Binary fixtures cover all
nine dialects, with AC1009 retaining absence because group 440 is not
expressible by its Binary group-code wire. Effective layer/block resolution,
percentage rounding, rendering, applicability, and `Complete` support remain
open.

M14.3an adds target-kind validation after unique common-reference resolution.
Extension dictionary group 360 accepts only an exact `DICTIONARY` record in
`OBJECTS`, material group 347 only `OBJECTS`/`MATERIAL`, and plot-style group
390 only `OBJECTS`/`ACDBPLACEHOLDER`. A unique handle aimed at another marker
or section is a typed incompatible target with exact target evidence. Null,
missing, ambiguous, invalid-field, absent, and generated-default outcomes keep
their prior precedence; owner group 330 is explicitly unreviewed because its
valid target kind is family- and placement-dependent. Paired ASCII/Binary
fixtures cover AC1009 through AC1032, wrong markers, wrong sections,
cancellation, source identity, and public bounds. Authoritative ownership,
dictionary membership, pointer lifecycle, applicability, reference-safe CRUD,
family graphs, and `Complete` support remain open.

M14.3ao validates common handle edits before generic singleton planning.
Identity group 5 requires a handle-remap operation and owner group 330 requires
a placement/ownership operation. Extension dictionary 360, material 347, and
plot style 390 require a non-null handle with exactly one same-document target
of the reviewed `OBJECTS` kind. Null, missing, ambiguous, wrong-marker, and
wrong-section proposals return typed issues and never enter the session queue.
The identity index is built lazily once per session. Public classification has
ASCII/Binary parity across AC1009 through AC1032; AC1012-through-AC1032 session
tests prove materialization, strict target semantics, and byte-identical
inverse restoration. AC1009 Binary rejects group 347 at its physical wire
gate; accepting an expressible AC1009 ASCII group does not establish dialect
applicability. Handle remap, owner placement, dictionary membership, pointer
lifecycle, cross-document clone/delete, applicability, family graphs, and
`Complete` support remain open.

M14.3ap validates common exact-text symbol edits before generic planning.
Layer group 8 requires one exact same-document `LAYER` table name and linetype
group 6 requires one exact `LTYPE` name, both from completely closed matching
tables. Missing and duplicate exact names fail typed and never enter the
session queue. Layout group 410 and color-name group 430 require future
dedicated resolvers and are no longer accepted as unchecked generic text. One
named-symbol directory is built lazily per session and reused; the exact-text
resource ceiling is enforced before lookup. Paired ASCII/Binary AC1009-through-
AC1032 tests prove admission, rejected no-op behavior, materialization, strict
semantic verification, and byte-identical inverse restoration. Case folding,
symbol-character validity, XREF naming, layout/color-book semantics, table-
record creation, applicability, family graphs, and `Complete` support remain
open.

M14.3aq resolves common group-410 layout names on the read side. The new
source-bound directory admits only exact `OBJECTS`/`LAYOUT` records from
completely closed sections and selects group 1 only inside the exact
`AcDbLayout` subclass. It retains missing, unique, and duplicate name
cardinality while excluding inherited plot-settings names, application-group
content, wrong subclasses, wrong sections, and incomplete sections. Common
layout semantics preserve raw field failures and distinguish absent, unique,
missing, and ambiguous exact targets without selecting an occurrence. Paired
ASCII/Binary fixtures span AC1009 through AC1032 and malformed structure.
Matching is still byte-exact and case-sensitive. Layout edit admission,
reciprocal block-record ownership, legal-name/case policy, color-book
resolution, applicability, lifecycle, family graphs, and `Complete` support
remain open.

M14.3ar validates common layout-name edits before generic singleton planning.
The public classifier distinguishes non-layout fields, wrong value kinds,
missing exact names, ambiguous exact names, and one same-document layout-object
target. `DxfEntityEditSession` constructs that directory lazily only for an
exact group-410 proposal and enforces the existing value-byte ceiling first.
Accepted modern ASCII/Binary edits use canonical insertion/replacement, strict
reparse, target-bearing layout semantics, semantic/raw verification, and exact
inverse restoration; rejected proposals queue nothing. Classifier parity spans
AC1009 through AC1032. AC1009 Binary group 410 remains unencodable and fails at
the typed wire gate. Matching is byte-exact and case-sensitive. Reciprocal
block-record ownership, layout lifecycle, case/legal-name policy, color-book
resolution, applicability, family graphs, and `Complete` support remain open.

M14.3as parses common color-name group 430 as the Autodesk-documented
`colorbook$colorname` envelope. Exactly one separator and two nonempty
components are required; valid components remain exact source spans. The
projection composes group 430 with the same entity's reviewed group-420
24-bit true color and group-62 indexed color, while preserving missing,
duplicate, malformed, and out-of-domain relation states as typed failures.
Group order is irrelevant. ASCII/Binary fixtures span AC1009 through AC1032;
AC1009 proves exact absence because its Binary group-code encoding cannot
represent 430. External `.acb` loading, color-name/RGB verification, edit
admission, applicability, family graphs, and `Complete` support remain open.

M14.3at validates common color-book edits before generic singleton planning.
The source-bound classifier requires the reviewed one-`$`, nonempty-component
syntax and usable existing group-420/group-62 semantics on the exact entity.
It preserves distinct value-kind, delimiter, relation, source, and cancellation
failures. The session builds the common-domain directory lazily and queues
nothing on rejection. Accepted AC1012-through-AC1032 ASCII/Binary edits pass
canonical materialization, strict reparse, structured color-book semantics,
semantic/raw verification, and byte-identical inverse restoration. AC1009
remains rejected. External `.acb` resolution, name/RGB verification, atomic
pending tuple edits, applicability, family graphs, and `Complete` support
remain open.

M14.3au adds `DxfEntityPatch::CommonColorBook`, a composite session operation
that inserts or replaces common groups 62, 420, and 430 atomically. Candidate
syntax, AC1009 wire parity, and pending conflicts are checked before queueing;
failure during any component plan truncates all tuple components added by that
request. Accepted AC1012-through-AC1032 ASCII/Binary tuples strict-reparse to
the requested indexed color, true color, and structured book/color name, pass
three generic semantic/raw postconditions, and restore the exact source via
the inverse journal. Debug output records only name byte count. External
color-book resolution, mapping verification, composite reset, reviewed
applicability, family graphs, and `Complete` support remain open.

M14.3av adds `DxfEntityPatch::ResetCommonColorBook`, the correlated inverse of
M14.3au. Unique explicit groups 62, 420, and 430 are deleted as one logical
request; absent members are already implicit, and a wholly absent tuple queues
no edit. Post-image semantics expose indexed color as the reviewed BYLAYER
default 256 and true color/color name as absent. Failure in a later component
truncates earlier tuple deletions from that request while retaining unrelated
pending edits. Paired ASCII/Binary fixtures cover complete, partial, and
implicit tuples for AC1009 through AC1032, strict generic postconditions, and
byte-identical inverse restoration. External color-book resolution,
applicability, family graphs, entity insert/clone/delete, and `Complete`
support remain open.

M14.3aw adds atomic composition of independently built source-bound transaction
plans. Each input must match the exact raw document; its patches are replayed
through the M11 builder so cross-plan conflicts, duplicate insertion points,
resource limits, plan-count limits, and cancellation fail typed before a
composed plan escapes. Result patches are in source order regardless of input
plan order and retain freshly captured inverse bytes. Verifiable entity plans
can absorb supplemental raw transactions without discarding field semantic
postconditions. Across all nine dialects and both physical formats, fixtures
compose M11 identity insertion and successor `$HANDSEED` replacement with an
M14 common-color reset, strict-reparse the result, verify the assigned handle,
seed, field default, raw transaction, and byte-identical inverse. Entity-draft
encoding, placement/owner selection, reservation, insert, clone, and delete
remain open.

M14.3ax adds source-bound container-start insertion placements. Every indexed
`ENTITIES` section anchors before its first content group or exact `ENDSEC`;
every closed BLOCK definition anchors before its first member or exact
`ENDBLK`. This avoids cutting an existing POLYLINE or INSERT/ATTRIB record
sequence. The directory retains unavailable assessments for interrupted or
unclosed containers and rejects orphan nonzero content instead of attaching it
to a future entity. All nine dialects have ASCII/Binary parity, and each ready
anchor is executed through the bounded transaction builder and strict reparse.
Draft encoding, handle allocation, group-330 owner selection/validation,
semantic insertion verification, clone, and delete remain open.

M14.3ay adds source-bound handle reservation for records that have not yet been
inserted. A ready M11.2a policy yields a bounded consecutive range plus one
transaction that advances the exact `$HANDSEED` payload to the uppercase-hex
successor. Zero-count requests are empty; unavailable policy, exhaustion,
source mismatch, limits, and cancellation fail typed. The encoder is shared
with existing-record assignment. All nine dialects and both physical formats
compose one reserved handle with an M14.3ax placement, strict-reparse the new
record identity and successor seed, and execute a byte-identical inverse.
Reservation is optimistic and source-bound, not a global/concurrent lock.
Typed draft encoding, group-330 ownership, semantic insertion verification,
clone, and delete remain open.

M14.3az adds source-bound placement-owner admission. Caller-selected owner
handles must be non-null, uniquely identified, and an exact named entry in a
completely closed `BLOCK_RECORD` table. A BLOCK placement additionally checks
the BLOCK marker's outside-application group 330 cardinality and target
resolution, then requires its declared owner record to match the requested
record. Missing, duplicate, malformed, null, dangling, ambiguous, wrong-kind,
and mismatched evidence remain separate typed outcomes. An `ENTITIES`
placement never guesses model/paper space from group 67 or layout 410.
ASCII/Binary parity spans all nine dialects; AC1009 represents BLOCK-owner
absence in both formats rather than inventing an unencodable Binary group 330.
This admission does not review version applicability, encode or insert a
draft, update ownership graphs, implement clone/delete, or advance any entity
to `Complete`.

M14.3ba adds typed draft identity preparation for the complete generated
name registry: 45 canonical topics and 14 reviewed aliases. Unknown/custom
names cannot enter the typed draft-name API. One exact-source M14.3az owner
binding is combined with one exact-source M14.3ay handle reservation, and the
reservation must contain exactly one handle. The prepared value retains the
exact wire name, canonical-topic relation, handle, placement, admitted owner
BLOCK_RECORD, allocation, and successor `$HANDSEED` transaction. Every one of
the 59 names is prepared for all nine ASCII/Binary dialect pairs; the seed
transaction strict-reparses and inverses byte-identically. Registry admission
does not establish version applicability or family support. No entity record
bytes are encoded or inserted, and semantic verification, clone/delete graph
closure, and `Complete` status remain open.

M14.3bb adds fail-closed dialect admission for a prepared draft identity. The
document must expose one supported `$ACADVER`, the identity and its reversible
reservation transaction must still match that exact source, and the generated
applicability descriptor must classify the selected name as `Applicable` for
that dialect. `NotApplicable`, `NotYetReviewed`, and absent, unsupported,
invalid, or ambiguous version states remain separate typed results. The
current reviewed matrix therefore admits DGN/DWF underlay aliases from AC1021
and PDF underlay from AC1024; all other name ranges remain unavailable rather
than guessed. Tests exercise all 59 names over all nine ASCII/Binary pairs.
This is an admission gate only: it does not encode or insert record bytes,
validate an underlay payload, or advance any entity to `Complete`.

M14.3bc adds the first canonical-topic applicability receipt. Autodesk's MESH
compatibility statement identifies the newer MESH object type with AutoCAD
2010, so generated admission now marks canonical MESH applicable from AC1024
and not applicable in the six earlier supported dialects. Its exact source
GUID and normalized one-row SHA-256 are carried by the generated descriptor.
Together with the three previously reviewed underlay aliases, four names now
have reviewed ranges and 55 remain `NotYetReviewed`. This version receipt does
not validate MESH topology, enable MESH insertion, or advance MESH support.

M14.3bd adds canonical MLEADER applicability from Autodesk's earlier-version
compatibility statement: multileaders are proxy objects before AutoCAD 2008.
Generated admission therefore marks canonical MLEADER applicable from AC1021
and not applicable in the five earlier supported dialects. The source GUID and
normalized one-row SHA-256 remain attached to the descriptor. The behavioral
`MULTILEADER` alias deliberately stays `NotYetReviewed`; the family statement
does not prove the exact alias wire marker's minimum version. Five names now
have reviewed ranges and 54 remain unreviewed. No nested MLEADER parsing,
reference resolution, geometry, CRUD, or support claim changes here.

M14.3be adds canonical LIGHT applicability. Autodesk requires lighting from
versions before AutoCAD 2007 to be converted to the AutoCAD 2007/2008 lighting
format, while its DXF reference defines the exact LIGHT entity. Generated
admission therefore marks LIGHT applicable from AC1021 and not applicable in
the five earlier supported dialects, with exact source GUID and normalized
one-row receipt. Six names now have reviewed ranges and 53 remain unreviewed.
This does not validate light types, vectors, photometric settings, rendering,
shadows, family CRUD, or `Complete` support.

M14.3bf adds exact ACAD_TABLE alias applicability. Autodesk's AutoCAD 2005 API
history marks the Table entity/API and `acTable` enum value as new; the existing
Autodesk TABLE DXF receipt independently fixes group 0 to `ACAD_TABLE`.
Generated admission therefore marks only ACAD_TABLE applicable from AC1018 and
not applicable in AC1009 through AC1015. Canonical `TABLE` remains
`NotYetReviewed` rather than being treated as an interchangeable wire marker.
Seven names now have reviewed ranges and 52 remain unreviewed. Cell grammar,
style references, layout, CRUD, and `Complete` support remain open.

M14.3bg adds canonical HELIX applicability. Autodesk's AutoCAD 2007 API history
marks `IAcadHelix` and its public constraint/twist enums as new, while the
Autodesk HELIX DXF page independently defines the exact HELIX entity and
`AcDbHelix` subclass. Generated admission therefore marks HELIX applicable from
AC1021 and not applicable in AC1009 through AC1018. Eight names now have
reviewed ranges and 51 remain unreviewed. Existing typed HELIX evidence is not
an analytic composition, writer, CRUD, or `Complete` support claim.

M14.3bh adds canonical LWPOLYLINE applicability. Autodesk's lightweight versus
old-style polyline guidance says 2D polylines are created as lightweight
entities as of Release 14 and earlier-release 2D polylines convert when opened.
The Autodesk LWPOLYLINE DXF page independently defines the exact entity and
`AcDbPolyline` subclass. Generated admission therefore marks LWPOLYLINE
applicable from AC1014 and not applicable in AC1009/AC1012. Nine names now have
reviewed ranges and 50 remain unreviewed. Existing field, segment, width, bulge,
OCS/WCS, and geometry coverage is not insert/CRUD/writer or `Complete` support.

M14.3bi promotes exact-name provenance for `SECTIONOBJECT` and the five surface
specializations from an AutoCAD inventory oracle to Autodesk's valid DXF object
name table. Autodesk's AutoCAD 2007 API history separately marks the matching
Section and Surface classes as new, so generated admission marks all six aliases
applicable from AC1021 and not applicable through AC1018. Canonical `SECTION`
and `SURFACE` remain `NotYetReviewed`; observed concrete aliases do not prove
generic group-0 records. Fifteen names now have reviewed ranges and 44 remain
unreviewed. Section/surface payloads, proprietary modeler data, CRUD, and
`Complete` support remain open.

M14.3bj admits the 16 exact canonical names Autodesk lists as introduced prior
to Release 13 from AC1009 onward: `3DFACE`, `ARC`, `ATTDEF`, `ATTRIB`, `CIRCLE`,
`DIMENSION`, `INSERT`, `LINE`, `POINT`, `POLYLINE`, `SEQEND`, `SHAPE`, `SOLID`,
`TEXT`, `VERTEX`, and `VIEWPORT`. AC1009 is the existing reviewed Release 11/12
dialect and the Core 1.0 minimum, so all nine supported versions are
`Applicable`. Thirty-one of 59 names now have reviewed ranges and 28 remain
unreviewed. This is a wire-version receipt only. In particular, Autodesk's
separate statement that `entmake` cannot create VIEWPORT entities is not
overridden; family encoding, insertion policy, CRUD, and `Complete` remain
open.

M14.3bk encodes one typed canonical `POINT` record across ASCII and Binary
AC1009 through AC1032. The plan is bound to the exact source, admitted POINT
applicability, one reserved handle, caller-selected placement/BLOCK_RECORD
owner, exact existing layer, and finite WCS location. R13+ output adds owner
and `AcDbEntity`/`AcDbPoint`; AutoCAD 2000+ output requires an explicit public
lineweight and, for `ENTITIES`, one exact same-document layout name. BLOCK-local
output rejects and omits layout. Tests compose the record with `$HANDSEED` and
the placement transaction, strict-reparse the result, recover POINT semantics,
and apply an executable byte-identical inverse for every supported dialect and
format. This checkpoint does not add the final edit-session insert operation,
optional POINT fields, clone/delete, placement-aware generic common-field
cardinality, or POINT `Complete` support.

M14.3bl consumes that typed record as one atomic insertion transaction. The
planner combines the exact placement-anchor patch with the reserved successor
`$HANDSEED` patch and returns a `DxfEntityEditPlan` carrying one logical POINT
insert expectation. Verification requires a unique allocated handle, canonical
POINT classification, the exact ENTITIES section or BLOCK definition, the
represented owner, explicit layer/layout/lineweight fields, and the expected
WCS location. The existing create-new writer then strict-reparses, verifies,
removes failed destinations, and journals a byte-exact inverse. Direct and
create-new coverage spans all nine ASCII/Binary dialect pairs; BLOCK placement
spans every R13+ dialect, and tampering proves typed missing/ambiguous handle,
family, common-field, and geometry failures. This is one prepared-record
operation, not yet `DxfEntityEditSession::insert`, multi-record reservation,
optional POINT payload support, clone/delete, or POINT `Complete` support.

M14.3bm adds the public unified-session POINT insertion operation with the
planned `insert(placement, draft)` shape. The typed draft carries an explicit
caller-selected BLOCK_RECORD owner; the session validates it, reserves one
handle, applies the generated dialect gate, encodes the record, and retains the
M14.3bl transaction and semantic expectation. The source remains immutable.
`finish` returns the composed insertion/`$HANDSEED` transaction, while
`finish_verifiable` also requires the inserted POINT postcondition before an
inverse is released. Compact receipts/issues preserve handle, entity name,
placement, owner, allocation, applicability, and record failure states without
payload bytes. Direct session coverage spans ASCII/Binary AC1009 through
AC1032. One insert is admitted per session; a second insert or any insert/update
mixture is rejected without queue growth. Multi-record insertion, mixed
ordinal-independent verification, optional POINT fields, clone/delete, and
POINT `Complete` remain open.

M14.3bn admits multiple POINT insertions in one unified session. Successful
admission copies the encoded record and semantic expectation into owned,
payload-redacted state and assigns a deterministic proposal handle; rejected
drafts neither grow the queue nor consume a handle ordinal. Finish creates one
shared consecutive reservation, verifies every proposed handle, advances
`$HANDSEED` once, and groups same-anchor records in caller/handle order before
transaction composition. Across all ASCII/Binary AC1009-through-AC1032 pairs,
three records allocate `0x40`, `0x41`, and `0x42`, publish successor seed
`0x43`, occupy one combined insertion patch plus one seed patch, satisfy all
three POINT semantic expectations, and inverse to byte-identical source.
Handle exhaustion remains typed. Optional POINT payload, mixed insert/update
sessions, non-POINT draft families, clone/delete, and POINT `Complete` remain
open.

M14.3bo adds the remaining public POINT payload to evidence, semantics, typed
drafts, canonical encoding, and insertion verification. POINT records now
retain thickness `39` and UCS X-axis angle `50` in source order alongside the
existing WCS location and extrusion tuple. Eight fixed cards preserve absence,
uniqueness, duplicates, invalid numerics, and raw provenance. Absent thickness
and angle default to zero; absent extrusion components default to `(0, 0, 1)`;
present invalid or duplicate evidence remains invalid. Drafts may explicitly
emit thickness, a complete nonzero extrusion, and angle after location in
Autodesk table order. Unspecified optional values remain omitted. The verifier
checks exact values and explicit-versus-defaulted state, so replacing or
silently omitting a requested field fails typed. Minimal and all-explicit
records pass every ASCII/Binary Core dialect, and POINT batches retain the same
strict-reparse/inverse guarantees. POINT update, clone/delete, mixed operation
sessions, display behavior, and `Complete` remain open.

M14.3bp adds the first typed POINT-family update to the unified edit session.
`DxfPointPatch::SetLocation` requires a canonical POINT with exactly one
existing WCS group `10/20/30` component, encodes three finite doubles in the
source ASCII/Binary dialect, and queues the tuple as one logical edit backed by
three atomic raw replacements. One POINT location update may compose with
independent common-property edits; insert/update mixing remains rejected.
Strict post-image verification resolves the same raw-record ordinal and
requires the exact typed location before releasing the inverse journal. Paired
fixtures span AC1009 through AC1032 and prove out-of-order source groups,
duplicate/missing components, wrong families, non-finite values, duplicate
patches, cancellation, tampering, common-field composition, strict reparse,
and byte-identical inverse restoration. Thickness, extrusion, angle, reset,
clone/delete, and POINT `Complete` remain open.

M14.3bq adds source-bound replacement of one existing explicit POINT thickness
group `39`. The typed patch rejects absent or duplicate occurrences instead of
inventing an insertion anchor or selecting evidence, and it encodes only finite
binary64 values for the source physical format and dialect. Thickness and
location patch identities may compose for the same POINT, while a second patch
of either kind is rejected without changing the queue. Strict verification
requires the exact thickness and explicit semantic state on the same raw-record
ordinal before releasing the inverse. Paired fixtures span AC1009 through
AC1032 in ASCII and Binary and cover missing/duplicate evidence, wrong family,
non-finite input, duplicate admission, location composition, cancellation,
tampering, strict reparse, and byte-identical inverse restoration. Absent-field
insertion, reset, extrusion/angle updates, mixed insert/update sessions,
clone/delete, display behavior, and POINT `Complete` remain open.

M14.3br extends that typed thickness patch to documented defaulted absence.
When group `39` is absent, all three source-backed POINT location components
must be unique; the last source occurrence becomes a zero-width insertion
predecessor. The original ASCII line ending or Binary group framing is retained,
and the new field is verified as the exact requested binary64 value in the
`Explicit` state before an inverse is released. A unique existing group remains
an exact replacement, while duplicate thickness or unusable location anchor
evidence queues nothing. All nine ASCII/Binary dialect pairs, CRLF insertion,
strict reparse, and byte-identical removal pass. Reset to implicit zero,
extrusion/angle updates, mixed insert/update sessions, clone/delete, display,
and POINT `Complete` remain open.

M14.3dg adds `DxfEntityXDataEntityDestinationDirectory`, composing every
M14.3df application result for an indexed entity with its exact M14.3cr source
capacity entry. Ready requires all applications ready and capacity within the
16,383-byte limit; zero-XDATA indexed records are explicitly ready with zero
used bytes. Unavailable state retains application counts, the number of
unavailable applications, and the complete within-limit, exceeded, or
indeterminate capacity state. Four ASCII/Binary pairings cover every Core
dialect plus cross-dialect boundaries, destination-only failure with exact
capacity, structural indeterminacy, independently exceeded capacity, zero-XDATA
records, dual-source identity, cancellation, bounds, and debug redaction.
Coordinate transforms, handle remaps, payload semantics, encoding/insertion,
destination mutation, cross-container clone, and POINT `Complete` remain open.

M14.3dh adds `DxfEntityXDataCoordinateDestinationDirectory`, composing each
M14.3dg entity result with exact M14.3cs affine tuple projection. Ready requires
the entity ready and every entity-bound tuple transformed successfully; zero-
tuple records remain ready. Unavailable retains the complete entity state plus
tuple and unavailable-tuple counts, while typed transform entries preserve
partial, invalid, and non-finite-derived reasons. Four ASCII/Binary pairings
cover every Core dialect plus cross-dialect boundaries, role-specific 1010
versus 1011 behavior, destination-only failure, partial tuples, dual-source
identity, cancellation, bounds, and redaction. Handle remaps, payload semantics,
encoding/insertion, mutation, cross-container clone, and POINT `Complete`
remain open.

M14.3di adds `DxfEntityXDataHandleComposedDestinationDirectory`, composing each
M14.3dh coordinate-readiness entry with caller-supplied group-1005 remaps that
M14.3cv validates against the independently parsed destination identity index.
Ready requires coordinate readiness and a unique destination identity for every
entity-bound group-1005 occurrence; entities with no handles remain ready when
their coordinate/base state is ready. Unavailable retains the complete
coordinate state plus exact total and unavailable handle counts, while every
typed remap/destination state remains reachable through the owned handle
directory. Four ASCII/Binary pairings cover all nine Core dialects plus
AC1009/AC1032 cross-dialect boundaries, independent coordinate and handle
failures, zero handles, dual-source identity, cancellation, bounds, and
redaction. Application payload semantics, encoding/insertion, destination
mutation, cross-container clone, and POINT `Complete` remain open.

M14.3dj adds `DxfEntityXDataPayloadDestinationDirectory`, composing each
M14.3di result with the exact source-bound typed occurrence and application
slices already owned by its coordinate chain. It counts logical payload values
separately from group-1001 application names and publishes exact orphan-value
counts. Ready requires the complete M14.3di state ready and zero orphan values;
unavailable preserves the entire handle-composed state plus both payload and
orphan counts. Four ASCII/Binary pairings cover all nine Core dialects plus
AC1009/AC1032 cross-dialect boundaries, valid application payloads, direct
orphan classification, handle and coordinate failures without orphans, zero
payloads, dual-source identity, cancellation, bounds, and redaction. This is
payload-envelope readiness only: application-specific meaning, destination
logical-value projection, encoding/insertion, mutation, cross-container clone,
and POINT `Complete` remain open.

M14.3dk adds `DxfEntityXDataLogicalDestinationDirectory`, publishing one
source/destination-bound logical state for every retained typed XDATA
occurrence. Exact source string, control, binary chunk, non-coordinate double,
and integer values remain source-referenced; group-1001 and group-1003 values
publish exact destination APPID/LAYER records; group-1005 publishes only a
unique remapped destination handle; and each complete coordinate component
publishes its M14.3cs transformed value. Orphan, invalid-source, APPID, LAYER,
handle, and coordinate blockers remain distinct typed unavailable states. Four
ASCII/Binary pairings cover all nine Core dialects plus AC1009/AC1032 cross-
dialect boundaries, every generic logical family, specialized/source-exact
independence, per-entity slicing, dual-source identity, cancellation, bounds,
and redaction. Application-specific meaning, destination text transcoding,
byte encoding/insertion, mutation, cross-container clone, and POINT `Complete`
remain open.

M14.3dl adds `DxfEntityXDataEncodedDestinationDirectory`, canonically encoding
every M14.3dk available logical occurrence as one complete destination-format
group without inserting it. The shared encoder supplies ASCII framing, Binary
value widths, uppercase handles and chunks, and the AC1009 extended-data group-
code escape. APPID/LAYER names come from their exact destination records;
source strings/controls/chunks are read through bounded provenance; transformed
doubles, remapped handles, and integers use their typed logical values. ASCII-
only source text is portable; non-ASCII source text requires the same reviewed
source/destination decoder or remains typed `TextTranscodingRequired`.
Unavailable logical, dialect, transcoding, and group-encoding states publish no
bytes. Four format pairings cover all nine Core dialects plus AC1009/AC1032
cross-dialect boundaries, canonical family bytes, decoder mismatch, dual-source
identity, cancellation, bounds, and redaction. Application-specific meaning,
actual text transcoding, application/entity grouping, insertion, mutation,
cross-container clone, and POINT `Complete` remain open.

M14.3dm adds `DxfEntityXDataEncodedApplicationDestinationDirectory`, composing
the M14.3dl occurrence results into one exact source-order set for every source
XDATA application without insertion. A set is ready only when its owning
M14.3dj payload envelope is ready and every application member has canonical
destination bytes. Unavailable sets retain the complete payload state, exact
member and unavailable-member counts, the first unavailable member ordinal,
the source application, and every underlying encoded result; aggregate bytes
remain unavailable unless the whole set is ready. Exact application ranges
exclude orphans by construction and preserve empty applications and nested
controls. Four format pairings cover all nine Core dialects plus AC1009/AC1032
cross-dialect boundaries, ready and unavailable siblings, source-exact and
transformed values, remapped handles, decoder mismatch, empty/nested lists,
orphans, cancellation, dual-source identity, bounds, and redaction.
Application-specific interpretation, actual text transcoding, per-entity
encoded grouping, insertion, mutation, cross-container clone, and POINT
`Complete` remain open.

M14.3dn adds `DxfEntityXDataEncodedEntityDestinationDirectory`, composing the
M14.3dm application sets into one exact encoded payload per indexed source
entity without insertion. Ready requires the M14.3dj payload envelope and every
owned application set ready; zero-XDATA entities remain explicit ready entries
with empty bytes. Unavailable entries retain the complete payload state,
application/member totals, unavailable counts, and first unavailable
application ordinal, while aggregate bytes remain unavailable. Four format
pairings cover all nine Core dialects plus AC1009/AC1032 cross-dialect
boundaries, ready, zero-XDATA, failed-handle and orphan entities, empty/nested
applications, cancellation, dual-source identity, bounds, and redaction.
Application-specific interpretation, actual text transcoding, insertion,
mutation, cross-container clone, and POINT `Complete` remain open.

M14.3do adds `DxfEntityXDataDraftRecordPlan`, composing one M14.3dn ready
source-entity payload with a canonical destination-bound entity draft record
without insertion. Composition requires the exact encoded-directory entry and
the destination draft's immutable source identity, appends the canonical XDATA
groups after the family record groups, preserves source-entity and encoded-state
evidence, and treats zero-XDATA as an exact no-op. Unavailable payloads,
cancelled work, foreign entries, and foreign destination drafts fail closed.
Four format pairings cover all nine Core dialects plus AC1009/AC1032 cross-
dialect boundaries, exact prefix/suffix bytes, zero-XDATA, orphan payloads,
dual-source identity, cancellation, bounded record growth, and redaction.
Insertion, post-write XDATA verification, application-specific interpretation,
actual text transcoding, cross-container clone completion, and POINT `Complete`
remain open.

M14.3dp adds `DxfEntityXDataDraftInsertPlan`, composing M14.3do with the
existing destination draft-insertion transaction without writing. It retains
the exact expected XDATA suffix, source entity, encoded entry/state, destination
identity, and family edit plan. The underlying atomic transaction reserves the
new handle and inserts the complete canonical family-plus-XDATA record; zero-
XDATA retains an explicit empty expectation. Cancellation and foreign
destination documents fail before a plan is published. Four format pairings
cover all nine Core dialects plus AC1009/AC1032 cross-dialect boundaries,
transaction shape, exact payload suffixes, zero-XDATA, destination identity,
cancellation, bounds, and redaction. No destination is written yet, and post-
write XDATA verification, application-specific interpretation, actual text
transcoding, cross-container clone completion, and POINT `Complete` remain
open.

M14.3dq adds strict post-image verification for M14.3dp. The existing family
verifier must first prove the exact transaction post-image, typed POINT
postconditions, and executable inverse. The XDATA verifier then resolves the
inserted handle uniquely in the strict-reparsed document, locates its indexed
entity, requires zero orphan XDATA values, reads the complete contiguous raw
XDATA span under the resource profile, and compares it byte-for-byte with the
retained expectation. A receipt binds source, destination, post-image, inserted
handle, application count, and XDATA byte count; the journal retains the family
verification and inverse. Four format pairings cover all nine Core dialects
plus AC1009/AC1032 cross-dialect boundaries, non-empty and zero-XDATA payloads,
inverse restoration, cancellation, foreign pre-images, tamper rejection,
bounds, and redaction. Create-new write cleanup under this XDATA wrapper,
application-specific interpretation, actual text transcoding, cross-container
clone completion, and POINT `Complete` remain open.

M14.3dr adds create-new writing for M14.3dp plans. It streams the atomic
destination transaction to a path that must not exist, strictly reparses the
new ASCII or Binary document, runs the complete M14.3dq family-plus-XDATA
verification, binds write and verification identities, and returns both
receipts with the executable inverse. Existing destinations are never modified;
pre-cancelled work creates nothing; write/reparse/verification failures remove
the newly created destination. Four format pairings cover all nine Core
dialects plus AC1009/AC1032 cross-dialect boundaries, non-empty and zero-XDATA
writes, byte-identical output, receipt identities, inverse restoration,
existing-file rejection, cancellation, final-progress tampering, cleanup,
bounds, and redaction. Application-specific interpretation, actual text
transcoding, generic source-entity-to-family draft projection, cross-container
clone completion, and POINT `Complete` remain open.

M14.3ds extracts the existing same-document POINT clone semantics into one
immutable source snapshot and projects it into a separately parsed destination
family draft. The caller supplies explicit destination-local layer, layout,
linetype, material, and plot-style bindings; existing destination draft
validation remains authoritative for missing, ambiguous, incompatible, and
dialect-inapplicable targets. All reviewed common scalar fields, exact proxy
graphics, and POINT geometry are retained. Four format pairings cover all nine
Core dialects, with AC1009/AC1032 boundary behavior, same-document rejection,
missing/ambiguous layer targets, unexpected bindings, cancellation, and debug
redaction. XDATA composition with this family projection, insertion/write as a
complete cross-container clone, application-specific XDATA interpretation,
actual text transcoding, and POINT `Complete` remain open.

M14.3dt selects one exact encoded-XDATA entity entry, derives its owning source
POINT, performs the M14.3ds family projection in XDATA-composition mode, and
immediately appends the M14.3dn payload through the existing M14.3do validator.
The result retains source key/dialect/placement/owner, dual-source identities,
the exact encoded entry, and the composed draft. Standalone family projection
continues to reject XDATA groups, preventing a public payload-dropping path.
Four format pairings cover all nine Core dialects plus the AC1009-to-AC1032
boundary, then consume the plan through insertion, strict post-image family-
plus-XDATA verification, and byte-exact inverse restoration. Cancellation,
foreign source identity, unavailable/orphan payloads, traits, bounds inherited
from both pipelines, and redaction remain fail-closed. A direct convenience
wrapper for insertion/create-new writing, application-specific XDATA meaning,
actual text transcoding, the reverse boundary where explicit modern fields are
inapplicable, and POINT `Complete` remain open.

M14.3du consumes one M14.3dt family-plus-XDATA draft into an atomic destination
insertion plan while retaining compact source POINT identity, dialect,
placement, and owner evidence. Dedicated verification and create-new write
outcomes carry the same provenance beside the existing strict XDATA journals,
receipts, and executable inverse. Four format pairings cover all nine Core
dialects plus AC1009-to-AC1032 through exact file creation and strict reparse.
Existing destinations remain unchanged; pre-cancellation creates nothing;
final-progress tampering is rejected and the created file is removed. Foreign
destination planning, identity, bounds inherited from the composed pipelines,
traits, and debug redaction remain fail-closed. Application-specific XDATA
meaning, actual text transcoding, reverse-boundary adaptation, higher entity
families, and POINT `Complete` remain open.

M14.3dv makes the AC1032-to-AC1009 POINT boundary usable only for two reviewed
semantic equivalences. A modern explicit layout is represented by the legacy
destination placement/owner binding, and explicit `BY_LAYER` lineweight is
omitted where group 370 is not applicable. A compact adaptation mask records
both decisions through family projection, XDATA composition, insertion,
verification, and write journals. Any other explicit lineweight returns typed
`DestinationFieldNotRepresentable`; no numeric approximation occurs. Four
format pairings cover both AC1009/AC1032 directions through strict create-new
write, reparse, verification, and exact inverse restoration. Application-
specific XDATA meaning, actual text transcoding, non-default modern common
fields, higher entity families, and POINT `Complete` remain open.

M14.3dw adds a bounded encoder for UTF-8 and every reviewed `encoding_rs`
legacy code page, with typed output-full, unmappable, and unavailable results
and no replacement output. A dual-document primitive reads one exact source
span, resolves both documents' text-storage decisions, decodes and encodes
within the selected value-byte ceiling, then decodes the destination bytes
again and requires byte-exact UTF-8 equality before publishing an immutable
plan. The plan binds both source identities, the exact source span, both
encoding resolutions, byte counts, and destination bytes while redacting text
from debug output. Four ASCII/Binary format pairings cover UTF-8/Windows-1252
both ways; focused cases cover same/cross-legacy values, empty text,
unmappable Unicode, unavailable Johab encoding, malformed and unsupported
source text, indeterminate destination encoding, cancellation, hostile spans,
identity/count evidence, traits, and redaction. This primitive is not yet
wired into POINT or XDATA field projection; symbol names remain governed by
explicit destination bindings and are never automatically transcoded.

M14.3dx integrates the M14.3dw primitive into exact group-1000 XDATA string
encoding. Non-ASCII values with different reviewed storage decisions now
decode, encode, and round-trip verify before canonical destination group
framing; ASCII values and non-ASCII values with the same available decoder
retain their exact source bytes. Each successful conversion stores a compact
dual-source receipt outside the hot entry metadata and exposes it only through
entry-identity validation. APPID group 1001 and LAYER group 1003 continue to
use exact destination symbol-table targets, while group-1002 controls remain
source-exact. Destination text above the 255-byte XDATA string ceiling is
typed unavailable. UTF-8/Windows-1252 both ways span all four ASCII/Binary
format pairs and remain ready through application/entity grouping; a Binary
AC1018-to-ASCII AC1021 POINT clone reaches create-new writing, strict reparse,
exact XDATA verification, and inverse restoration. Unmappable, malformed,
unsupported, indeterminate, Johab-unavailable, over-limit, cancellation,
identity, metadata-bound, trait, and redaction paths remain fail-closed.
Application-specific XDATA meaning, automatic symbol creation, other entity
text fields, higher entity families, and POINT `Complete` remain open.

M14.3dy applies verified storage transcoding to the reviewed POINT group-430
color-book name. The source clone snapshot now retains exact text provenance
alongside copied bytes for layer, layout, linetype, and color name, but only
color name is eligible for automatic storage conversion; document-local
symbols still require explicit destination bindings. Portable ASCII and equal
available decoders remain byte-exact. A conversion receipt binds the exact
source field span, both document identities/resolutions, and byte counts, and
survives family projection, XDATA draft composition, insertion, strict
verification, and create-new write journals without disclosing text. UTF-8/
Windows-1252 conversions run both ways over all four ASCII/Binary format pairs;
a Binary AC1018-to-ASCII AC1021 color-book clone passes strict write/reparse,
typed color/XDATA verification, and exact inverse restoration. Unmappable text
is typed at projection, and group 430 remains non-representable before AC1015.
The transcode verifier also reserves four bounded scratch bytes so a legacy
decoder can finish an exact-length round trip without increasing accepted
payload size. Other free-text common fields, application-specific XDATA
meaning, automatic symbol creation, higher entity families, and POINT
`Complete` remain open.

M14.3dz publishes the first typed entity-completion ledger without broadening
the underlying support evidence. POINT is explicitly audited at
`VerifiedMutation`, level 5 of the six-level entity subplan: exact evidence,
cardinality, typed semantics, WCS geometry, and verified create/update/clone/
delete materialization are satisfied. `ReleaseQualified` remains false with
exact blockers `PrivateCorpusQualification` and
`CurrentCheckpointSixNativeCi`. The committed corpus policy is not a passing
private-corpus receipt, and six-native run `30557566354` belongs to commit
`222eec2c9d9b18fbb7ff1b8d5f0120ba30633365`, not this checkpoint. The other 44
public topics remain unaudited in this ledger rather than receiving inferred
levels. Rendering, POINT display behavior, application-specific XDATA meaning,
and automatic symbol creation remain separate capabilities, not additional
entity-completion levels.

M14.3ea adds bounded on-demand point evaluation for analytically ready SPLINE
records. A source-bound raw-record lookup accepts one finite parameter inside
the exact active knot interval and evaluates unit or explicit positive weights
in four-dimensional homogeneous coordinates with the De Boor recurrence. The
fixed degree ceiling is 64, bounding scratch storage to 65 homogeneous points
and the quadratic recurrence to 4,096 interpolation steps; higher degrees
remain typed rather than consuming unbounded CPU. Endpoints, internal spans,
rational division, canonical positive zero, cancellation, fallible allocation,
and derived non-finite arithmetic are explicit. All nine Core dialects in both
ASCII and Binary produce identical evaluated bits for the authored quadratic
fixture. Missing records return no result; unavailable analytic inputs,
non-finite/out-of-domain parameters, non-finite stored coordinates/weights,
degenerate active knot intervals, overflow, and invalid homogeneous weights
remain typed. This is point evaluation, not adaptive sampling, derivatives,
tessellation, rendering, HELIX evaluation, SPLINE CRUD/write, or SPLINE
`Complete` support.

M14.3eb adds the matching bounded rational first derivative. The shared
preflight now owns parameter/domain/degree/input validation and source-local
homogeneous controls for both point and derivative paths, while one common De
Boor kernel retains the already verified point results. The derivative control
polygon uses the degree-scaled knot-interval difference, then a degree-minus-
one De Boor pass produces the homogeneous derivative. Cartesian output applies
the rational quotient rule and returns the evaluated point and unnormalized
first-derivative vector together. At degree 64 the two passes use at most 129
four-component scratch points and 4,096 combined recurrence steps. All nine
Core dialects in ASCII and Binary agree bit-for-bit for quadratic endpoint,
midpoint, and derivative results; an independent rational fixture proves the
nonconstant-weight quotient rule. Existing typed parameter, analytic,
non-finite, degenerate-knot, overflow, cancellation, allocation, lookup, and
degree-limit outcomes are reused. A zero derivative remains a valid vector;
normalization, tangent-frame policy, curvature, adaptive sampling,
tessellation, rendering, HELIX evaluation, SPLINE CRUD/write, and SPLINE
`Complete` remain open.

M14.3ec audits the curve-family completion boundary before M14.4. SPLINE now
has an explicit `Geometry` assessment, level 4 of 6: exact evidence,
cardinality, typed analytic readiness, bounded rational point evaluation, and
bounded rational first derivatives are evidenced. Its remaining blockers are
`VerifiedMutation`, `PrivateCorpusQualification`, and
`CurrentCheckpointSixNativeCi`. HELIX has an explicit `TypedSemantics`
assessment, level 3 of 6: exact subclass-scoped fields, fixed cardinality,
vectors/scalars/relations, and embedded-SPLINE readiness are evidenced, but
Autodesk's warning against inherited NURBS operations leaves exact stored-curve
evaluation unqualified. Its additional blocker is
`PublicGeometryQualification`. POINT remains unchanged at level 5. The ledger
is sorted by canonical topic ordinal and uses deterministic binary lookup; the
other 42 topics remain unaudited rather than inferred. Neither curve is called
`Complete`, and mutation closure remains assigned to M14.11.

M14.4a starts fills and meshes with one deliberately untyped evidence layer.
Exact canonical uppercase HATCH and MESH records in complete BLOCKS or ENTITIES
sections retain each exact `AcDbHatch` or `AcDbSubDMesh` subclass occurrence
and every non-application raw field until the next subclass marker. Record,
subclass, and field ranges preserve source order, group occurrence, group code,
payload span, source identity, and family kind. Missing, reversed, near-case,
and duplicate subclass markers remain distinguishable; nested HATCH and MESH
code collisions are not assigned premature roles. ASCII/Binary fixtures cover
all nine Core dialects. Recognition remains dialect-neutral and does not raise
HATCH applicability or weaken the reviewed AC1024 MESH applicability boundary.
Cardinality, typed values, boundary paths, topology, geometry, subdivision,
CRUD/write, and completion status remain open.

M14.4b projects only the 25 HATCH roles whose group codes are unambiguous across
the top-level, boundary-path, pattern-line, seed-point, and gradient grammar.
Exact text spans remain source-backed; doubles retain IEEE-754 bits; codes in
the 60-79 range decode as signed Int16, including group 78 pattern-line count;
codes 90-99 and 450-459 decode as signed Int32. Invalid ASCII numerics and all
duplicates remain source ordered. Each exact `AcDbHatch` subclass receives an
independent compact range, while MESH and near/missing subclass markers remain
excluded. The AC1009 paired fixture omits group 450-470 because its Binary DXF
entity group-code grammar is one byte; this is a wire observation, not HATCH
applicability evidence. Codes whose roles depend on nested state remain raw.
Cardinality, defaults/domains, tuple assembly, boundary/path partitioning,
pattern lines, gradients relations, geometry, CRUD/write, applicability, and
completion status remain open.

M14.4c adds exactly 25 source-stable cardinality cards per exact `AcDbHatch`
subclass in the M14.4b directory. Each card retains its subclass entry, role,
compact member range, and `Absent`, `Unique`, or `Multiple` state; members are
ordinal references back to the original typed occurrences rather than copied
payloads. Duplicate subclass markers in one raw record receive independent card
sets, and raw-record lookup returns their contiguous aggregate. Invalid numeric
payloads do not alter occurrence count or cause value selection. All nine Core
dialects have ASCII/Binary card/member parity, including nine absent high-code
cards for AC1009. Defaults, domains, semantic relations, tuples, nested HATCH
state, geometry, applicability, CRUD/write, and completion remain open.

M14.4d publishes 25 stable four-state singleton semantics per exact HATCH
subclass. Unique values retain exact field and raw provenance; duplicates are
typed invalid without choosing a member; malformed ASCII and non-finite Binary
doubles remain source-anchored invalid states. Only the documented optional
extrusion direction receives component defaults `(0,0,1)`. Flags, style,
pattern type, nonnegative counts, gradient reserved fields, color-mode/count,
shift, and tint receive their documented individual domains. Gradient defaults
are not applied while the optional group-450 envelope is absent; envelope and
tuple relations remain separate. All nine Core dialects have ASCII/Binary
semantic parity, with the nine high-code roles absent for AC1009. Tuple
assembly, cross-field relations, nested HATCH state, geometry, applicability,
CRUD/write, and completion remain open.

M14.4e assembles one exact extrusion tuple for every exact `AcDbHatch`
subclass from the M14.4d singleton semantics. Each component retains whether
it was explicit or defaulted, partial explicit tuples use only the reviewed
component defaults, and the vector is returned bit-exact without
normalization. Duplicate, malformed, or non-finite component evidence produces
an exact unavailable-component mask; an all-zero vector, including signed
zero, produces a separate typed issue. Duplicate subclasses remain isolated,
and lookup remains source anchored and cancellation aware. Elevation cannot yet
be assembled safely because its top-level group 10/20 components collide with
boundary and seed-point fields; that requires stateful HATCH partitioning.
This checkpoint adds no elevation tuple, coordinate transform, geometry,
applicability, CRUD/write, or completion claim.

M14.4f adds the first stateful HATCH grammar partition without decoding a
boundary path. For every exact `AcDbHatch` subclass, unique group 91 and group
75 fences in canonical order divide the exact field inventory into a top-level
header through group 91, opaque boundary-path payload, and a trailing span
beginning with group 75. All fields and raw payloads remain in source order;
the partition only publishes ranges and the exact fence fields. Missing or
duplicate fences and reversed order remain typed issues with no guessed span.
Duplicate subclasses stay independent and all nine Core dialects have paired
ASCII/Binary partitions. The isolated header now prevents boundary group 10/20
collisions from contaminating later elevation work, but this checkpoint does
not select elevation, compare declared path counts, decode path/edge types,
partition pattern or seed data, derive geometry, establish applicability, add
CRUD/write behavior, or claim completion.

M14.4g selects the required HATCH elevation point only from the M14.4f header
range. Unique groups 10, 20, and 30 decode as exact binary64 values with their
raw groups retained; X and Y must equal zero as documented, including accepted
signed-zero encodings, while Z may be any finite value. Boundary-path and seed
group 10/20 decoys, and even unrelated group 30 payloads outside the header,
cannot affect the tuple. Missing or duplicate components, malformed ASCII,
non-finite Binary values, nonzero X/Y, and an unavailable boundary partition
remain typed states; no component default is invented. Duplicate subclasses
remain independent and all nine Core dialects have ASCII/Binary parity. This
checkpoint does not transform OCS to WCS, reconcile path counts, decode nested
HATCH state, derive geometry, establish applicability, add CRUD/write behavior,
or claim completion.

M14.4h groups the M14.4f opaque boundary span into conservative path envelopes.
Every exact group 92 starts one path; its raw marker remains separate and its
payload extends to the next group 92 or the boundary fence. Fields before the
first anchor are retained as an explicit orphan slice rather than attached to a
guessed path. The unique group 91 fence decodes as signed Int32 and reports a
matched or mismatched declared-versus-observed count, malformed ASCII, or a
negative declared count. Partition failures remain typed and publish no path
slice. Duplicate subclasses stay independent and all nine Core dialects have
ASCII/Binary path/payload parity. This checkpoint does not interpret group-92
flags, choose polyline versus edge grammar, decode edge/path values or source
handles, validate path payload completeness, derive geometry, establish
applicability, add CRUD/write behavior, or claim completion.

M14.4i decodes each M14.4h group-92 marker as signed Int32 and accepts only the
documented bit mask: External 1, Polyline 2, Derived 4, Textbox 8, and Outermost
16. Every valid combination retains the raw marker and exposes stable helpers;
bit 2 classifies the payload as Polyline, otherwise as Edges. Zero is a valid
Edges path. Malformed ASCII, negative values, and any bit outside `0x1F` remain
typed issues and publish no classification. Duplicate subclasses stay isolated
and all nine Core dialects have ASCII/Binary parity. Payload grammar,
cardinality, vertices/edges, handles, geometry, applicability, CRUD/write, and
completion remain open.

M14.4j selects the required Polyline boundary header only for M14.4i Polyline
paths. Groups 72 and 73 are unique signed Int16 booleans for bulge presence and
closure; group 93 is a unique nonnegative signed Int32 declared vertex count.
Edges remain explicitly NotPolyline. Invalid path flags, absent or duplicate
fields, malformed ASCII, and out-of-domain values remain typed. All nine Core
dialects have ASCII/Binary parity. Vertex grouping, count reconciliation,
payload geometry, handles, applicability, CRUD/write, and completion remain
open.

M14.4k groups recognized Polyline vertex fields conservatively: every group 10
starts a vertex, while following group 20 and optional group 42 occurrences
belong to that vertex until the next group 10. Recognized 20/42 fields before
the first anchor remain explicit orphans. Fixed X/Y/bulge cardinality and the
group-93 declared-versus-observed vertex relation remain source stable. Edges
and invalid headers publish no vertices. All nine dialects have ASCII/Binary
parity. Numeric vertex semantics, bulge defaults, geometry, applicability,
CRUD/write, and completion remain open.

M14.4l selects each grouped Polyline vertex component into a source-anchored
four-state numeric semantic. Unique X/Y/bulge values decode as exact finite
binary64; malformed ASCII and non-finite Binary payloads remain invalid with
raw provenance. Absent Y/bulge stays absent, and duplicates stay invalid
without selecting a member. Edges and invalid headers publish no numeric
entries, while all nine dialects have ASCII/Binary parity. Required-Y and
has-bulge relations, bulge defaults, OCS/WCS geometry, applicability,
CRUD/write, and completion remain open.

M14.4m resolves effective Polyline bulges against the unique group-72 header.
An absent group 42 receives the documented `+0.0` default without raw
provenance. When `has-bulge` is true, explicit and invalid M14.4l numerics pass
through with their source evidence; when false, any present group 42 becomes a
typed relation failure while the complete numeric directory remains retained.
Edges and invalid headers publish no bulge entries, and all nine dialects have
ASCII/Binary parity. Required-Y semantics, closed-path topology, OCS/WCS
geometry, applicability, CRUD/write, and completion remain open.

M14.3bs adds reset-to-default semantics under the same POINT thickness patch
identity. One unique explicit group `39` is deleted by exact source span and
must reparse as the documented zero value in the `Defaulted` state before the
inverse is released. An absent group is reported as `AlreadyImplicit` and does
not consume a session edit slot; duplicate groups remain unselectable. POINT
receipts now distinguish insertion, replacement, reset, and no-op default
retention. Paired fixtures span all nine ASCII/Binary Core dialects and prove
typed duplicate/conflict rejection, cancellation, semantic tamper rejection,
strict reparse, and byte-identical inverse restoration. Extrusion/angle
updates, mixed insert/update sessions, clone/delete, display, and POINT
`Complete` remain open.

M14.3bt adds atomic replacement of a complete explicit POINT extrusion tuple.
Unique groups `210/220/230` are encoded and replaced as three exact source
spans under one `Extrusion` patch identity. Missing or duplicate components,
zero requested direction, wrong family, non-finite values, duplicate admission,
resource limits, and cancellation queue nothing. Location, thickness, and
extrusion remain independent logical edits and can compose. Strict post-image
verification requires the exact requested tuple with all three semantic states
`Explicit` before releasing the byte-identical inverse. Paired fixtures span
all nine ASCII/Binary Core dialects and include typed malformed evidence and
tamper rejection. Absent/default extrusion insertion or reset, angle updates,
mixed insert/update sessions, clone/delete, display, and POINT `Complete`
remain open.

M14.3bu extends the POINT extrusion patch to the fully absent, documented-
default tuple. The requested nonzero direction is encoded as canonical groups
`210/220/230` and inserted in one zero-width patch after a unique thickness
group or, if thickness is absent, after the last unique location component.
Duplicate thickness and unusable fallback location evidence queue nothing.
ASCII insertion preserves LF, CRLF, or CR and Binary preserves its exact group
framing. The receipt reports `Inserted`; strict verification requires the exact
tuple with all components `Explicit`, and the inverse removes the inserted
sequence byte-for-byte. Paired fixtures cover all nine Core dialects plus CRLF
and malformed anchors. Partial explicit/default extrusion tuples, extrusion
reset, angle updates, mixed insert/update sessions, clone/delete, display, and
POINT `Complete` remain open.

M14.3bv completes all six partial absent/unique extrusion masks. Existing
components are replaced at their exact spans and consecutive missing runs are
inserted at canonical `210/220/230` gaps under one logical `Extrusion` edit.
Receipts report `Composite`; transactions contain the exact two or three
physical patches implied by the mask. Partial components must occur in
canonical source order, and reordered or duplicate evidence queues nothing.
ASCII replacements and insertions retain local LF, CRLF, or CR; Binary retains
exact group framing. Every partial mask passes all nine ASCII/Binary dialect
pairs with strict explicit-state verification and byte-identical inverse
restoration. Extrusion reset, angle updates, mixed entity insert/update
sessions, clone/delete, display, and POINT `Complete` remain open.

M14.3bw adds atomic reset of every complete or partial explicit POINT extrusion
tuple. Each unique present `210`, `220`, or `230` group is deleted at its exact
source span under one logical `Extrusion` edit; receipts report `Reset`. A fully
absent tuple reports `AlreadyImplicit`, queues no patch, and does not reserve
the extrusion patch identity. Duplicate component evidence fails typed before
mutation. Strict verification requires the documented `(0,0,1)` tuple with all
three semantic states `Defaulted`, and the inverse restores the complete source
bytes. All seven nonempty masks pass all nine ASCII/Binary dialect pairs. Angle
updates, mixed entity insert/update sessions, clone/delete, display, and POINT
`Complete` remain open.

M14.3bx adds atomic set/update of optional POINT UCS X-axis angle group `50`.
A unique explicit angle is replaced; an absent angle is inserted after the last
unique explicit extrusion component, otherwise after unique thickness or the
last of three required location components. Duplicate angle or ambiguous
predecessor evidence queues nothing. The distinct `UcsXAxisAngle` patch kind
composes with location, thickness, and extrusion edits. Local ASCII line
endings and Binary framing are preserved. Exact explicit-state verification and
byte-identical inverse restoration pass replacement plus all eight extrusion-
mask insertion states on every supported ASCII/Binary dialect. Angle reset,
mixed entity insert/update sessions, clone/delete, display, and POINT
`Complete` remain open.

M14.3by adds atomic reset of optional POINT UCS X-axis angle group `50`. One
unique explicit angle is deleted under the existing `UcsXAxisAngle` patch
identity; absent evidence reports `AlreadyImplicit` without queueing or
reserving that identity. Duplicate angle evidence fails typed, and queued set
or reset requests reject a second angle operation. Strict verification requires
both the documented zero value and `Defaulted` state, distinguishing reset from
an explicit zero. Every supported ASCII/Binary dialect passes strict reparse,
semantic verification, and byte-identical inverse restoration; CRLF deletion
is covered separately. Mixed entity insert/update sessions, clone/delete,
display, and POINT `Complete` remain open.

M14.3bz allows one unified session to mix existing-record common/POINT updates
with one or more POINT insertions in either API order. One final transaction
contains handle reservation, successor `$HANDSEED`, new records, common-field
patches, and POINT-family patches. Same-offset insertion fragments retain
existing-record updates before new records. Verification ordinals for existing
entities shift by the exact number of inserted records placed before their
source marker, including insertion in an earlier ENTITIES section. Both call
orders and ordinal shifts pass every supported ASCII/Binary dialect with strict
semantic verification and byte-identical inverse restoration. Clone/delete,
display, and POINT `Complete` remain open.

M14.3ca adds standalone whole-record deletion for canonical POINT entities
whose non-null handle identity is unique. Before queueing the exact raw-record
span deletion, the session resolves every pointer/owner occurrence and rejects
any uniquely resolved incoming reference from another record. Missing,
invalid, null, multiple, or document-ambiguous identities remain visible as
typed failures; wrong families, cancellation, and mixing with other session
operations also queue nothing. Strict post-image verification requires the
deleted handle to be absent before the executable inverse is released. Every
supported ASCII/Binary dialect passes exact deletion and byte-identical
restoration. Handleless deletion, mixed/multi-delete sessions, clone, display,
and POINT `Complete` remain open.

M14.3cb adds canonical semantic POINT cloning into a fresh reserved handle.
The source and requested placement must identify the same entity container,
and AC1012+ owner identity must match. Only the canonical group envelope
currently representable by `DxfPointDraft` is admitted, preventing silent loss
of unsupported common properties, XDATA, application groups, or opaque data.
The clone preserves exact layer/layout names, applicable lineweight, location,
and explicit-versus-defaulted thickness, extrusion, and angle state before
reusing verified insertion, `$HANDSEED`, strict reparse, and inverse
restoration. Pending source updates, invalid/partial semantics, placement or
owner drift, and unsupported groups queue nothing. Defaulted clones pass every
supported ASCII/Binary dialect; an all-explicit fixture proves complete POINT
payload retention. Cross-container/owner clone, broader common-property clone,
handleless/mixed deletion, display, and POINT `Complete` remain open.

M14.3cc admits standalone deletion of canonical POINT records with exactly
absent handle identity. A distinct handleless receipt preserves the stronger
handle-backed receipt contract. The verification plan stores the expected
post-image entity count and releases the executable inverse only after strict
reparse proves the one-entity reduction; exact transaction-byte verification
still proves the selected group-zero-delimited record was removed. Invalid,
null, multiple, and ambiguous identity remains fail-closed. Every supported
ASCII/Binary dialect proves exact deletion, neighboring LINE retention,
semantic verification, and byte-identical restoration. Mixed/multi-delete,
display, and POINT `Complete` remain open.

M14.3cd permits a bounded delete-only session to contain multiple distinct
canonical POINT records. Each admission independently enforces unique
non-null-or-absent identity and incoming-reference safety, and the exact
raw-record plans compose into one source-order transaction. Every handled
identity must disappear; handleless members require the exact final entity
count after all removals. Duplicate keys queue nothing and retain the existing
batch. Two handled records pass all supported ASCII/Binary dialects, and one
mixed handled/handleless batch proves combined semantic verification and exact
inverse restoration. Delete/update/insert mixing, display, and POINT `Complete`
remain open.

M14.3ce composes deletion with unrelated existing-record updates and new POINT
insertions in either API order. A source key cannot be both updated/cloned and
deleted, but independent keys share the resource-bounded session. Updated
record ordinals account for inserted records before their marker and deleted
records before their raw ordinal. Finalization combines update patches, handle
reservation, `$HANDSEED`, record insertion, and exact deletion spans; a
handleless expectation uses the final source-minus-deletes-plus-inserts entity
count. Delete/update and delete/insert pass every supported ASCII/Binary
dialect in both orders with strict semantics and exact inverse restoration.
Broader clone, display, and POINT `Complete` remain open.

M14.3cf preserves reviewed singleton scalar common properties when cloning a
canonical POINT. Explicit paper/model space (67), indexed color (62), linetype
scale (48), visibility (60), true color (420), transparency (440), and shadow
mode (284) flow through typed draft fields, canonical ASCII/Binary encoding,
strict reparse, semantic post-image checks, and exact inverse restoration.
Defaulted/absent values remain omitted and invalid, duplicate, or
version-inapplicable values queue nothing. The coverage matrix includes all
nine Core versions in both physical formats. Linetype/material/color-book/
plot-style references, proxy graphics, XDATA/application groups, extension
dictionaries, ownership graphs, display, and POINT `Complete` remain open.

M14.3cg adds exact same-document linetype preservation to canonical POINT
clone. Explicit group 6 must match one LTYPE table entry byte-for-byte; the
typed draft emits it in common-field order and strict post-image verification
requires the same semantic value. Missing, ambiguous, wrong-scope, and unknown
symbols fail without queueing. The test matrix covers all nine Core versions in
ASCII and Binary with exact inverse restoration. Material, plot-style,
color-book, graph common properties, display, and POINT `Complete` remain open.

M14.3ch preserves material (347) and plot-style (390) handles for AC1012+
canonical POINT clone only when each same-document target is unique and has the
reviewed OBJECTS marker: MATERIAL and ACDBPLACEHOLDER respectively. Canonical
encoding revalidates target kind and strict post-image semantics require both
explicit handles. Every applicable modern version passes ASCII/Binary clone
and inverse coverage. Color-book, graph common properties, display, and POINT
`Complete` remain open.

M14.3ci preserves the AC1012+ common color-book tuple (62/420/430) during
canonical POINT clone. Group 430 must contain exactly one non-edge `$`
separator, and both related color fields must be explicit and valid. Canonical
encoding and strict post-image verification retain the indexed color, true
color, and exact color-book bytes across every applicable ASCII/Binary dialect.
Malformed or incomplete tuples queue nothing. Graph common properties, display,
and POINT `Complete` remain open.

M14.3cj preserves matched AC1012+ proxy graphics (92/310) during canonical
POINT clone. ASCII hexadecimal sequences and Binary chunks become the same
bounded decoded-byte payload, canonical output rechunks that payload, and
strict post-image verification compares the exact bytes as well as the declared
size relation. Missing, mismatched, or malformed size/data relations queue
nothing. Graph-scoped handles, application groups/XDATA, display, and POINT
`Complete` remain open.

M14.3ck makes standalone POINT deletion fail closed when the selected raw
record contains any application-control group 102 or hard-owner group 360.
The typed rejection retains the exact group occurrence and code, and no edit is
queued. AC1012 through AC1032 ASCII/Binary fixtures cover reactor,
extension-dictionary, and unscoped hard-owner shapes. This is an orphan-
prevention boundary, not graph-aware cascade deletion or graph clone; XDATA,
display, and POINT `Complete` remain open.

M14.3cl adds `DxfEntityXDataDirectory` over every entity in the unified
directory. Exact group-1001 occurrences start separate application lists, and
the following group codes in the inclusive 1000..=1071 range retain their raw
groups and source order until the next 1001, a normal group, or the record
boundary. Duplicate application names are not merged; XDATA codes before a
1001 remain explicit orphans; a normal group marks the open list interrupted;
and XDATA-shaped codes inside group-102 application controls are excluded.
ASCII/Binary parity spans all nine Core dialects. APPID lookup, application-name
or group-1002 brace validation, typed value projection, 16-KiB policy, payload
meaning, clone/write, and POINT `Complete` remain open.

M14.3cm extends the exact closed-table scanner to APPID and adds
`DxfEntityXDataAppIdResolutionDirectory`. Every group-1001 application name is
hashed only to bound candidate selection, then compared byte-for-byte against
the exact group-2 APPID name span. Results retain missing, unique, or ambiguous
state and a unique source-backed target. Wrong-table records, case near-matches,
duplicate names, malformed table declarations, and tables without an exact
`ENDTAB` remain fail closed. All nine Core dialects have ASCII/Binary parity.
Application-name syntax, group-1002 braces, typed values, 16-KiB policy,
payload meaning, handle remap, clone/write, and POINT `Complete` remain open.

M14.3cn adds `DxfEntityXDataStructureDirectory`. Group-1001 payloads over the
documented 31-byte ceiling are typed invalid. Group 1002 accepts only exact
single-byte `{` and `}` controls; nested lists balance per application, while
invalid controls, premature closes, leftover opens, and interruption by a
normal entity group remain distinct source-anchored issues. All nine Core
dialects have ASCII/Binary parity. Full symbol-name character policy, typed
value projection, 16-KiB enforcement, payload meaning, handle remap,
clone/write, and POINT `Complete` remain open.

M14.3co adds `DxfEntityXDataTypedDirectory`, a one-to-one typed projection of
every retained application, value, and orphan occurrence. It preserves exact
group-1000 and group-1003 text spans, validates exact group-1002 controls and
the 255-byte string ceiling, validates and caller-buffer decodes group-1004
chunks up to 127 bytes, and exposes group-1005 handles, all documented
1010--1042 double roles, group-1070 signed 16-bit integers, and group-1071
signed 32-bit integers. Unsupported codes and malformed values remain typed
source evidence. All nine Core dialects have ASCII/Binary parity. Layer-name
resolution, point/vector tuple grouping and transforms, per-entity 16-KiB
enforcement, payload semantics, handle target resolution/remap, clone/write,
and POINT `Complete` remain open.

M14.3cp adds `DxfEntityXDataPointTupleDirectory`. Immediately adjacent members
with matching suffixes form maximal Point, WorldPosition, WorldDisplacement, or
WorldDirection candidates in documented X/Y/Z order. Every component retains
its compact link to the generic typed entry. Complete and partial axis sets are
explicit, invalid numeric members remain typed, and normal-group gaps,
application changes, entity changes, and source changes cannot be bridged. All
nine Core dialects have ASCII/Binary parity. Transformation application,
layer-name resolution, per-entity 16-KiB accounting, payload semantics, handle
target resolution/remap, clone/write, and POINT `Complete` remain open.

M14.3cq adds `DxfEntityXDataLayerResolutionDirectory`. Every retained
group-1003 value, including orphan evidence, resolves by exact source bytes
against group-2 names admitted only from completely closed LAYER tables. A
sorted SHA-256 index bounds candidate selection; byte-for-byte source-span
comparison remains authoritative and collision safe. Unique, missing, and
ambiguous targets are distinct. Near-case names, wrong table kinds, malformed
multi-name records, and unclosed tables fail closed. All nine Core dialects
have ASCII/Binary parity. Per-entity 16-KiB accounting, coordinate transforms,
payload semantics, handle target resolution/remap, clone/write, and POINT
`Complete` remain open.

M14.3cr adds `DxfEntityXDataCapacityDirectory` for every indexed entity,
including an exact zero-XDATA result. AutoCAD 2027 Core Console `xdsize` and
`xdroom` observations establish a 16,383-byte per-entity ceiling and the
logical-value formula: each nonempty application contributes three bytes;
strings contribute three plus twice the decoded Unicode-scalar count; list
controls contribute two; resolved layers three; binary chunks two plus decoded
length; handles nine; complete 3D tuples twenty-five; other doubles nine;
signed 16-bit integers three; and signed 32-bit integers five. Empty
applications contribute zero. Exact totals distinguish within-limit from
exceeded; orphan values, unresolved/ambiguous APPIDs or layers, invalid
structure/value/text, and partial point tuples instead publish an accounted
lower bound with compact typed issues. The exact 16,383/16,384 boundary, raw
UTF-8, CIF surrogate pairs, all logical value families, and all fail-closed
states have nine-dialect ASCII/Binary parity. Coordinate transforms, payload
semantics, group-1005 target resolution/remap, clone/write, and POINT
`Complete` remain open.

M14.3cs adds validated and composable translation, uniform-scale, axis-rotation,
and mirror-plane channels plus one source-bound transformed entry for every
generic XDATA 3D tuple. AutoCAD 2027 Core Console observations establish the
role split: group 1010 is invariant; group 1011 receives the complete position
affine transform; group 1012 receives scale, rotation, and mirror but no
translation; group 1013 receives rotation and mirror but neither translation
nor scale. Transform factories reject non-finite inputs, zero scale, zero
rotation axes, and zero mirror normals; composition overflow, partial tuples,
invalid component values, and derived overflow remain typed unavailable states.
All four roles and the composed oracle sequence have nine-dialect ASCII/Binary
parity. Application-specific payload semantics, group-1005 target
resolution/remap, clone/write, and POINT `Complete` remain open.

M14.3ct adds `DxfEntityXDataHandleResolutionDirectory` over every retained
generic XDATA group-1005 value. It composes typed XDATA scope with the existing
document-local handle reference/identity index, preserving invalid, null,
missing, ambiguous, and unique states. Only a unique result carries its exact
`DxfHandleIdentityMatch`, including target record and identity candidate;
application values and orphans remain independently addressable. Group-1005
payload inside group-102 application controls remains excluded by the XDATA
scope boundary even though generic handle evidence retains it. All states have
nine-dialect ASCII/Binary parity. Cross-document handle remap, application-
specific payload semantics, clone/write, and POINT `Complete` remain open.

M14.3cu adds `DxfEntityXDataHandleRemapDirectory` over M14.3ct source
resolution. Caller-supplied non-null source/destination candidates are copied,
sorted, and matched only to uniquely resolved source identities. Invalid,
null, missing, and ambiguous source states remain unusable; unique sources are
separately unmapped, mapped by exactly one candidate, or ambiguous when two or
more candidates share the source. Only the exactly-one case publishes a
destination handle. Entries retain source identity, source target evidence,
and compact resolution links; foreign ordinal lookups fail closed. All states
have nine-dialect ASCII/Binary parity. Destination-document identity
validation, application-specific payload semantics, clone/write, and POINT
`Complete` remain open.

M14.3cv adds `DxfEntityXDataHandleDestinationDirectory` across separate source
and destination documents. M14.3cu remap-unavailable states propagate exactly.
For an exactly mapped handle, the destination identity index classifies
missing, unique, or ambiguous target count; only unique resolution exposes the
exact destination `DxfHandleIdentityMatch`. Entry lookup binds both source and
destination identities, while destination target evidence is derived from the
owned identity directory rather than copied per entry. ASCII-to-ASCII,
ASCII-to-Binary, Binary-to-ASCII, and Binary-to-Binary pairs have parity across
all nine Core dialects. Application-specific payload semantics, replacement
encoding, clone/write, and POINT `Complete` remain open.

M14.3cw adds exact destination-dialect group-1005 replacement encoding over
M14.3cv. Only a unique destination identity is ready; every missing, ambiguous,
or remap-unavailable result is retained and publishes no replacement bytes. A
ready entry exposes one complete canonical group with uppercase handle spelling,
including the full 64-bit handle domain, ASCII line framing, Binary NUL framing,
and the AC1009 extended-data group-code escape. Entries and byte lookup bind
both source and destination identities, while debug output reports only byte
counts. All four source/destination format pairs pass all nine Core dialects.
Application-specific payload semantics, transaction composition, clone/write,
and POINT `Complete` remain open.

M14.3cx binds each M14.3cw ready group to its exact source group-1005 occurrence
and complete raw span through `DxfEntityXDataHandleReplacementPatch`. Patch
lookup revalidates the replacement entry, source identity, destination identity,
target handle, and source group before releasing bytes. Foreign patches and all
unavailable replacement states expose no byte slice. Source-span association
and foreign rejection pass all four format pairings across all nine Core
dialects. Application-specific payload semantics, transaction composition,
clone/write, and POINT `Complete` remain open.

M14.3cy adds entity-level all-or-nothing replacement readiness. Source-ordered
group-1005 entries are grouped by exact `DxfEntityRef`; a set is `Ready` only
when every member can publish an M14.3cx patch. Unavailable sets retain exact
member count, unavailable count, and first unavailable replacement ordinal.
Set membership and patch lookup bind both document identities, preventing a
foreign member or a partially remapped entity from entering later assembly.
Ready, mixed, and fully unavailable entities pass all four format pairings
across all nine Core dialects. Application-specific payload semantics,
transaction composition, clone/write, and POINT `Complete` remain open.

M14.3cz composes one complete M14.3cy ready set into a source-bound
`DxfTransactionPlan` only when source and destination format plus supported
`$ACADVER` state match exactly. Every member replaces its complete source group
with M14.3cw canonical bytes in source order, while inverse bytes retain the
exact original groups. Unavailable sets and format/dialect mismatch return typed
outcomes without patches. Same-format ASCII and Binary transactions plus all
cross-format rejections pass all nine Core dialects. The plan is a staging
primitive; destination clone/write verification, application-specific payload
semantics, and POINT `Complete` remain open.

M14.3da verifies a staged post-image byte-for-byte through the shared inverse
materializer, reparses every replaced occurrence as a handle, and requires its
exact M14.3cx destination target before releasing a receipt and executable
inverse. Receipts bind source, destination, post-image, and replacement count.
Cancellation and any byte tampering fail closed. Verification and exact inverse
restoration pass same-format ASCII/Binary across all nine Core dialects.
Destination clone/write integration, application-specific payload semantics,
and POINT `Complete` remain open.

M14.3db executes one ready replacement-set transaction through the existing
create-new writer, strictly reparses the written file in its exact ASCII/Binary
format, and reruns M14.3da byte and handle verification before releasing a
combined write/verification receipt and executable inverse. Exact source,
destination, set, post-image, patch-count, and replacement-count bindings are
preserved. Existing paths are never modified; foreign evidence, source
mismatch, cancellation, strict-reparse failure, and raw tampering leave no
partial destination. The verified write and byte-identical inverse pass all
nine same-format ASCII/Binary dialect pairs. The output remains a staged source
clone rather than an insertion into the separately parsed destination document;
cross-container clone, application-specific payload semantics, and POINT
`Complete` remain open.

M14.3dc adds `DxfEntityXDataAppIdDestinationDirectory`, binding the M14.3cm
source APPID resolution evidence to a separately opened destination symbol
directory. Missing or ambiguous source APPIDs propagate before destination
lookup. For a source-unique application, a sorted SHA-256 index narrows exact
destination APPID candidates and bounded byte comparison across both documents
remains authoritative. Destination missing, unique, and ambiguous states retain
dual-source identity and exact counts; only the unique state publishes an owned
destination APPID target. All four ASCII/Binary format pairings cover every
Core dialect, cross-dialect AC1009/AC1032 boundaries, long chunked names,
near-case and duplicate names, malformed, wrong-table, and unclosed evidence,
cancellation, foreign identity, bounds, and debug redaction. APPID syntax and
length, record creation/edit, group-1002 and payload semantics,
coordinate/layer/handle composition, XDATA encoding/insertion, destination
mutation, cross-container clone, and POINT `Complete` remain open.

M14.3dd adds `DxfEntityXDataLayerDestinationDirectory`, binding every retained
M14.3cq group-1003 source LAYER resolution to a separately opened destination
symbol directory. Source-missing and source-ambiguous states take precedence.
Only source-unique layer names enter a shared sorted SHA-256 destination index;
bounded byte comparison across both documents remains authoritative.
Destination missing, unique, and ambiguous states retain dual-source identity
and exact counts, and only a unique result publishes an owned destination LAYER
target. All four ASCII/Binary source-destination pairings cover every Core
dialect plus AC1009/AC1032 cross-dialect boundaries, long chunked names,
orphan and application occurrences, case-near and duplicate names, malformed,
wrong-table, multi-name, and unclosed evidence, cancellation, foreign identity,
bounds, and debug redaction. Layer-name syntax and length, record creation/edit,
application payload semantics, coordinate/handle composition, XDATA encoding
or insertion, destination mutation, cross-container clone, and POINT
`Complete` remain open.

M14.3de adds `DxfEntityXDataSymbolDestinationDirectory` as an exact per-
application composition of M14.3dc APPID and M14.3dd application-bound LAYER
destination evidence. A result is `Ready` only when the application APPID and
every group-1003 occurrence in that application resolve uniquely in the
independently parsed destination. Otherwise a compact source-ordered issue range
retains every exact APPID and LAYER blocker without disclosing symbol bytes.
Ready entries derive their owned destination APPID and exact unique LAYER
occurrences through the two owned directories. All four ASCII/Binary source-
destination pairings cover every Core dialect plus AC1009/AC1032 cross-dialect
boundaries, source/destination missing and ambiguity precedence, accumulated
APPID/LAYER failures, case-near rejection, malformed tables, dual-source
identity, cancellation, bounds, and debug redaction. Orphan group-1003 values
remain outside any application result and visible in the owned LAYER directory.
This symbol-only result does not claim structure, capacity, coordinate, handle,
payload, encoding, insertion, mutation, or cross-container clone readiness.

M14.3df adds `DxfEntityXDataApplicationDestinationDirectory`, composing each
M14.3de symbol result with the exact M14.3cn group-1002 list-structure result for
the same source application. `Ready` requires both unique destination symbols
and valid source list structure. `Unavailable` retains independent symbol and
structure issue counts, while the owned directories derive every exact blocker
without copying source bytes. Four ASCII/Binary source-destination pairings
cover every Core dialect plus AC1009/AC1032 cross-dialect boundaries, symbol-
only failure, structure-only failure, simultaneous symbol/structure failure,
complete list controls, unclosed lists, entity slicing, dual-source identity,
cancellation, bounds, and debug redaction. Orphan XDATA remains outside any
application result. Capacity, transformed coordinates, handle remaps, payload
semantics, encoding, insertion, destination mutation, cross-container clone,
and POINT `Complete` remain open.

M14.2m classifies modern embedded MTEXT column type, count, width, gutter,
automatic-height, flow-reversal, shared height, and source-order individual
heights into typed scalar domains. It rejects unsupported type codes, negative
counts/heights/gutters, nonpositive width, non-Boolean flags, duplicate
singletons, and invalid numerics without losing raw provenance.

M14.2n validates modern embedded fields as no-column, static, dynamic
automatic-height, or dynamic manual-height mode. It requires the documented
active dimensions, validates positive static/manual counts where applicable,
separates shared-height from per-column-height strategies, and checks repeated
height cardinality. A source-observed zero terminal manual height remains
usable and source-anchored. Legacy flat/R2007 XDATA unification, layout
geometry, edit, and write remain unclaimed.

M14.2o recognizes complete exact `ACAD` R2007-era MTEXT column-info XDATA
blocks. It retains typed field IDs and values for type, automatic height,
count, reversed flow, width, gutter, declared height count, and repeated
heights with source identity. AC1009 extended Binary XDATA codes and all later
dialects have ASCII/Binary parity. Semantic unification with the modern
embedded representation, layout geometry, edit, and write remain unclaimed.

M14.2p projects Embedded and `ACAD` XDATA evidence through the same typed
scalar and mode-relation implementation. Each result identifies its physical
source envelope; shared fields and source-order individual heights have
identical semantics and provenance across storage forms. R2007
dynamic-manual height arrays are usable.

M14.2q recognizes only a complete exact
`ACAD_MTEXT_DEFINED_HEIGHT_BEGIN`/`END` block containing selector 46 and one
1040 value after a complete same-record column-info envelope. Its value maps
to the shared-height scalar, so R2007 static, dynamic-automatic, and
dynamic-manual storage can use the common mode semantics. Inexact or
incomplete blocks publish no partial value. Linked-column-handle XDATA,
direct flat group-50 framing, geometry, edit, and write remain unclaimed.

M14.2r retains complete exact `ACAD_MTEXT_COLUMNS_BEGIN`/`END` evidence with
selector 47, its typed 1070 declared count, and the contiguous source-order
1005 handle slice. Each handle reuses the core's source-anchored 1005
soft-pointer parsing, including lexical failures and original spelling.
Inexact or interrupted envelopes publish no entry or partial handle slice.
Association with column-info, count relationships, target resolution, direct
flat group-50 framing, geometry, edit, and write remain unclaimed.

M14.2s associates a linked-column block only with the latest complete
column-info block that ends before it in the same MTEXT record. Its group-1005
handles reuse document-local generic resolution and distinguish invalid,
null, missing, ambiguous, unique-MTEXT, and unique-other-record targets while
retaining the unique target record. This is resolution evidence, not a claim
that the link graph or declared count is valid. Count relationships, direct
flat group-50 framing, geometry, edit, and write remain unclaimed.

M14.2t isolates direct MTEXT column evidence for groups 75, 76, 78, 79, 48,
and 49 before any exact group-101 embedded-object boundary. When such evidence
exists, every direct group 50 remains source ordered and explicitly
`RotationOrColumnHeight`; a group 50 alone does not create a column entry.
Scalar unification, group-50 disambiguation, geometry, edit, and write remain
unclaimed.

M14.2u adds flat/direct storage to the common column source enum and scalar
projector. Type, count, flow, auto-height, width, and gutter now share the
same validation and mode relations as Embedded and XDATA storage. Ambiguous
group 50 is excluded from height projection; dynamic-auto flat columns can be
usable without silently consuming rotation evidence. Group-50 height
disambiguation, geometry, edit, and write remain unclaimed.

M14.2v exposes a typed height-framing disposition for every unified column
source. Embedded/XDATA framing is unambiguous; flat sources distinguish no
height evidence from ambiguous direct group-50 sets while retaining the exact
count and first raw provenance. Static and dynamic-manual flat modes surface a
dedicated unsupported-ambiguity issue, while dynamic-auto remains usable
without consuming group 50. Direct group-50 disambiguation, geometry, edit,
and write remain unclaimed.

M14.2w constructs the effective MTEXT WCS X-axis direction from M14.2k's
source-order winner. Rotation becomes a cosine/sine/zero vector; group
11/21/31 retains its exact vector and also exposes a normalized unit vector.
Partial, invalid, zero-length, non-finite-length, absent, and ambiguous
orientation inputs remain distinct typed states. Placement, extrusion-basis
composition, text metrics, glyph geometry, edit, and write remain unclaimed.

M14.2x selects the TEXT placement anchor in OCS. Left/baseline layout selects
the required group-10/20/30 point; any nonzero horizontal or vertical
justification selects group 11/21/31. Missing optional components, missing
required components, invalid numerics, duplicates, and unsupported
justification codes remain distinct typed failures with exact provenance. OCS
to WCS transformation, style resolution, metrics, glyph geometry, edit, and
write remain unclaimed.

M14.2y transforms a usable selected TEXT OCS anchor into WCS using the
normalized extrusion and SeaCad's shared Autodesk arbitrary-axis basis.
Success retains the first/second selection, finite WCS point, normalized
normal, canonical positive zero, and complete lower-layer evidence. Invalid
anchors or extrusion components, non-finite Binary values, zero extrusion,
basis failure, and derived overflow remain typed. Text rotation, style
metrics, glyph geometry, edit, and write remain unclaimed.

M14.2z introduced shared text-symbol extrusion/OCS projection code and an
initial SHAPE placement projection. M14.2aa corrects its coordinate-system
boundary from Autodesk's normative SHAPE reference: groups 10/20/30 are
already WCS and therefore remain bit-exact instead of being multiplied by an
OCS basis. Extrusion is independently normalized into the WCS normal. Missing,
invalid, duplicate, non-finite, zero-extrusion, and basis failures remain
typed; a finite maximum-magnitude WCS point remains usable. SHAPE definition
resolution, rotation, style metrics, glyph geometry, edit, and write remain
unclaimed.

M14.2ab projects optional SHAPE group 50 rotation, in degrees, onto the
normalized extrusion plane and exposes finite orthonormal WCS x/y axes plus
the normal. Zero degrees preserves the arbitrary-axis basis, while the exact
WCS insertion remains unchanged. Invalid or duplicate rotation/extrusion
evidence, non-finite Binary values, zero extrusion, and basis failure remain
typed. SHAPE definition resolution, style metrics, glyph geometry, edit, and
write remain unclaimed.

M14.2ac projects TEXT group 50 rotation and the documented group 71 backward
and upside-down bits into WCS glyph axes on the normalized extrusion plane.
Unknown generation bits remain preserved, and the selected WCS placement plus
all lower evidence remains reachable. Invalid or duplicate
rotation/flags/extrusion, non-finite Binary values, zero extrusion, and basis
failure remain typed. Style metrics, oblique/width/height geometry, glyph
outlines, edit, and write remain unclaimed.

M14.2ad composes TOLERANCE's required WCS insertion and WCS x-axis direction
with its normalized extrusion normal. Both WCS vectors remain bit-exact; no
OCS transform, x-axis normalization, or invented y-axis is applied. Missing,
invalid, duplicate, non-finite, zero x-axis/extrusion, and basis failures
remain typed. Dimension-style resolution, tolerance-string interpretation,
glyph geometry, edit, and write remain unclaimed.

M14.2ae indexes exact named DIMSTYLE records only from completely closed exact
table envelopes and resolves TOLERANCE group-3 names by bounded byte-exact
source comparison. Missing, unique, ambiguous, and unusable outcomes remain
distinct; duplicate targets retain source order. Interrupted/unclosed or
wrong-case tables, malformed names, and application-group decoys are excluded.
DIMSTYLE field semantics, tolerance-string interpretation, glyph geometry,
edit, and write remain unclaimed.

M14.2af projects all 68 documented DIMSTYLE-specific field codes through one
reviewed registry. Exact text, binary64, signed-16-bit, and handle values retain
source order, duplicates, raw provenance, and typed lexical failure outside
application groups. Every admitted exact named DIMSTYLE record exposes 68
fixed absent/unique/multiple cards. This is typed field and cardinality
evidence only: no occurrence selection, defaults, normalization, domain
validation, handle target resolution, tolerance-string interpretation, glyph
geometry, edit, or write is claimed. AC1009 Binary parity keeps codes above
255 absent because its one-byte group-code header cannot represent them.

M14.2ag lazily selects any registered DIMSTYLE field only when exactly one
occurrence exists. Explicit, absent, invalid-number, invalid-handle, and
duplicate states retain exact field/raw provenance without inventing defaults.
Group-70 standard flags expose the documented externally-dependent, resolved,
and referenced bits while preserving unknown bits. Field-domain validation,
handle target resolution, tolerance-string interpretation, glyph geometry,
edit, and write remain unclaimed.

| Format | Version | Read | Preserve | Semantic | Edit/Write |
|---|---|---:|---:|---:|---:|
| DXF ASCII | AC1009-AC1032 | Raw framing + dialect/structure/text resolution + exact 15-token ANSI registry | Verified Verbatim only | Shared HEADER views + raw records + bidirectional owner evidence + BLOCK topology/semantics + DIMSTYLE name/field semantics + SPLINE numeric evidence + POINT/LINE, CIRCLE/ARC, ELLIPSE, RAY/XLINE, TEXT/MTEXT/SHAPE/TOLERANCE numeric/text-field/layout semantics + TEXT/SHAPE orientation and TEXT/SHAPE/TOLERANCE WCS placement + LWPOLYLINE OCS geometry + classic POLYLINE OCS/WCS segment geometry | Not implemented |
| DXF Binary | AC1009-AC1032 | Encoding-verified immutable raw snapshot + EOF envelope + section/group-zero index | Verified Verbatim only | Shared HEADER views + raw records + bidirectional owner evidence + BLOCK topology/semantics + DIMSTYLE name/field semantics + SPLINE numeric evidence + POINT/LINE, CIRCLE/ARC, ELLIPSE, RAY/XLINE, TEXT/MTEXT/SHAPE/TOLERANCE numeric/text-field/layout semantics + TEXT/SHAPE orientation and TEXT/SHAPE/TOLERANCE WCS placement + LWPOLYLINE OCS geometry + classic POLYLINE OCS/WCS segment geometry | Not implemented |
| DWG | Any | Out of scope | Out of scope | Out of scope | Out of scope |
| DGN V7/V8 | Any | Out of scope | Out of scope | Out of scope | Out of scope |

A cell changes only after its milestone closes with deterministic evidence.
M6.7 exercises the 184 numeric, 25 exact-text, and five handle slots in schema
order over all nine supported dialects and both physical formats. AC1009 parity
fixtures keep group codes above 255 absent in both formats because its one-byte
Binary group-code header cannot represent them. Absence in that fixture is
physical evidence, not a version-applicability rule. Defaults, enum meaning,
ranges, filesystem resolution, handle topology, and cross-variable semantics
remain unreviewed.
M7.1a classifies only the numeric role of codes `5`, `105`, `320..369`,
`390..399`, `480..481`, and `1005`. It does not scan records, parse arbitrary
group payloads, resolve a handle, validate a target, infer ownership, or expose
dictionary, reactor, XDATA container, or reference-graph semantics.
M7.1b reads at most 16 payload bytes to project a selected handle-valued raw
group. Missing occurrences, non-handle groups, exact values, and empty,
overlong, or invalid hexadecimal spelling remain distinct. The projection does
not establish a record boundary, object identity, target existence, or graph
edge, and it does not traverse dictionaries, reactors, or XDATA containers.
M7.2a indexes raw group-zero-delimited chunks only inside completely closed
`CLASSES`, `TABLES`, `BLOCKS`, `ENTITIES`, and `OBJECTS` sections. HEADER,
thumbnail, unknown sections, `SECTION`/`ENDSEC`/`EOF`, and partial content from
interrupted or unclosed recognized sections are not indexed as records. TABLE,
ENDTAB, BLOCK, ENDBLK, entity, and object markers remain exact uninterpreted
type spelling; no handle identity or topology is inferred.
M7.2b scans those raw record ranges for source-anchored group-code `5` and `105`
identity candidates. Each record reports absent, one parsed, one lexically
invalid, or multiple candidates. Only a single parsed candidate participates in
lookup; duplicate parsed values across records return an ambiguous match set.
Raw spelling remains authoritative, null is not rejected semantically, and no
record type, target existence, ownership, pointer edge, dictionary, reactor, or
XDATA meaning is inferred.
M7.3a indexes source-order occurrences whose numeric code classifies as a soft
or hard pointer or a soft or hard owner inside the complete M7.2a records.
Every entry retains its record and M7.1b raw handle evidence. Identity codes
`5`/`105` and arbitrary codes `320..329` are excluded. Code `1005` is retained
as a soft-pointer occurrence without claiming that surrounding groups form a
valid XDATA container. No target lookup, dangling-reference status, ownership
enforcement, purge behavior, dictionary/reactor context, INSERT/XREF mutation,
or graph is implemented.
M7.3b performs document-local lookup of those pointer/owner occurrences against
the M7.2b uniquely parsed identity index. Lexically invalid, null, missing,
unique, and ambiguous targets remain distinct. Records with invalid or multiple
identity candidates cannot become targets; duplicate identities are returned as
an ambiguous shared match slice. Resolution does not validate whether a
particular record type may use the code, enforce one-owner or purge rules,
traverse a graph, interpret containers, or perform clone/INSERT/XREF rewriting.
M7.4a retains only soft-owner and hard-owner occurrences in a source-order
ownership-evidence directory. All five M7.3b resolution states stay visible;
only unique resolutions enter a compact incoming-link index grouped by target
record, so zero, one, or multiple incoming ownership-class links can be queried
without copying target arrays. This does not infer a complete owner from
context-specific `330` pointers, validate record-type legality, enforce the
one-owner rule, apply hard/soft purge behavior, diagnose cycles, or interpret
dictionaries and other containers.
M7.4b adds one target entry per raw record with a compact incoming-link range
and typed zero, one, or multiple uniquely resolved ownership-class cardinality.
The target entries cover records without identities and records with invalid or
duplicate identities as well as ordinary targets. Cardinality remains a view of
incoming `350..369` evidence only; it is not a legal-owner or one-owner
conformance decision and does not incorporate context-specific `330` owner
pointers.
M7.4c adds source-order evidence for every group-code `102` application control
inside complete raw records. Exact `{ACAD_REACTORS` and
`{ACAD_XDICTIONARY` starts are distinguished from other starts, closing braces,
and invalid controls. Group ranges are record-local and retain closed,
interrupted, or unclosed state. This is lexical container evidence only; handle
occurrences inside the ranges retain their existing M7.3/M7.4 classifications
until a later context join.
M7.4d performs that context join for every M7.3b pointer/owner resolution.
Each source-order entry preserves its numeric reference class and target state,
plus outside, reactor, extension-dictionary, or other application-group context
and the exact optional M7.4c group entry. This does not yet reinterpret a
context/class combination as a semantic reactor, extension dictionary, or
ordinary owner relation.
M7.4e adds a separate conservative role entry for every M7.4d contextual
reference. Exact closed `330` soft pointers inside `{ACAD_REACTORS`, exact
closed `360` hard owners inside `{ACAD_XDICTIONARY`, and exact outside-group
`330` soft pointers receive candidate roles only in complete `TABLES`,
`BLOCKS`, `ENTITIES`, or `OBJECTS` records. All other shapes remain generic
pointer or ownership-class evidence; interrupted and unclosed groups never
receive specialized candidates. Invalid, null, missing, unique, and ambiguous
target-resolution states remain independently accessible. Candidate roles do
not validate record-type legality, target existence, authoritative ownership,
one-owner conformance, dictionary membership, reactor payloads, lifecycle,
purge behavior, cycles, edits, or writes.
M7.4f filters every M7.4e `CommonOwnerPointerCandidate` into a source-order
candidate directory and creates one compact entry for every raw record. Each
record reports `NoCandidate`, `UniqueCandidate`, or `MultipleCandidates` and a
half-open candidate slice. Invalid, null, missing, unique, and ambiguous target
states all remain present, and the exact identity-match slice is available per
candidate. This cardinality does not compare the candidate target with the
source records of incoming ownership-class links, select an authoritative
owner, establish one-owner conformance, validate a record type, or implement
graph/lifecycle/edit/write behavior.
M7.4g creates one comparison entry per raw record from the M7.4f common-owner
card and M7.4b incoming ownership card. Only exactly one common-owner candidate
with a unique target and exactly one incoming ownership-class link are
comparable. The candidate target record ordinal is compared with the source
record ordinal of the incoming link, yielding `Matched` or `Conflicting`;
every other combination remains `NotComparable`. Both original cards, compact
evidence ordinals, target match, incoming link, and incoming source evidence
remain accessible. The comparison does not select an authoritative owner,
declare either direction valid, apply the one-owner rule, validate record or
target types, or implement graph/lifecycle/edit/write behavior.
M8.1a recognizes exact uppercase `POINT` and `LINE` markers only in complete
`BLOCKS` or `ENTITIES` record ranges. It indexes documented WCS location/start,
WCS endpoint, and optional extrusion components by numeric group code without
depending on group order. Source order, duplicates, empty slices, invalid ASCII
numbers, and exact binary64 bits remain visible. It does not choose canonical
components, apply omitted defaults, validate entity completeness, normalize an
extrusion vector, transform coordinate systems, expose thickness or POINT angle,
or claim complete POINT/LINE geometry support.
M8.1b adds a stable card for every documented M8.1a role applicable to each
recognized record: six cards for `POINT` and nine for `LINE`. Card members are
compact ordinals back into the source-order occurrence directory, so all
duplicates and lexical failures remain inspectable without copying component
values. `Absent`, `Unique`, and `Multiple` describe occurrence cardinality only;
they do not establish validity, completeness, a canonical component, an
extrusion default, or an assembled/transformed geometry value.
M8.1c lazily maps unique valid WCS location/start/endpoint cards to exact
component values with field and raw provenance. Missing required components,
invalid ASCII numbers, and duplicate occurrences become typed invalid states.
Absent extrusion X/Y/Z cards independently default to `0`, `0`, and `1` as
documented for POINT and LINE; explicit invalid or duplicate extrusion evidence
is never hidden by a default. Triple helpers return a value only when all three
components are explicit or reviewed defaults. This is component semantics, not
coordinate transformation, extrusion normalization, subclass/version
validation, editing, rendering, or complete geometry support.
M8.2a recognizes exact uppercase `CIRCLE` and `ARC` markers only in complete
`BLOCKS` or `ENTITIES` record ranges and keeps their OCS contract separate from
the POINT/LINE WCS directory. It indexes center `10/20/30`, radius `40`, ARC
start/end angle `50/51`, and optional extrusion `210/220/230` occurrences by
numeric group code. Source order, duplicates, empty slices, invalid ASCII
numbers, exact binary64 bits, and raw spans remain visible. Thickness `39`,
cardinality, defaults, required-field or positive-radius validation, angle
normalization, sweep interpretation, OCS transformation, and assembled circular
geometry remain deferred.
M8.2b adds seven stable cards per CIRCLE for OCS center X/Y/Z, radius, and
extrusion X/Y/Z, plus nine per ARC by inserting start/end angle before
extrusion. Compact member ordinals point back into the retained M8.2a evidence
directory. `Absent`, `Unique`, and `Multiple` report occurrence count only;
invalid ASCII values remain card members and no value is copied or selected.
The cards do not establish required fields, valid geometry, defaults, angle or
radius constraints, canonical values, coordinate transformation, or assembled
circular semantics.
M8.2c lazily maps unique valid CIRCLE/ARC OCS center and radius cards to exact
semantic values; ARC additionally exposes start/end values explicitly named as
DXF-file degrees. Missing, invalid ASCII, and duplicate required values remain
typed invalid with available raw provenance. Absent extrusion X/Y/Z components
independently default to `0`, `0`, and `1`, while present invalid or duplicate
evidence never defaults. Tuple/scalar helpers return values only from explicit
or reviewed-default states. Negative radius and out-of-range angle values remain
exact rather than silently validated or normalized. This is component semantics,
not radius conformance, ARC sweep interpretation, OCS transformation, or
assembled circular geometry.
M8.3a recognizes exact uppercase `ELLIPSE` markers only in complete `BLOCKS` or
`ENTITIES` record ranges. It indexes WCS center `10/20/30`, the WCS major-axis
endpoint vector relative to center `11/21/31`, minor-to-major axis ratio `40`,
start/end parameters `41/42`, and optional extrusion `210/220/230` by numeric
group code. Source order, duplicates, empty slices, invalid ASCII numbers,
exact binary64 bits, and raw spans remain visible. Parameters remain parameters
rather than degree angles. Cardinality, defaults, required-field, ratio or
parameter-range validation, subclass/version applicability, ellipse assembly,
coordinate transformation, thickness, editing, and rendering remain deferred.
M8.3b adds twelve stable cards per ELLIPSE for WCS center X/Y/Z, relative
major-axis endpoint X/Y/Z, minor-to-major axis ratio, start/end parameters, and
extrusion X/Y/Z. Compact member ordinals point back into the retained M8.3a
evidence directory. `Absent`, `Unique`, and `Multiple` report occurrence count
only; invalid ASCII values remain card members and no value is copied or
selected. The cards do not establish required fields, valid geometry, defaults,
ratio or parameter constraints, canonical values, coordinate transformation,
or assembled ellipse semantics.
M8.3c lazily maps unique valid ELLIPSE WCS center, relative major-axis endpoint,
minor-to-major axis ratio, and start/end parameter cards to exact semantic
values. Missing, invalid ASCII, and duplicate required values remain typed
invalid with available raw provenance. Absent extrusion X/Y/Z independently
defaults to `0`, `0`, and `1`, while present invalid or duplicate evidence never
defaults. Tuple/scalar helpers return values only from explicit or
reviewed-default states. Negative ratio and out-of-range parameter values remain
exact rather than silently validated or normalized. This is component
semantics, not ratio/parameter conformance, endpoint or sweep inference,
coordinate transformation, or assembled ellipse geometry.
M8.4a recognizes exact uppercase `RAY` and `XLINE` markers only in complete
`BLOCKS` or `ENTITIES` record ranges. It indexes the RAY start/XLINE first point
`10/20/30` and unit direction vector `11/21/31`, all documented in WCS, by
numeric group code. Source order, entity kind, duplicates, empty slices, invalid
ASCII numbers, exact binary64 bits, and raw spans remain visible. Cardinality,
required-field validation, unit-vector validation or normalization, direction
or extent interpretation, subclass/version applicability, coordinate
transformation, editing, and rendering remain deferred.
M8.4b adds six stable cards per RAY/XLINE for WCS start/first-point X/Y/Z and
unit-direction X/Y/Z. Compact member ordinals point back into the retained
M8.4a evidence directory. `Absent`, `Unique`, and `Multiple` report occurrence
count only; invalid ASCII values remain card members and no value is copied or
selected. The cards do not establish required fields, valid unit vectors,
direction or extent semantics, canonical values, coordinate transformation, or
assembled infinite-line geometry.
M8.4c lazily maps unique valid RAY/XLINE WCS start/first-point and
unit-direction cards to exact semantic values. Missing, invalid ASCII, and
duplicate required components remain typed invalid with available raw
provenance. Tuple helpers succeed only when all three components are explicit.
Non-unit and zero direction triples remain exact rather than silently rejected
or normalized. This is component semantics, not vector conformance, direction
or extent inference, coordinate transformation, or assembled infinite-line
geometry.
M9.1a recognizes exact uppercase `LWPOLYLINE` markers only in complete
`BLOCKS` or `ENTITIES` sections and indexes group codes `38`, `39`, `43`,
`10`, `20`, `40`, `41`, `42`, `210`, `220`, and `230` by documented
floating-point role. Source order, duplicates, empty slices, invalid ASCII
numbers, exact binary64 bits, raw spans, and the containing raw record remain
visible. Group codes `90`, `70`, and `91` remain intact in raw evidence but
their integer meaning is not projected by this checkpoint. Vertex grouping,
defaults, count/order/cardinality validation, width precedence, bulge
interpretation, version applicability, OCS transformation, editing, and
assembled polyline geometry remain deferred.
M9.1b separately indexes every `90`, `70`, and `91` occurrence in recognized
LWPOLYLINE records as vertex count, flags, or vertex identifier. Group `70`
retains its exact signed 16-bit domain; groups `90/91` retain exact signed
32-bit domains. ASCII syntax/range failures, Binary values, source order,
duplicates, empty slices, raw spans, and containing records stay explicit.
This is integer wire evidence, not canonical-value selection, flag-bit
interpretation, non-negative-domain validation, vertex association, declared-
versus-observed count reconciliation, version applicability, or assembled
polyline geometry.
M9.1c starts a conservative vertex at every retained LWPOLYLINE OCS-X group
`10`. OCS-Y `20`, start/end width `40/41`, bulge `42`, and identifier `91`
occurrences following that anchor belong to its evidence slice until the next
group `10`. Six fixed cards preserve absent, unique, or multiple cardinality
independently from lexical validity. Vertex-scoped groups before the first
anchor remain source-ordered orphan members; entity-level floating and integer
roles remain outside the vertex grouping. This is deterministic grouping
evidence, not default application, canonical selection, declared-count
reconciliation, flag/bulge interpretation, identifier uniqueness, version
applicability, OCS transformation, or assembled segment geometry.
M9.1d lazily maps every grouped LWPOLYLINE vertex card set into typed semantic
states. Unique valid OCS X/Y values are explicit; missing Y, invalid ASCII, and
duplicate required coordinates remain invalid with available raw provenance.
Absent local start/end width and bulge fields receive documented zero defaults;
present invalid or duplicate values remain invalid. Identifier is explicit,
absent, or invalid without a default. These are local field semantics: a zero
default for absent `40/41` does not select an effective width when record-level
constant width `43` exists. Width precedence, range validation, declared-count
reconciliation, flags, bulge geometry, version applicability, OCS
transformation, closure, and segment assembly remain deferred.
M9.1e adds eight fixed cards per recognized LWPOLYLINE record for vertex count
`90`, flags `70`, OCS elevation `38`, thickness `39`, constant width `43`, and
extrusion X/Y/Z `210/220/230`. Compact members resolve to exact M9.1a/M9.1b
floating or signed-integer evidence. Vertex-scoped `10/20/40/41/42/91` values
cannot enter these cards. `Absent`, `Unique`, and `Multiple` describe
occurrence cardinality independently from lexical validity; the cards do not
select canonical values, apply defaults, interpret flags or widths, reconcile
declared and observed counts, validate version applicability, transform OCS,
or assemble geometry.
M9.1f lazily maps the eight record cards into typed semantic states. Vertex
count `90` is required; flags `70`, elevation `38`, thickness `39`, constant
width `43`, and extrusion `210/220/230` receive only their documented defaults.
Closed bit `1` and Plinegen bit `128` have direct helpers while all signed flag
bits remain exact. The usable declared count is compared with the number of
retained group-10 anchors, preserving matched, mismatched, and not-comparable
states. Explicit constant-width and per-vertex-width occurrence shapes remain
neutral as none, constant only, variable only, or both. This does not validate
count/width domains, select width precedence, publish effective segment widths,
transform OCS, infer closure geometry, or assemble segments.
M9.1g builds guaranteed consecutive segments between adjacent retained
group-10 vertices. A unique usable Closed flag adds the last-to-first segment;
absent flags default open, while invalid or duplicate flags keep closure
indeterminate and do not add it. This yields `n-1` segments for open records and
`n` for closed nonempty records, including a self-closing single-vertex case.
Each lazy segment view exposes its start/end grouped vertex semantics, the
start vertex's local width pair, and its bulge. Usable signed zero bulge is
straight, usable nonzero bulge is an exact arc factor, and invalid vertex
semantics remain indeterminate. This does not choose effective width, compute
arc center/radius/sweep, transform OCS to WCS, validate count agreement or
geometry, edit/write, or render.
M9.1h maps a usable zero bulge to a straight segment retaining the exact OCS
endpoint values. A usable nonzero bulge derives a finite circular OCS center,
positive radius, and signed included-angle sweep; positive/negative bulges
retain counterclockwise/clockwise orientation. Missing or invalid endpoint or
bulge semantics, a zero chord with nonzero bulge, and non-finite intermediate
arithmetic fail typed without publishing NaN or infinity. These derived
binary64 values are not raw evidence or guaranteed cross-platform canonical
bits. This does not select effective width, transform OCS to WCS, validate
geometric tolerances or version applicability, edit/write, or render.
M9.2a recognizes exact uppercase classic `POLYLINE`, `VERTEX`, and `SEQEND`
record markers only in complete `BLOCKS` or `ENTITIES` sections. Zero or more
consecutive VERTEX records belong to the preceding recognized POLYLINE until
SEQEND closes the sequence, the first unexpected record interrupts it, or the
containing section ends with it unclosed. Closed and interrupted entries retain
their exact boundary record; orphan VERTEX/SEQEND records and non-exact marker
spellings are not guessed into a sequence. Group `66` is deliberately ignored.
This is record topology only, not value/cardinality/default semantics, 2D/3D/
mesh/polyface classification, vertex-face resolution, coordinate transforms,
geometry, editing, writing, or rendering.
M9.2b retains all numeric groups documented on each recognized classic
POLYLINE record: dummy/elevation `10/20/30`, thickness `39`, default widths
`40/41`, obsolete entities-follow `66`, flags `70`, mesh counts/densities/type
`71`-`75`, and extrusion `210/220/230`. The nine double roles and seven signed
16-bit roles remain distinct, source ordered, and source anchored; duplicates,
ASCII syntax/range failures, exact Binary values, and empty slices remain
visible for closed, interrupted, and unclosed sequences. Numeric groups inside
the following VERTEX records are excluded. This does not choose a canonical
value, apply defaults, require dummy zero, interpret group `66`, flags, or
surface type, validate domains, classify polyline families, decode VERTEX
payloads, transform coordinates, assemble geometry, edit/write, or render.
M9.2c retains every documented numeric group on each VERTEX record belonging
to an M9.2a classic POLYLINE sequence: location `10/20/30`, start/end width
`40/41`, bulge `42`, curve-fit tangent direction `50`, flags `70`, polyface
indices `71`-`74`, and vertex identifier `91`. Seven double, five signed-16-
bit, and one signed-32-bit roles remain distinct, source ordered, and source
anchored. Entries retain their parent POLYLINE and sequence-local ordinal;
duplicates, ASCII syntax/range failures, exact Binary values, empty slices,
and all three sequence states remain visible. POLYLINE, other VERTEX, and
SEQEND payloads cannot cross record boundaries. This does not select values,
apply defaults, interpret flags, bulge, tangent, or signed indices, classify
2D/3D/mesh/polyface vertices, validate domains, resolve polyface faces,
transform coordinates, assemble geometry, edit/write, or render.
M9.2d adds thirteen fixed cards to every M9.2c VERTEX entry in stable role
order. Cards report `Absent`, `Unique`, or `Multiple` and retain compact member
references to every source-order M9.2c occurrence. Numeric parsing success or
failure does not change cardinality; an empty VERTEX has thirteen absent cards,
and the contract remains available for closed, interrupted, and unclosed
sequences. This does not select canonical values, apply defaults, interpret
fields, classify vertex families, validate domains, resolve faces, transform
coordinates, assemble geometry, edit/write, or render.
M9.2e lazily maps the seven double-role cards on every retained VERTEX.
Location X/Y/Z are required components; absent start/end width and bulge use
only their documented zero defaults; absent curve-fit tangent direction stays
absent. Unique valid values remain exact, while missing required values,
invalid ASCII, and duplicates remain typed invalid with raw provenance when
available. This does not decide OCS versus WCS, require a tangent from flag bit
`2`, interpret integer fields, classify vertex families, validate ranges,
resolve faces, transform coordinates, assemble geometry, edit/write, or
render.
M9.2f adds sixteen fixed cards to every M9.2b POLYLINE record entry in stable
role order. Cards report `Absent`, `Unique`, or `Multiple` and retain compact
member references to every source-order M9.2b occurrence. Numeric parsing
success or failure does not change cardinality; an empty POLYLINE has sixteen
absent cards, and VERTEX values cannot enter record cards. The contract remains
available for closed, interrupted, and unclosed sequences. This does not select
values, apply defaults, interpret fields, classify polyline families, validate
domains, transform coordinates, assemble geometry, edit/write, or render.
M9.2g lazily maps fifteen meaningful POLYLINE record roles. Dummy X/Y and
elevation are required; absent thickness, default widths, flags, mesh
counts/densities/type, and extrusion receive only their documented defaults.
Unique values remain exact, and missing required values, invalid ASCII, or
duplicates remain typed invalid with raw provenance when available. Eight
helpers independently expose the documented flag bits. Obsolete group `66`
remains accessible through record cards/evidence but is ignored by semantic
policy. This does not enforce dummy zero, validate widths/counts/densities/
surface type or extrusion, reconcile contradictory flags, classify 2D/3D/
mesh/polyface families, apply mesh metadata to vertices, resolve faces,
transform coordinates, assemble geometry, edit/write, or render.
M9.2h lazily maps flags `70`, polyface indices `71`-`74`, and identifier `91`
on every retained VERTEX. Absent fields remain absent; unique signed values
stay exact; invalid ASCII and duplicates remain typed invalid with raw
provenance when available. Seven helpers expose only meaningful flag bits;
unused bit `4` has no helper. This does not require integer fields, interpret
negative-index edge visibility or zero termination, classify vertex families,
validate indices, resolve faces, apply parent mesh metadata, transform
coordinates, assemble geometry, edit/write, or render.
M9.2i classifies usable, non-conflicting parent flags as 2D, 3D, polygon mesh,
or polyface mesh and VERTEX flags as 2D, 3D, polygon mesh, polyface coordinate,
or polyface face. Parent/vertex comparison retains `Matched`, `Mismatched`, or
`NotComparable`; missing/invalid/duplicate flags remain unavailable and
multiple family bits remain conflicting. This does not enforce consistency,
interpret indices, validate applicability, resolve faces, transform
coordinates, assemble geometry, edit/write, or render.
M9.2j creates consecutive and flag-proven closing topology only for complete
`SEQEND`-terminated classic 2D/3D sequences whose VERTEX family evidence all
matches the parent. Segments retain exact endpoint entries and ordinals.
Incomplete sequences, mesh/polyface families, indeterminate parents, and
inconsistent vertices emit no segments and retain typed record state. This
does not project coordinates/effective widths, derive bulge geometry, transform
OCS/WCS, build meshes, resolve faces, edit/write, or render.
M9.2k lazily binds every proven segment to parent and endpoint semantic values.
It preserves the 2D OCS versus 3D WCS boundary and keeps start-vertex local
widths separate from parent default widths. Invalid endpoint, width, bulge, or
tangent fields remain independently unavailable. This does not choose effective
widths, derive arc geometry, transform coordinates, tessellate, edit/write, or
render.
M9.2l derives classic 2D straight/circular segment geometry in OCS from VERTEX
X/Y, parent elevation, and start-vertex bulge; stored 2D VERTEX Z is retained
by earlier evidence but ignored geometrically as Autodesk documents. Classic
3D segments retain exact WCS endpoints and accept only straight geometry.
Unavailable endpoints/elevation/bulge, nonzero 3D bulge, degenerate arc
chords, and non-finite derivation fail typed. This does not transform OCS to
WCS, validate extrusion, select effective widths, tessellate, edit/write, or
render.
M9.2m transforms usable classic 2D segment geometry from OCS to WCS with the
normalized extrusion direction and Autodesk arbitrary-axis algorithm. It
retains transformed WCS endpoints/centers, normalized normal, radius, signed
sweep, and bulge; classic 3D WCS lines pass through exactly without consulting
extrusion. Source geometry, unavailable/zero-length extrusion, and non-finite
transform failures remain typed. This does not claim external-engine
conformance, effective widths, tessellation, edit/write, or rendering.
M9.2n selects each classic 2D segment's effective start/end width independently.
Explicit VERTEX `40/41` wins even when zero; an omitted VERTEX component uses
only the corresponding parent POLYLINE default. Origin and typed vertex/parent
failure remain visible, while classic 3D widths are explicitly unsupported.
This does not validate width sign/range, construct wide geometry or joins,
tessellate, edit/write, or render.
M9.2o maps complete family-consistent classic polygon meshes into row-major
quadrilateral cells when positive M/N counts exactly match the retained VERTEX
count. Cells retain named grid-corner evidence and independent M/N wrap state.
Incomplete sequences, family inconsistency, unusable counts, or count mismatch
emit no cells with typed record state. This does not assign winding/normals,
project coordinates, interpret smoothing, edit/write, or render.
M9.2p partitions complete family-consistent classic polyface meshes into exact
coordinate (`128|64`) and face-definition (`128`) VERTEX ranges. Parent groups
`71/72` remain reported coordinate/face counts beside independent observed
counts; mismatch does not reject the partition, and coordinates after faces
are retained with `Odd` ordering evidence. Incomplete sequences and family
failures emit typed zero-member states. This does not interpret face indices,
resolve edge visibility, assemble face geometry, edit/write, or render.
M9.2q resolves usable groups `71`-`74` on each retained polyface face against
record-local coordinate order, including coordinates after oddly ordered
faces. The first zero or absent slot terminates the face; the sign preserves
visibility for the edge beginning at that corner. Invalid, post-terminator,
overflowing, and out-of-range indices emit typed zero-corner states. This does
not validate degeneracy/winding, assemble coordinate tuples, triangulate,
edit/write, or render.
M9.2r assembles each resolved polyface corner's referenced coordinate VERTEX
`10/20/30` values into an exact WCS point while retaining signed-index and edge-
visibility evidence. Face-resolution failures or unavailable coordinate
components emit typed zero-point face states. Odd ordering remains supported.
This does not consult irrelevant face-record locations, derive edges, validate
degeneracy/winding/planarity/manifoldness, calculate normals, triangulate,
edit/write, or render.
M9.2s assembles the four named VERTEX `10/20/30` coordinates of every M9.2o
polygon-mesh cell into exact WCS corner tuples. Unusable coordinate components
emit a typed state naming the first failed corner and all component states;
topology/wrap evidence remains attached. This does not assign winding, derive
edges, validate degeneracy/planarity, calculate normals, interpret smoothing,
triangulate, edit/write, or render.
M9.2t retains polygon-mesh smooth-surface M/N densities `73/74` in their exact
signed semantic domain and classifies `75` as none, quadratic B-spline, cubic
B-spline, or Bezier. Invalid metadata, unknown type codes, and topology failure
remain typed; no undocumented density range is imposed. This does not fit new
vertices/cells, change topology or WCS corners, validate continuity, tessellate,
edit/write, or render.
M10.1a recognizes exact uppercase `BLOCK` and `ENDBLK` records only in complete
`BLOCKS` sections. It retains every intervening group-zero record as an exact
source-order member slice. An exact `ENDBLK` closes a definition, another exact
`BLOCK` interrupts the preceding definition and starts a new one, and section
end leaves a definition unclosed. Orphan boundaries, non-exact spelling, other
sections, and partial `BLOCKS` sections publish no false definition topology.
This does not decode BLOCK values or member entities, resolve INSERT/xref
references, apply transforms, edit/write, or render.
M10.1b retains primary and secondary BLOCK names `2/3`, flags `70`, base-point
components `10/20/30`, xref path `1`, and optional description `4` in source
order. Text values retain exact source spans and document encoding for
caller-buffer decoding without replacement; numeric values retain exact
double/i16 domains or typed ASCII lexical failure. Duplicates, empty text, and
closed/interrupted/unclosed definitions stay visible. Group-102 application
payloads, member-entity values, and ENDBLK values are excluded. This does not
select occurrences, reconcile names, interpret flags, assemble points, resolve
xrefs, bind INSERT, transform geometry, edit/write, or render.
M10.1c adds eight fixed cards to every M10.1b BLOCK record in documented-role
order. Each card reports absent, unique, or multiple cardinality and retains
compact references to every exact source-order value occurrence. Text/numeric
validity does not change cardinality; empty records and all M10.1a definition
states keep the same eight-card shape. This does not select or decode canonical
values, reconcile names, apply defaults, interpret flags, assemble points,
resolve xrefs, bind INSERT, transform geometry, edit/write, or render.
M10.1d lazily projects required primary/secondary names, flags, and base-point
X/Y/Z plus optional xref path and description. Missing required fields,
duplicates, and invalid ASCII numbers fail typed; optional omission remains
absent and no undocumented default is invented. Seven helpers expose the
documented flag bits independently, while a base-point tuple requires all three
usable components. This does not enforce path presence from xref flags,
reconcile names, validate empty text/finiteness, resolve xrefs, bind INSERT,
transform member geometry, edit/write, or render.
M10.1e compares usable primary and secondary BLOCK names as exact same-document
raw bytes. Equal bytes match, unequal bytes conflict, and unusable name
semantics remain not comparable. Fixed 4-KiB comparison chunks bound memory
without decoding, case folding, normalization, or proportional allocation.
Empty names and every definition state remain visible. This does not choose a
canonical name, reject conflicts, build a lookup index, resolve INSERT/xrefs,
edit/write, or render.
M10.1f indexes only M10.1e matched BLOCK names. Bounded SHA-256 source-span
digests group candidates, then fixed 4-KiB source reads recheck exact bytes so a
digest collision cannot create a false match. Exact lookup preserves duplicate
records and reports missing, unique, or ambiguous; it does not decode,
case-fold, normalize, or retain name-sized buffers. Empty names and every
definition state remain indexable. Conflicting and not-comparable records stay
visible through retained consistency evidence but are not targets. This does
not validate a legal block namespace, choose among duplicates, resolve INSERT
or xrefs, edit/write, or render.
M10.1g discovers uppercase byte-exact INSERT records in completely indexed
BLOCKS and ENTITIES sections. It retains block name `2`, insertion point
`10/20/30`, scale `41/42/43`, rotation `50`, column/row counts `70/71`,
column/row spacing `44/45`, attributes-follow `66`, and extrusion
`210/220/230` in source order. Duplicates and invalid ASCII numbers remain
visible, and group `102` application content is excluded. This does not apply
defaults, select duplicates, assemble typed semantics, resolve block names,
follow ATTRIB/SEQEND, transform geometry, edit/write, or render.
M10.1h publishes 16 fixed cards per M10.1g INSERT record in documented role
order. Each card reports absent, unique, or multiple independently of lexical
validity and retains compact references to every source-order value occurrence.
Cards remain record-local across BLOCKS and ENTITIES. This does not apply
defaults, select duplicates, assemble typed semantics, resolve names, follow
ATTRIB/SEQEND, transform geometry, edit/write, or render.
M10.1i projects required block name and insertion point plus documented scale
`(1,1,1)`, rotation `0`, array count `(1,1)`, spacing `(0,0)`,
attributes-follow `0`, and extrusion `(0,0,1)` defaults. Explicit, defaulted,
and invalid states retain field/raw provenance; invalid or multiple values do
not receive defaults, and composite accessors fail closed. This does not
validate empty names, finiteness, count/flag ranges, resolve a block target,
follow ATTRIB/SEQEND, form an OCS transform, edit/write, or render.
M10.1j resolves usable INSERT block-name source spans against the M10.1f exact
matched-name index with fixed 4-KiB reads and no name-sized query allocation.
Results preserve unusable-name, missing, unique, or ambiguous state and every
duplicate target in source-record order. Conflicting/unindexable BLOCK names
cannot become targets. This does not require closed definitions, choose among
ambiguous targets, detect recursion, follow ATTRIB/SEQEND, form transforms,
edit/write, or render.
M10.1k builds the uniquely resolved BLOCK-member expansion graph and classifies
each INSERT target as not uniquely resolved, target definition not closed,
recursive expansion, or eligible. Iterative cancellation-aware graph traversal
detects self, indirect, and deeper reachable cycles while only traversing
closed targets. This does not validate numeric/count domains, follow
ATTRIB/SEQEND, calculate OCS axes/transforms, expand geometry, edit/write, or
render.
M10.1l derives one finite row-major 3x4 affine transform for every eligible
single INSERT. It subtracts the target BLOCK base point, applies per-axis scale
and rotation in OCS, adds the OCS insertion point, and maps through the
documented arbitrary-axis OCS basis to WCS. Unavailable semantics/base points,
non-finite inputs, zero extrusion, derived overflow, and non-finite point
application remain typed failures. This does not expand array rows/columns,
convert block units, follow ATTRIB/SEQEND, recursively transform member
geometry, edit/write, or render.
M10.1m derives constant-space rectangular-array layouts from positive
column/row counts and finite spacing. Column and row steps follow the rotated
INSERT OCS axes without multiplying spacing by BLOCK scale. The layout retains
counts and two WCS step vectors, reports its exact `u64` instance count, and
computes only a requested in-range transform; it never allocates the expanded
array. Invalid semantics/counts/spacing and placement overflow remain typed.
This does not enumerate every instance, convert BLOCK units, follow
ATTRIB/SEQEND, recursively transform member geometry, edit/write, or render.
M10.1n retains exact INSERT/ATTRIB/SEQEND sequence topology in complete
BLOCKS/ENTITIES sections. Zero attributes-follow consumes nothing; every
nonzero value scans consecutive exact uppercase ATTRIB records and records a
closed SEQEND, the first interruption, or section-end unclosed state while
preserving the original signed-16-bit flag. Invalid/duplicate flags remain
unavailable and consume nothing. This does not decode ATTRIB fields, associate
ATTDEF definitions, validate ownership, apply attribute transforms, expand
geometry, edit/write, or render.
M10.1o retains 23 classic ATTRIB roles from every M10.1n attribute record in
exact text, binary64, or signed-16-bit source order. Subclass tracking admits
legacy, AcDbText, and AcDbAttribute values, excludes group-102 application
content, ignores unrelated subclass contexts, and stops before AcDbXrecord so
MText extension codes cannot impersonate classic roles. The two documented
group-280 meanings remain neutral `VersionOrLockPosition` occurrences. This
does not assign cardinality, select/default values, validate domains, interpret
flags/justification, decode MText extensions, associate ATTDEF definitions,
transform attributes, edit/write, or render.
M10.1p publishes 23 fixed cards per M10.1o ATTRIB record. Each card reports
absent, unique, or duplicate-preserving multiple state and retains compact
source-order references independently of lexical validity and sequence state.
Empty records receive 23 absent cards, and both group-280 occurrences remain
one neutral multiple card. This does not select values, apply defaults,
distinguish group-280 meanings, interpret flags/justification, decode MText,
associate ATTDEF definitions, transform attributes, edit/write, or render.
M10.1q lazily projects the 14 classic ATTRIB double roles. Text-start X/Y/Z
and text height are required; absent thickness, rotation, relative X scale,
oblique angle, and extrusion receive only documented defaults. Alignment-point
components remain independently optional. Invalid ASCII and duplicate values
remain typed with raw provenance and never fall back to defaults; tuple helpers
require every component to be usable. This does not validate numeric domains,
interpret justification, project remaining text/integer fields, decode MText,
associate ATTDEF definitions, transform attributes, edit/write, or render.
M10.1r lazily projects the three classic ATTRIB text roles. Text/default value
and tag are required source-anchored values; an absent style name receives the
documented `STANDARD` default without invented raw provenance. Invalid
cardinality remains typed, and explicit text decoding stays bounded,
same-document, and replacement-free. This does not validate empty text or tag
spaces, resolve style-table names, interpret formatting/escapes, project
numeric fields, decode MText, associate ATTDEF definitions, transform
attributes, edit/write, or render.
M10.1s lazily projects the five unambiguous classic ATTRIB signed-16-bit roles.
Attribute flags are required; field length, text-generation flags, and both
justification values receive documented zero defaults. Helpers expose four
attribute bits and two text-generation bits without discarding unknown bits.
Invalid ASCII and duplicate values remain typed with raw provenance. Neutral
group `280` stays exact card evidence because version and lock-position share
one wire code. This does not validate field length/unknown bits, classify
justification, determine alignment-point applicability, decode MText,
associate ATTDEF definitions, transform attributes, edit/write, or render.
M10.1t classifies horizontal justification codes `0..5` and vertical codes
`0..3` exactly as published by Autodesk. Unsupported or unavailable codes stay
typed invalid. Alignment-point applicability is true when either usable code
is nonzero, false when both are usable zero values, and otherwise unavailable.
This does not validate code combinations, select coordinate tuples,
recalculate placement, measure styled text, decode MText, associate ATTDEF
definitions, transform attributes, edit/write, or render.
M10.1u selects text-start `10/20/30` only for usable baseline/left
justification and selects alignment point `11/21/31` when either usable
justification code is nonzero. The selected tuple must be wholly usable;
failures in the unselected tuple are retained but do not invalidate the
anchor. Justification, text-start, and alignment unavailability remain
distinct. This does not recalculate stored points, validate justification
combinations, apply extrusion/rotation/style metrics/INSERT transforms, decode
MText, associate ATTDEF definitions, edit/write, or render.
M10.1v maps a usable selected OCS anchor through the ATTRIB extrusion with the
shared normalized arbitrary-axis algorithm and exact `1/64` branch. Finite WCS
points and normals use canonical positive zero. Placement/extrusion
unavailability, non-finite Binary values, zero normals, basis failure, and
derived overflow stay typed. This does not apply text rotation, oblique/width/
generation flags, style metrics, INSERT/BLOCK transforms, or ATTDEF
association; recalculate stored points; decode MText; edit/write, or render.
M10.1w indexes exact uppercase `ATTDEF` records only inside the retained member
ranges of M10.1a BLOCK definitions. Each entry preserves its exact raw record,
owning closed/interrupted/unclosed definition, zero-based member position, and
definition-local ATTDEF position. Outside-definition and non-exact markers are
excluded. This does not decode ATTDEF fields, interpret flags, compare tags,
associate ATTRIB records, transform geometry, edit/write, or render.
M10.1x retains 24 classic ATTDEF roles from every M10.1w record in exact text,
binary64, or signed-16-bit source order. Subclass tracking admits legacy,
AcDbText, and AcDbAttributeDefinition values, excludes group-102 application
content, ignores unrelated subclass contexts, and stops before AcDbXrecord so
MText extension codes cannot impersonate classic roles. The two documented
group-280 meanings remain neutral `VersionOrLockPosition` occurrences. This
does not assign cardinality, select/default values, validate domains, interpret
flags/justification, decode MText extensions, compare ATTRIB tags, associate
inserted attributes, transform geometry, edit/write, or render.
M10.1y publishes 24 fixed cards per M10.1x ATTDEF record. Each card reports
absent, unique, or duplicate-preserving multiple state and retains compact
source-order references independently of lexical validity and owning BLOCK
state. Empty records receive 24 absent cards, and both group-280 occurrences
remain one neutral multiple card. This does not select values, apply defaults,
distinguish group-280 meanings, interpret flags/justification, decode MText,
compare ATTRIB tags, associate inserted attributes, transform geometry,
edit/write, or render.
M10.1z lazily projects the 14 classic ATTDEF double roles. Text-start X/Y/Z
and text height are required; absent thickness, rotation, relative X scale,
oblique angle, and extrusion receive only documented defaults. Alignment-point
components remain independently optional. Invalid ASCII and duplicate values
remain typed with raw provenance and never fall back to defaults; tuple helpers
require every component to be usable. This does not validate numeric domains,
interpret justification, project remaining text/integer fields, decode MText,
compare ATTRIB tags, associate inserted attributes, transform geometry,
edit/write, or render.
M10.1aa lazily projects the four classic ATTDEF text roles. Default value,
prompt, and tag are required source-anchored values; an absent style name
receives the documented `STANDARD` default without invented raw provenance.
Invalid cardinality remains typed, and explicit text decoding stays bounded,
same-document, and replacement-free. This does not validate empty text or tag
spaces, resolve style-table names, interpret formatting/escapes, project
integer fields, decode MText, compare ATTRIB tags, associate inserted
attributes, transform geometry, edit/write, or render.
M10.1ab lazily projects the five unambiguous classic ATTDEF signed-16-bit
roles. Attribute flags are required; absent field length, text-generation
flags, and horizontal/vertical justification receive their documented zero
defaults. Exact values preserve unknown bits while helpers test only the four
published attribute bits and two published text-generation bits. Invalid
ASCII, missing required flags, and duplicates remain typed with raw provenance.
The overloaded group `280` remains neutral card evidence and is not selected.
This does not validate field-length or justification ranges, classify
justification/alignment applicability, distinguish group-280 meanings, decode
MText, compare ATTRIB tags, associate inserted attributes, transform geometry,
edit/write, or render.
M10.1ac lazily classifies published ATTDEF horizontal codes `0..5` and vertical
codes `0..3` while preserving explicit/defaulted state and raw provenance.
Unsupported signed codes and underlying integer failures remain typed.
Alignment-point applicability is true when either usable code is nonzero,
false only when both usable codes are zero, and unavailable otherwise. This
does not validate code combinations, require or select text-start/alignment
tuples, recalculate placement, measure styled text, decode MText, compare
ATTRIB tags, associate inserted attributes, transform geometry, edit/write, or
render.
M10.1ad lazily selects the stored ATTDEF text-start tuple for usable zero/zero
justification and the stored alignment tuple when either usable code is
nonzero. The selected tuple must have three usable components; unavailable
justification, text start, and alignment point remain distinct states. Invalid
or missing components in the ignored tuple remain inspectable without
contaminating the selected anchor. This does not recalculate stored points,
validate justification combinations, apply extrusion, rotation, style metrics,
or INSERT/BLOCK transforms, decode MText, compare ATTRIB tags, associate
inserted attributes, edit/write, or render.
M10.1ae lazily projects a usable selected ATTDEF OCS anchor through its usable
extrusion using the shared arbitrary-axis basis. Success retains a finite WCS
point, normalized finite normal, canonical positive zero, and the underlying
placement evidence. Placement/extrusion unavailability, non-finite Binary
inputs, zero extrusion, basis failure, and transformed overflow remain typed.
This does not apply text rotation, oblique/width/generation flags, style
metrics, INSERT/BLOCK transforms, or ATTRIB association. It does not
recalculate stored points, decode MText, edit/write, or render.
M10.1af indexes usable ATTDEF tags under each exact owning BLOCK. SHA-256
narrows lookup candidates, exact bounded raw-span comparison confirms equality,
and duplicates remain in definition-local order. Every BLOCK exposes total,
indexed, and unusable ATTDEF-tag counts, including empty blocks; same-document
source spans can query without a tag-sized buffer. This does not validate tag
syntax, uppercase/decode tags, collapse duplicates, require a closed BLOCK,
treat missing as definitive when unusable tags exist, resolve INSERT targets,
associate ATTRIB records, edit/write, or render.
M10.1ag resolves each retained ATTRIB tag only for an INSERT with one exact
BLOCK target, then queries that BLOCK's exact ATTDEF tag index by source span.
It preserves target-unavailable, tag-unavailable, definitive missing,
indeterminate missing when unusable ATTDEF tags remain, unique, and
duplicate-preserving ambiguous outcomes. It does not normalize/decode tags,
choose ambiguous targets/definitions, require closed sequences/definitions,
validate ownership or attribute flags/defaults, transform placement,
edit/write, or render.
M11.1a constructs immutable raw-byte transaction plans bound to the opened
source identity, length, and physical format. Source-order patches own their
replacement bytes and captured inverse bytes; replace/delete and deterministic
boundary insertion are supported. Overlaps, insertions inside replacements,
duplicate insertion offsets, out-of-bounds spans, cancellation, and resource
limits fail before mutating builder state. Plans redact payload bytes from
`Debug`. This does not apply patches, calculate a post-image identity,
materialize an executable inverse transaction, validate resulting DXF,
allocate handles, write a destination, or publish a snapshot.
M11.1b stream-verifies that an independently opened same-format post-image is
the exact result of a transaction plan. Fixed 4-KiB comparisons cover unchanged
source ranges and owned replacement bytes; length mismatch and the first
different post-image byte remain stable typed errors. Exact matches materialize
an immutable inverse plan bound to the post-image identity. Adjacent deletions
mapping to one insertion offset are coalesced in original source order, and
inverse-to-redo materialization preserves the exact post-image bytes. This does
not write/apply plans, accept an unopenable result, allocate handles, replace a
filesystem destination, or publish a snapshot.
M11.2a exposes an immutable allocation-policy directory tied to the same source
identity as its exact `$HANDSEED` and record-local group-code 5/105 identity
evidence. Allocation is ready only for one parsed nonzero seed strictly above
all uniquely parsed, nonzero, globally unique object identities. Missing,
invalid, ambiguous, null, multiple, duplicate, and stale evidence remain
distinct typed states. Bounded eligible requests return a constant-space
consecutive range and successor seed or a typed arithmetic-exhaustion outcome.
This does not validate pointer/owner references, repair stale evidence, reuse
gaps, reserve handles, encode object identities, update `$HANDSEED`, build or
apply a transaction, write a destination, or publish a snapshot.
M11.2b turns a caller-ordered set of identity-absent raw records into one
immutable source-bound transaction containing every new identity plus the
successor `$HANDSEED`. Duplicate, missing, already-identified, CLASSES,
ENDTAB, and TABLES records without group 2 fail as typed target states before a
plan escapes. BLOCKS/ENTITIES/OBJECTS use group 5 after group 0; table objects
and entries use group 5 after group 2 except exact TABLES `DIMSTYLE`, which
uses group 105. ASCII preserves the selected anchor's line ending and Binary
uses the declared dialect's group-code width and NUL-terminated hexadecimal.
Materialized ASCII/Binary plans re-open with the assigned identities and a
ready successor allocation policy. This does not infer which records need
identities, handle CLASSES or incomplete table records, validate references,
apply the transaction, write or replace a destination, or publish a snapshot.
M12.1a applies any immutable transaction plan to a create-new file in fixed
64-KiB chunks. It rehashes the complete live source while applying patches,
hashes the projected bytes while writing, flushes and syncs, then reopens the
output to verify exact length and identity. The receipt retains source/output
identities plus byte and patch counts. Source mismatch, output length mismatch,
and output identity mismatch remain typed; cancellation or failure after file
creation attempts to remove the incomplete output, while an existing
destination is never modified. This does not replace paths, automatically
materialize an inverse from the output, canonicalize either physical format,
infer edits, or publish a snapshot.
M12.1b adds a stronger create-new path that strictly reparses the verified
M12.1a output using its exact ASCII/Binary format and selected resource
profile. The reparsed identity must match the write receipt, then M11.1b
verifies the complete post-image and returns an executable inverse plan paired
with that receipt. Applying the inverse through the same API restores a strict
original and returns the corresponding redo journal. Strict-reparse or inverse
failure removes the new output. This does not replace paths, provide
crash-atomic rename, canonicalize either format, infer edits, or publish a
snapshot.
M12.2a writes ASCII raw groups with minimal signed-decimal group codes and LF
framing while retaining each exact value payload. Reviewed Compatible envelope
recoveries close to one strict terminal `0`/`EOF`: BOM/padded EOF is
normalized, missing EOF is appended, and opaque trailing bytes are omitted.
Preflight enforces selected source/value/record limits and exact projected
counts; streaming rehashes the complete live source, verifies and syncs the
create-new output, then strictly reparses it. This is canonical ASCII framing,
not numeric, text, handle, binary-chunk, or semantic value normalization. It
does not convert Binary input, replace a path, or publish a snapshot.
M12.2b writes Binary raw groups after the exact canonical 22-byte sentinel with
the group-code encoding required by the declared dialect: one byte plus the
reviewed XDATA escape for AC1009, or signed 16-bit little-endian for AC1012+.
Every non-EOF value wire span remains byte-exact, including string terminators,
binary-chunk length prefixes, and fixed-width numeric bits. Missing EOF is
appended, opaque trailing bytes are omitted, and the result passes the same
bounded create-new hash/length verification, sync, and strict-reparse contract
as M12.2a. This is Binary physical framing canonicalization only; it does not
reinterpret payloads, convert ASCII input, replace a path, or publish a
snapshot.
M13.1a adds a deterministic CycloneDX 1.6 inventory for all 29 packages in the
complete locked workspace graph. All 26 registry packages carry exact
`Cargo.lock` SHA-256 checksums, crates.io package URLs, declared SPDX license
expressions, dependency edges, and package/version presence in
`THIRD_PARTY_NOTICES.md`; all three workspace packages retain the proprietary
license reference. The generated-file gate runs on every CI platform. This is
dependency/notices inventory evidence, not a distributable legal bundle,
private-corpus evidence, six-native release closure, or a Core 1.0 release
claim.
M13.1b adds a deterministic distributable legal directory containing the
three project legal/notice files and all 54 root license artifacts extracted
byte-for-byte from the 26 locked crates.io source trees. Its manifest records
package/version, SPDX expression, crate archive checksum, lockfile identity,
and exact byte count/SHA-256 for every file; the gate rejects missing, changed,
extra, symlinked, or non-regular artifacts. This closes legal-file packaging
for the current locked graph, not native archive/signature, installer,
private-corpus, six-native, or Core 1.0 release evidence.
M13.2a preserves the Q2.2 ceiling-only corpus manifest/receipt v1 and adds a
release-only aggregate receipt v2. The committed v2 policy requires at least
1,000 strictly verified DXF files and 10 GiB with zero invalid files under
independent hard ceilings of 2,000 files, 20 GiB, 20,000 entries, and depth 32.
The receipt reports file, byte, and zero-invalid threshold states separately;
shortfall is a redacted failed receipt, not a fabricated parse error. This
implements the corpus release gate but is not evidence that a private corpus
has passed it, a corpus commitment, or Core 1.0 authorization.
M13.2b adds a fail-closed artifact directory assembler for the six reviewed
Linux, Windows, and macOS x64/ARM64 targets. Each staged directory contains one
native `seacad` CLI, README, deterministic SBOM, complete legal bundle, and a
sorted receipt binding all 61 payload files by relative path, byte count, and
SHA-256 to the target, source commit, package version, and Rust 1.97.1. A
manual, contents-read-only workflow builds on the six native hosted runners
and uploads the directories with exact checkout/upload action pins. This is a
packaging contract and workflow definition, not evidence that those workflow
jobs ran successfully, not a signed or reproducible archive, not an installer
or published release, and not Core 1.0 authorization.
M13.2c adds a same-run six-artifact verifier. It requires the exact target set,
binds package/version/Rust/source commit, re-hashes every downloaded payload,
rejects missing/extra/symlinked/non-regular or out-of-bound entries, and
compares README, SBOM, and the entire legal path/hash set with the checkout
rather than trusting artifact receipts alone. Its aggregate receipt records
the SHA-256 of all six per-target receipts and their payload totals. This
defines six-target evidence aggregation but does not claim that the manual
workflow has run successfully, publish or sign artifacts, satisfy corpus or
twenty-night gates, or authorize Core 1.0.
M13.2d generates the locked CycloneDX dependency graph as the deterministic
union of explicit Cargo metadata for the six reviewed native target triples.
Package identities and resolved edges are merged and sorted after verifying a
single workspace-member set. M13.2e adds bounded committed/generated hashes and
semantic edge deltas to a stale-SBOM failure. M13.2f normalizes Cargo.lock
identity to LF, rejects lone carriage returns, and pins lockfile checkout line
endings. These change release-evidence reproducibility only; they do not add
dependencies, DXF behavior, target support, or a successful six-native
workflow claim.
M13.2g records the first successful manual Native Release Artifacts workflow:
run `30557566354` built all six reviewed native packages and its aggregate job
verified every receipt and payload against commit
`222eec2c9d9b18fbb7ff1b8d5f0120ba30633365`. The seven uploaded artifacts are
temporary workflow evidence, not signed or permanently retained releases.
Q2.2 adds an offline strict-verification receipt harness whose output is
aggregate-only and path-redacted. Its 1,000-file and 10-GiB manifest values are
hard traversal ceilings, not achieved corpus evidence, performance evidence,
or increased support claims. Final corpus scale and six-native release
receipts remain M13 work.
M5.2a requires the canonical opening and verifies `$ACADVER` against the
selected group-code encoding. M5.2b requires or explicitly recovers terminal
EOF, retains compatible trailing bytes as one opaque span, and accounts every
accepted group through the shared section/group-zero index. M5.2c reuses the
verified create-new Verbatim path and the stable JSON v1 CLI contract.
"Envelope structure" recognizes exact section boundaries and record starts
without interpreting section payloads. Unknown section names remain exact
source-backed bytes. "Text-storage policy" means UTF-8 by documented modern
version or a provenance-backed legacy declaration. The decoder is a low-level,
caller-buffer view tied to one document group occurrence. Escape
interpretation is a second bounded layer: exact CIF controls and all five MIF
selectors decode without replacement. CP1361 uses a frozen exhaustive table
that matches Windows strict NLS; invalid and undefined codes fail closed.
Selectors outside 1 through 5 remain literal, matching the AutoCAD 2027
oracle. MTEXT/percent tokenization is lexical: it uses an explicit entity
context, retains raw/payload spans, enforces the documented eight-block limit,
and does not parse formatting values, evaluate fields, shape glyphs, or render
text. Semantic strings and DOS/OEM pages are not implemented.
"Verified Verbatim" only creates a new byte-identical file and is not an edit
or canonical writer.
