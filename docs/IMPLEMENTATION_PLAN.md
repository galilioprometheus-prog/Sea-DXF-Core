# DXF Core 1.0 Implementation Plan

Status: M5 completed through M5.2c verified Binary replay and CLI integration;
M6.7 closes the source-anchored 206-row, 214-slot HEADER inventory with explicit
ASCII/Binary parity evidence across all nine supported AC1009-AC1032 dialects;
Q1 enforces the reviewed cargo-deny dependency policy; Q2.1 stages native CI
coverage across Linux, Windows, and macOS on both x64 and ARM64; Q2.2 adds the
bounded aggregate-only offline corpus manifest and cross-platform receipt
harness without publishing private corpus identifiers; M7.4g conservatively
compares a uniquely resolved common-owner candidate with a unique incoming
ownership-class link and retains matched, conflicting, or non-comparable state;
M8.1a adds exact POINT/LINE coordinate-component evidence without assembling or
transforming geometry; M8.1b adds fixed per-role cardinality cards that retain
every occurrence through compact evidence-member references; M8.1c lazily
projects required WCS tuples and reviewed per-component extrusion defaults into
source-anchored semantic states; M8.2a adds a separate exact CIRCLE/ARC defining-
value evidence directory that preserves their OCS boundary; M8.2b adds fixed
per-role CIRCLE/ARC cardinality cards with compact evidence references; M8.2c
adds lazy required-value semantics and reviewed extrusion defaults; M8.3a adds
exact ELLIPSE WCS center, relative major-axis endpoint, ratio, parameter, and
extrusion-component evidence without assembling ellipse geometry; M8.3b adds
twelve fixed per-role ELLIPSE cardinality cards with compact evidence
references; M8.3c adds lazy required-value semantics and reviewed extrusion
defaults without range validation or ellipse assembly; M8.4a adds exact
RAY/XLINE WCS point and unit-direction evidence without normalization or
infinite-line assembly; M8.4b adds six fixed per-role RAY/XLINE cardinality
cards with compact evidence references; M8.4c adds lazy required-value
semantics without unit-vector validation or normalization; M9.1a adds exact
LWPOLYLINE floating-point evidence without grouping vertices or assembling a
polyline; M9.1b adds exact signed integer evidence for LWPOLYLINE count, flags,
and vertex identifiers without interpreting or reconciling them; M9.1c groups
vertex-scoped evidence by exact group-10 anchors with fixed cardinality cards
and explicit pre-anchor orphans; M9.1d lazily projects required OCS X/Y,
defaulted local widths/bulge, and optional vertex identifiers without effective
width or segment assembly; M9.1e adds eight fixed record-level cardinality
cards for count, flags, elevation, thickness, constant width, and extrusion
without admitting vertex-scoped fields or applying record semantics; M9.1f
adds typed record defaults, flag helpers, declared-versus-observed vertex-count
comparison, and neutral constant/variable width coexistence evidence; M9.1g
builds consecutive/closing segment topology and binds each segment to the
starting vertex's local widths and bulge without assembling OCS geometry;
M9.1h lazily derives finite straight or circular OCS segment geometry from
usable endpoints and bulge with typed unavailable/degenerate/overflow failures;
M9.2a indexes exact classic POLYLINE records with consecutive VERTEX records
and typed closed, interrupted, or unclosed SEQEND boundaries; M9.2b retains all
sixteen documented classic POLYLINE numeric roles in exact double or signed
16-bit wire domains without selecting values or interpreting flags; M9.2c
retains all thirteen documented classic VERTEX numeric roles in exact double,
signed-16-bit, or signed-32-bit wire domains without classifying vertices or
resolving polyface indices; M9.2d adds thirteen fixed cardinality cards to each
retained VERTEX without selecting values or coupling lexical validity to
occurrence count; M9.2e lazily projects required location, defaulted local
widths/bulge, and optional curve-fit tangent direction without classifying
vertices or interpreting integer fields; M9.2f adds sixteen fixed cardinality
cards to each classic POLYLINE record without selecting values or admitting
VERTEX payloads; M9.2g lazily projects required dummy/elevation components,
documented record defaults, and exact flag-bit helpers while keeping obsolete
group `66` outside semantic policy and avoiding mesh classification; M9.2h
lazily projects optional VERTEX flags, four polyface indices, and identifier
with seven meaningful flag helpers but no vertex classification or face
resolution; M9.2i classifies non-conflicting POLYLINE and VERTEX family bits
and compares parent/child evidence while retaining unavailable, conflicting,
and mismatched states; M9.2j builds consecutive and closing topology only for
complete, family-consistent classic 2D/3D sequences; M9.2k lazily binds each
proven segment to parent and endpoint semantics while
keeping local and parent width evidence separate; M9.2l derives planar 2D OCS
line/arc geometry with parent elevation and exact 3D WCS line geometry while
rejecting unavailable, contradictory, degenerate, or non-finite inputs; M9.2m
projects classic 2D segment geometry into WCS with the documented arbitrary-axis
algorithm while preserving native 3D WCS lines; M9.2n selects effective classic
2D segment widths with explicit VERTEX values taking precedence over parent
POLYLINE defaults independently for start and end; M9.2o builds fail-closed
row-major quadrilateral topology for complete, count-consistent classic polygon
meshes with independent M/N closure; M9.2p tolerantly partitions complete,
family-consistent classic polyface meshes into exact coordinate and face
VERTEX evidence while retaining reported-versus-observed counts and odd
ordering; M9.2q resolves valid signed 1-based polyface face indices to exact
coordinate evidence and preserves per-corner edge visibility

1. M0: toolchain, clean private repository, workspace, policy, and CI.
2. M1: provenance audit of earlier tests, fixtures, documents, and code.
3. M2: bounded file/byte source, progress/cancel, diagnostics, and SHA-256.
4. M3: lossless ASCII framing, strict/compatible modes, verbatim writer, and
   initial `inspect`/`verify` CLI.
