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
