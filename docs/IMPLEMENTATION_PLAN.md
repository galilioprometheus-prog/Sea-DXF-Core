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
coordinate evidence and preserves per-corner edge visibility; M9.2r assembles
the resolved polyface coordinates into exact WCS point tuples with typed
per-corner component failure; M9.2s assembles every proven polygon-mesh cell's
four named VERTEX coordinates into exact WCS corner tuples; M9.2t classifies
polygon-mesh smooth-surface type metadata while retaining exact signed
densities; M10.1a indexes exact BLOCK/member/ENDBLK definition topology in
complete BLOCKS sections without decoding definition fields or member entities;
M10.1b retains all eight documented BLOCK defining-value roles as exact text,
double, or signed-16-bit evidence without selecting occurrences or applying
defaults; M10.1c adds eight fixed cardinality cards per BLOCK with compact
source-order member references independent of lexical validity; M10.1d lazily
projects required names, flags, and base point plus optional xref
path/description without inventing undocumented defaults; M10.1e compares
usable primary/secondary BLOCK names as exact bounded raw bytes while retaining
matched, conflicting, or not-comparable state; M10.1f indexes only matched
names for exact duplicate-preserving missing/unique/ambiguous lookup; M10.1g
retains every documented defining value from exact INSERT records; M10.1h adds
16 fixed per-record cardinality cards with compact source-order members;
M10.1i projects required and Autodesk-defaulted typed INSERT semantics;
M10.1j resolves usable INSERT names against exact matched BLOCK names; M10.1k
requires closed unique targets and rejects reachable recursive expansion;
M10.1l derives finite single-instance BLOCK-to-WCS affine transforms from
eligible targets with typed fail-closed input and application states; M10.1m
derives constant-space rectangular-array layouts and bounded per-index
instance transforms; M10.1n retains exact INSERT/ATTRIB/SEQEND sequence
boundaries under zero, nonzero, and unavailable attributes-follow evidence;
M10.1o retains source-order classic ATTRIB defining values without conflating
AcDbXrecord/AcDbMText extension payloads; M10.1p adds 23 fixed cardinality
cards per classic ATTRIB record with compact source-order members; M10.1q
lazily projects classic ATTRIB double fields with documented defaults and
typed required/optional component states; M10.1r lazily projects required
source-anchored ATTRIB text/tag and the documented STANDARD style default;
M10.1s lazily projects the five unambiguous classic ATTRIB integer roles and
their documented flag/default semantics; M10.1t classifies documented
horizontal/vertical justification codes and alignment-point applicability;
M10.1u selects the applicable source placement tuple without coupling errors
from the ignored tuple; M10.1v projects the selected OCS anchor into WCS with
the shared arbitrary-axis implementation and typed finite-input failures;
M10.1w indexes exact uppercase ATTDEF member records under their owning BLOCK
definitions without decoding fields or associating inserted attributes;
M10.1x retains source-order classic ATTDEF defining values without admitting
application-group or AcDbXrecord/MText extension payloads; M10.1y adds 24
fixed cardinality cards per ATTDEF with compact source-order members; M10.1z
projects classic ATTDEF double fields with documented required/default/optional
states; M10.1aa projects required source-anchored default/prompt/tag text and
the documented STANDARD style default; M10.1ab projects the five unambiguous
classic ATTDEF signed-16-bit roles with required flags, documented zero
defaults, and exact flag-bit helpers while leaving group `280` neutral;
M10.1ac classifies the published ATTDEF horizontal/vertical justification
codes and exposes alignment-point applicability without selecting coordinates;
M10.1ad selects the applicable stored ATTDEF text-start or alignment tuple
without coupling failures from the ignored tuple; M10.1ae projects each usable
selected ATTDEF OCS anchor into WCS through the shared arbitrary-axis basis
with typed finite-input and extrusion failures; M10.1af builds a collision-safe
block-local exact ATTDEF-tag index with duplicate-preserving matches and
explicit unusable-tag summaries; M10.1ag resolves each retained ATTRIB against
the exact ATTDEF tags of a uniquely targeted BLOCK while preserving
fail-closed target, tag, missing, indeterminate, unique, and ambiguous states;
M11.1a introduces immutable source-bound raw-byte transaction plans with
non-conflicting source-order patches and captured inverse bytes; M11.1b
stream-verifies an opened post-image and materializes a source-bound executable
inverse plan with adjacent deletion coalescing; M11.2a adds a fail-closed,
constant-space monotonic object-handle allocation policy from exact
`$HANDSEED` and record-identity evidence; M11.2b atomically plans new
record-identity groups and the successor `$HANDSEED` as one source-bound
transaction; M12.1a streams any immutable transaction plan to a verified
create-new output with cleanup-on-failure; M12.1b strictly reparses that output
and returns an executable inverse journal; M12.2a writes canonical ASCII
framing with strict EOF recovery closure while retaining exact value payloads;
M12.2b writes canonical Binary sentinel/group-code framing for the declared
dialect with the same strict EOF closure and exact non-EOF value wire bytes;
M13.1a locks a deterministic CycloneDX 1.6 inventory to the complete Cargo
resolution, lockfile checksums, dependency edges, and third-party notice rows;
M13.1b packages canonical-LF project/legal notices and every byte-exact root
license artifact from the 26 reviewed crates with a deterministic hash manifest;
M13.2a adds a distinct redacted corpus receipt v2 whose verified state requires
at least 1,000 strictly verified files and 10 GiB with zero invalid inputs;
M13.2b adds fail-closed native artifact assembly and a manual pinned workflow
for all six reviewed host targets without publishing a release; M13.2c
downloads the same-run artifacts, re-verifies every payload against its
receipt and committed evidence, and emits one exact six-target matrix receipt;
M13.2d makes SBOM dependency resolution host-independent by unioning explicit
Cargo metadata for all six reviewed target triples; M13.2e exposes bounded
semantic edge and hash differences for stale SBOM failures; M13.2f
canonicalizes Cargo.lock identity to LF and pins its checkout line endings;
M13.2g records the first successful six-package and aggregate receipt workflow

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
   M6.5o-r7 moves reusable human help/report text out of command construction,
   execution, and rendering into complete compile-time English and Vietnamese
   catalogs. Adding another language no longer expands those business modules;
   JSON v1 remains language-neutral, explicit language selection and English
   fallback behavior remain unchanged, and no dependency or support claim is
   added. The accompanying architecture audit separates handwritten
   production, inline/integration tests, and generated code before identifying
   any size-driven follow-up.
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
    M9.2r assembles each M9.2q corner's referenced coordinate VERTEX groups
    `10/20/30` into an exact WCS point while retaining the resolved signed index
    and edge visibility. Resolution failures and unavailable X/Y/Z components
    emit typed zero-point face states with per-component semantic status. Oddly
    ordered coordinates remain usable through the M9.2p partition. This does
    not use the irrelevant location fields on face-definition VERTEX records,
    derive edges, validate degeneracy/winding/planarity/manifoldness, calculate
    normals, triangulate, edit, write, or render.
    M9.2s assembles each M9.2o cell's named `(m0,n0)`, `(m0,n1)`, `(m1,n1)`,
    and `(m1,n0)` VERTEX groups `10/20/30` into exact WCS points. A missing or
    invalid component emits a typed cell state naming the first unusable corner
    and all three component states. Topology wrap evidence and exact VERTEX
    identity remain attached. This does not assign winding, derive edges,
    validate degeneracy/planarity, calculate normals, apply smoothing metadata,
    triangulate, edit, write, or render.
    M9.2t retains exact signed smooth-surface M/N density semantics from groups
    `73/74` and classifies group `75` only as none, quadratic B-spline, cubic
    B-spline, or Bezier. Invalid metadata, unsupported type codes, and M9.2o
    topology failures remain typed. No undocumented density range is imposed.
    This does not generate fitted vertices/cells, alter M9.2o topology or M9.2s
    WCS corners, validate surface continuity, tessellate, edit, write, or render.