5. M4: AC1009-AC1032 dialects, encoding, sections, and group-0 indexes.
6. M5: lossless Binary DXF record stream and malformed-input bounds.
   M5.1a freezes explicit pre-R13/R13+ group-code decoding and the documented
   value-family registry. M5.1b adds bounded streaming value framing. M5.2a
   adds encoding/dialect agreement and the immutable raw document; M5.2b adds
   envelope/index; M5.2c adds verified unchanged replay and CLI integration.
7. M6: provenance-backed schema/codegen and lazy document semantics. M6.1a
   adds a deterministic internal Rust generator and shape-only HEADER bootstrap
   for `$ACADVER`, `$DWGCODEPAGE`, and `$HANDSEED`. M6.1b moves normative source
   identities into a shared, strictly validated registry and requires generated
   code to match on every CI platform. M6.2 adds compact field/raw provenance
   and distinguishes `Explicit`, `Defaulted`, `Absent`, and `Invalid` without
   conflating states. M6.3a connects the generated schema to the already
   evidence-backed `$ACADVER` report through one O(1) typed HEADER view shared by
   ASCII and Binary. M6.3b runs the same `$DWGCODEPAGE` state machine over both
   physical formats and exposes reviewed/unrecognized declarations separately
   from decoder policy. M6.3c runs one exact `$HANDSEED` state machine in both
   parse loops, preserves source spelling/provenance, and exposes the parsed
   64-bit handle without claiming allocation or topology policy. M6.4a indexes
   every exact group-code 9 marker and its complete value-group range in exact
   HEADER sections, including unknown and multi-value variables, without a
   second source scan or semantic guesses. M6.4b exposes both validated physical
   representations through one allocation-free borrowed document/group API,
   including shared reports and indexes, so later semantic code has one input
   path. M6.4c adds exact raw-byte HEADER name lookup with duplicate evidence,
   keyed per-document fingerprints only as a candidate filter, and bounded
   source comparison that remains authoritative even on a fingerprint
   collision. M6.4d resolves every generated HEADER schema field together in
   one explicit lazy directory, avoiding a field-by-variable scan while
   retaining exact duplicate evidence. M6.5a extends the generator with
   reviewed `Double` and `Int16` wire families and adds shape-only entries for
   `$ACADMAINTVER`, `$ANGBASE`, `$ANGDIR`, `$ATTMODE`, `$AUNITS`, and
   `$AUPREC`; it does not yet claim defaults, applicability, enum meaning, or
   typed numeric values. M6.5b reads those six fields through the shared raw
   document and schema directory, preserving exact provenance and separating
   absence, structural conflicts, ASCII syntax/range failures, and explicit
   values. `DxfDouble` retains IEEE-754 bits; no default, enum, range, or unit
   conversion is applied. M6.5c resolves every generated numeric field by its
   schema ordinal into one ordered directory, then projects the existing
   six-field typed view from that directory. Adding a reviewed numeric schema
   row therefore does not require another handwritten resolution state machine.
   It does not add defaults, enum meaning, applicability, ranges, or units.
   M6.5d adds generated `Double2` and `Double3` wire shapes and the first eight
   reviewed coordinate rows: `$EXTMAX`, `$EXTMIN`, `$INSBASE`, `$LIMMAX`,
   `$LIMMIN`, `$PEXTMAX`, `$PEXTMIN`, and `$PINSBASE`. Each component is a
   separate source-anchored semantic value so a malformed axis cannot erase
   valid sibling components. The generic storage names deliberately avoid
   claiming point/vector meaning, coordinate transforms, or default extents.
   M6.5e adds `$PLIMMAX`, `$PLIMMIN`, `$PUCSORG`, `$PUCSXDIR`, `$PUCSYDIR`,
   `$UCSORG`, `$UCSXDIR`, and `$UCSYDIR` through generated schema rows only.
   The existing directory and tuple decoder require no production parser
   branch, demonstrating that reviewed fields now scale by data rather than
   handwritten resolution logic. M6.5f adds the six paper-space and six
   model-space orthographic origin rows (`BACK`, `BOTTOM`, `FRONT`, `LEFT`,
   `RIGHT`, and `TOP`). Schema manifest order becomes authoritative and
   append-only so all prior public ordinals remain unchanged; stable field ids
   remain the canonical persisted identity. M6.5g appends `$CECOLOR`,
   `$CELTSCALE`, `$CHAMFERA`, `$CHAMFERB`, `$CHAMFERC`, `$CHAMFERD`,
   `$CMLJUST`, `$CMLSCALE`, `$ELEVATION`, `$FILLETRAD`, `$FILLMODE`, and
   `$LTSCALE` as reviewed `Int16` or `Double` wire shapes. The generic numeric
   directory resolves them without field-specific parser branches; meanings,
   ranges, defaults, applicability, and cross-variable rules remain unclaimed.
   M6.5h appends `$LIMCHECK`, `$LUNITS`, `$LUPREC`, `$MAXACTVP`,
   `$MEASUREMENT`, `$MIRRTEXT`, `$ORTHOMODE`, `$PDMODE`, `$PDSIZE`,
   `$PELEVATION`, `$PLIMCHECK`, and `$PLINEWID` through the same generic
   `Int16` and `Double` directory. The checkpoint preserves source values and
   provenance without assigning enum, boolean, unit, range, or default
   semantics. M6.5i appends `$PLINEGEN`, `$PROXYGRAPHICS`, `$PSLTSCALE`,
   `$PSVPSCALE`, `$PUCSORTHOVIEW`, `$QTEXTMODE`, `$REGENMODE`, `$SHADEDGE`,
   `$SHADEDIF`, `$SHADOWPLANELOCATION`, `$SKETCHINC`, and `$SKPOLY` through
   the same generated numeric path. Display, regeneration, percentage,
   orthographic-view, and sketch meanings remain unclaimed until their
   semantic rules are separately reviewed. M6.5j appends `$SPLINESEGS`,
   `$SPLINETYPE`, `$SURFTAB1`, `$SURFTAB2`, `$SURFTYPE`, `$SURFU`, `$SURFV`,
   `$TEXTSIZE`, `$THICKNESS`, `$TILEMODE`, `$TRACEWID`, and `$TREEDEPTH`
   through the generated scalar path. Surface, text, thickness, layout, trace,
   and index-depth meanings remain unclaimed. M6.5k appends `$TDCREATE`,
   `$TDUCREATE`, `$TDUPDATE`, and `$TDUUPDATE` as exact Julian-date scalars,
   plus `$TDINDWG` and `$TDUSRTIMER` as exact elapsed-day scalars. Derived
   day splitting is finite and `i64`-bounded; the raw IEEE payload remains
   authoritative, and no timezone or calendar conversion is inferred. M6.5l
   appends five group-280 `Int16` fields and five group-290 strict Boolean
   fields. Boolean accepts only `0/1`; other source values remain typed invalid
   with raw provenance. AC1009 cannot physically encode those group codes in
   Binary DXF, while AC1012-AC1032 exercise both representations. A 1 GiB
   evidence gate is required before any large-file claim. M6.5m appends the
   remaining sixteen reviewed numeric or Boolean HEADER fields outside the
   `$DIM*` family. Ten group-62/70 values remain physically representable in
   AC1009 Binary; six group-280/290/370/380 values are absent from its standard
   parity fixture because the one-byte pre-R13 group-code header cannot encode
   them. AC1012-AC1032 exercise all sixteen through the same generic generated
   directory. Enum, color, lineweight, units, display, xref, timer, default,
   range, and applicability meanings remain unclaimed. M6.5n appends nineteen
   group-40 `$DIM*` double scalars through the same generic directory. All are
   physically representable in AC1009-AC1032 ASCII and Binary and retain exact
   IEEE-754 payloads and raw provenance. Dimension units, scale, tolerance,
   layout, sign, zero, default, range, applicability, and cross-variable
   meanings remain unclaimed. M6.5o appends fourteen group-70 `$DIM*`
   formatting and precision fields through the same generic directory. Their
   exact signed 16-bit values and raw provenance remain authoritative across
   AC1009-AC1032 ASCII and Binary. No precision, unit, bitmask, separator
   character, enum, default, range, applicability, or cross-variable semantics
   are inferred. M6.5o-r1 moves the existing strict ASCII `i16` and `f64`
   token parsers into one private module with focused unit tests. The public
   `DxfAsciiNumericIssue` re-export, exact IEEE-754 conversion, invalid-state
   mapping, streaming reads, provenance, dependencies, and support claims
   remain unchanged. M6.5o-r2 moves `DxfDouble`, `DxfDayParts`,
   `DxfJulianDate`, and `DxfElapsedDays` into one private `header_scalar`
   module. Their crate-root re-exports, exact bit preservation, finite
   `i64`-bounded day splitting, calendar/timezone non-interpretation, and all
   HEADER decoding behavior remain unchanged. Mutation testing adds explicit
   finite-value and `i64::MIN` boundary coverage before the move. M6.5o-r3
   moves `DxfHeaderNumericIssue`, `DxfHeaderNumericValue`, and their tuple
   state/provenance projections into one private `header_numeric_value` module.
   Crate-root paths, enum shapes, semantic states, exact raw provenance, and
   decoder behavior remain unchanged. Mutation testing adds direct coverage for
   defaulted/absent tuples and provenance that first appears in a later tuple
   component. M6.5o-r4 moves the CLI's serializable report data and stable
   enum-to-output names into one private `report` module. Command execution,
   rendering, exit codes, JSON schema v1, localized text, path redaction, DXF
   decoding, dependencies, streaming, and support claims remain unchanged.
   Mutation testing directly locks every published core enum variant used in
   report output. M6.5o-r5 moves JSON and localized human rendering together
   with the stable CLI error codes into one private `output` module. The moved
   renderer is byte-for-byte identical after visibility normalization.
   Contract tests lock every published status, physical format, read mode,
   resource profile, conformance, severity, and localized error-code mapping.
   Command execution, report construction, JSON schema v1, exit behavior,
   redaction, DXF core behavior, and support claims remain unchanged.
   M6.5o-r6 moves the Clap command tree, parsed `CliOptions`, action names, and
   help writer into one private `command` module. The moved command block is
   byte-for-byte identical after visibility normalization. A direct contract
   test locks root help plus inspect/verify usage in English and Vietnamese.
   Argument names, defaults, validation, exit behavior, execution, rendering,
   DXF core behavior, dependencies, and support claims remain unchanged.
   M6.5p appends `$DIMALT`, `$DIMASO`, `$DIMLIM`, `$DIMSAH`, `$DIMSD1`,
   `$DIMSD2`, `$DIMSE1`, `$DIMSE2`, `$DIMSHO`, `$DIMSOXD`, `$DIMTIH`,
   `$DIMTIX`, `$DIMTOFL`, `$DIMTOH`, `$DIMTOL`, and `$DIMUPT` as exact
   group-70 signed 16-bit values. Values are not coerced to Boolean, and the
   obsolete `$DIMASO` field is not merged with or used as a fallback for
   `$DIMASSOC`. Flag, suppression, placement, default, range, applicability,
   and cross-variable meanings remain unclaimed.
