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
and absence remains `Absent` without an invented default. Other numeric ranges,
background/column semantics, rotation precedence, and geometry remain later
checkpoints.

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