14. M10: blocks, text, hatch, dimensions, leaders, layouts, underlays, and
    exact-opaque ACIS/proxy/custom payloads.
    M10.1a recognizes exact uppercase `BLOCK` and `ENDBLK` markers only in
    complete `BLOCKS` sections. Every definition retains all intervening
    group-zero member records in source order. Exact `ENDBLK` closes the
    definition; another exact `BLOCK` interrupts it and begins a separately
    indexed definition; section end leaves it unclosed. Orphan/non-exact
    markers and BLOCK/ENDBLK spelling outside `BLOCKS` do not create topology.
    This does not decode names, flags, base points, xref paths, descriptions,
    handles, member entities, INSERT references, transforms, edits, writes, or
    rendering.
    M10.1b layers source-order defining-value evidence over every M10.1a BLOCK
    record. It retains primary/secondary names `2/3`, flags `70`, base point
    `10/20/30`, xref path `1`, and optional description `4` in their exact text,
    double, or signed-16-bit wire domains. Text stays source-anchored with the
    document encoding resolution and decodes only on explicit request without
    replacement. Duplicates, empty text, ASCII numeric failures, exact Binary
    values, raw spans, and every definition state remain explicit. Values inside
    group-102 application-control payloads, member entities, and ENDBLK cannot
    enter the BLOCK slice. This does not select names, reconcile groups `2/3`,
    apply defaults, interpret flags, assemble the base point, resolve paths,
    inspect members, bind INSERT, transform geometry, edit, write, or render.
    M10.1c assigns eight stable per-role cards to every M10.1b BLOCK record.
    Each card reports `Absent`, `Unique`, or `Multiple` and retains compact
    source-order member references back to the exact M10.1b value occurrences.
    Text/numeric validity remains independent from cardinality; empty BLOCK
    records receive eight absent cards, and closed/interrupted/unclosed
    definition states retain the same card contract. This does not select or
    decode a canonical value, reconcile names, apply defaults, interpret flags,
    assemble points, resolve xrefs, bind INSERT, transform geometry, edit,
    write, or render.
    M10.1d lazily projects all eight M10.1c cards. Primary/secondary names,
    flags, and base-point X/Y/Z are required; xref path and description remain
    explicitly absent when omitted. Unique text remains source-anchored and
    unique numeric values preserve exact double/i16 domains. Invalid ASCII,
    missing required fields, and duplicates fail typed with raw provenance when
    available. Helpers expose all seven documented flag bits independently,
    including the two bits Autodesk says are ignored on input, and assemble a
    base-point tuple only when all three components are usable. This does not
    require an xref path from flag bit `4`, reconcile names, validate empty
    text or numeric finiteness, resolve xrefs, bind INSERT, transform member
    geometry, edit, write, or render.
    M10.1e compares each M10.1d usable primary name `2` with its usable
    secondary name `3` as exact same-document raw bytes. Equal bytes are
    `Matched`, unequal bytes are `Conflicting`, and missing, duplicate, or
    otherwise unusable names are `NotComparable`. Comparison is bounded in
    fixed 4-KiB chunks and performs no decoding, case folding, normalization,
    or allocation proportional to an untrusted name. Empty names and all
    M10.1a definition states remain representable. This evidence does not
    choose a canonical name, reject a conflict, build a name index, resolve
    INSERT, apply xref policy, edit, write, or render.
    M10.1f admits only M10.1e `Matched` records to a source-anchored exact-name
    index. SHA-256 digests group candidates without retaining untrusted name
    bytes; every digest candidate is rechecked against exact source bytes in
    fixed 4-KiB chunks. Lookup preserves all duplicates and returns `Missing`,
    `Unique`, or `Ambiguous` without decoding, case folding, normalization, or
    choosing among duplicates. Empty names and all M10.1a definition states
    remain indexable evidence. Conflicting and not-comparable records remain
    available through the retained consistency directory but cannot become
    lookup targets. This evidence does not resolve INSERT, validate a legal
    block namespace, apply xref policy, edit, write, or render.
    M10.1g discovers exact uppercase `INSERT` records in completely indexed
    BLOCKS and ENTITIES sections and retains documented block name `2`,
    insertion point `10/20/30`, scales `41/42/43`, rotation `50`, array counts
    `70/71`, array spacing `44/45`, attributes-follow `66`, and extrusion
    `210/220/230`. Values remain in source order with duplicate and invalid
    ASCII-number evidence; group `102` application content cannot impersonate
    an INSERT field. Text stays source-backed and numbers preserve exact
    binary64 or signed-16-bit domains. This evidence does not apply documented
    defaults, select duplicate values, assemble typed INSERT semantics, bind a
    block name, follow ATTRIB/SEQEND, transform geometry, edit, write, or
    render.
    M10.1h publishes 16 fixed cards per exact M10.1g INSERT record in documented
    role order. Each card independently reports `Absent`, `Unique`, or
    duplicate-preserving `Multiple`, and compact members reference every
    source-order M10.1g value occurrence. Cardinality remains independent of
    ASCII lexical validity and record-local across BLOCKS/ENTITIES. This
    evidence does not apply defaults, select a duplicate, assemble typed
    semantics, resolve a block name, follow ATTRIB/SEQEND, transform geometry,
    edit, write, or render.
    M10.1i lazily projects the required block name and insertion point plus
    Autodesk-documented optional defaults: scale `(1,1,1)`, rotation `0`,
    column/row counts `(1,1)`, spacing `(0,0)`, attributes-follow `0`, and
    extrusion `(0,0,1)`. Every field retains explicit/defaulted/invalid state
    and raw/field provenance; invalid ASCII numbers and multiple values remain
    invalid instead of falling back to defaults. Composite accessors fail
    closed when any component is unusable. This evidence does not validate
    empty names, numeric finiteness, count ranges, or attributes-follow values;
    resolve a block name; follow ATTRIB/SEQEND; form an OCS transform; edit,
    write, or render.
    M10.1j resolves each usable M10.1i INSERT block-name source span against the
    M10.1f exact matched-name index without allocating a name-sized query
    buffer. Resolution preserves `UnusableName`, `Missing`, `Unique`, or
    duplicate-preserving `Ambiguous` state and retains every exact target in
    source-record order. Hash candidates are rechecked against same-document
    source bytes in fixed 4-KiB chunks. Conflicting or otherwise unindexable
    BLOCK names cannot become targets. This evidence does not require a target
    to be a closed definition, choose among ambiguous targets, detect recursive
    references, follow ATTRIB/SEQEND, form an OCS transform, edit, write, or
    render.
    M10.1k derives a BLOCK expansion graph from uniquely resolved INSERT member
    records and classifies every M10.1j entry as not uniquely resolved,
    target-definition-not-closed, recursive expansion, or eligible. Only
    `Closed` unique targets are traversable. Iterative three-color graph
    traversal detects direct self-reference, indirect cycles, and cycles
    reachable below an ENTITIES INSERT with cancellation and memory bounded by
    indexed definitions. This evidence does not validate numeric/count domains,
    follow ATTRIB/SEQEND, calculate OCS axes or a transform matrix, expand
    geometry, edit, write, or render.
    M10.1l derives one finite row-major 3x4 affine matrix for each M10.1k
    eligible INSERT. The matrix subtracts the target BLOCK base point, applies
    documented per-axis scale and rotation in the INSERT OCS, adds the
    insertion point, then maps OCS to WCS with Autodesk's arbitrary-axis
    algorithm. Derived negative zero is canonicalized for stable exact
    evidence. Unavailable typed semantics/base points, non-finite inputs,
    zero-length extrusion, non-finite matrix results, and non-finite point
    application remain typed failures. This evidence covers one INSERT
    instance only: it does not expand row/column arrays, convert block units,
    follow ATTRIB/SEQEND, transform member geometry recursively, edit, write,
    or render.
    M10.1m derives a constant-space rectangular-array layout from each usable
    M10.1l transform plus the documented column/row counts and spacing.
    Column and row step vectors follow the INSERT OCS axes after its rotation,
    because Autodesk specifies that MINSERT rotation applies to the individual
    insertions and the entire array. SeaCad treats the documented spacing as
    drawing-unit array offsets independent of BLOCK scale. Counts must be
    positive and spacing finite. The layout exposes an exact
    `u64` instance count and computes only a requested in-range row/column
    transform, so maximum signed-16-bit counts do not allocate an expanded
    array. Placement overflow remains typed. This evidence does not enumerate
    every instance, convert BLOCK units, follow ATTRIB/SEQEND, recursively
    transform member geometry, edit, write, or render.
    M10.1n evaluates every M10.1i attributes-follow semantic while retaining
    its exact signed-16-bit value. Zero consumes no following records.
    Any nonzero value scans consecutive exact uppercase ATTRIB records in the
    same complete BLOCKS/ENTITIES section and retains the exact SEQEND,
    first interrupting record, or section-end boundary. This reconciles the
    INSERT page's value-`1` wording with the SEQEND page's nonzero wording
    without normalizing the flag. Invalid or duplicate flags remain
    `FlagUnavailable` and consume nothing. This evidence does not decode
    ATTRIB fields, associate ATTDEF definitions, validate ownership, apply
    attribute transforms, expand geometry, edit, write, or render.
    M10.1o layers source-order classic ATTRIB value evidence over every exact
    attribute record retained by M10.1n. It retains thickness, OCS text start,
    height/value/tag, attribute and text flags, field length, rotation, width,
    oblique angle, style, justification, optional alignment point, extrusion,
    and both source occurrences of group `280` in exact text, binary64, or
    signed-16-bit wire domains. Because Autodesk assigns group `280` both
    version and lock-position roles without a distinct code, M10.1o keeps a
    neutral `VersionOrLockPosition` role rather than inferring order.
    Subclass tracking admits legacy, `AcDbText`, and `AcDbAttribute` payloads,
    ignores other subclass contexts, and stops before `AcDbXrecord`; group
    `102` application content is excluded. This does not assign cardinality,
    select values, apply defaults, validate text/numeric domains, interpret
    flags/justification, decode the MText extension, associate ATTDEF
    definitions, transform attributes, edit, write, or render.
    M10.1p publishes 23 fixed cards per M10.1o ATTRIB record in documented
    classic role order. Each independently reports `Absent`, `Unique`, or
    duplicate-preserving `Multiple`, with compact members referencing every
    source-order M10.1o occurrence. Cardinality remains independent of ASCII
    lexical validity, sequence boundary state, and sequence-local record
    ordinal; an empty ATTRIB still receives 23 absent cards. The neutral
    group-280 role can therefore report two occurrences without selecting
    version versus lock position by order. This does not select/decode a
    canonical value, apply defaults, distinguish group-280 meanings, interpret
    flags/justification, decode MText extensions, associate ATTDEF definitions,
    transform attributes, edit, write, or render.
    M10.1q lazily projects the 14 classic ATTRIB double roles from M10.1p.
    Text-start X/Y/Z and text height are required. Thickness, rotation,
    relative X scale, oblique angle, and extrusion receive only their
    documented `0`, `0`, `1`, `0`, and `(0,0,1)` defaults. Alignment-point
    X/Y/Z remain independently optional because their applicability depends on
    justification semantics not yet interpreted. Unique values preserve exact
    binary64 bits; invalid ASCII, missing required values, and duplicates stay
    typed with raw provenance when available. Tuple helpers fail closed unless
    all three components are usable. This does not validate finiteness,
    positivity, angles, justification-dependent alignment requirements, or
    extrusion length; project text/flags/integers; decode MText extensions;
    associate ATTDEF definitions; transform attributes; edit, write, or
    render.
    M10.1r lazily projects the three classic ATTRIB text roles from M10.1p.
    Text/default value `1` and attribute tag `2` are required source-anchored
    values. An absent text-style name `7` receives the documented `STANDARD`
    default as typed semantic data without inventing raw provenance; an
    explicit style remains source-anchored. Missing required roles and
    duplicates fail typed with raw provenance when available. Decoding remains
    explicit, bounded, tied to the same document, and replacement-free through
    the M10.1o text view. This does not reject empty values or spaces in tags,
    compare style-table names, interpret formatting/escapes, project numeric
    fields, decode MText extensions, associate ATTDEF definitions, transform
    attributes, edit, write, or render.
    M10.1s lazily projects the five unambiguous classic ATTRIB signed-16-bit
    roles from M10.1p. Attribute flags `70` are required; field length `73`,
    text-generation flags `71`, horizontal justification `72`, and vertical
    justification `74` receive only their documented zero defaults. Helpers
    expose the four documented attribute-flag bits and the two documented text-
    generation bits while retaining the exact underlying signed value and any
    unknown bits. Invalid ASCII, missing required flags, and duplicates stay
    typed with raw provenance when available. Neutral group `280` evidence is
    deliberately not selected because its version and lock-position meanings
    share one wire code and record order is not authoritative. This does not
    validate field length or unknown flag bits, classify justification codes,
    determine alignment-point applicability, distinguish group-280 meanings,
    decode MText extensions, associate ATTDEF definitions, transform
    attributes, edit, write, or render.
    M10.1t classifies usable M10.1s horizontal justification `72` as left,
    center, right, aligned, middle, or fit and vertical justification `74` as
    baseline, bottom, middle, or top. Exact signed codes and explicit/defaulted
    provenance remain attached through typed semantic values. Codes outside
    Autodesk's published domains fail typed instead of being normalized.
    Alignment-point applicability is `true` when either classified code is
    nonzero and `false` only when both are usable zero values; unavailable or
    unsupported codes keep applicability unknown. This does not validate
    horizontal/vertical combinations, require or select coordinate tuples,
    recalculate text placement, measure styled text, decode MText extensions,
    associate ATTDEF definitions, transform attributes, edit, write, or
    render.
    M10.1u joins M10.1t justification applicability with M10.1q double
    semantics to select one exact OCS placement anchor per ATTRIB. Usable
    baseline/left justification selects required text-start `10/20/30`;
    any usable nonzero horizontal or vertical justification selects optional
    alignment point `11/21/31`. The selected tuple must have three usable
    components, while invalid or missing components in the unselected tuple do
    not contaminate the result. Unavailable justification, text start, and
    alignment point remain distinct typed states, and successful anchors retain
    exact binary64 components plus both underlying semantic views. This does
    not recalculate AutoCAD's stored points, validate horizontal/vertical
    combinations, apply extrusion, rotation, style metrics, or INSERT
    transforms, decode MText extensions, associate ATTDEF definitions, edit,
    write, or render.
    M10.1v projects each usable M10.1u OCS anchor into WCS using the ATTRIB's
    usable M10.1q extrusion and the same normalized arbitrary-axis
    implementation used by INSERT transforms, including Autodesk's exact
    `1/64` polar-cap branch. Successful projections retain the finite WCS point,
    normalized WCS normal, positive-zero canonicalization, and underlying
    placement evidence. Placement/extrusion unavailability, non-finite Binary
    inputs, zero-length extrusion, non-finite basis derivation, and transformed
    overflow remain typed. This does not apply text rotation, oblique/width/
    generation flags, style metrics, INSERT/BLOCK transforms, or ATTDEF
    association; recalculate stored points; decode MText extensions; edit,
    write, or render.
    M10.1w indexes exact uppercase `ATTDEF` group-zero records only when they
    occur in the retained member range of an M10.1a BLOCK definition. Every
    entry retains the exact raw record, owning definition and its
    closed/interrupted/unclosed state, zero-based member position, and
    definition-local ATTDEF position. Records outside BLOCK definitions and
    non-exact marker spellings remain excluded. Complete-section publication,
    cancellation, source identity, and bounded ordinal lookups are inherited
    from the BLOCK topology. This evidence does not decode ATTDEF fields,
    interpret attribute flags, compare tags, associate ATTRIB records, apply
    INSERT/BLOCK transforms, edit, write, or render.
    M10.1x layers source-order classic ATTDEF value evidence over every exact
    definition record retained by M10.1w. It retains thickness, OCS text start,
    height/default value, prompt/tag, attribute and text flags, field length,
    rotation, width, oblique angle, style, justification, optional alignment
    point, extrusion, and both source occurrences of group `280` in exact text,
    binary64, or signed-16-bit wire domains. Because Autodesk assigns group
    `280` both version and lock-position roles without a distinct code, M10.1x
    keeps a neutral `VersionOrLockPosition` role rather than inferring order.
    Subclass tracking admits legacy, `AcDbText`, and
    `AcDbAttributeDefinition` payloads, ignores other subclass contexts, and
    stops before `AcDbXrecord`; group `102` application content is excluded.
    This does not assign cardinality, select values, apply defaults, validate
    text/numeric domains, interpret flags/justification, decode the MText
    extension, compare ATTRIB tags, associate inserted attributes, transform
    geometry, edit, write, or render.
    M10.1y publishes 24 fixed cards per M10.1x ATTDEF record in documented
    classic role order. Each independently reports `Absent`, `Unique`, or
    duplicate-preserving `Multiple`, with compact members referencing every
    source-order M10.1x occurrence. Cardinality remains independent of ASCII
    lexical validity, owning BLOCK boundary state, and definition-local ATTDEF
    ordinal; an empty ATTDEF still receives 24 absent cards. The neutral
    group-280 role can therefore report two occurrences without selecting
    version versus lock position by order. This does not select/decode a
    canonical value, apply defaults, distinguish group-280 meanings, interpret
    flags/justification, decode MText extensions, compare ATTRIB tags,
    associate inserted attributes, transform geometry, edit, write, or render.
    M10.1z lazily projects the 14 classic ATTDEF double roles from M10.1y.
    Text-start X/Y/Z and text height are required. Thickness, rotation,
    relative X scale, oblique angle, and extrusion receive only their
    documented `0`, `0`, `1`, `0`, and `(0,0,1)` defaults. Alignment-point
    X/Y/Z remain independently optional because their applicability depends on
    justification semantics not yet interpreted. Unique values preserve exact
    binary64 bits; invalid ASCII, missing required values, and duplicates stay
    typed with raw provenance when available. Tuple helpers fail closed unless
    all three components are usable. This does not validate finiteness,
    positivity, angles, justification-dependent alignment requirements, or
    extrusion length; project ATTDEF text/flags/integers; decode MText
    extensions; compare ATTRIB tags; associate inserted attributes; transform
    geometry; edit, write, or render.
    M10.1aa lazily projects the four classic ATTDEF text roles from M10.1y.
    Default value `1`, prompt `3`, and attribute tag `2` are required
    source-anchored values. An absent text-style name `7` receives the
    documented `STANDARD` default as typed semantic data without inventing raw
    provenance; an explicit style remains source-anchored. Missing required
    roles and duplicates fail typed with raw provenance when available.
    Decoding remains explicit, bounded, tied to the same document, and
    replacement-free through the M10.1x text view. This does not reject empty
    text or spaces in tags, compare style-table names, interpret formatting/
    escapes, project ATTDEF integer fields, decode MText extensions, compare
    ATTRIB tags, associate inserted attributes, transform geometry, edit,
    write, or render.
    M10.1ab lazily projects the five unambiguous classic ATTDEF signed-16-bit
    roles from M10.1y. Attribute flags `70` are required. Absent field length
    `73`, text-generation flags `71`, horizontal justification `72`, and
    vertical justification `74` receive only their documented zero defaults.
    Unique values preserve the exact signed wire value, including unknown flag
    bits; helpers test the four documented attribute bits and two documented
    text-generation bits without rewriting the value. Invalid ASCII, missing
    required flags, and duplicates stay typed with raw provenance when
    available. The overloaded group `280` remains card-level evidence and is
    intentionally not selected. This does not validate field-length or
    justification ranges, classify justification or alignment applicability,
    distinguish group-280 meanings, decode MText extensions, compare ATTRIB
    tags, associate inserted attributes, transform geometry, edit, write, or
    render.
    M10.1ac lazily classifies usable M10.1ab horizontal justification `72` as
    left, center, right, aligned, middle, or fit and vertical justification
    `74` as baseline, bottom, middle, or top. Explicit/defaulted state and raw
    provenance remain intact; unsupported signed codes and nested integer
    failures remain typed. Alignment-point applicability is true when either
    classified code is nonzero, false only when both are usable zero values,
    and unavailable otherwise. This does not validate horizontal/vertical
    combinations, require or select text-start/alignment tuples, recalculate
    placement, measure styled text, decode MText extensions, compare ATTRIB
    tags, associate inserted attributes, transform geometry, edit, write, or
    render.
    M10.1ad lazily selects one placement anchor from M10.1z using M10.1ac
    applicability. Usable baseline/left justification selects text start;
    either usable nonzero code selects alignment point. The selected tuple must
    have all three usable components. Unavailable justification, unavailable
    text start, and unavailable alignment point remain distinct states;
    invalid or missing components in the ignored tuple remain inspectable but
    do not contaminate the selected anchor. This does not recalculate stored
    points, validate justification combinations, apply extrusion, rotation,
    text-style metrics, or INSERT/BLOCK transforms, decode MText extensions,
    compare ATTRIB tags, associate inserted attributes, edit, write, or render.
    M10.1ae lazily projects each usable M10.1ad OCS anchor through the usable
    M10.1z extrusion by reusing the shared arbitrary-axis implementation.
    Successful entries retain a finite WCS point, normalized finite normal,
    canonical positive zero, and complete placement evidence. Placement or
    extrusion unavailability, non-finite Binary inputs, zero-length extrusion,
    non-finite basis derivation, and transformed overflow remain distinct
    typed failures. This does not apply text rotation, oblique/width/generation
    flags, text-style metrics, INSERT/BLOCK transforms, or ATTRIB association.
    It does not recalculate stored points, decode MText extensions, edit, write,
    or render.
    M10.1af indexes every usable M10.1aa ATTDEF tag under its exact owning
    BLOCK. SHA-256 narrows candidates, exact bounded raw-span comparison
    confirms collision-safe equality, and duplicate tags remain ordered by
    definition-local ordinal. Every BLOCK publishes total, indexed, and
    unusable ATTDEF-tag counts, including empty blocks. Same-document source
    spans can drive lookup without a tag-sized buffer. This does not validate
    tag syntax, uppercase or decode tags, collapse duplicates, require a closed
    BLOCK, interpret a missing exact match as definitive when unusable tags
    exist, resolve INSERT targets, associate ATTRIB records, edit, write, or
    render.
    M10.1ag resolves each retained M10.1r ATTRIB tag only when its owning
    INSERT has one exact M10.1j BLOCK target. Same-document source spans query
    M10.1af without a tag-sized buffer. Results preserve target-unavailable,
    tag-unavailable, definitive missing, indeterminate missing caused by
    unusable ATTDEF tags, unique, or duplicate-preserving ambiguous state.
    Exact matches remain ordered by definition-local ordinal and retain their
    target BLOCK. This does not normalize/decode tags, choose among duplicate
    BLOCKs or ATTDEFs, require a closed attribute sequence or BLOCK definition,
    validate ownership, compare flags/default values, transform attribute
    placement, edit, write, or render.