8. Q1: automate dependency policy with `cargo-deny 0.20.2`. The required
   `cargo deny --locked check` gate covers advisories, licenses, duplicate and
   wildcard dependencies, exact reviewed features, audited build scripts, and
   approved package sources. The CI action and checkout action are pinned by
   commit SHA. This checkpoint changes packaging and quality gates only; it
   does not change the Rust API, DXF behavior, CLI output, or `Cargo.lock`.
9. Q2: stage six-native-platform evidence before semantic expansion. Q2.1
   retains full required quality checks on Linux x64, Windows x64, and macOS
   ARM64 for every push and pull request; adds native schema, workspace build,
   and core smoke coverage on Linux ARM64, Windows ARM64, and macOS x64; and
   runs the full workspace gate on all six platforms nightly and on manual
   dispatch. Supplemental runners remain staged until twenty consecutive
   nightly runs pass. Q2.2 adds the redacted offline corpus manifest and
   receipt harness without placing private DXF bytes, paths, or per-file hashes
   in the repository.
10. M6 closure: complete the documented HEADER inventory before topology.
    M6.5q appends `$DIMASSOC`, `$DIMATFIT`, `$DIMCLRD`, `$DIMCLRE`, `$DIMCLRT`,
    `$DIMJUST`, `$DIMLWD`, `$DIMLWE`, `$DIMTAD`, `$DIMTMOVE`, `$DIMTOLJ`,
    `$USERI1`-`$USERI5`, and `$USERR1`-`$USERR5` through the generic numeric
    directory. The five-field user-variable expansions retain their exact
    published range-row evidence; runtime parsing, defaults, ranges, semantic
    interpretation, and version applicability remain unchanged or unclaimed.
    M6.6a adds one source-anchored generic text directory and appends
    `$CELTYPE`, `$CLAYER`, `$CMLSTYLE`, `$DIMAPOST`, `$DIMBLK`, `$DIMBLK1`,
    `$DIMBLK2`, `$DIMLDRBLK`, `$DIMPOST`, `$DIMSTYLE`, and `$DIMTXSTY`.
    M6.6b appends the remaining twelve text, name, path, and GUID fields
    through the generic exact-text directory without filesystem resolution,
    symbol lookup, GUID validation, or silent normalization. M6.6c adds the four
    remaining HEADER handle fields through a generic source-anchored handle
    directory. It retains exact hexadecimal spelling and provenance without
    resolving pointer, ownership, or object identity semantics. M6.7 closes the
    206 documented rows and 214 expanded field slots with explicit ASCII/Binary
    parity evidence across all nine supported AC1009-AC1032 dialects. Text
    values retain exact source spelling under the existing encoding policy;
    AC1009 fixtures leave post-R12 numeric and handle group codes absent in both
    physical formats because its one-byte Binary header cannot encode them.
    This closure does not add defaults, applicability, enums, ranges, topology,
    filesystem behavior, or new support beyond the reviewed wire shapes.
