# M5.2b Binary DXF envelope/index contract

Status: frozen 2026-07-27

M5.2b adds exact Binary `0/EOF` conformance and an eager source-anchored
section/group-zero index to the M5.2a immutable raw document. It does not yet
expose Binary through the CLI or write/replay Binary DXF; those remain M5.2c.

## Terminal envelope

An exact terminal marker is a Binary string group code 0 whose payload is
exactly `EOF`. Matching is byte-exact, case-sensitive, and does not trim or
decode text.

- Strict requires one exact EOF and physical source end immediately after its
  NUL terminator. Missing EOF is `DXF-E0219`; any trailing physical byte range
  is `DXF-E0220`.
- Compatible may open a completely framed stream without EOF and records
  `DXF-W0210`. It may stop at the first exact EOF, retain all remaining source
  bytes as one opaque trailing span, and record `DXF-W0211`.
- Compatible never treats `eof`, ` EOF `, or another value as EOF and never
  parses or normalizes bytes in the retained trailing span.

The full source, including an allowed compatible tail, remains covered by the
document SHA-256 identity. Groups through EOF retain exact metadata. A missing
EOF stream retains every successfully framed group.

## Shared structure tracker

The existing ASCII tracker now receives a format-neutral internal observation:
occurrence, numeric code, exact payload bytes, exact payload span, and the
already-classified EOF flag. ASCII and Binary use the same state machine and
diagnostic codes; existing ASCII public types and behavior remain unchanged.

Binary public aliases expose group ranges, known/unknown/invalid section names,
closure state, sections, and the structure index without duplicating state.
The index eagerly records:

- every exact `0/SECTION` envelope and immediate group-code 2 name;
- documented HEADER, CLASSES, TABLES, BLOCKS, ENTITIES, OBJECTS, and
  THUMBNAILIMAGE names plus exact unknown-name spans;
- Closed, Interrupted, and Unclosed section ranges;
- every numeric group-code 0 occurrence and record range;
- total, inside-section, and outside-section group accounting.

Every parsed group through the accepted envelope is counted exactly once.
Malformed section topology remains visible as bounded diagnostics and never
causes semantic guessing.

## Resource and deferred boundary

Binary group framing, group metadata, section vectors, zero-group vectors, and
diagnostics use fallible allocations and remain bounded by the selected source,
record, value, and diagnostic profile. The later release performance gate must
still measure the combined raw-document/index peak on a 2 GiB corpus fixture;
M5.2b does not claim that benchmark early.

M5.2c will add verified byte-identical replay and CLI `inspect`/`verify` with
stable JSON v1 behavior. Text policy integration, numeric semantics, canonical
writers, edits, conversion, and geometry remain later milestones.
