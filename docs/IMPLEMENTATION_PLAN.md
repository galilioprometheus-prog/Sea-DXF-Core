# DXF Core 1.0 Implementation Plan

Status: M5 completed through M5.2c verified Binary replay and CLI integration;
M6.7 closes the source-anchored 206-row, 214-slot HEADER inventory with explicit
ASCII/Binary parity evidence across all nine supported AC1009-AC1032 dialects;
Q1 enforces the reviewed cargo-deny dependency policy; Q2.1 stages native CI
coverage across Linux, Windows, and macOS on both x64 and ARM64; Q2.2 adds the
bounded aggregate-only offline corpus manifest and cross-platform receipt
harness without publishing private corpus identifiers; M7.4e adds conservative
semantic-role evidence over exact handle code, reference class, application
context, closed-group state, and common record-bearing section while retaining
the independent target-resolution state

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
    semantics stay deferred.
12. M8: exact basic geometry and coordinate-system preservation.
13. M9: polyline, mesh, spline, and helix families.
14. M10: blocks, text, hatch, dimensions, leaders, layouts, underlays, and
    exact-opaque ACIS/proxy/custom payloads.
15. M11: immutable atomic transactions, inverse journals, and handle policy.
16. M12: preserve-patch and canonical ASCII/Binary writers with reparse.
17. M13: evidence closure, 1,000-file/10-GB corpus gates, six native receipts,
    SBOM/notices, and DXF Core 1.0 release.

Every item is split into reviewable micro-milestones and stops after its own
passing checkpoint.