11. M7: handles, ownership, references, dictionaries, XDATA, and reactors.
    M7.1a adds a context-neutral, public classification registry for every
    documented handle-valued numeric group-code family: object identity,
    arbitrary handles, soft/hard pointers, and soft/hard owners. Plot-style
    and `480..481` handles retain hard-pointer behavior, while XDATA `1005`
    retains soft-pointer behavior. This checkpoint classifies codes only; it
    does not parse record values, resolve targets, validate existence, assign
    ownership, or construct document topology.
    M7.1b adds a bounded, source-anchored projection for one raw group
    occurrence. It distinguishes a missing occurrence, a non-handle group, and
    a handle-valued group with an exact parsed value or typed lexical failure.
    ASCII and Binary share the same API and fixed 16-byte parser; raw spelling
    remains readable only from the matching source document. Target lookup,
    record identity, dangling-reference checks, and topology remain deferred.
    M7.2a adds one format-neutral directory of group-zero-delimited raw chunks
    inside completely closed `CLASSES`, `TABLES`, `BLOCKS`, `ENTITIES`, and
    `OBJECTS` sections. It excludes framing markers and unknown/non-record
    sections, retains exact group ranges, and reports interrupted or unclosed
    recognized sections without indexing partial records. Record type
    interpretation, handle identity, ownership, and reference resolution remain
    deferred.
    M7.2b layers source-anchored group-code `5` and `105` identity evidence over
    every M7.2a raw record. Absent, uniquely parsed, uniquely invalid, and
    multiple-candidate records remain distinct; exact candidate spelling stays
    readable from the matching source. Only records with one lexically parsed
    candidate enter the handle-sorted lookup, where duplicate values remain an
    explicit ambiguous result. Parsed and null values are evidence rather than
    proof of semantic validity, and reference/ownership resolution stays
    deferred.
    M7.3a adds a source-order directory of raw record occurrences classified as
    soft/hard pointers or soft/hard owners. Each entry retains its exact record,
    raw group, class, spelling access, and parsed or invalid handle evidence.
    Object identities and `320..329` arbitrary handles are excluded; the latter
    are not translated references. AC1009 parity admits `1005` through the
    documented pre-R13 Binary escape while keeping unavailable group codes above
    255 absent in both physical fixtures. Target resolution, existence checks,
    ownership enforcement, and container semantics remain deferred.
    M7.3b resolves every M7.3a occurrence against the uniquely parsed M7.2b
    identity index with five explicit states: lexical `Invalid`, `Null`,
    `Missing`, `Unique`, and duplicate-preserving `Ambiguous`. Multiple or
    invalid identity candidates never become targets, and null remains null
    even if a record carries identity value zero. The directory owns one shared
    identity index and one shared reference index, keeping storage linear rather
    than copying target lists per reference. This is document-local lookup only;
    ownership validity, graph traversal, container meaning, and edit behavior
    remain deferred.
    M7.4a filters that resolved evidence to soft-owner and hard-owner
    occurrences, retains every invalid, null, missing, unique, or ambiguous
    occurrence in source order, and groups only uniquely resolved links by
    target record. Zero, one, and multiple incoming ownership-class links are
    therefore observable without copying target lists. This is not yet a
    complete ownership graph: context-specific owner pointers such as common
    group code `330`, legal record-type usage, one-owner conformance, lifecycle
    behavior, cycles, and container semantics remain deferred.
    M7.4b materializes one compact target entry for every raw record. Each
    entry retains its exact record, half-open slice of uniquely resolved
    incoming ownership-class links, and an explicit `NoIncomingLink`,
    `UniqueIncomingLink`, or `MultipleIncomingLinks` state. Construction is a
    linear merge over target-sorted M7.4a links. These states describe only
    incoming `350..369` evidence and are not one-owner conformance results.
    M7.4c indexes every group-code `102` occurrence inside complete raw records
    as an exact known start, other application start, close, or invalid control.
    Known starts distinguish `{ACAD_REACTORS` and `{ACAD_XDICTIONARY`; each
    start retains closed, interrupted-by-another-start, or unclosed state plus
    exact source/content ranges bounded to its record. This lexical context does
    not yet reinterpret the `330` or `360` handles inside it.
    M7.4d joins every M7.3b resolution entry to the optional M7.4c group whose
    content range contains the original handle occurrence. Context remains one
    of outside, `{ACAD_REACTORS`, `{ACAD_XDICTIONARY`, or another application
    group, with the exact group entry and malformed closure state still
    available. Numeric pointer/owner class and target-resolution state remain
    independent evidence. M7.4e conservatively classifies every contextual
    reference as a generic pointer, generic ownership-class reference,
    persistent-reactor candidate, extension-dictionary candidate, or common
    owner-pointer candidate. Candidate roles require the exact documented code,
    numeric class, application context, closed-group state where applicable,
    and one of `TABLES`, `BLOCKS`, `ENTITIES`, or `OBJECTS`; malformed groups
    fall back to generic evidence. Roles do not erase invalid, null, missing,
    unique, or ambiguous target state. Record-type legality, authoritative owner
    reconciliation, one-owner conformance, graph topology, and container
    semantics stay deferred. M7.4f filters those common-owner pointer candidates
    into one source-order evidence stream and materializes one compact card per
    raw record with `NoCandidate`, `UniqueCandidate`, or `MultipleCandidates`.
    Every invalid, null, missing, unique, or ambiguous candidate remains in the
    record slice, and its exact target-match slice stays accessible. This is
    candidate cardinality only; comparison with incoming ownership-class links,
    authoritative owner selection, and one-owner conformance stay deferred.
    M7.4g materializes one comparison entry per raw record. Comparison occurs
    only when the record has exactly one common-owner candidate resolved to one
    record and exactly one incoming ownership-class link. It compares the
    candidate's target record with the incoming link's source record and emits
    `Matched` or `Conflicting`; every absent, multiple, invalid, null, missing,
    or ambiguous shape remains `NotComparable` with both underlying dimensions
    intact. This is bidirectional evidence, not authoritative owner selection,
    record-type validation, or one-owner conformance.