15. M11: immutable atomic transactions, inverse journals, and handle policy.
    M11.1a uses a mutable builder only as a bounded construction boundary, then
    freezes an immutable plan bound to the opened source identity, length, and
    physical format. Each replace/delete/insertion patch retains its exact
    source span, owned replacement bytes, and source bytes captured for a
    future inverse journal. Patches are published in source order; overlapping
    nonempty spans, insertions inside replaced spans, and duplicate insertions
    at one offset fail with stable typed errors, while boundary insertions are
    deterministic. Per-patch, patch-count, accumulated journal, and projected
    snapshot limits reuse the selected resource profile. Cancellation before
    or after inverse capture leaves the builder unchanged. This checkpoint
    does not apply a plan, calculate post-image source identity, materialize a
    directly executable inverse plan, validate DXF syntax or semantics, assign
    handles, write a destination, or publish a new snapshot.
    M11.1b accepts the original raw document plus an independently opened
    same-format post-image. It first enforces the M11.1a source precondition and
    exact projected length, then streams unchanged source ranges and owned
    replacement bytes against the post-image in fixed 4-KiB chunks. Stable
    typed errors retain length mismatch or the first differing post-image byte
    without exposing payloads. Exact post-image spans are mapped back to the
    captured original bytes to produce another immutable M11.1a plan bound to
    the post-image identity. Adjacent forward deletions that map to one inverse
    insertion offset are coalesced in original source order. Applying that
    inverse later projects the original length, and materializing its inverse
    yields an exact redo plan. This checkpoint does not write/apply either
    plan, open unvalidated bytes, allocate handles, choose a destination,
    perform filesystem replacement, or publish a snapshot.
    M11.2a treats Autodesk's next-available `$HANDSEED` and drawing-local
    unique object handles as allocation preconditions. It requires one parsed,
    nonzero seed, rejects invalid/multiple/null/duplicate object identities,
    and requires the seed to be strictly above every occupied identity.
    Eligible requests produce a constant-space consecutive handle range and
    representable successor seed under the selected record limit; arithmetic
    exhaustion remains a typed outcome. The policy does not repair stale
    seeds, reuse gaps, reserve a proposal, validate reference topology, encode
    handles, update `$HANDSEED`, apply a transaction, write a destination, or
    publish a snapshot.
    M11.2b accepts caller-ordered raw-record ordinals only when M11.2a is ready
    and every target has absent identity evidence. It rejects duplicate,
    missing, already-identified, CLASSES, ENDTAB, and structurally incomplete
    TABLES targets before a plan escapes. Group 5 is inserted after the record
    marker for BLOCKS/ENTITIES/OBJECTS and after group 2 for table objects and
    entries; exact TABLES `DIMSTYLE` entries alone use group 105. ASCII
    insertion preserves the anchor's CR/LF/CRLF ending, while Binary insertion
    uses the declared pre-R13 or R13+ group-code width and NUL-terminated
    uppercase hexadecimal. The successor `$HANDSEED` and all identities are
    emitted in one immutable M11.1a plan whose materialized bytes re-open with
    M11.2a ready. This checkpoint does not discover edit intent, assign handles
    to CLASSES or incomplete table records, validate reference topology, apply
    a transaction, write or replace a destination, or publish a snapshot.
