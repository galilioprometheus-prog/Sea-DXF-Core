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
   staged full or smoke checks across all six reviewed platforms. Q2.1a makes
   that staging budget-aware: non-document pushes and pull requests run one
   complete Linux x64 gate including dependency policy, Markdown-only changes
   do not start runners, and manual dispatch runs the complete gate on all six
   platforms. Automatic schedules and the redundant standalone dependency job
   are removed; concurrency still cancels superseded runs. The standalone
   dependency workflow remains available manually. Twenty consecutive
   six-platform receipts therefore remain explicitly unachieved and must be
   collected through deliberate runs or a later approved schedule. Q2.2 adds
   the redacted offline corpus manifest and receipt harness without placing
   private DXF bytes, paths, or per-file hashes in the repository.
   Q2.1b makes development local-first after hosted billing exhaustion. Push
   and pull-request events allocate no GitHub-hosted runner; the ordinary CI
   workflow is a manual Windows x64 diagnostic routed to the repository's
   `seacad` self-hosted runner. Required gates run locally on the final batch
   commit before one deliberate push. The separate manual Native Release
   Artifacts workflow remains the only six-hosted-runner path and continues to
   bind all six native packages and the aggregate receipt to one exact commit.
   This policy changes execution cost, not existing native evidence or support
   claims.
   Q2.1c-d select the provisioned Windows PowerShell 5.1 shell and scope its
   execution-policy bypass to each Actions-generated process without changing
   machine policy. Q2.1e records the first successful end-to-end self-hosted
   manual quality run at commit `9e7c47a2af2a80f79c59f51ef151f62090572c91`.
   This operational Windows x64 receipt is not six-native release evidence.
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
    M14.2z extracted the shared text-symbol extrusion/OCS projection helper and
    added the first SHAPE placement projection. M14.2aa corrects that
    checkpoint after a normative re-audit: Autodesk defines SHAPE groups
    10/20/30 directly in WCS, unlike TEXT's OCS alignment points. SHAPE now
    retains its exact finite WCS insertion unchanged and uses the shared helper
    only to normalize extrusion for the WCS normal. A finite `f64::MAX` point
    remains usable, proving no accidental basis multiplication. Missing,
    invalid, duplicate, non-finite, zero-extrusion, and basis failures remain
    typed. TEXT continues to use the helper's OCS-to-WCS point transform with
    unchanged regression evidence.
    M14.2ab projects SHAPE group 50 rotation onto the normalized extrusion
    plane and exposes finite orthonormal WCS x/y axes plus the normal. The
    default zero-degree rotation preserves the arbitrary-axis basis; positive
    angles rotate its x/y axes in degrees. The exact SHAPE WCS insertion point
    remains untouched. Invalid or duplicate rotation/extrusion evidence,
    non-finite Binary values, zero extrusion, and basis failure remain typed.
    Shape-definition resolution, style metrics, glyph geometry, edit, and
    write remain unclaimed.
    M14.2ac projects TEXT group 50 rotation and the documented group 71
    backward/upside-down bits into WCS glyph axes on the normalized extrusion
    plane. Unknown generation bits remain preserved without changing the known
    axis effects. The selected OCS-to-WCS placement anchor and all lower
    evidence remain reachable. Invalid or duplicate rotation/flags/extrusion,
    non-finite Binary values, zero extrusion, and basis failure remain typed.
    Style metrics, oblique/width/height geometry, glyph outlines, edit, and
    write remain unclaimed.
    M14.2ad composes TOLERANCE's required WCS insertion and WCS x-axis
    direction with its normalized extrusion normal. The two WCS vectors remain
    bit-exact; no OCS transform, x-axis normalization, or invented y-axis is
    applied. Missing, invalid, duplicate, non-finite, zero x-axis/extrusion,
    and basis failures remain typed with complete scalar evidence. Dimension
    style resolution, tolerance-string interpretation, glyph geometry, edit,
    and write remain unclaimed.
    M14.2ae indexes exact group-2 names only from DIMSTYLE records inside
    completely closed exact `TABLE`/`DIMSTYLE`/`ENDTAB` envelopes, then resolves
    TOLERANCE group-3 names by bounded byte-exact source-span comparison.
    Missing, unique, ambiguous, and unusable names remain distinct; duplicate
    targets are preserved in source order. Interrupted/unclosed/wrong-case
    tables, malformed names, and application-group decoys are not admitted.
    A shared bounded span hash/comparator replaces three duplicated BLOCK and
    ATTDEF implementations without changing their public behavior. DIMSTYLE
    field semantics, tolerance-string interpretation, glyph geometry, edit,
    and write remain unclaimed.
    M14.2af publishes one reviewed registry for all 68 documented
    DIMSTYLE-specific fields and retains their exact source-order text,
    binary64, signed-16-bit, or handle evidence. Every admitted named record
    exposes 68 fixed cards with absent, unique, or duplicate-preserving
    multiple state; invalid ASCII numbers and handles remain typed, and
    application-group content cannot impersonate a field. AC1009 Binary
    parity keeps codes above 255 absent because its one-byte group-code header
    cannot represent them. No field value is selected, defaulted, normalized,
    range-validated, or resolved to another table record. Tolerance-string
    interpretation, glyph geometry, edit, and write remain unclaimed.
    M14.2ag lazily selects any requested DIMSTYLE-specific field only when its
    M14.2af card is unique. Explicit, absent, invalid-number, invalid-handle,
    and duplicate states retain stable field/raw provenance; no absent value
    receives an invented default. Group-70 standard flags additionally expose
    the documented externally-dependent, resolved, and referenced bits while
    preserving every unknown bit. Domain/range validation, handle target
    resolution, tolerance-string interpretation, glyph geometry, edit, and
    write remain unclaimed.
    M14.2ah composes the five DIMSTYLE handle-field cards with the shared
    document-local handle resolver. Each text-style, leader-arrow, common-
    arrow, first-arrow, and second-arrow role retains absent, duplicate-field,
    invalid, null, missing, unique, or ambiguous target state and exposes every
    generic identity match without selecting a duplicate. AC1009 Binary keeps
    these above-255 groups absent. Target record-type validation, name
    resolution, tolerance-string interpretation, glyph geometry, edit, and
    write remain unclaimed.
    M14.2ai validates each uniquely resolved DIMSTYLE handle against exact
    membership in a completely closed, matching STYLE or BLOCK_RECORD table.
    The shared named-symbol scanner also retains DIMSTYLE membership without
    duplicating table-envelope logic. Unique targets are classified as the
    expected table kind, another reviewed named-symbol kind, or another raw
    record; non-unique lower-layer states pass through unchanged. This does
    not resolve target names, validate block/style contents, apply defaults,
    interpret tolerance strings, construct glyphs, edit, or write.
    M14.3a retains every documented SPLINE signed-16-bit and binary64 defining
    value in source order from exact records in complete BLOCKS or ENTITIES
    sections. All 24 roles preserve duplicates, lexical failures, raw spans,
    integer domains, and float bits while group-102 application content is
    excluded. No values are selected, no defaults or version applicability
    are applied, and counts, flags, point grouping, curve validity, geometry,
    HELIX subclass data, edit, and write remain unclaimed.
    M14.3b adds 24 fixed per-role cardinality cards to every retained SPLINE
    record. Absent, unique, and duplicate-preserving multiple states remain
    independent of lexical validity, and compact members point back to M14.3a
    evidence without copying or selecting values. Defaults, flag/count
    semantics, point grouping, curve validity, geometry, HELIX, edit, and write
    remain unclaimed.
    M14.3c lazily selects SPLINE group-70 flags only when its M14.3b card is
    unique. Absent, invalid, and duplicate states remain explicit; documented
    closed, periodic, rational, planar, and linear helpers preserve every
    unknown bit rather than rejecting real extended flags. Degree/count/
    tolerance semantics, point grouping, validity, geometry, HELIX, edit, and
    write remain unclaimed.
    M14.3d selects the unique degree, three declared counts, and three
    tolerance scalars through M14.3b cards. Missing counts remain absent;
    missing knot/control/fit tolerances receive only Autodesk's documented
    `1e-7`, `1e-7`, and `1e-10` defaults. Explicit, defaulted, absent, invalid,
    and duplicate states retain evidence without range validation. Count
    reconciliation, point grouping, curve validity/geometry, HELIX, edit, and
    write remain unclaimed.
    M14.3e compares each unique nonnegative declared knot, control-point, and
    fit-point count with the number of retained group-40, group-10, or group-11
    anchor occurrences. Matched and mismatched counts remain explicit; absent,
    duplicate, invalid, and negative declarations remain typed and retain the
    lower-layer evidence. This occurrence relation does not infer complete
    coordinate tuples, validate numeric members, knots/topology, or geometry,
    process HELIX, edit, or write.
    M14.3f freezes the canonical 45-topic Autodesk entity inventory as reviewed
    schema input and deterministic generated Rust. Every entry exposes one
    stable ordinal, schema id, exact canonical group-zero spelling, and the
    shared normalized source receipt. Exact byte lookup is case-sensitive;
    aliases and unknown spellings remain unclassified. The generator rejects
    missing/extra topics, duplicate ids or names, an invalid namespace, and a
    non-topic-list source. This checkpoint does not scan records, map aliases,
    define version applicability or fields, project common properties, parse
    any new entity, construct geometry, edit, or write.
    M14.3g adds 14 reviewed group-zero alias/specialization names to the same
    generated platform. Five mappings are normative Autodesk facts
    (`MPOLYGON`, `ACAD_TABLE`, and the DGN/DWF/PDF underlay names); nine
    concrete dimension, MLEADER, SECTION, and SURFACE spellings are explicitly
    labeled behavioral AutoCAD-oracle evidence. Public exact-byte
    classification returns canonical, alias, or unknown and maps aliases to a
    canonical topic without hiding their exact marker. Per-source normalized
    receipts reject mapping or provenance drift. Section validity, dialect
    applicability, record scanning, semantics, geometry, edit, and write remain
    unclaimed.
    M14.3h adds one generated applicability row for every canonical topic and
    reviewed alias across all nine supported `$ACADVER` dialects. A row is
    either a source-backed inclusive range or explicitly `NotYetReviewed`;
    missing evidence is never converted into a guessed version floor. Autodesk
    compatibility guidance establishes DWF and DGN underlays at AC1021 and PDF
    underlays at AC1024. All other rows remain unreviewed. The generator rejects
    missing/reordered rows, unsupported or inverted ranges, metadata attached
    to unreviewed rows, wrong source kinds, and stale per-source receipts. This
    checkpoint does not scan records, validate section placement, gate writers,
    add fields or semantics, construct geometry, edit, or advance support.
    M14.3i adds the unified source-anchored entity directory over completely
    indexed raw records. Canonical topics, exact aliases, unknown records in
    `BLOCKS`/`ENTITIES`, and reviewed names in other record-bearing sections
    remain distinct; structural `BLOCK`/`ENDBLK` controls are excluded.
    `DxfEntityRef` retains source identity, raw-record ordinal, exact marker,
    section, classification, and an ordered slice of exact group-100 subclass
    markers. Group-100 occurrences inside group-102 application envelopes are
    excluded from the semantic subclass path without changing raw evidence.
    ASCII/Binary parity covers all nine dialects, exact case sensitivity,
    interrupted/unclosed sections, cancellation, source mismatch, and public
    metadata bounds. This checkpoint does not add common fields, typed family
    semantics, geometry, edit, write, or a support-state advancement.
    M14.3j freezes 19 common entity-property roles from Autodesk's common-code
    table into generated descriptors. Stable field ordinals map exact group
    codes to wire type, required/optional singleton or optional-sequence
    cardinality, documented defaults, entity-preamble/`AcDbEntity`/extension-
    dictionary scope, non-coordinate classification, explicit field-version
    `NotYetReviewed`, and per-row provenance. The schema distinguishes the
    BLOCK_RECORD owner from group-330 reactor content and keeps group-360 in
    the `ACAD_XDICTIONARY` application scope. Generator validation rejects
    count/order drift, duplicate ids/codes, invalid wire families, defaults,
    scope facts, source kind, and stale normalized receipts. This checkpoint
    does not scan field occurrences, evaluate cardinality, decode values,
    apply defaults, edit, write, or advance support.
    M14.3k scans the 19 generated common-field roles once per unified semantic
    entity and builds fixed descriptor-order cardinality cards. Exact raw
    occurrences remain in source order; required/optional absence, unique
    singleton, duplicate singleton, and sequence counts are distinct.
    Preamble fields, `AcDbEntity` fields, and `ACAD_XDICTIONARY` group-360
    content are scope-separated; reactor and family-subclass collisions are
    excluded. Extension-dictionary evidence retains a resolvable application-
    group ordinal and closure state. Canonical, alias, and unknown records in
    `BLOCKS`/`ENTITIES` participate, while wrong-section classifications do
    not. Nine-dialect ASCII/Binary parity, source identity, cancellation,
    metadata bounds, and debug redaction are covered. This checkpoint does not
    decode values, apply defaults, validate domains/references/proxy byte
    counts, edit, write, or advance support.
    M14.3l groups the retained SPLINE group-10/20/30 control-point and
    group-11/21/31 fit-point sequences by the ordinal within each component
    role. This produces two fixed entries per SPLINE record and compact tuples
    that point back to exact evidence-card members without copying values.
    Complete and every partial component shape remain explicit, including
    missing X or Y and omitted Z; invalid numeric evidence remains attached.
    Pairing is independent of how different group codes are interleaved.
    Nine-dialect ASCII/Binary parity, empty sequences, cancellation, lookup
    bounds, and compact public metadata are covered. This checkpoint does not
    apply coordinate defaults, validate tuple/count relations, add weights or
    tangent/normal vectors, construct NURBS geometry, process HELIX, edit,
    write, or advance support.
    M14.3m adds group-41 weight evidence to the fixed SPLINE role/card set and
    projects one weight relation plus start-tangent, end-tangent, and normal
    vector structure per record. Missing weights represent Autodesk's implicit
    unit-weight sequence; explicit weight counts are compared with group-10
    control-point anchors without validating their numeric domain. Each vector
    component remains absent, unique with an exact card member, or duplicate
    with a count; aggregate state is absent, present with an exact component
    mask, or ambiguous without selecting a duplicate. The SPLINE evidence scan
    now accepts only legacy pre-subclass or exact `AcDbSpline` scope, excluding
    later subclass collisions. Nine-dialect ASCII/Binary parity, mismatch,
    partial/duplicate/invalid evidence, cancellation, and public bounds are
    covered. Vector defaults/domains, spline invariants, analytic NURBS data,
    HELIX, CRUD, writes, and support advancement remain unclaimed.
    M14.3n projects M14.3m evidence into typed effective weight and vector
    values. Missing group-41 sequences retain an implicit unit-weight state;
    explicit matched, count-mismatched, valid, invalid-numeric, and
    behavioral nonpositive weight states remain distinct with exact evidence.
    A wholly absent tangent/normal stays absent. Otherwise unique valid X is
    required, optional Y/Z components default to zero, and the effective vector
    records which components were explicit. Duplicate components, invalid
    numerics, or missing X produce one unavailable state with independent
    component masks and no occurrence selection. Nine-dialect ASCII/Binary
    parity, defaults, mismatch/domain evidence, cancellation, and metadata
    bounds are covered. Rational/planar/linear relations, zero-normal policy,
    point semantics, degree/knot invariants, analytic NURBS data, HELIX, CRUD,
    writes, and support advancement remain unclaimed.
    M14.3o composes the exact group-70 state with M14.3n weights and normal.
    Linear-plus-planar is a typed satisfied or contradictory relation per the
    Autodesk flag definition. Rational and non-rational flags are paired with
    implicit-unit, explicit-matched, or explicit-count-mismatched weights and
    retain invalid/nonpositive member totals without imposing an undocumented
    equivalence. Planar normals are missing, explicit with an exact-zero
    observation, or unavailable; nonplanar normals remain typed unexpected
    observations because Autodesk says they are omitted. Missing, duplicate,
    and invalid flags fail all dependent projections closed. Nine-dialect
    ASCII/Binary parity, relation edges, cancellation, lookup, and bounds are
    covered. Point semantics, degree/knot/periodic invariants, analytic NURBS
    data, HELIX, CRUD, writes, and support advancement remain unclaimed.
    M14.3p adds one bounded topology entry per SPLINE. Degree is absent,
    duplicate, invalid, nonpositive, or explicit with exact evidence. Knot
    order is empty, nondecreasing, decreasing at its first exact right-member
    index, or unavailable with an invalid-member count. Positive degree is
    compared with observed group-10 control anchors and with the defining
    NURBS relation `knot_count = control_point_count + degree + 1`; expected
    count overflow stays typed. All four closed/periodic flag combinations are
    observations, not an undocumented validity equivalence. Nine-dialect
    ASCII/Binary parity, degree/count/order edges, cancellation, lookup, and
    bounds are covered. Point values, analytic NURBS projection, HELIX, CRUD,
    writes, and support advancement remain unclaimed.
    M14.3q1 materializes analytic-value sequences over the existing evidence.
    Knots retain exact group-40 values and provenance. Control/fit tuples
    require valid X/Y, use a behavioral zero default only for omitted Z, and
    retain both tuple ordinal and explicit-Z state. Control weights become
    implicit units or a fully matched positive explicit sequence. Knot,
    control, fit, and weight failures accumulate without publishing partial
    ranges. Nine-dialect ASCII/Binary parity, out-of-order component groups,
    invalid/partial values, cancellation, lookup, and bounds are covered. Full
    topology/count/multiplicity/parameter-domain and vector readiness remains
    M14.3q2; HELIX, CRUD, writes, and support advancement remain unclaimed.
    M14.3q2 adds the final source-backed SPLINE analytic-readiness projection.
    It composes q1 value ranges with unique valid flags and degree, matched
    declared knot/control counts, an absent-or-matched fit count, nondecreasing
    knots, degree/control and NURBS count relations, bounded knot multiplicity,
    a positive active parameter interval, usable optional tangents, and the
    documented planar-normal relation. Every failed prerequisite accumulates
    in a typed issue mask and no partial analytic data is published. The
    closed/periodic combination remains observational. Nine-dialect
    ASCII/Binary parity, multi-issue failures, cancellation, lookup, and public
    bounds are covered. Evaluation, tessellation, HELIX, CRUD, writes, and
    support advancement remain unclaimed.
    M14.3r begins HELIX with one source-anchored evidence directory over the
    unified entity index. Sixteen Autodesk-defined `AcDbHelix` roles retain
    exact groups and typed raw numeric results: two Int32 version fields, nine
    WCS point/vector components, three double parameters, BooleanByte
    handedness, and Int16 constraint type. Exact subclass transitions prevent
    shared group codes such as 10 and 40 from colliding with embedded SPLINE
    data; application groups, wrong sections, and near-match markers do not
    enter the family projection. Duplicate and invalid values remain present
    without selection. Nine-dialect ASCII/Binary parity accounts for the
    AC1009 binary group-code limit; malformed/scope fixtures, cancellation,
    lookup, and bounds are covered. HELIX cardinality, semantics, embedded
    SPLINE composition, geometry, CRUD, writes, applicability advancement, and
    support completion remain unclaimed.
    M14.3s adds 16 fixed cardinality cards per indexed HELIX record. Every card
    retains a compact source-order member range into M14.3r evidence and is
    classified as absent, unique, or multiple; invalid numeric syntax remains
    a present occurrence. Capacity arithmetic is checked, allocations are
    fallible, source identity is verified, and cancellation is checked before
    and during card construction. No duplicate singleton is selected and no
    required/optional or default policy is inferred from the Autodesk table.
    Nine-dialect ASCII/Binary parity, deliberately out-of-order fields, empty
    and duplicate records, invalid members, cancellation, lookup, and bounds
    are covered. HELIX typed semantics, point/vector grouping, parameter
    relationships, embedded SPLINE composition, geometry, CRUD, writes,
    applicability advancement, and support completion remain unclaimed.
    M14.3t adds seven provenance-bearing HELIX scalar semantics using the
    platform `DxfSemanticValue<T, I>`. Version integers and finite
    radius/turns/turn-height doubles remain exact; documented handedness 0/1
    and constraint 0/1/2 values map to typed enums. Absence remains `Absent`
    because no default or requiredness is inferred. Duplicate singletons,
    invalid ASCII numbers, Binary non-finite doubles, and out-of-domain enum
    values become typed `Invalid` states without value selection. Negative or
    zero versions/parameters remain explicit pending a separately evidenced
    relationship/domain milestone. Nine-dialect ASCII/Binary parity,
    malformed/domain/non-finite fixtures, cancellation, provenance, lookup,
    and bounds are covered. HELIX WCS tuple semantics, axis and parameter
    relationships, embedded SPLINE composition, geometry, CRUD, writes,
    applicability advancement, and support completion remain unclaimed.
    M14.3u adds three WCS vector semantic entries per HELIX: axis base, start
    point, and axis vector. Nine component values use the platform semantic
    contract with exact raw and field provenance. Fully usable triples are
    published without coordinate transformation; absent, duplicate, invalid
    ASCII, and Binary non-finite components remain distinct and suppress the
    tuple. No Y/Z default is inferred from the Autodesk HELIX table. A
    fully explicit zero axis remains observable rather than being rejected in
    this component milestone. Nine-dialect ASCII/Binary parity, deliberately
    out-of-order groups, partial/duplicate/invalid/non-finite cases,
    cancellation, lookup, provenance, and bounds are covered. HELIX axis and
    parameter relationships, embedded SPLINE composition, analytic geometry,
    CRUD, writes, applicability advancement, and support completion remain
    unclaimed.
    M14.3v composes HELIX scalar and WCS-vector semantics into one relation
    entry per retained record. A complete nonzero axis is normalized, the
    start-minus-axis-base radial vector yields a derived base radius, and an
    exact orthogonality residual reports whether the documented perpendicular
    relation is bit-exact without inventing a tolerance. Zero axes and every
    non-finite derived stage remain typed. The stored radius is classified as
    negative or nonnegative, turns as nonpositive, within the documented
    command limit of 500, or above it, and axial height is derived as turns
    times turn height with explicit overflow and flat-height states. Autodesk
    permits height zero, so the legacy oracle's zero-turn-height rejection is
    not imported. Nine-dialect ASCII/Binary parity, unavailable inputs,
    residual/domain/overflow cases, cancellation, lookup, source identity,
    and bounds are covered. The group-40 radius is not relabeled as top or
    base radius beyond Autodesk's DXF wording. Embedded SPLINE composition,
    analytic HELIX geometry, CRUD, writes, applicability advancement, and
    support completion remain unclaimed.
    M14.3w extends the shared SPLINE evidence and analytic directories to
    classify exact `SPLINE` and `HELIX` record kinds. HELIX spline values are
    admitted only inside the exact `AcDbSpline` subclass, so colliding
    `AcDbHelix` groups cannot enter the embedded curve. One HELIX analytic
    entry joins relation and spline readiness by exact raw-record ordinal and
    requires exactly one ordered `AcDbSpline` then `AcDbHelix` marker, an
    available embedded curve, exact axis perpendicularity, nonnegative stored
    radius, positive turns, and finite compared height. Above-500 turns remain
    readable existing-data state; flat zero height remains valid. Failures
    accumulate in a typed mask and publish no partial analytic data.
    Nine-dialect ASCII/Binary parity, malformed subclass paths, invalid curve
    and relation inputs, cancellation, lookup, source identity, and a 512-byte
    entry bound are covered. This exposes analytic representation only; it
    does not evaluate NURBS calls on HELIX, sample/tessellate geometry, add
    CRUD or writes, settle applicability, or advance support completion.
    M14.3x returns to the unified platform and projects all 19 generated common
    entity fields into typed semantics. Singleton fields use the shared
    provenance-bearing four-state value contract: exact signed integers,
    finite binary64, parsed handles, and source-backed exact text remain in
    separate wire domains. Reviewed omitted values materialize only the
    generated defaults, including the distinct material `ByLayer` state.
    Missing required fields, duplicate singletons, invalid ASCII numbers,
    invalid handles, and Binary non-finite doubles fail typed without selecting
    an occurrence. Proxy group-310 data remains an opaque sequence whose exact
    members resolve through the evidence directory. Nine-dialect ASCII/Binary
    parity, all wire domains, defaults/absence, malformed values, cancellation,
    source identity, exact text decoding, lookup, and a 320-byte entry bound are
    covered. `MissingRequired` reflects schema shape while field applicability
    is unreviewed. This checkpoint does not validate property domains/references,
    proxy byte counts, application-group closure, CRUD, writes, or advance any
    entity to `Complete`.
    M14.3y begins the CRUD kernel with one format- and dialect-aware encoder
    for generated common-field descriptors. Borrowed edit values preserve six
    distinct wire domains: exact raw text, handle, finite binary64, Int16,
    Int32, and one bounded binary chunk. ASCII output uses minimal group-code
    and numeric spelling, uppercase handle/chunk hexadecimal, and LF framing.
    Binary output uses one-byte AC1009 group codes, the documented AC1009 XDATA
    escape range, or R13-and-later little-endian two-byte codes, plus exact
    little-endian payloads, NUL-terminated strings, and length-prefixed chunks.
    Wire mismatch, non-finite doubles, framing bytes in exact text, over-128-
    byte chunks, and group codes unavailable in a dialect fail typed; resource
    exhaustion and cancellation remain fatal. Nine-dialect outputs strictly
    reparse in both formats and exact-byte tests cover all six wire domains.
    This checkpoint does not transcode Unicode into legacy code pages, create
    sequence operations, choose insertion anchors, allocate handles/owners,
    edit a source, build a transaction plan, or claim CRUD/writer completion.
    M14.3z introduces source-bound `DxfEntityKey` values and composes M14.3k
    cards, M14.3y encoding, and the existing M11 transaction/inverse kernel for
    one explicit common-field replacement. Only an existing `Unique`
    non-sequence card is eligible. Missing required/optional fields remain
    typed and require future canonical insertion anchors; duplicate singletons
    fail without occurrence selection; group-310 cards always require a
    sequence operation; wrong-section entities, unavailable dialects, and
    encoder failures remain typed. One exact raw group span is replaced and
    every other source byte remains outside the plan. Nine-dialect paired
    ASCII/Binary outputs strictly reparse, expose the new field semantic, and
    produce executable inverse plans that restore byte-identical input. This
    checkpoint does not insert/reset fields, validate property domains or
    references, combine patches in an edit session, write a destination, or
    claim complete entity update/CRUD support.
    M14.3aa adds an explicit reset-to-default planner for one common-field
    optional singleton. `AbsentOptional` is already implicit and produces no
    transaction. `Unique` deletes the exact complete raw group through one M11
    patch, while required fields and duplicate singletons fail typed without
    occurrence selection. Optional sequences always require a sequence
    operation, and an extension-dictionary handle inside the documented
    `ACAD_XDICTIONARY` group requires a nested-structure operation so the
    singleton planner cannot leave an empty group-102 wrapper. Nine-dialect
    paired ASCII/Binary post-images strictly reparse, expose the generated
    omitted semantic default or absence, and produce inverse plans that restore
    byte-identical input. This checkpoint does not remove whole application
    groups, insert fields, combine patches, validate domains/references, write
    a destination, or claim complete entity update/CRUD support.
    M14.3ab freezes a writer-only canonical order ordinal for all 19 generated
    common-field descriptors. The order follows the usual presentation in the
    reviewed Autodesk common entity-code table: handle, extension dictionary,
    owner, then the remaining common properties. Generator validation rejects
    missing, duplicate, or reordered ordinals, and the public descriptor keeps
    registry ordinal and write order separate. Readers continue to accept
    arbitrary group order and preserve unknown groups. This checkpoint does
    not calculate record-specific insertion anchors, insert a group, mutate a
    source, resolve subclass envelopes, or advance CRUD/support claims.
    M14.3ac uses that ordinal to plan one source-bound between-group anchor for
    an absent common-field singleton. Handle anchors immediately after the
    entity marker; owner anchors after complete preamble application groups and
    before the exact `AcDbEntity` subclass. AC1012 and later require exactly one
    such subclass, while AC1009 scans a bounded legacy preamble and refuses an
    unclosed/interrupted group-102 envelope. AcDbEntity fields constrain the
    anchor after every earlier known common field and before every later one;
    conflicting existing order fails typed, and unknown groups remain in place.
    The result exposes the exact byte offset plus immediate neighbor group
    occurrences. Existing fields, sequences, extension dictionaries, wrong
    sections, absent dialects, source mismatches, and cancellation remain typed.
    This checkpoint does not encode or insert a group, build a transaction,
    mutate a source, validate domains/references, or claim CRUD completion.
    M14.3ad composes M14.3ac anchors, M14.3y typed encoding, and the M11 raw
    transaction/inverse kernel to insert one absent common-field singleton.
    The source span is empty at the exact anchor and the owned replacement is
    one complete group. ASCII replaces the encoder's two canonical separators
    with the preceding group's exact LF, CR, or CRLF ending; Binary bytes remain
    dialect-correct. Anchor failures and encoder failures remain distinct typed
    outcomes before a plan escapes. Across AC1009 through AC1032, paired strict
    ASCII/Binary post-images publish the inserted semantic and executable
    inverse plans restore byte-identical source. This checkpoint does not insert
    sequences or extension-dictionary envelopes, batch fields in an edit
    session, validate domains/references, allocate handles/owners, write a
    create-new destination, or claim complete entity update/CRUD support.
    M14.3ae introduces the first public `DxfEntityEditSession` and typed
    `DxfEntityPatch::CommonField` update path. Explicit set dispatches to exact
    replacement or absent-field insertion; reset dispatches to the existing
    optional-singleton reset contract. Every accepted edit remains source-
    bound and payload-redacted. A second queued edit for the same entity/field
    fails typed, while already-implicit resets remain no-ops. `finish` sorts raw
    spans, merges same-entity empty-span insertions by generated writer order,
    and returns one immutable M11 transaction. Five logical edits across two
    records collapse to three non-overlapping patches in paired strict
    ASCII/Binary tests for all nine dialects, and the executable inverse
    restores byte-identical source. This checkpoint does not add topic-family
    patch variants, sequences/nested grammar, insert/clone/delete, domain or
    reference validators, or the create-new verified destination pipeline.
    M14.3af adds `DxfEntityEditPlan` as the verifiable finish path for an edit
    session. Each queued singleton retains its record ordinal, field, requested
    explicit typed value or implicit reset state, while exact text is owned and
    every `Debug` view remains payload-redacted. Verification requires the
    source precondition and post-image format/length, resolves the same raw
    record ordinal in a fresh common-field semantic directory, checks unique
    explicit versus absent optional cardinality, checks explicit/defaulted/
    absent state, and compares exact text, handle, finite double, Int16, or
    Int32 values. Only then does M11.1b verify every raw output byte and publish
    an executable inverse journal. Paired strict ASCII/Binary tests cover all
    nine dialects; semantic value mismatch and unrelated raw-byte mismatch stay
    distinct. This checkpoint does not call the M12 create-new writer, remove a
    destination after semantic failure, validate domains/references, add family
    patches, or establish broader CRUD support.
    M14.3ag connects that verifiable entity plan to the existing M12 create-new
    writer. `write_reparse_verify_and_journal_to_new_file` first streams and
    independently hashes a new destination, then strictly reparses the exact
    ASCII/Binary format, checks every requested common-field postcondition,
    verifies every transaction byte, and returns paired write/semantic receipts
    with the executable inverse. Semantic unavailability and every strict/raw/
    cancellation error after creation remove the destination; cleanup failure
    replaces the primary result, and a pre-existing path remains untouched.
    Paired success coverage spans all nine dialects, while controlled post-hash
    tampering proves typed semantic mismatch, unrelated raw mismatch, strict
    reparse failure, and cleanup. This checkpoint does not add domain/reference
    validation, family patches, sequence/nested edits, entity insert/clone/
    delete, handle/owner assignment, or full CRUD support.
    M14.3ah adds a pure common-field edit-domain classifier and admits it before
    replacement/insertion planning. Autodesk-backed closed domains cover group
    67 model/paper space, group 62 BYBLOCK/ACI/BYLAYER and negative layer-off
    colors, public `AcDb::LineWeight` values, nonnegative group-48 linetype
    scale, group-60 visibility, nonnegative group-92 proxy byte count, group-420
    24-bit RGB, and group-284 shadow mode. Each valid result is a typed value;
    wrong value kinds and out-of-domain scalars return typed issues, while
    unreviewed fields remain explicitly classified as such. Invalid edits never
    enter the session queue or produce a transaction. Boundary tests and paired
    strict ASCII/Binary session tests cover all nine dialects and exact inverse
    restoration. This checkpoint does not validate existing raw-domain values,
    names, handles/references, transparency, proxy count/data agreement,
    cross-field relations, applicability, or family-specific patches.
    M14.3ai composes the existing-document common-field semantic directory with
    the M14.3ah domain classifier. The eight reviewed scalar fields retain the
    shared explicit/defaulted/absent/invalid state and exact raw provenance;
    valid usable values become typed domain values, while scalar decode,
    duplicate, missing-required, and domain failures remain distinguishable.
    Every unreviewed common field passes through its exact prior semantics rather
    than receiving an implied validity claim. ASCII/Binary parity spans AC1009
    through AC1032, including out-of-order groups, invalid scalars, defaults,
    absence, opaque sequences, source identity, cancellation, and public API
    bounds. This checkpoint does not validate names or references, transparency,
    proxy count/data agreement, cross-field relations, applicability, family
    patches, or complete CRUD.
    M14.3aj composes the generic common-field semantics with the M7.3b handle
    resolution directory for all five handle-valued common fields. Group 5
    remains lexical identity evidence. Owner group 330, closed
    `ACAD_XDICTIONARY` group 360, material group 347, and plot-style group 390
    preserve field invalidity and the shared four-state model while adding
    typed null, missing, unique, and ambiguous document-local target outcomes.
    Material omission remains the generated `ByLayer` default and is never
    resolved as a fabricated handle. ASCII/Binary parity spans all nine
    dialects; AC1009 retains its exact missing-required/absent/defaulted states
    because applicability is still unreviewed. This checkpoint does not validate
    target record kinds, authoritative ownership, dictionary membership,
    pointer lifecycle, names, applicability, or reference-safe edits.
    M14.3ak extends the closed named-symbol inventory with `LAYER` and `LTYPE`
    records and projects the four exact-text common fields. Layer group 8 and
    linetype group 6 use SHA-256-bounded exact same-document lookup followed by
    byte comparison, retaining unique, missing, and duplicate-name outcomes.
    Omitted linetype remains the reviewed schema `BYLAYER` default rather than a
    fabricated table target. Layout group 410 and color-name group 430 pass
    through their exact prior semantics as explicitly unreviewed. Paired
    ASCII/Binary fixtures span all nine dialects and preserve wrong-section
    inventory, field cardinality, raw provenance, source identity, cancellation,
    and debug redaction. This checkpoint does not define case folding, symbol
    character validity, XREF name policy, layout-object or color-book resolution,
    applicability, or name-safe edits.
    M14.3al reconciles the reviewed group-92 proxy byte count with the exact
    opaque group-310 sequence. ASCII chunks are validated as hexadecimal and
    counted by decoded byte length; Binary chunks use the validated payload
    span after the wire length prefix. The projection never concatenates or
    interprets payload data and retains the first malformed chunk's source
    provenance. Typed states distinguish complete absence, missing size,
    matched counts, mismatches, invalid size, and invalid ASCII chunks.
    ASCII/Binary parity spans all nine dialects; AC1009 proves the expressible
    zero-size/no-data case. This checkpoint does not decode proxy graphics,
    infer a missing size, repair mismatches, validate version applicability,
    or add sequence edit/clone/delete support.
    M14.3am reviews common transparency group 440. Public values distinguish
    `ByLayer`, `ByBlock`, and `ByAlpha { alpha }`; alpha follows ObjectARX's
    zero-clear to 255-opaque contract. Autodesk's method enumeration plus an
    AutoCAD 2027 native CHPROP receipt establish method bytes 0, 1, and 2 in
    the high byte, with alpha in the low byte. Reserved payload bits and other
    method bytes fail typed. The domain composes with existing-document
    semantics, edit admission, strict post-image verification, and exact
    inverse restoration. ASCII/Binary fixtures cover every dialect; AC1009
    retains absence and rejects inexpressible insertion through the existing
    wire gate. This checkpoint does not resolve layer/block effective alpha,
    map UI percentage rounding, render transparency, or claim applicability.
    M14.3an composes the common-handle projection with exact target-record
    classification for the three references whose public target kinds are
    reviewed. Extension dictionary group 360 requires an `OBJECTS`/`DICTIONARY`
    record, material group 347 requires `OBJECTS`/`MATERIAL`, and plot-style
    group 390 requires `OBJECTS`/`ACDBPLACEHOLDER`. A uniquely resolved handle
    with another exact marker or section becomes a typed incompatible-target
    state; null, missing, ambiguous, raw-field, absent, and generated-default
    states retain their existing precedence and provenance. Owner group 330
    remains explicitly unreviewed because its valid target kind depends on
    entity family and placement. ASCII/Binary fixtures cover all nine dialects,
    wrong marker, wrong section, resolution failures, cancellation, source
    identity, and bounds. This checkpoint does not validate authoritative
    ownership, dictionary membership, lifecycle or reference-safe edits,
    applicability, family graphs, or `Complete` support.
    M14.3ao applies those reviewed target-kind rules before a generic singleton
    edit enters `DxfEntityEditSession`. Handle identity group 5 now requires a
    handle-remap operation and owner group 330 requires a placement/ownership
    operation. Extension dictionary, material, and plot-style edits require a
    non-null handle, one unique document target, and the exact reviewed
    `OBJECTS` marker; wrong-kind, wrong-section, missing, and ambiguous targets
    fail typed without queueing a patch. The session builds the shared identity
    directory lazily and reuses it across requests. Classifier parity spans
    ASCII/Binary AC1009 through AC1032; verified materialization and exact
    inverse span the eight modern dialects. AC1009 Binary retains its physical
    group-code rejection, while ASCII applicability remains explicitly
    unreviewed. This checkpoint does not implement handle remap, owner changes,
    dictionary membership, lifecycle, cross-document remap, applicability,
    clone/delete, or family graph CRUD.
    M14.3ap applies exact named-symbol resolution before common exact-text edits
    enter the session. Layer group 8 must match one `LAYER` table group-2 name;
    linetype group 6 must match one `LTYPE` name. Matching remains exact and
    case-sensitive because case-folding and legal-name policy are not reviewed.
    Missing and duplicate table names fail typed without queueing. Layout group
    410 and color-name group 430 now require dedicated layout/color-book
    resolution rather than passing through generic raw-text replacement. The
    named table directory is built lazily once per session and shared by layer
    and linetype requests; oversized exact text still fails the resource gate
    before lookup. Paired ASCII/Binary classification, materialization, strict
    semantic verification, and exact inverse cover all nine dialects. This
    checkpoint does not define name folding, XREF naming, symbol characters,
    layout/color-book resolution, symbol-table record creation, applicability,
    or family graph CRUD.
    M14.3aq adds a source-bound `OBJECTS`/`LAYOUT` directory and resolves common
    group 410 on the read side. Layout names come only from group 1 inside the
    exact `AcDbLayout` subclass; group 1 inherited from `AcDbPlotSettings`,
    application-group content, wrong subclasses, wrong sections, and unclosed
    sections cannot become targets. Each layout record retains typed missing,
    unique, or duplicate name cardinality. Common layout semantics then retain
    raw field failures and distinguish absent, exact unique, missing, and
    ambiguous target names without selecting an occurrence. ASCII/Binary
    fixtures span all nine dialects plus subclass collision, application-group,
    wrong-section, cancellation, source identity, and public-bound cases. Name
    matching remains byte-exact and case-sensitive. This checkpoint does not
    admit layout edits, validate reciprocal block-record ownership, define case
    folding or legal names, resolve color books, claim applicability, or add
    layout lifecycle and family graph CRUD.
    M14.3ar composes the exact layout-object directory with common-field edit
    admission. `classify_entity_common_layout_edit` distinguishes non-layout
    fields, wrong value kinds, missing names, ambiguous names, and one exact
    same-document target without changing source bytes. The edit session builds
    and reuses the layout directory lazily, after the existing exact-text
    resource ceiling. Accepted modern ASCII/Binary edits flow through canonical
    insertion/replacement, strict reparse, target-bearing layout semantics,
    semantic/raw verification, and byte-identical inverse restoration. Public
    classifier parity spans all nine dialects; AC1009 Binary remains physically
    unable to encode group 410 and fails typed without queueing. Matching stays
    byte-exact and case-sensitive. This checkpoint does not validate reciprocal
    block-record ownership, layout lifecycle, case folding, legal names,
    color-book resolution, applicability, or family graph CRUD.

    M14.3as replaces the final unreviewed common exact-text projection with a
    typed color-book envelope. Group 430 requires exactly one `$` and nonempty
    book/color components, returned as source-backed spans. Structured
    semantics additionally require usable reviewed group-420 true color and
    group-62 indexed color states from the same entity; related absence,
    cardinality, decode, and domain failures remain typed without selecting or
    repairing values. Parsing is group-order independent and cancellation-
    aware. Paired ASCII/Binary fixtures span AC1009 through AC1032 and cover
    delimiter, relation, provenance, source, and bound failures. This
    checkpoint does not load color-book files, validate external mappings,
    admit group-430 edits, review applicability, or add family graph CRUD.

    M14.3at admits a group-430 edit only after source-bound candidate syntax
    and same-entity color relations succeed. The public classifier retains
    typed non-color, value-kind, missing/empty/multiple separator, and unusable
    related group-420/group-62 outcomes. The edit session lazily reuses one
    common-domain directory and leaves its queue unchanged on rejection.
    Accepted modern ASCII/Binary edits flow through canonical insertion or
    replacement, strict reparse, structured color-book projection, generic
    semantic/raw verification, and byte-identical inverse restoration. AC1009
    proves typed rejection. This checkpoint does not access `.acb` files,
    validate external mappings, compose pending 62/420/430 tuple edits, review
    applicability, or add family graph CRUD.

    M14.3au introduces a typed composite common color-book patch for atomic
    groups 62/420/430 CRUD. The operation validates name syntax and pending-
    field conflicts before planning, uses reviewed indexed/true-color types,
    and rolls back every newly queued component if any later component fails.
    AC1009 rejects the tuple consistently in ASCII and Binary because the
    pre-R13 Binary group-code wire cannot encode it. Modern paired fixtures
    prove full insertion and replacement, strict structured semantics, generic
    three-field verification, exact inverse, duplicate-source rollback,
    duplicate-pending rejection, cancellation/resource inheritance, and debug
    redaction. This checkpoint does not access `.acb` data, validate external
    mappings, reset a complete tuple, review broader applicability, or add
    family graph CRUD.

    M14.3av adds `ResetCommonColorBook` as the atomic inverse of the composite
    set operation. Each unique explicit 62/420/430 singleton becomes an exact
    deletion, absent optional members remain no-ops, and a wholly implicit
    tuple reports `AlreadyImplicit`. Indexed color then exposes the reviewed
    BYLAYER default 256 while true color and color name expose absence. A
    duplicate or other component failure rolls the request back to its exact
    queue checkpoint and preserves earlier unrelated edits. Paired fixtures
    cover complete, partial, and absent tuples in ASCII/Binary across all nine
    dialects, strict generic semantic verification, and exact inverse. This
    checkpoint does not access `.acb` data, review applicability, add family
    patch CRUD, or implement entity insert/clone/delete graphs.

    M14.3aw introduces `compose_transaction_plans` over exact source-bound M11
    plans. It validates every input against one raw document before rebuilding
    all patches through the existing resource-bounded builder, which preserves
    source order and rejects cross-plan overlap or duplicate insertion anchors.
    Empty input is an empty plan; plan count, replacement size, cancellation,
    and source mismatch remain typed. The composed transaction captures fresh
    inverse bytes from the unchanged source. A verifiable entity edit plan can
    absorb supplemental raw plans without losing its field postconditions.
    Paired AC1009-through-AC1032 ASCII/Binary fixtures compose M11 handle
    assignment and `$HANDSEED` advancement with an M14 field reset, then
    strict-reparse, verify semantics/raw bytes, and restore the exact source.
    This checkpoint does not encode entity drafts, choose placement/owner,
    reserve handles across sessions, or implement insert/clone/delete.

    M14.3ax introduces a source-bound placement directory for the two entity-
    bearing containers. Indexed `ENTITIES` sections use the byte before their
    first content group (or exact `ENDSEC` when empty); closed BLOCK definitions
    use the byte before their first member (or exact `ENDBLK` when empty).
    Container-start insertion cannot split an existing entity sequence. Every
    assessment retains its exact section or BLOCK-record target, and malformed
    section closure, BLOCK closure, or orphan nonzero content fails typed
    without guessing. AC1009-through-AC1032 paired ASCII/Binary fixtures apply
    a minimal record at every anchor through the bounded transaction builder
    and strict-reparse the result. This checkpoint does not choose owner group
    330, allocate handles, encode drafts, or implement insert/clone/delete.

    M14.3ay prepares handle ranges for records that are not yet present. A
    source-bound `DxfHandleReservationPlan` retains the existing M11.2a
    consecutive proposal and one transaction that advances the exact
    `$HANDSEED` payload. Zero-count requests yield an empty transaction;
    policy unavailability, exhaustion, source mismatch, resource limits, and
    cancellation remain typed. M11.2b assignment and the reservation planner
    now share one uppercase hexadecimal encoder. Across all nine dialects and
    both physical formats, a reserved handle is composed with an M14.3ax
    placement, an identified record is inserted, the post-image strict-reparses
    with the expected identity and successor seed, and the composed inverse
    restores the exact source. This is optimistic source-bound planning, not a
    global/concurrent reservation lock. Typed draft encoding, owner group 330,
    and insert/clone/delete remain open.

    M14.3az binds an M14.3ax placement to one caller-selected owner handle
    before draft encoding. The handle must be non-null, uniquely identified,
    and admitted as an exact uniquely named record from a completely closed
    `BLOCK_RECORD` table. For a BLOCK placement, the BLOCK marker must also
    expose exactly one outside-application group 330 candidate whose uniquely
    resolved target is the same record. Cardinality, lexical, null, missing,
    ambiguous, wrong-record-kind, and mismatch failures are distinct and queue
    no mutation. `ENTITIES` placement does not infer model/paper space from
    group 67 or layout 410. Paired AC1009-through-AC1032 ASCII/Binary fixtures
    preserve parity; AC1009 exercises typed BLOCK-owner absence because the
    pre-R13 Binary wire cannot carry group 330. This checkpoint produces a
    source-bound preparation value only. Applicability, draft encoding,
    insertion bytes, clone/delete graphs, and full CRUD remain open.

    M14.3ba introduces `DxfEntityDraftName` and a source-bound identity
    preparation plan. Only the generated 45 canonical topics and 14 reviewed
    aliases can be selected; unknown/custom names cannot enter this typed path.
    Preparation joins one exact M14.3az placement-owner binding with an M14.3ay
    reservation from the same document and requires reservation cardinality
    exactly one. The result retains the chosen exact wire name, canonical topic
    relation, reserved handle, placement, owner BLOCK_RECORD, and successor
    `$HANDSEED` transaction. All 59 names are exercised across nine dialects
    and both physical formats; one plan per pair strict-reparses the advanced
    seed and materializes an exact inverse. This is registry and identity
    preparation, not applicability or semantic support for all names. Family
    draft payloads, record encoding, insertion, post-insert semantic checks,
    clone/delete, and full CRUD remain open.

    M14.3bb adds a fail-closed dialect gate before any prepared draft can reach
    record encoding. The identity, reservation transaction, and document must
    retain one exact source identity, `$ACADVER` must be one supported value,
    and the generated descriptor must classify the exact canonical or alias
    name as `Applicable`. `NotApplicable`, `NotYetReviewed`, absent,
    unsupported, invalid, and ambiguous version evidence remain distinct.
    With the current reviewed registry, DGN/DWF underlay names are admitted
    from AC1021 and PDF underlay from AC1024; the other 56 names fail closed
    until their applicability ranges receive normative evidence. Paired tests
    cover all 59 names over all nine ASCII/Binary dialects and retain the
    reversible seed transaction. This checkpoint does not encode entity
    envelopes or payloads, insert records, verify inserted semantics, clone,
    delete, or complete any topic.

    M14.3bc records the first canonical-topic applicability range. Autodesk
    describes the newer MESH object type as implemented in the AutoCAD 2010
    context for subdivision-surface workflows. The generated registry now
    admits canonical MESH from AC1024 through the open-ended supported range
    and returns `NotApplicable` for AC1009 through AC1021. Its source GUID and
    normalized one-row facts receipt remain attached to the descriptor. Matrix
    tests keep all 59 names and both physical formats closed: four names have
    reviewed ranges and 55 remain `NotYetReviewed`. No MESH topology, family
    payload, record encoding, insertion, semantic postcondition, or CRUD claim
    changes in this evidence-only checkpoint.

    M14.3bd records canonical MLEADER applicability from Autodesk's statement
    that multileaders display as proxy objects in versions before AutoCAD 2008.
    The generated descriptor starts at AC1021 and keeps its source GUID and
    normalized one-row facts receipt. The separately classified behavioral
    `MULTILEADER` wire alias remains `NotYetReviewed`: a family compatibility
    statement is not treated as proof of that exact marker's version floor.
    Matrix tests therefore contain five reviewed names and 54 fail-closed
    names. Nested grammar, MLEADERSTYLE resolution, geometry, encoding,
    insertion, and CRUD remain unchanged.

    M14.3be records canonical LIGHT applicability from Autodesk's conversion
    boundary between pre-2007 lighting and the AutoCAD 2007/2008 lighting
    format. The generated descriptor starts at AC1021 and carries the exact
    compatibility-page GUID and normalized one-row facts receipt. Matrix tests
    now contain six reviewed names and 53 fail-closed names. LIGHT fields,
    type/vector relations, photometric settings, rendering, shadows, encoding,
    insertion, CRUD, and `Complete` support remain unchanged.

    M14.3bf records exact ACAD_TABLE alias applicability. Autodesk's AutoCAD
    2005 API history marks the Table entity/API as new, and the normative TABLE
    DXF page separately fixes its group-0 marker as `ACAD_TABLE`. The generated
    descriptor starts at AC1018 with the API-history GUID and normalized facts
    receipt. Canonical `TABLE` remains `NotYetReviewed`; the documentation topic
    is not assumed to be an interchangeable exact marker. Matrix tests now
    contain seven reviewed names and 52 fail-closed names. Cell grammar, style
    resolution, layout, encoding, CRUD, and `Complete` remain unchanged.

    M14.3bg records canonical HELIX applicability. Autodesk's AutoCAD 2007 API
    history marks `IAcadHelix`, its constraint enum, and its twist enum as new;
    the normative HELIX DXF page separately defines HELIX entity fields and the
    `AcDbHelix` subclass. The generated descriptor starts at AC1021 with the
    API-history GUID and normalized facts receipt. Eight names now have reviewed
    ranges and 51 stay fail-closed. Existing field evidence does not advance to
    analytic composition, encoding, CRUD, or `Complete` in this checkpoint.

    M14.3bh records canonical LWPOLYLINE applicability. Autodesk's legacy
    polyline guidance states that 2D polylines are created as lightweight
    entities from Release 14 and converts earlier-release 2D polylines on open;
    the normative LWPOLYLINE DXF page separately defines `AcDbPolyline` fields.
    The generated descriptor starts at AC1014 with the guidance GUID and
    normalized facts receipt. Nine names now have reviewed ranges and 50 stay
    fail-closed. Existing semantic/geometry coverage does not establish entity
    insertion, full CRUD, writer closure, or `Complete`.

    M14.3bi moves `SECTIONOBJECT`, `EXTRUDEDSURFACE`, `LOFTEDSURFACE`,
    `PLANESURFACE`, `REVOLVEDSURFACE`, and `SWEPTSURFACE` from AutoCAD-oracle
    alias provenance to Autodesk's valid-DXF-name inventory. AutoCAD 2007 API
    history independently marks their public Section/Surface classes as new,
    so exact alias admission begins at AC1021. Canonical `SECTION` and
    `SURFACE` remain fail-closed because neither topic label is assumed to be a
    concrete group-0 marker. Fifteen names now have reviewed ranges and 44 stay
    unreviewed; payload semantics, modeler data, CRUD, and `Complete` do not
    advance.

    M14.3bj records a shared AC1009 floor for the 16 canonical DXF names in
    Autodesk's table of entities introduced before Release 13: `3DFACE`, `ARC`,
    `ATTDEF`, `ATTRIB`, `CIRCLE`, `DIMENSION`, `INSERT`, `LINE`, `POINT`,
    `POLYLINE`, `SEQEND`, `SHAPE`, `SOLID`, `TEXT`, `VERTEX`, and `VIEWPORT`.
    The established dialect registry identifies AC1009 as Release 11/12, so
    the range covers every Core 1.0 dialect without claiming older formats.
    Thirty-one names now have reviewed ranges and 28 remain unreviewed.
    VIEWPORT's separately documented `entmake` restriction and every family's
    payload validation, insertion policy, CRUD, and `Complete` state remain
    unchanged.

    M14.3bk adds the first typed whole-record family draft. `DxfPointDraft`
    carries an exact existing layer, finite WCS location, and the explicit
    placement-conditioned common values required to emit canonical `POINT`
    bytes. AC1009 uses its legacy envelope; AC1012+ adds owner and
    `AcDbEntity`/`AcDbPoint`; AC1015+ requires an explicit reviewed lineweight,
    while an `ENTITIES` placement additionally requires an exact uniquely
    resolved same-document layout. BLOCK-local records reject layout and omit
    group 410. ASCII/Binary tests cover all nine dialects, both placement
    families where owner evidence is representable, strict post-image reparse,
    POINT semantics, `$HANDSEED` composition, inverse byte identity, and typed
    applicability/reference/lexical failures. This record plan does not yet
    expose the final edit-session `insert`, clone/delete closure, or POINT
    `Complete` support.

    M14.3bl adds the first atomic typed entity insertion plan. The document
    consumes one M14.3bk record, inserts its canonical bytes at the retained
    placement anchor, and composes the insertion with its `$HANDSEED`
    transaction. The result is the existing verified `DxfEntityEditPlan`, not
    a parallel writer path. Its new family expectation resolves the allocated
    handle, proves canonical POINT classification and exact ENTITIES/BLOCK
    membership, checks the represented owner and explicit common fields, and
    compares the WCS location. All nine ASCII/Binary dialect pairs pass direct
    verification, create-new strict-reparse/write/journal, and byte-identical
    inverse restoration; typed tamper cases cover missing/duplicate identity,
    wrong family, common-field mismatch, and geometry mismatch. Unified
    session insertion, multi-record reservation, optional POINT fields, and
    the remaining CRUD ladder stay open.

    M14.3bm adds `DxfEntityEditSession::insert(placement, draft)` for canonical
    POINT. `DxfEntityDraft` now optionally retains the exact caller-selected
    BLOCK_RECORD owner; session insertion requires it and never infers space
    from layout or group 67. Admission composes owner validation, one handle
    reservation, generated dialect applicability, typed record encoding, and
    the M14.3bl verified insert plan without changing the source. `finish`
    releases its raw transaction and `finish_verifiable` releases the family
    postcondition. Compact typed outcomes cover owner, allocation,
    applicability, record, duplicate-insert, and mixed-update failures. Paired
    ASCII/Binary tests span all nine dialects and strict semantic/inverse
    verification. Multi-record inserts and mixed insert/update sessions remain
    fail-closed until one shared reservation and ordinal-independent verifier
    are implemented.

    M14.3bn batches multiple POINT insert calls under one final handle
    reservation. The session stores owned encoded records and expectations,
    not caller references or cloned `$HANDSEED` transactions. Each admission
    proposes the next handle from the exact source policy, so a rejected draft
    leaves both queue length and the next successful handle unchanged. Finish
    validates the final consecutive allocation, groups equal anchors in
    handle/caller order, emits one insertion patch per anchor, and composes
    those patches with one successor-seed update. Three-record ASCII/Binary
    tests span all nine dialects, prove `0x40..0x42` identities and successor
    `0x43`, semantic verification, and byte-identical inverse restoration.
    Insert/update mixing and multi-family batches remain fail-closed.

    M14.3bo completes the public POINT family payload on the read and insert
    paths. Source-order evidence and fixed cards now include thickness `39` and
    UCS X-axis angle `50`; typed semantics apply their documented zero defaults
    only when absent and retain invalid/duplicate evidence. `DxfPointDraft`
    accepts optional thickness, a complete nonzero extrusion direction, and
    the optional angle. The writer omits absent defaults and emits explicit
    values in canonical location/thickness/extrusion/angle order. Post-image
    verification distinguishes explicit from defaulted state and checks exact
    binary64 values for every field. Minimal and all-explicit ASCII/Binary
    records pass all nine dialects, including typed tamper and inverse tests.
    POINT update, clone/delete, and mixed insert/update sessions remain open.

    M14.3bp adds atomic WCS-location replacement for an existing canonical
    POINT through the unified `DxfEntityPatch` surface. Admission requires
    unique source-backed groups `10`, `20`, and `30`; missing or duplicate
    components, a wrong family, duplicate family patch, non-finite encoding,
    resource limits, and cancellation leave the session unchanged. The three
    raw replacements count as one logical edit and compose with independent
    common-property updates. Verification reparses the output, resolves the
    same raw-record ordinal, checks the exact typed location, and returns the
    byte-identical inverse journal. Paired ASCII/Binary tests cover every Core
    dialect. Other POINT fields, reset, clone/delete, and mixed insert/update
    sessions remain open.

    M14.3bq adds atomic replacement of one existing explicit POINT thickness
    group `39`. Admission requires a unique source-backed thickness occurrence;
    absence, duplicates, wrong classification, duplicate thickness patches,
    non-finite encoding, resource limits, and cancellation leave the session
    unchanged. Location and thickness use distinct patch identities and may
    compose as independent logical edits, while insert/update mixing remains
    fail-closed. Verification requires the exact typed thickness and explicit
    semantic state on the same raw-record ordinal before returning the
    byte-identical inverse. Paired ASCII/Binary tests cover all nine Core
    dialects. Missing-field insertion, reset to the documented zero default,
    extrusion/angle updates, clone/delete, and POINT `Complete` remain open.

    M14.3br extends the same POINT thickness patch to an absent group `39`.
    Unique explicit thickness still uses exact source-span replacement, while
    documented defaulted absence now uses one zero-width insertion immediately
    after the last unique source-backed location component. Missing or duplicate
    location evidence cannot provide an insertion predecessor and fails typed;
    duplicate thickness remains unselectable. The shared insertion-byte helper
    preserves LF, CRLF, CR, and Binary framing without adding a parallel writer.
    Verification requires the requested exact binary64 value in the `Explicit`
    state, and the inverse removes the inserted bytes exactly. Paired fixtures
    cover all nine Core dialects in ASCII and Binary, plus CRLF and malformed
    anchor evidence. Reset to the implicit zero default, extrusion/angle
    updates, mixed insert/update sessions, clone/delete, and POINT `Complete`
    remain open.

    M14.3bu extends `SetExtrusion` to the fully absent tuple whose three
    components use the documented `(0,0,1)` default. A unique thickness group
    `39` is the preferred insertion predecessor; when thickness is absent, the
    last of three unique source-backed location components is used. Duplicate
    thickness or unusable location evidence fails typed. The three encoded
    groups are concatenated in `210/220/230` order into one zero-width patch,
    preserving LF, CRLF, CR, or Binary framing. The receipt reports `Inserted`,
    strict verification requires the requested tuple in the `Explicit` state,
    and the inverse removes the complete inserted sequence. Paired
    ASCII/Binary fixtures cover all nine Core dialects. Partial
    explicit/default extrusion tuples, extrusion reset, angle updates, mixed
    insert/update sessions, clone/delete, and POINT `Complete` remain open.

    M14.3bs adds `DxfPointPatch::ResetThickness` under the existing thickness
    patch identity. One unique explicit group `39` becomes one exact deletion
    patch and verifies as the documented zero value in the `Defaulted` state.
    Documented absence returns an `AlreadyImplicit` receipt without consuming
    an edit slot or reserving the patch identity, so a later set remains
    admissible. Multiple thickness occurrences remain a typed failure, while a
    queued set or reset blocks a second thickness request. The POINT receipt now
    distinguishes inserted, replaced, reset, and already-implicit outcomes.
    Paired ASCII/Binary fixtures cover all nine Core dialects, strict semantic
    verification, cancellation, tampering, and byte-identical inverse
    restoration. Extrusion/angle updates, mixed insert/update sessions,
    clone/delete, and POINT `Complete` remain open.

    M14.3bt adds atomic replacement of one complete explicit POINT extrusion
    tuple through `DxfPointPatch::SetExtrusion`. Admission requires unique
    source-backed groups `210`, `220`, and `230` and a nonzero requested
    direction. Missing or duplicate components, a zero vector, wrong family,
    duplicate extrusion patches, non-finite encoding, resource limits, and
    cancellation queue nothing. Location, thickness, and extrusion retain
    distinct patch identities and may compose as separate logical edits.
    Verification resolves the same raw-record ordinal, requires the exact
    requested tuple with all three components in the `Explicit` state, and
    returns the byte-identical inverse. Paired ASCII/Binary fixtures cover all
    nine Core dialects. Absent/default extrusion insertion or reset, angle
    updates, mixed insert/update sessions, clone/delete, and POINT `Complete`
    remain open.

    M14.3bv completes every partial absent/unique POINT extrusion tuple. Each
    unique explicit component is replaced in place, while every consecutive
    missing run is encoded in canonical `210/220/230` order and inserted at
    its source-order gap. One logical `Extrusion` edit may therefore contain
    two or three physical patches and reports `Composite`. Partial source
    evidence must already retain canonical component order; reordered or
    duplicate components fail typed and queue nothing. Replacement and
    insertion preserve local LF, CRLF, CR, or exact Binary framing. All six
    partial masks pass all nine ASCII/Binary Core dialect pairs with strict
    post-image verification and byte-identical inverse restoration. Extrusion
    reset, angle updates, mixed insert/update sessions, clone/delete, and POINT
    `Complete` remain open.

    M14.3bw adds `DxfPointPatch::ResetExtrusion` under the existing extrusion
    patch identity. Every unique explicit component in any complete or partial
    tuple becomes one exact deletion patch; a fully absent tuple returns
    `AlreadyImplicit` without consuming an edit slot or reserving the patch
    kind. Duplicate components fail typed before any deletion is admitted, and
    a queued set or reset blocks a second extrusion request. Verification
    requires the documented `(0,0,1)` value with all three component states
    `Defaulted`, so an explicit encoding of the same numeric tuple does not
    satisfy reset. All seven nonempty masks pass all nine ASCII/Binary Core
    dialect pairs with exact inverse restoration. Angle updates, mixed
    insert/update sessions, clone/delete, and POINT `Complete` remain open.

    M14.3bx adds `DxfPointPatch::SetUcsXAxisAngle` for optional POINT group
    `50`. One unique explicit angle is replaced at its exact span; an absent
    angle is inserted after the last unique explicit extrusion component, or
    after the unique thickness/required-location fallback when the extrusion
    tuple is fully absent. Duplicate angle or ambiguous predecessor evidence
    fails typed and queues nothing. Angle owns a distinct patch identity and
    composes with location, thickness, and extrusion edits. Replacement and
    insertion preserve LF, CRLF, CR, or Binary framing, while verification
    requires the exact binary64 value in the `Explicit` state. Replacement and
    all eight extrusion-mask insertion states pass all nine ASCII/Binary Core
    dialect pairs. Angle reset, mixed insert/update sessions, clone/delete, and
    POINT `Complete` remain open.

    M14.3by adds `DxfPointPatch::ResetUcsXAxisAngle` under the existing angle
    patch identity. One unique explicit group `50` becomes one exact deletion
    patch; an absent angle returns `AlreadyImplicit`, queues no edit, and does
    not reserve the angle patch kind. Duplicate angle evidence fails typed
    before a transaction escapes, while a queued set or reset blocks a second
    angle request. Post-image verification requires both the documented zero
    value and `Defaulted` semantic state, so an explicit zero does not satisfy
    reset. Every ASCII/Binary Core dialect pair passes strict reparse,
    verification, and byte-identical inverse restoration. Mixed entity
    insert/update sessions, clone/delete, and POINT `Complete` remain open.

    M14.3bz removes the session-level insert/update exclusion. Common-field and
    POINT-family updates may be queued before or after one or more POINT drafts;
    one combined edit limit covers every logical operation. Finalization
    composes handle reservation, `$HANDSEED`, record insertions, common-field
    patches, and POINT patches into one source-bound transaction. Same-offset
    insertion fragments keep existing-record updates before new entity records.
    Each existing-record semantic expectation shifts by the exact number of
    inserted records placed before its marker, so earlier-section insertion
    cannot invalidate later-record verification. Both API orders and earlier-
    section ordinal shifts pass every ASCII/Binary Core dialect with strict
    verification and byte-identical inverse restoration. Clone/delete and
    POINT `Complete` remain open.

    M14.3ca adds the first reference-safe whole-record delete operation. The
    session admits one standalone canonical POINT only when its record owns one
    non-null parsed handle, the document resolves that handle uniquely back to
    the selected record, and no uniquely resolved pointer or owner from another
    record targets it. Admission deletes the exact complete raw-record span and
    retains a postcondition requiring the handle to disappear before the
    byte-identical inverse is released. Missing, invalid, null, multiple, or
    ambiguous identity, incoming references, wrong families, cancellation, and
    queued-operation mixing fail typed without mutating the session. Paired
    fixtures cover all nine ASCII/Binary Core dialects. Handleless deletion,
    mixed/multi-delete sessions, clone, and POINT `Complete` remain open.

    M14.3cb adds `DxfEntityEditSession::clone_entity` for canonical semantic
    POINT cloning. Admission reconstructs a typed draft only when every source
    group belongs to the currently modeled POINT envelope, the destination is
    the same entity container, modern owner identity is unchanged, and layer,
    layout/lineweight, location, thickness, extrusion, and angle semantics are
    usable without collapsing partial or invalid state. Unsupported common
    properties, XDATA/application groups, pending source updates, placement
    drift, and owner drift fail typed. Successful clones allocate a fresh
    handle and reuse the verified insertion/`$HANDSEED`/inverse pipeline.
    Defaulted clones cover every ASCII/Binary Core dialect, while an explicit
    fixture retains every POINT payload value. Cross-container/owner clone,
    broader common-property clone, handleless/mixed deletion, and POINT
    `Complete` remain open.

    M14.3cc admits standalone canonical POINT deletion when record-local handle
    identity is exactly `Absent`. The public outcome distinguishes handleless
    admission from handle-backed deletion without weakening the existing
    handle receipt. Strict verification retains the expected post-image entity
    count, so the exact raw-record deletion must also prove a one-entity
    semantic reduction before returning the byte-identical inverse. Invalid,
    null, multiple, or ambiguous identities remain typed failures. Every
    ASCII/Binary Core dialect passes exact removal, retained neighboring LINE,
    strict reparse, semantic verification, and restoration. Mixed/multi-delete
    sessions and POINT `Complete` remain open.

    M14.3cd replaces the single pending delete slot with a resource-bounded
    delete-only batch. Each distinct canonical POINT is admitted under the
    existing identity and incoming-reference rules, then all exact raw-record
    transactions compose in source order. Semantic verification requires every
    handle-backed identity to be absent and every handleless expectation to
    observe the one exact final entity count. Duplicate keys remain typed and
    leave the accepted batch unchanged. Two handled deletes cover every
    ASCII/Binary Core dialect; a handled/handleless batch proves the combined
    expectation path and exact inverse. Delete/update/insert mixing and POINT
    `Complete` remain open.

    M14.3ce removes the session-wide delete exclusion while preserving
    per-record conflict safety. Updates and clones reject a source key selected
    for deletion, and delete rejects a key with pending common or POINT edits;
    unrelated work and new POINT inserts compose in either order. Verification
    ordinals apply both insertion shifts and earlier-delete reductions. The
    final transaction combines common/POINT updates, handle reservation,
    `$HANDSEED`, record insertion, and raw-record deletion, while handleless
    postconditions use the exact source-minus-deletes-plus-inserts entity
    count. Delete/update and delete/insert pass every ASCII/Binary Core dialect
    in both API orders with exact inverse restoration. Broader clone and POINT
    `Complete` remain open.

    M14.3cf retains reviewed common scalar properties during canonical POINT
    clone. Typed draft fields cover explicit paper/model space, indexed color,
    linetype scale, visibility, true color, transparency, and shadow mode;
    encoding and verification use their exact public wire domains. Defaulted
    and absent source values remain omitted, while invalid, duplicate, or
    version-inapplicable values fail before queueing. The clone matrix passes
    all ASCII/Binary Core dialects with strict semantic verification and exact
    inverse restoration. Reference/text properties, proxy graphics, XDATA,
    extension dictionaries, ownership graphs, and POINT `Complete` remain
    open.

    M14.3cg admits exact linetype preservation for canonical POINT clone. A
    group 6 source value must be uniquely resolvable in the same document's
    LTYPE table, then flows through the typed draft encoder and common-field
    semantic verifier. Invalid scope, missing/ambiguous symbols, and unsupported
    payloads remain fail-closed. Every ASCII/Binary Core dialect passes clone,
    strict reparse, and exact inverse verification. Material, plot-style,
    color-book, graph common properties, and POINT `Complete` remain open.

    M14.3ch adds same-document material and plot-style reference preservation
    for AC1012+ canonical POINT clone. Non-null groups 347 and 390 must resolve
    uniquely to the reviewed MATERIAL and ACDBPLACEHOLDER OBJECTS targets. The
    typed draft revalidates target kind, and semantic post-image verification
    checks both handles. Every applicable ASCII/Binary dialect passes clone,
    strict reparse, and exact inverse restoration. Color-book, graph common
    properties, and POINT `Complete` remain open.

    M14.3ci adds atomic color-book tuple preservation for AC1012+ canonical
    POINT clone. Explicit group 430 syntax is validated and admitted only with
    usable explicit groups 62 and 420. Canonical encoding retains the exact
    name bytes and typed colors; strict semantic post-image verification and
    inverse restoration cover every applicable ASCII/Binary dialect. Graph
    common properties and POINT `Complete` remain open.

    M14.3cj adds opaque proxy-graphics preservation for AC1012+ canonical POINT
    clone. Only exact group-92/group-310 size relations are admitted. ASCII hex
    and Binary chunks are normalized to bounded decoded bytes, canonically
    rechunked, and compared byte-for-byte during strict post-image verification.
    Invalid relations leave the queue unchanged. Graph-scoped handles,
    application groups/XDATA, and POINT `Complete` remain open.

    M14.3ck closes the standalone-delete orphan gap before graph mutation is
    implemented. Canonical POINT deletion scans the complete selected record
    before identity admission and rejects any group-102 application control or
    group-360 hard-owner occurrence with an exact typed source location. This
    covers persistent-reactor, extension-dictionary, custom, malformed, and
    unscoped ownership payload without interpreting or silently dropping it.
    AC1012+ ASCII/Binary fixtures prove reactor, extension-dictionary, and
    unscoped hard-owner rejection leaves the session unchanged. Graph-aware
    cascade/remap, application groups/XDATA clone, and POINT `Complete` remain
    open.

    M14.3cl adds one source-anchored, format-neutral entity XDATA directory as
    the prerequisite for any payload-preserving clone. Exact group 1001 opens
    a distinct application list; following codes in 1000..=1071 retain raw
    source order until another 1001, a normal group, or the record boundary.
    Duplicate application names remain separate, pre-name values remain
    orphans, and a normal group marks the open list interrupted. XDATA-shaped
    codes inside group-102 application controls remain excluded. Paired ASCII
    and Binary fixtures cover every Core dialect, source identity,
    cancellation, duplicate names, orphans, interruption, and public metadata
    bounds. APPID resolution, lexical/value validation, the per-entity 16-KiB
    policy, application semantics, clone/write, and POINT `Complete` remain
    open.

    M14.3cm extends the exact closed named-symbol table scanner with APPID and
    resolves every entity XDATA group-1001 application name. A resource-bounded
    sorted SHA-256 index selects candidate APPID names, while byte-exact source
    span comparison remains authoritative and collision safe. Missing, unique,
    and ambiguous outcomes remain typed; malformed, wrong-table, near-case,
    duplicate, and unclosed table evidence fails closed. Paired ASCII and
    Binary fixtures cover every Core dialect, source identity, cancellation,
    lookup limits, and public metadata bounds. Application-name syntax,
    group-1002 braces, typed values, the 16-KiB policy, payload semantics,
    handle remap, clone/write, and POINT `Complete` remain open.

    M14.3cn adds source-bound structural validation for every generic entity
    XDATA application. Group-1001 names above the documented 31-byte limit are
    typed invalid. Exact group-1002 `{` and `}` controls maintain nested-list
    balance; invalid controls, premature closes, leftover opens, and normal-
    group interruption remain distinct deterministic issues. Paired ASCII and
    Binary fixtures cover every Core dialect, source identity, cancellation,
    lookup limits, metadata bounds, and non-disclosing debug output. Full
    symbol-name character policy, typed values, 16-KiB enforcement, payload
    semantics, handle remap, clone/write, and POINT `Complete` remain open.

    M14.3co adds a one-to-one typed projection for every retained generic XDATA
    occurrence. Exact text/control spans, bounded group-1004 caller-buffer
    chunks, group-1005 handles, all 15 documented binary64 roles, group-1070
    signed 16-bit values, and group-1071 signed 32-bit values retain raw
    provenance. The documented 255-byte string and 127-byte decoded chunk
    ceilings are enforced; malformed, non-finite, and unsupported values remain
    explicit typed evidence. Paired ASCII and Binary fixtures cover all nine
    Core dialects, source identity, cancellation, lookup and allocation bounds,
    and non-disclosing debug output. Layer-name resolution, point/vector tuple
    grouping and transforms, per-entity 16-KiB enforcement, payload semantics,
    handle target resolution/remap, clone/write, and POINT `Complete` remain
    open.

    M14.3cp groups the four documented XDATA 3D families as immediately
    adjacent, same-context 101x/102x/103x candidates. Complete X/Y/Z tuples and
    partial axis sets remain exact; invalid numeric members keep their typed
    provenance. Compact member handles resolve through the owned typed
    directory, and grouping cannot cross normal-group gaps, application,
    entity, or source boundaries. Paired ASCII and Binary fixtures cover all
    nine Core dialects plus partial, reordered, duplicate, interrupted, orphan,
    source-identity, cancellation, lookup, metadata, and debug-redaction
    boundaries. Coordinate transforms, layer-name resolution, per-entity
    16-KiB accounting, payload semantics, handle target resolution/remap,
    clone/write, and POINT `Complete` remain open.

    M14.3cq resolves every retained generic XDATA group-1003 occurrence against
    exact group-2 names admitted from completely closed LAYER tables. A sorted
    SHA-256 index bounds candidate selection, while byte-exact source-span
    comparison remains authoritative and collision safe. Unique, missing, and
    ambiguous results remain typed; near-case, wrong-table, multi-name, and
    unclosed-table evidence fails closed. Paired ASCII and Binary fixtures
    cover every Core dialect, application/orphan context, source identity,
    cancellation, lookup limits, compact metadata, and non-disclosing debug
    output. Per-entity 16-KiB accounting, coordinate transforms, payload
    semantics, handle target resolution/remap, clone/write, and POINT
    `Complete` remain open.

    M14.3cr adds source-bound AutoCAD-compatible XDATA capacity accounting for
    every indexed entity. AutoCAD 2027 `xdsize`/`xdroom` observations establish
    the exact 16,383-byte ceiling and logical-value costs, including decoded
    Unicode-scalar string length rather than physical wire length. Exact totals
    publish within-limit or exceeded states; unresolved APPID/layer names,
    invalid structure or typed values, partial 3D tuples, or replacement-free
    text/escape decoding failures publish only an accounted lower bound and
    compact typed issues. Empty and multiple applications, every logical value
    family, Unicode, exact 16,383 and exceeded 16,384 totals, source identity,
    cancellation, metadata bounds, and debug redaction pass all nine paired
    ASCII/Binary Core dialects. Coordinate transforms, payload semantics,
    group-1005 target resolution/remap, clone/write, and POINT `Complete`
    remain open.

    M14.3cs adds a validated, composable affine projection over every M14.3cp
    XDATA 3D tuple. AutoCAD 2027 MOVE, SCALE, ROTATE, and MIRROR observations
    establish four distinct channels: group 1010 remains unchanged; group 1011
    receives translation, uniform scale, rotation, and mirror; group 1012 omits
    translation; and group 1013 omits both translation and scale. Public
    factories retain separate position, displacement, and direction matrices
    and reject non-finite inputs, zero scale, zero axes/normals, and composition
    overflow. One source-bound entry per tuple publishes original and transformed
    values only when the tuple is complete, typed, and finite; partial, invalid,
    and derived-overflow cases remain typed unavailable. The four roles, a
    composed scale/rotate/move/mirror sequence, constructor failures, source
    identity, cancellation, compact metadata, lookup bounds, and debug redaction
    pass all nine paired ASCII/Binary Core dialects. Application payload
    semantics, group-1005 target resolution/remap, clone/write, and POINT
    `Complete` remain open.

    M14.3ct composes M14.3co typed XDATA group-1005 values with the existing
    document-local generic handle resolution. One compact source-bound entry is
    emitted per retained group-1005 application value or orphan. Invalid, null,
    missing, ambiguous, and unique states remain exact; only unique resolution
    retains the target record and identity candidate. Typed and generic lexical
    states are cross-checked, foreign entries fail lookup, and group-1005-shaped
    values inside group-102 application controls remain outside the XDATA
    projection. Invalid/null/missing/ambiguous/unique cases, application/orphan
    context, group-102 exclusion, source identity, cancellation, lookup and
    metadata bounds, and debug redaction pass all nine paired ASCII/Binary Core
    dialects. Cross-document remap, application-specific payload semantics,
    clone/write, and POINT `Complete` remain open.

    M14.3cu adds fail-closed cross-document remap planning over M14.3ct unique
    XDATA handle targets. Public mapping candidates reject null source or
    destination handles; the directory copies and sorts caller input before
    bounded lookup. Invalid/null/missing/ambiguous source resolution remains
    typed unusable. A unique source with no mapping remains `Unmapped`, exactly
    one candidate publishes `Mapped`, and duplicate candidates publish only an
    ambiguous candidate count. Every result is source-bound and keeps its exact
    source target through the composed resolution directory. All remap/source
    states, shuffled mappings, duplicate mappings, constructor failures, source
    identity, cancellation, lookup and metadata bounds, and debug redaction pass
    all nine paired ASCII/Binary Core dialects. Destination identity validation,
    application-specific payload semantics, clone/write, and POINT `Complete`
    remain open.

    M14.3cv validates each M14.3cu mapped destination handle against a separately
    supplied destination document's exact identity index. Remap-unavailable
    states propagate without destination lookup. Mapped handles become missing,
    unique, or ambiguous with exact target count; only unique resolution can
    derive its exact destination record and identity candidate. Compact entries
    retain both source and destination identities and resolve destination target
    evidence through the owned identity directory without per-entry copying.
    Every remap/destination state, unique evidence, duplicate identities,
    dual-source foreign lookup, cancellation, metadata/lookup bounds, and debug
    redaction pass ASCII-to-ASCII, ASCII-to-Binary, Binary-to-ASCII, and Binary-
    to-Binary combinations across all nine Core dialects. Application-specific
    payload semantics, replacement encoding, clone/write, and POINT `Complete`
    remain open.

    M14.3cw encodes an exact complete group-1005 replacement only after M14.3cv
    proves one unique destination identity. The existing canonical group
    encoder supplies destination-specific ASCII or Binary framing, uppercase
    full-width handle spelling, and the AC1009 Binary extended-data group-code
    escape. Missing, ambiguous, and remap-unavailable states retain their exact
    typed evidence and publish no bytes. Compact entries bind both source and
    destination identities; foreign lookup fails closed and debug output omits
    encoded bytes. All four format pairings pass all nine Core dialects with
    cancellation, lookup, and metadata bounds. Application-specific payload
    semantics, transaction composition, clone/write, and POINT `Complete`
    remain open.

    M14.3cx binds every M14.3cw ready byte group to the exact complete source
    group-1005 span that it supersedes. `DxfEntityXDataHandleReplacementPatch`
    follows the owned destination, remap, resolution, and typed directories
    without copying payload evidence, retaining the target plus source and
    destination identities. Patch byte lookup revalidates the complete binding;
    foreign and unavailable entries expose neither patch nor bytes. All four
    format pairings and nine Core dialects cover span association, foreign
    rejection, cancellation, lookup bounds, and debug redaction. Application-
    specific payload semantics, transaction composition, clone/write, and POINT
    `Complete` remain open.

Every item is split into reviewable micro-milestones and stops after its own
passing checkpoint.