12. M8: exact basic geometry and coordinate-system preservation. M8.1a indexes
    exact uppercase `POINT` and `LINE` records in complete `BLOCKS` and
    `ENTITIES` sections and retains source-order WCS location/start, WCS
    endpoint, and extrusion-component occurrences with exact ASCII/Binary
    double evidence. Duplicates, invalid ASCII values, and empty component
    slices remain explicit. It does not select canonical components, apply
    defaults, validate completeness, or transform coordinate systems.
    M8.1b materializes six stable role cards for every `POINT` and nine for
    every `LINE`. Each card reports `Absent`, `Unique`, or `Multiple` and owns a
    compact member slice pointing back to the M8.1a occurrence directory.
    Lexical validity remains independent from cardinality; no occurrence is
    copied, selected, defaulted, or transformed.
    M8.1c adds lazy typed POINT and LINE component semantics. Unique valid WCS
    location/start/endpoint components become explicit values; absent,
    lexically invalid, or duplicate required components remain typed invalid.
    Absent extrusion X/Y/Z components receive the reviewed `0/0/1` defaults
    independently, while explicit invalid or duplicate extrusion evidence stays
    invalid. No coordinate-system transformation or vector normalization occurs.
    M8.2a indexes exact uppercase `CIRCLE` and `ARC` records in complete
    `BLOCKS` and `ENTITIES` sections through a separate OCS-aware evidence
    directory. It retains center `10/20/30`, radius `40`, ARC start/end angle
    `50/51`, and extrusion `210/220/230` occurrences in source order with exact
    ASCII/Binary double evidence. It does not select values, apply defaults,
    interpret angle ranges, validate geometry, or transform OCS to WCS.
    M8.2b materializes seven stable role cards for every `CIRCLE` and nine for
    every `ARC`. Each card reports `Absent`, `Unique`, or `Multiple` and owns a
    compact member slice pointing back to M8.2a values. Numeric validity remains
    independent from cardinality; no occurrence is copied, selected, defaulted,
    validated, normalized, or transformed.
    M8.2c lazily projects required OCS center and radius values for CIRCLE and
    ARC, plus required degree-valued start/end angles for ARC. Missing, invalid,
    or duplicate required values remain typed invalid. Absent extrusion X/Y/Z
    independently receives the reviewed `0/0/1` defaults; present invalid or
    duplicate evidence is never hidden. Radius constraints, angle normalization,
    ARC sweep, OCS transformation, and geometry assembly remain deferred.
    M8.3a indexes exact uppercase `ELLIPSE` records in complete `BLOCKS` and
    `ENTITIES` sections through a separate WCS/parameter-aware evidence
    directory. It retains WCS center `10/20/30`, WCS major-axis endpoint vector
    `11/21/31`, minor-to-major axis ratio `40`, start/end parameters `41/42`,
    and extrusion `210/220/230` occurrences in source order with exact
    ASCII/Binary double evidence. It does not select values, apply defaults,
    validate ratio or parameter ranges, assemble an ellipse, or transform
    coordinate systems.
    M8.3b materializes twelve stable role cards for every `ELLIPSE`, covering
    WCS center X/Y/Z, relative major-axis endpoint X/Y/Z, ratio, start/end
    parameters, and extrusion X/Y/Z. Each card reports `Absent`, `Unique`, or
    `Multiple` and owns a compact member slice pointing back to M8.3a values.
    Numeric validity remains independent from cardinality; no occurrence is
    copied, selected, defaulted, validated, normalized, or transformed.
    M8.3c lazily projects required WCS center, relative major-axis endpoint,
    minor-to-major axis ratio, and start/end parameter cards into exact semantic
    values. Missing, invalid, or duplicate required values remain typed invalid.
    Absent extrusion X/Y/Z independently receives the reviewed `0/0/1`
    defaults; present invalid or duplicate evidence is never hidden. Ratio and
    parameter constraints, endpoint/sweep inference, coordinate transformation,
    and ellipse assembly remain deferred.
    M8.4a indexes exact uppercase `RAY` and `XLINE` records in complete
    `BLOCKS` and `ENTITIES` sections through a separate infinite-line evidence
    directory. It retains the RAY start/XLINE first point `10/20/30` and unit
    direction vector `11/21/31` in WCS, in source order, with exact
    ASCII/Binary double evidence. It does not select values, validate
    completeness or unit length, infer extent/direction semantics, validate
    version applicability, or assemble infinite geometry.
    M8.4b materializes six stable role cards for every `RAY` and `XLINE`,
    covering WCS start/first-point X/Y/Z and unit-direction X/Y/Z. Each card
    reports `Absent`, `Unique`, or `Multiple` and owns a compact member slice
    pointing back to M8.4a values. Numeric validity remains independent from
    cardinality; no occurrence is copied, selected, defaulted, validated,
    normalized, or transformed.
    M8.4c lazily projects required WCS start/first-point and unit-direction
    cards for RAY/XLINE into exact semantic values. Missing, invalid, or
    duplicate components remain typed invalid with available raw provenance.
    Non-unit and zero direction triples remain exact; vector validation,
    normalization, direction/extent inference, coordinate transformation, and
    infinite-line assembly remain deferred.