16. M12: preserve-patch and canonical ASCII/Binary writers with reparse.
    M12.1a validates the transaction source precondition before creating a
    destination, then reads the complete current source in fixed 64-KiB chunks
    while emitting unchanged ranges and owned replacements in source order.
    The source is rehashed during application, the projected output is hashed
    while written, and the flushed/synced create-new file is reopened to verify
    exact length and identity. Existing destinations are never modified; any
    cancellation or failure after creation attempts to remove the incomplete
    output. The receipt retains source/output identities, byte count, and patch
    count. This does not replace an existing path, calculate or return an
    executable inverse from the reopened output, select edit intent, perform
    canonical serialization, or publish a snapshot.
    M12.1b composes M12.1a with an internal strict reopen using the plan's exact
    ASCII/Binary physical format and the caller-selected resource profile. The
    reparsed identity must equal the verified write receipt before M11.1b
    stream-verifies the post-image and materializes an executable inverse plan.
    The returned journal owns that inverse plus the M12.1a receipt; applying
    the inverse through the same API restores a strictly reparsed original and
    yields an exact redo journal. Reparse and inverse failures remove the new
    output. The writer's progress interval remains the M12.1a source-apply plus
    output-verification work; the internal strict reparse uses a no-op
    observer. This does not replace an existing path, provide crash-atomic
    rename, canonicalize either format, infer edits, or publish a snapshot.
    M12.2a projects any opened ASCII raw document into minimal signed-decimal
    group codes, one LF after each code and value, and the exact original value
    payload bytes. The three reviewed Compatible envelope recoveries become one
    strict terminal `0`/`EOF`: padded/BOM EOF is normalized, missing EOF is
    appended, and opaque trailing bytes are omitted. A preflight computes exact
    output length/group count and enforces the selected source/value/record
    limits. Writing hashes the complete live source, verifies and syncs the
    create-new output, then strictly reparses it before returning source/output
    identities, counts, and envelope action. This canonicalizes ASCII physical
    framing only; it does not normalize text, numeric, handle, binary-chunk, or
    semantic value payloads, convert Binary input, replace a path, or publish a
    snapshot.
    M12.2b projects any opened Binary raw document into the exact 22-byte
    sentinel followed by canonical dialect-specific group codes: AC1009 uses
    one byte with the reviewed three-byte XDATA escape and AC1012+ uses signed
    16-bit little-endian codes. Exact non-EOF value wire bytes remain
    authoritative, including string terminators, binary-chunk length prefixes,
    and fixed-width numeric bits. Compatible missing EOF appends the canonical
    group-code `0` plus `EOF\0`; opaque trailing bytes are omitted. Preflight,
    complete live-source hashing, create-new output verification/sync, strict
    reparse, cleanup-on-failure, and the typed receipt match M12.2a while also
    retaining the selected group-code encoding. This canonicalizes Binary
    physical framing only; it does not normalize value semantics, convert
    ASCII input, replace a path, or publish a snapshot.
