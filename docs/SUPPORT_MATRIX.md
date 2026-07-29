# Format Support Matrix

SeaCad through M7.4e can open an immutable raw ASCII framing document, enforce
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
common-owner candidates; these remain evidence rather than a validated
topology. Completely closed
known record-bearing sections expose format-neutral group-zero record chunks,
while record types remain uninterpreted. Date or elapsed-day values do not infer
calendars or timezones. Each
storage decode receipt retains source ID, occurrence, raw span, encoding, and
terminal status. The CLI exposes `inspect` and `verify` with
English/Vietnamese human output, JSON v1, stable exits, and path redaction.
These M4 reports and decode views remain core APIs and are not exposed in CLI
JSON v1 yet.

| Format | Version | Read | Preserve | Semantic | Edit/Write |
|---|---|---:|---:|---:|---:|
| DXF ASCII | AC1009-AC1032 | Raw framing + dialect/structure/text resolution + exact 15-token ANSI registry | Verified Verbatim only | Shared HEADER views + raw records + contextual handle target/ownership/role evidence; no validated topology | Not implemented |
| DXF Binary | AC1009-AC1032 | Encoding-verified immutable raw snapshot + EOF envelope + section/group-zero index | Verified Verbatim only | Shared HEADER views + raw records + contextual handle target/ownership/role evidence; no validated topology | Not implemented |
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
The Binary row claims physical raw-document, envelope/index opening, verified
unchanged replay, and CLI `inspect`/`verify` only.
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