13. M9: polyline, mesh, spline, and helix families. M9.1a indexes exact
    uppercase `LWPOLYLINE` records in complete `BLOCKS` and `ENTITIES`
    sections and retains source-order elevation, thickness, constant width,
    OCS vertex X/Y, per-vertex start/end width, bulge, and extrusion-component
    occurrences with exact ASCII/Binary double evidence. Duplicates, invalid
    ASCII values, and empty floating-value slices remain explicit. Integer
    count, flags, and vertex identifiers remain available in the raw record but
    are not interpreted here. It does not group values into vertices, apply
    defaults, validate count/order/cardinality, transform OCS, or assemble a
    polyline.
    M9.1b indexes every LWPOLYLINE vertex-count `90`, flag `70`, and vertex-
    identifier `91` occurrence in source order. The wire domains remain exact:
    group `70` is signed 16-bit while groups `90/91` are signed 32-bit, with
    typed ASCII syntax/range failures and exact Binary values. Duplicates and
    empty integer slices remain explicit. It does not select a canonical count
    or flag value, interpret flag bits, require non-negative values, associate
    identifiers with vertices, compare declared and observed counts, validate
    version applicability, or assemble a polyline.
    M9.1c conservatively starts one LWPOLYLINE vertex at each exact OCS-X group
    `10` and assigns subsequent OCS-Y `20`, start/end width `40/41`, bulge `42`,
    and vertex identifier `91` evidence through the next group `10`. Six fixed
    cards per vertex report `Absent`, `Unique`, or `Multiple` while compact
    members retain every M9.1a/b occurrence. Vertex-scoped evidence before the
    first group `10` remains in a separate orphan slice rather than being
    guessed onto a vertex. Entity-level values stay outside vertex grouping.
    This does not apply defaults, select values, validate lexical content,
    reconcile declared count, interpret flags/bulges, transform OCS, or
    assemble segments.
    M9.1d lazily projects each grouped vertex into source-anchored semantic
    states. OCS X/Y are required; missing Y, invalid ASCII, or duplicates fail
    typed. Absent local start/end width and bulge fields receive the documented
    zero defaults, while present invalid or duplicate evidence is never hidden.
    Vertex identifiers remain explicit, absent, or invalid without a default.
    Local width defaults do not claim effective width when constant width `43`
    exists. Constant/variable-width precedence, count reconciliation, flag and
    bulge interpretation, OCS transformation, closure, and segment assembly
    remain deferred.
    M9.1e materializes eight stable record-level cards for vertex count `90`,
    flags `70`, OCS elevation `38`, thickness `39`, constant width `43`, and
    extrusion `210/220/230`. Each card reports `Absent`, `Unique`, or
    `Multiple` and owns compact members that resolve to the exact M9.1a/b
    floating or integer evidence. Vertex coordinates `10/20`, local widths
    `40/41`, bulge `42`, and identifier `91` remain excluded from these cards.
    Lexical validity stays independent from occurrence cardinality; no value is
    selected, defaulted, interpreted, reconciled, validated, or transformed.
    M9.1f lazily projects required signed-i32 vertex count, defaulted signed-i16
    flags, defaulted elevation/thickness/constant-width doubles, and defaulted
    extrusion components from the M9.1e cards. Flag helpers expose only the
    documented Closed and Plinegen bits while retaining all raw bits. The
    declared count is compared with the exact number of retained group-10
    anchors as matched, mismatched, or not comparable. Separate width evidence
    reports no explicit width, constant only, variable only, or both; it does
    not choose precedence when constant `43` and per-vertex `40/41` coexist.
    Count-domain validation, effective segment widths, OCS transformation,
    closure, and segment assembly remain deferred.
    M9.1g creates one guaranteed consecutive segment for each adjacent retained
    vertex pair. A unique usable Closed flag additionally creates the documented
    last-to-first segment; absent flags default open, while invalid or duplicate
    flags leave closure indeterminate and never invent that segment. Closed
    records therefore have one segment per retained vertex, including one
    self-closing segment for a single retained vertex. Lazy segment semantics
    bind OCS endpoints, local start/end widths, and bulge to the segment's start
    vertex. Zero bulge reports straight, nonzero bulge reports an arc factor,
    and unusable bulge remains indeterminate. This does not compute arc center,
    radius, sweep, or endpoint transforms; select effective width; validate
    count agreement; transform OCS; edit/write; or render geometry.
    M9.1h maps zero bulge to an exact-endpoint straight OCS segment. Nonzero
    bulge derives a circular center, positive radius, and signed included-angle
    sweep from Autodesk's `bulge = tan(sweep / 4)` definition. Positive and
    negative bulges retain counterclockwise and clockwise orientation. Missing
    or invalid endpoint/bulge semantics, a nonzero-bulge zero chord, and any
    non-finite or unrepresentable intermediate result remain typed failures;
    no NaN or infinity is published. Derived binary64 results are not raw source
    evidence or a cross-platform canonicalization. Effective width, OCS-to-WCS
    transformation, geometric tolerance/conformance, editing, and rendering
    remain deferred.
    M9.2a recognizes exact uppercase classic `POLYLINE` records only in
    complete `BLOCKS` and `ENTITIES` sections. It retains each consecutive
    exact uppercase `VERTEX` record until exact uppercase `SEQEND`, another
    record, or the containing section boundary. Sequence state is explicitly
    `Closed`, `Interrupted`, or `Unclosed`; a closed entry retains its SEQEND,
    and an interrupted entry retains the first unexpected record. The obsolete
    group-66 entities-follow flag is ignored as documented. This is exact
    section-local record topology, not POLYLINE or VERTEX value decoding, flag
    interpretation, polyline/mesh/face classification, coordinate semantics,
    geometry, editing, writing, or rendering.
    M9.2b layers source-order numeric evidence over every M9.2a POLYLINE
    sequence entry. It retains dummy/elevation `10/20/30`, thickness `39`,
    default widths `40/41`, obsolete entities-follow `66`, flags `70`, mesh
    counts/densities/type `71`-`75`, and extrusion `210/220/230` in their exact
    documented double or signed-16-bit wire domains. Duplicates, ASCII lexical
    or range failures, binary values, raw spans, interrupted/unclosed sequence
    states, and empty value slices remain explicit. VERTEX record values cannot
    enter the POLYLINE slice. This does not apply defaults, require dummy zero,
    use group `66`, interpret flags or surface type, validate counts/densities/
    widths/extrusion, classify polyline families, decode vertices, transform
    coordinates, assemble geometry, edit, write, or render.
    M9.2c layers source-order numeric evidence over every VERTEX retained by
    M9.2a. It retains location `10/20/30`, widths `40/41`, bulge `42`, curve-fit
    tangent direction `50`, flags `70`, polyface indices `71`-`74`, and vertex
    identifier `91` in their exact documented double, signed-16-bit, or signed-
    32-bit wire domains. Every entry remains linked to its POLYLINE record and
    sequence-local ordinal; duplicates, ASCII lexical or range failures, exact
    Binary values, raw spans, empty value slices, and closed/interrupted/
    unclosed sequence states remain explicit. POLYLINE and SEQEND payloads
    cannot enter a VERTEX slice. This does not select canonical occurrences,
    apply defaults, interpret flags/bulge/tangent/index signs, classify 2D/3D/
    mesh/polyface vertices, validate coordinates or widths, resolve faces,
    transform OCS/WCS, assemble geometry, edit, write, or render.
    M9.2d assigns thirteen stable per-role cards to every M9.2c VERTEX entry.
    Each card reports `Absent`, `Unique`, or `Multiple` and retains compact
    source-order member references back to the exact M9.2c value occurrences.
    Lexical validity remains independent from cardinality; empty VERTEX records
    receive thirteen absent cards, and closed/interrupted/unclosed sequence
    states retain the same card contract. This does not select or decode a
    canonical value, apply defaults, interpret flags/bulge/tangent/index signs,
    classify vertex families, validate domains, resolve faces, transform
    coordinates, assemble geometry, edit, write, or render.
    M9.2e lazily projects the seven double-role M9.2d cards. Location X/Y/Z are
    required semantic components; absent start/end width and bulge receive only
    their documented zero defaults; absent curve-fit tangent direction remains
    explicitly absent. Unique valid values remain exact, while invalid ASCII,
    missing required components, and duplicate occurrences fail typed with
    available raw provenance. This neutral projection does not decide whether
    location is OCS or WCS, require a tangent from flag bit `2`, interpret flags
    or polyface indices, classify vertex families, validate ranges, resolve
    faces, transform coordinates, assemble geometry, edit, write, or render.
    M9.2f assigns sixteen stable per-role cards to every M9.2b POLYLINE record
    entry. Cards report `Absent`, `Unique`, or `Multiple` and retain compact
    source-order references back to exact M9.2b values. Lexical validity remains
    independent from cardinality; an empty POLYLINE receives sixteen absent
    cards, and VERTEX values remain excluded. Closed/interrupted/unclosed
    sequence states retain the same record-card contract. This does not select
    values, apply defaults, interpret flags or counts, classify polyline
    families, validate domains, transform coordinates, assemble geometry,
    edit, write, or render.
    M9.2g lazily projects fifteen meaningful M9.2f record roles. Dummy X/Y and
    elevation `10/20/30` are required; absent thickness `39`, default widths
    `40/41`, flags `70`, mesh counts/densities/type `71`-`75`, and extrusion
    `210/220/230` receive only their documented zero or `(0, 0, 1)` defaults.
    Unique values preserve exact double bits or signed-i16 values; invalid
    ASCII, missing required components, and duplicates remain typed invalid
    with raw provenance when available. Helpers expose each of the eight
    documented flag bits independently. Obsolete entities-follow `66` stays
    available only through M9.2b/f evidence and is ignored semantically. This
    does not enforce dummy zero, validate numeric domains or smooth-surface
    codes, reconcile contradictory flags, classify polyline families, apply
    mesh metadata to VERTEX records, resolve faces, transform coordinates,
    assemble geometry, edit, write, or render.
    M9.2h lazily projects the six integer-role M9.2d VERTEX cards: flags `70`,
    four polyface indices `71`-`74`, and identifier `91`. Absent fields remain
    explicitly absent because Autodesk publishes no default for them. Unique
    signed-i16/i32 values remain exact; invalid ASCII and duplicates fail typed
    with available raw provenance. Helpers expose the seven meaningful flag
    bits independently while bit `4`, documented as not used, receives no
    semantic helper. This does not require flags or identifiers, interpret
    negative-index edge visibility or zero termination, classify vertex
    families, validate indices, resolve faces, apply parent mesh metadata,
    transform coordinates, assemble geometry, edit, write, or render.
    M9.2i lazily classifies parent flags as 2D, 3D, polygon mesh, or polyface
    mesh, and usable VERTEX flags as 2D, 3D, polygon mesh, polyface coordinate,
    or polyface face evidence. Multiple simultaneous parent/vertex family bits
    remain `Conflicting`; absent, invalid, or duplicate flags remain
    `Unavailable`. Parent/vertex pairs are `Matched`, `Mismatched`, or
    `NotComparable` without precedence. This does not validate subclass or
    version applicability, require every VERTEX to match, interpret indices,
    resolve faces, transform coordinates, assemble geometry, edit, write, or
    render.
    M9.2j builds consecutive segments, plus a last-to-first segment only when
    parent flag bit `1` is usable and set, for complete `SEQEND`-terminated 2D
    or 3D sequences whose every VERTEX family matches. Each segment retains
    exact start/end VERTEX entries and source-local ordinals. Incomplete
    sequences, mesh/polyface parents, indeterminate parents, or inconsistent
    vertices receive typed record states and zero segments. This does not
    project coordinates or effective widths, interpret bulge geometry, apply
    elevation/extrusion transforms, build mesh topology, resolve faces, edit,
    write, or render.
    M9.2k binds each M9.2j segment to exact start/end VERTEX semantics and its
    parent POLYLINE record semantics. It exposes the 2D OCS versus 3D WCS
    boundary, endpoint tuples, start-vertex local widths/bulge/tangent, and
    parent default widths without choosing effective widths or requiring every
    value to be usable. Per-field invalidity remains independent. This does not
    derive arc geometry, resolve width precedence, apply elevation/extrusion
    transforms, tessellate, edit, write, or render.
    M9.2l lazily derives geometry for each M9.2j segment. Classic 2D segments
    use VERTEX X/Y in OCS plus the parent POLYLINE elevation; a zero bulge
    retains a straight segment and a usable nonzero bulge derives finite
    center, radius, and signed sweep. The otherwise-ignored VERTEX Z does not
    alter 2D elevation. Classic 3D segments retain exact WCS endpoints and are
    straight only; a nonzero bulge is a typed contradiction. Missing or
    invalid endpoints/elevation/bulge, zero-length arc chords, and non-finite
    derived values fail typed per segment. This does not transform OCS to WCS,
    validate extrusion, choose effective widths, tessellate, edit, write, or
    render.
    M9.2m normalizes each usable classic 2D POLYLINE extrusion direction and
    applies Autodesk's arbitrary-axis algorithm, including its exact `1/64`
    square polar-cap branch, to transform OCS line/arc start, end, center, and
    elevation into finite WCS coordinates. Transformed arcs retain their
    normalized WCS normal, radius, signed sweep, and bulge. Classic 3D segments
    remain exact native WCS lines and do not depend on their irrelevant
    extrusion field. Source-geometry failure, unavailable extrusion, zero-
    length extrusion, and non-finite derivation remain typed per segment. This
    does not validate planarity against external engines, select effective
    widths, tessellate, edit, write, or render.
    M9.2n resolves effective start and end width independently for every proven
    classic 2D segment. A source-explicit VERTEX `40` or `41`, including exact
    zero, wins; only an omitted/defaulted VERTEX field selects the corresponding
    parent POLYLINE default. Each result retains `Vertex` or `ParentDefault`
    origin. Invalid vertex fields, unusable parent defaults, and classic 3D
    segments remain typed without cross-component fallback. This does not
    validate nonnegative widths, construct wide outlines or joins, tessellate,
    edit, write, or render.
    M9.2o builds row-major quadrilateral cells for complete classic polygon-
    mesh sequences whose parent/vertex family evidence matches and whose
    positive M/N counts multiply to the observed VERTEX count. Each cell
    retains named `(m0,n0)`, `(m0,n1)`, `(m1,n1)`, and `(m1,n0)` VERTEX
    evidence plus independent M/N wrap state from parent flag bits `1` and
    `32`. Incomplete sequences, other/indeterminate families, inconsistent
    vertices, unavailable/nonpositive counts, and count mismatch emit zero
    cells with typed record state. This does not assign face winding or normals,
    project vertex coordinates, apply smoothing metadata, edit, write, or
    render.
    M9.2p partitions every complete family-consistent classic polyface sequence
    into exact coordinate VERTEX (`128|64`) and face-definition VERTEX (`128`)
    ranges. It retains parent groups `71/72` as reported coordinate/face counts,
    independently reports observed counts, and marks coordinate records that
    appear after a face as odd ordering without rejecting them. Incomplete
    sequences, other/indeterminate parents, and inconsistent vertices expose
    typed zero-member states. Reported count disagreement is evidence, not a
    rejection, because readers must tolerate incorrect counts. This does not
    interpret groups `71`-`74` on face records, resolve indices or edge
    visibility, assemble face geometry, edit, write, or render.
    M9.2q resolves each usable face index in groups `71`-`74` against the
    record-local M9.2p coordinate order, including coordinates retained after
    oddly ordered faces. The first explicit zero or omitted slot terminates the
    face; positive/negative indices select the same 1-based coordinate while
    retaining visible/invisible state for the edge beginning at that corner.
    Invalid values, nonzero values after termination, `i16::MIN`, and indices
    outside the retained coordinate range emit typed zero-corner face states.
    Empty, point, line, triangle, and quadrilateral face cardinalities remain
    representable. This does not validate geometric degeneracy or winding,
    assemble coordinate tuples, triangulate, edit, write, or render.
14. M10: blocks, text, hatch, dimensions, leaders, layouts, underlays, and
    exact-opaque ACIS/proxy/custom payloads.
15. M11: immutable atomic transactions, inverse journals, and handle policy.
16. M12: preserve-patch and canonical ASCII/Binary writers with reparse.
17. M13: evidence closure, 1,000-file/10-GB corpus gates, six native receipts,
    SBOM/notices, and DXF Core 1.0 release.

Every item is split into reviewable micro-milestones and stops after its own
passing checkpoint.