17. M13: evidence closure, 1,000-file/10-GB corpus gates, six native receipts,
    SBOM/notices, and DXF Core 1.0 release.
    M13.1a generates a deterministic CycloneDX 1.6 SBOM from the complete
    `cargo metadata --locked` resolution. Every registry component is joined
    to its exact `Cargo.lock` SHA-256 checksum, crates.io package URL, declared
    SPDX license expression, dependency edges, and an exact package/version
    row in `THIRD_PARTY_NOTICES.md`; workspace components retain the
    proprietary license reference. The committed SBOM records the lockfile
    identity and pinned Rust version but omits timestamps, host paths, and
    machine-specific identifiers. `--check` is a required gate on all six CI
    platforms. This closes deterministic locked dependency inventory only; it
    does not yet package standalone license texts, prove a distributable
    archive, satisfy the private corpus threshold, advance the current 2/20
    consecutive six-native nightly receipts, or authorize Core 1.0 release.
    M13.1b derives a distributable legal directory from the same locked Cargo
    resolution. It retains canonical-LF project `LICENSE`, `NOTICE`, and
    `THIRD_PARTY_NOTICES.md` plus every byte-exact regular root file beginning with
    LICENSE, LICENCE, COPYING, UNLICENSE, or NOTICE from each of the 26
    crates.io source trees. A deterministic manifest binds all 54 package
    license artifacts to package/version, declared SPDX expression, exact
    crate archive checksum, per-file byte count/SHA-256, and the lockfile
    identity. The freshness gate rejects missing, changed, unexpected,
    non-regular, or symlinked committed legal artifacts. This closes legal-file
    packaging for the locked graph only; it does not construct/sign native
    archives, validate an installer, satisfy corpus/native receipts, or
    authorize Core 1.0 release.
    M13.2a preserves the Q2.2 ceiling-only manifest/receipt v1 and adds a
    release-only v2 contract. The committed v2 policy requires at least 1,000
    strictly verified DXF files and 10 GiB of selected bytes with zero invalid
    files, under independent hard traversal ceilings of 2,000 files, 20 GiB,
    20,000 entries, and depth 32. Its aggregate-only receipt exposes separate
    file-count, byte-count, and zero-invalid threshold Booleans plus their
    conjunction. A threshold shortfall produces a completed redacted failed
    receipt without inventing a parser failure. Requirements above configured
    bounds or outside implementation ceilings fail before traversal. This
    creates the real release gate but does not claim that a private corpus has
    met it, identify corpus contents, provide a cryptographic corpus
    commitment, close six-native receipts, or authorize Core 1.0 release.
    M13.2b adds one native artifact directory contract for each reviewed
    Linux, Windows, and macOS x64/ARM64 target. A bounded Rust packager rejects
    unreviewed targets, invalid commit identities, existing destinations,
    symlinked/non-regular inputs, and non-portable paths; it copies the native
    CLI, README, deterministic SBOM, and complete legal bundle, then records
    every payload byte count and SHA-256 in a sorted receipt bound to target,
    commit, version, and Rust 1.97.1. A manually dispatched, read-only GitHub
    workflow builds natively on the same six hosted runners and uploads each
    directory with exact action pins. This implements artifact staging only:
    no workflow run is claimed, no GitHub Release or tag is created, no
    archive/service digest is treated as the payload receipt, no signing or
    reproducible-build claim is made, and corpus/nightly/final authorization
    remain open.
    M13.2c adds a bounded verifier after all six native package jobs. It
    requires exactly one package for every reviewed target and rejects target,
    package, version, Rust, or source-commit disagreement. Every downloaded
    regular payload is matched to the strictly sorted per-artifact receipt and
    streamed through SHA-256; symlinks, special entries, missing/extra paths,
    excessive depth/count/bytes, and altered payloads fail closed. README,
    SBOM, and the complete legal path/hash set are additionally compared with
    the checkout at the same commit rather than trusted from the artifact
    receipt alone. One aggregate receipt binds the six per-artifact receipt
    hashes and payload totals. The same manually dispatched read-only workflow
    uploads that aggregate for 14 days using exact action pins. This closes the
    local verification process definition, not evidence that the workflow ran,
    permanent retention, signing, reproducibility, corpus/nightly closure, or
    Core 1.0 authorization.
    The M13.2c push exposed a remote Windows ARM64 finding in the M13.1a
    freshness gate. M13.2d hardened the graph by invoking locked metadata with
    an explicit platform filter for every reviewed Linux, Windows, and macOS
    x64/ARM64 triple, verifying one workspace-member set, unioning package
    identities and resolved dependency edges, and sorting the result. CI run
    `30556204169` showed that this graph hardening alone did not close the
    finding. M13.2e added bounded stale-SBOM hashes and semantic edge deltas;
    run `30556646352` proved both edge sets identical. Replacing only the
    committed LF Cargo.lock hash with the checkout's CRLF hash reproduced the
    remote generated SBOM hash exactly. M13.2f therefore canonicalizes
    Cargo.lock bytes to LF before hashing, rejects lone carriage returns, and
    pins `Cargo.lock text eol=lf` in `.gitattributes`. These checkpoints close
    the diagnosed cross-checkout generation semantics subject to a fresh
    remote run; they do not broaden platform support or close the
    corpus/nightly/final release gates.
    M13.2g records GitHub Actions Native Release Artifacts run `30557566354`
    at commit `222eec2c9d9b18fbb7ff1b8d5f0120ba30633365`. All six native
    package jobs succeeded, the aggregate verifier re-hashed and cross-checked
    the exact matrix in 29 seconds, and the overall workflow succeeded in five
    minutes with seven retained artifacts. The aggregate artifact digest is
    `d287c406f7c12e1d0e3686f294b1f4b5cd50567b25d3565c8e6fe0ba1cf6d97d`.
    This closes the first native artifact workflow receipt, not permanent
    retention, signatures, private corpus scale, the twenty-night sequence, or
    final Core 1.0 authorization.
18. M14: complete the documented DXF entity semantic inventory while M13
    release evidence continues independently in the background.
    `docs/DXF_ENTITY_COMPLETION_PLAN.md` defines the 45-topic Autodesk entity
    inventory, observed on-wire aliases, six completion levels, and the
    ordered M14.1--M14.11 family queue. Public group codes become typed and
    source-anchored; proprietary ACIS, OLE, proxy, raster, font, and external
    reference payloads remain bounded opaque data unless a later named
    milestone supplies a public exact decoder. Unknown future records remain
    lossless raw records.
    M14.1a recognizes exact uppercase `3DFACE`, `SOLID`, and `TRACE` markers
    only in complete raw records belonging to `BLOCKS` or `ENTITIES`. It
    retains all twelve documented corner components plus 3DFACE invisible-edge
    flags or SOLID/TRACE thickness and extrusion components in source order,
    preserving duplicates, exact binary64 bits, signed-16-bit flags, raw group
    spans, and typed ASCII lexical failures across ASCII and Binary AC1009--
    AC1032. This is occurrence evidence only: it does not select values, apply
    corner/extrusion/thickness defaults, reorder SOLID/TRACE corners, transform
    OCS to WCS, validate flags or normals, assemble faces, edit, or write.
    M14.1b builds a fixed per-role cardinality directory over that evidence:
    thirteen stable cards per 3DFACE record and sixteen per SOLID/TRACE record.
    Each absent, unique, or multiple state retains source-order references to
    the M14.1a occurrences; lexical validity remains independent, and roles
    from the wrong family have no card. This still does not select, default,
    validate, reorder, transform, assemble, edit, or write values.
    M14.1c lazily selects unique typed double or signed-16-bit values with exact
    field/raw provenance. Missing required corners, invalid ASCII numbers,
    duplicates, partial fourth corners, and unavailable default sources remain
    typed. It applies only Autodesk-documented defaults: 3DFACE edge flags
    zero; SOLID/TRACE thickness zero and extrusion (0,0,1); and an entirely
    absent 3DFACE/SOLID fourth corner inherits the usable third corner. TRACE
    has no fourth-corner default. This milestone does not interpret edge bits,
    validate finite geometry/normals, reorder corners, transform OCS to WCS,
    assemble faces, edit, or write.
    M14.1d derives finite WCS corner geometry. 3DFACE corners remain in their
    documented WCS/source order. SOLID/TRACE corners are reordered from stored
    order to perimeter order (first, second, fourth, third), then transformed
    through the shared arbitrary-axis OCS basis; normalized WCS normal and
    finite thickness remain attached. Missing semantic corners/thickness/
    extrusion, non-finite inputs, zero extrusion, and non-finite derived
    coordinates are typed failures. Positive zero is canonicalized. Edge-bit
    interpretation, thickness extrusion surfaces, BLOCK expansion, edit,
    write, render, and tessellation remain later work.
    M14.1e interprets 3DFACE group-70 bits 1, 2, 4, and 8 as invisibility of
    the edge beginning at each source-order corner. The signed source value and
    exact 16-bit pattern remain available; helpers retain and report all
    unknown bits rather than rejecting or erasing them. Defaulted zero means
    all four edges are visible. Invalid or duplicate flags leave every derived
    edge state unavailable. SOLID/TRACE edge flags, mesh topology, editing, and
    writing remain unclaimed.
    M14.2a starts text-and-symbol coverage with exact source evidence for
    `TEXT`, `MTEXT`, `SHAPE`, and `TOLERANCE` in closed BLOCKS/ENTITIES
    sections. Every Autodesk-documented per-entity field is assigned a typed
    role and its text, binary64, signed-16-bit, or signed-32-bit wire domain;
    source order, duplicates, invalid ASCII numbers, raw text spans, and
    repeated MTEXT chunks remain intact. MTEXT group 50 remains deliberately
    `RotationOrColumnHeight` until later column cardinality can disambiguate it;
    group 420/430 ranges remain ambiguous with common entity color fields until
    M14.11 owns subclass-aware common properties. This milestone does not
    select values, apply defaults, decode text, validate layouts, resolve
    styles, or derive geometry.
    M14.2b adds stable per-role cardinality over the M14.2a evidence: 19 TEXT,
    33 MTEXT, 12 SHAPE, and 11 TOLERANCE cards per record. Card members refer
    back to exact source occurrences; absent, unique, and multiple states are
    independent of lexical validity. Multiple MTEXT group 3 chunks and group
    50 occurrences remain neutral cardinality evidence for later ordered-chunk
    and column semantics. No value is selected, copied, defaulted, decoded, or
    validated.
    M14.2c selects the unambiguous numeric scalar surface for `TEXT` and
    `SHAPE`. Required alignment/insertion coordinates, TEXT height, and SHAPE
    size fail typed when absent; unique valid numbers remain explicit with
    exact raw provenance; and duplicates retain the first raw occurrence only
    as diagnostic provenance. Autodesk defaults are applied for thickness,
    rotation, width factor, oblique angle, generation flags, justification,
    and extrusion. Optional TEXT second-alignment components remain `Absent`
    instead of receiving invented coordinates. This checkpoint does not decode
    content/style/shape names, validate enum or numeric ranges, choose the
    applicable TEXT alignment point, transform coordinates, derive glyph
    geometry, or select MTEXT/TOLERANCE values.
    M14.2d selects unambiguous numeric MTEXT and TOLERANCE values through the
    same required/defaulted/absent/invalid provenance contract. Required MTEXT
    insertion, nominal height, reference width, attachment, and drawing
    direction fields and required TOLERANCE insertion/x-axis fields fail typed
    when absent. MTEXT optional x-axis components, read-only extents, line
    spacing, background controls, and column controls remain independently
    `Absent` or `Explicit`; MTEXT and TOLERANCE extrusion alone receives the
    documented `(0, 0, 1)` default. Group 50 is not selected until
    rotation/x-axis/column precedence is modeled, and group 420/430 stays
    evidence-only until M14.11 owns subclass-aware common properties. No enum
    or range validation, chunk ordering, text decoding, coordinate transform,
    style resolution, glyph geometry, editing, or writing is claimed.
    M14.2e selects the unambiguous source-backed text/name surface. Required
    TEXT content, SHAPE name, and TOLERANCE dimension-style/content fields fail
    typed when absent; unique values stay explicit with exact raw provenance;
    duplicates retain first-occurrence diagnostic provenance. TEXT and MTEXT
    alone receive Autodesk's `STANDARD` style default. MTEXT group 3 chunks
    followed by the terminal group 1 value remain individually addressable and
    in source order; zero or multiple terminals and any group 3 after a
    terminal remain preserved with typed structural counts. This checkpoint
    does not concatenate or decode text, resolve STYLE/DIMSTYLE tables,
    interpret MTEXT inline formatting, validate chunk byte lengths, derive
    glyph geometry, edit, or write.
    M14.2f projects TEXT generation flags and justification from M14.2c scalar
    states. Horizontal codes `0..=5` and vertical codes `0..=3` become typed
    enums; unsupported codes retain exact raw provenance and make alignment
    applicability unavailable. Generation values preserve the complete signed
    16-bit source pattern while exposing Autodesk's backward/upside-down bits
    and every unknown bit. A valid nonzero horizontal or vertical
    justification requires the second alignment point; left/baseline uses the
    first. This checkpoint does not verify that the required point components
    are usable, select coordinates, transform OCS/WCS, or derive glyph
    geometry.
    M14.2g projects the three MTEXT code domains that Autodesk enumerates
    completely: attachment `1..=9`, drawing direction `1/3/5`, and optional
    line-spacing style `1/2`. Unsupported explicit values retain raw
    provenance; missing required attachment/direction values keep the M14.2d
    failure; and an absent line-spacing style remains `Absent` without an
    invented default. Background/column code interpretation, numeric range
    validation, rotation/x-axis precedence, coordinate transforms, style
    resolution, and glyph geometry remain separate checkpoints.
    M14.2h validates the optional MTEXT group-44 line-spacing factor against
    Autodesk's inclusive `0.25..=4.00` domain. Accepted values preserve their
    exact IEEE-754 payload; out-of-range values remain typed with raw
    provenance; and absence remains `Absent` without an invented default.
    Other numeric ranges, background/column interpretation, rotation
    precedence, transforms, styles, and glyph geometry remain separate
    checkpoints.
    M14.2i classifies the three MTEXT group-90 settings that Autodesk publishes:
    `0` background off, `1` explicit fill color, and `2` drawing-window color.
    Other signed values, invalid ASCII, and duplicates remain typed with exact
    provenance; absence receives no undocumented default. Background-color
    ownership, other numeric ranges, columns, rotation precedence, transforms,
    styles, and glyph geometry remain separate checkpoints.
    M14.2j validates Autodesk's MTEXT width invariant: read-only group 42 is
    equal to or less than reference rectangle group 41. Equality and smaller
    values are explicit usable relations; excess values retain both exact
    IEEE-754 inputs and group-42 provenance in a typed failure; absent or
    unusable inputs remain independently typed. Other numeric relationships,
    background ownership, columns, rotation precedence, transforms, styles,
    and glyph geometry remain separate checkpoints.
    M14.2k resolves source-order precedence between the MTEXT group-50 rotation
    input and group-11/21/31 X-axis input. When no column fields make group 50
    ambiguous, the input form whose final source occurrence is later is
    effective. Invalid or duplicate rotation remains typed when effective, and
    a later X-axis input remains selectable without erasing the rotation
    diagnostic. Coexisting column fields and group 50 fail closed pending
    column-height structure validation. Axis-vector assembly, normalization,
    transforms, column heights, styles, and glyph geometry remain separate
    checkpoints.
    M14.2l isolates modern MTEXT embedded-column evidence at the exact
    group-101 `Embedded Object` boundary. Main MTEXT cards no longer consume
    groups owned by that embedded object, and a separate source-order directory
    retains observed groups 70, 41, 46, 71, 72, 44, 45, 73, and 74 with typed
    numeric results and raw group provenance. ASCII and Binary parity covers
    AC1009 through AC1032. Column-domain validation, relationships among count
    and heights, legacy flat/R2007 XDATA unification, geometry, edit, and write
    remain separate checkpoints.
    M14.2m adds a bounded scalar-semantic projection over the modern embedded
    evidence. Column type is one of no/static/dynamic; count is a nonnegative
    integer; width is positive; height and gutter measurements are nonnegative;
    and automatic-height/flow-reversed accept only Boolean 0/1.
    Missing type, duplicate singleton fields, invalid numerics, and
    out-of-domain inputs stay typed and source-anchored. Individual heights
    retain source order in a separate bounded slice.
    M14.2n validates type-dependent field relationships. Static mode requires
    positive count and shared height, active modes require usable width and
    gutter, dynamic automatic mode excludes individual heights, and dynamic
    manual mode requires positive count plus either a positive shared height or
    exactly one source-order height per column. The observed terminal zero in a
    valid-looking R2018 dynamic-manual height sequence is retained. Legacy
    storage unification, geometry, edit, and write remain separate checkpoints.
    M14.2o adds exact source evidence for R2007-era MTEXT columns stored in
    `ACAD` XDATA. It recognizes only the complete
    `ACAD_MTEXT_COLUMN_INFO_BEGIN`/`END` envelope, retains the 1070 field IDs
    75/79/76/78/48/49, and models field 50 as a declared 1070 height count
    followed by its source-order 1040 values. Inexact apps/markers and
    incomplete blocks cannot publish partial evidence. Semantic unification
    with modern embedded storage remains separate.
    M14.2p unifies Embedded and `ACAD` XDATA column evidence behind one
    source-entry enum and one generic scalar projection. Both sources now reuse
    identical type/count/dimension/flag/height issues and feed the existing
    mode-relation directory in marker source order. R2007 dynamic-manual XDATA
    with a declared height array reaches the same usable mode as modern
    embedded storage.
    M14.2q recognizes the separate exact
    `ACAD_MTEXT_DEFINED_HEIGHT_BEGIN`/`END` block only when it contains selector
    46, one 1040 value, and follows a complete same-record column-info envelope.
    It appends source-anchored defined-height evidence to that envelope and maps
    it through the existing shared-height scalar. R2007 static and
    dynamic-automatic modes now use the same cross-field relation path as
    modern storage. Linked-column handles and direct flat group-50 framing
    remain later checkpoints.
    M14.2r recognizes complete exact
    `ACAD_MTEXT_COLUMNS_BEGIN`/`END` blocks containing selector 47, one typed
    1070 declared count, and a contiguous source-order sequence of group-1005
    handles. The evidence reuses the existing raw-handle parser and preserves
    invalid hexadecimal spelling as a typed result. Inexact, interrupted, or
    wrong-wire blocks publish no entry or partial handle slice. Column-info
    association, count relationships, target resolution, and direct flat
    group-50 framing remain later checkpoints.
    M14.2s associates a linked-column block only with the latest complete
    column-info block that ends before it in the same MTEXT record. Each
    group-1005 handle reuses the document-local generic resolver and retains
    distinct invalid, null, missing, ambiguous, unique-MTEXT, and
    unique-other-record outcomes plus the unique target record where present.
    This is target-resolution evidence, not validation of graph membership or
    the undocumented declared-count relationship. Direct flat group-50
    framing remains a later checkpoint.
    M14.2t isolates Autodesk's direct MTEXT column groups
    75/76/78/79/48/49 before the exact embedded-object boundary. If at least
    one unambiguous direct column field exists, every coexisting group 50 is
    retained in source order as `RotationOrColumnHeight`; group 50 alone
    remains orientation evidence and cannot create a column entry. Scalar
    unification and group-50 height disambiguation remain later checkpoints.
    M14.2u adds direct/flat storage to the common column source enum and generic
    scalar projector. Type, count, flow, auto-height, width, and gutter reuse
    the same validation and mode relationships as Embedded/XDATA sources.
    `RotationOrColumnHeight` intentionally maps to no scalar role: flat
    dynamic-auto mode can become usable, while static/manual height modes
    remain fail-closed pending unambiguous group-50 framing.
    M14.2v closes the published direct group-50 boundary without inventing a
    record-order discriminator. Each unified source now exposes whether height
    framing is unambiguous, absent, or an ambiguous direct group-50 set with
    exact count and first-value provenance. Static and dynamic-manual flat
    modes return a dedicated unsupported-ambiguity relation issue; dynamic
    automatic mode remains usable because it does not consume height evidence.
    Autodesk's dual assignment and the legacy behavioral notes do not provide
    enough evidence to classify direct group 50 as rotation versus shared or
    individual height, so no height value is fabricated.
    M14.2w projects the M14.2k effective orientation input into one WCS X-axis
    direction. A selected rotation becomes the documented cosine/sine vector;
    a selected 11/21/31 vector retains its exact components and separately
    exposes a normalized unit direction. Partial, invalid, zero-length, and
    non-finite-length vectors remain typed, and M14.2v group-50 ambiguity
    propagates unchanged instead of becoming geometry.
    M14.2x selects the exact TEXT placement anchor in OCS from M14.2f layout
    semantics. Left/baseline layout uses the required group-10/20/30 point;
    any nonzero horizontal or vertical justification uses the optional
    group-11/21/31 point. Missing, invalid, or duplicate point components and
    unavailable justification remain distinct typed failures with source
    provenance. This checkpoint does not transform the selected point to WCS,
    resolve styles, measure text, or derive glyph geometry.
    M14.2y transforms the selected M14.2x TEXT OCS anchor into WCS with the
    normalized extrusion direction and the shared Autodesk arbitrary-axis
    basis. The result retains the first/second selection, finite WCS point,
    normalized normal, canonical positive zero, underlying scalar/layout
    evidence, and typed provenance. Invalid anchors or extrusion components,
    non-finite Binary inputs, zero extrusion, basis failure, and transformed
    overflow remain typed. Rotation, style metrics, glyph geometry, edit, and
    write remain unclaimed.
    M14.2z transforms the required SHAPE insertion point from OCS to WCS with
    its documented extrusion default and the same shared arbitrary-axis
    projection used by TEXT. It retains a finite WCS point, normalized normal,
    canonical positive zero, scalar provenance, and typed failures for
    insertion/extrusion evidence, non-finite Binary values, zero extrusion,
    basis failure, and derived overflow. The checkpoint also extracts the
    shared 129-line text-symbol projection helper and reduces the TEXT WCS
    module from 309 to 271 lines without changing its public contract.

Every item is split into reviewable micro-milestones and stops after its own
passing checkpoint.
