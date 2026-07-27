# M4.3b2 source-anchored text view contract

Status: frozen 2026-07-27

This contract connects document encoding evidence to bounded decoding of one
raw ASCII group value. Raw bytes remain authoritative. Escape interpretation
and semantic text are later layers.

## Parse-time encoding resolution

`DxfTextEncodingReport::resolution()` is fixed during the original framing
pass from the same borrowed bytes used to compute the document `SourceId`.
It has four states:

- `Utf8(version)` for supported AC1021-or-later storage;
- `Legacy { version, code_page }` for one exact supported legacy declaration;
- `UnsupportedLegacy { version, declaration_span }` for one structurally valid
  legacy declaration with no reviewed decoder;
- `Indeterminate` when version or declaration evidence is absent, invalid, or
  ambiguous.

Modern version evidence outranks a stale legacy declaration. Therefore an
unknown token in AC1021-or-later still resolves to UTF-8 and does not emit a
legacy-unsupported diagnostic. A legacy unknown token emits stable
`DXF-E0423`, anchored to its exact declaration value span, with no fallback.

The original token is never reread to select a decoder. This prevents a later
source read from silently changing the interpretation associated with the
document snapshot.

## Group-value API and provenance

`DxfAsciiRawDocument::decode_group_value_to_utf8_without_replacement()` accepts
one group occurrence from that document and caller-owned UTF-8 destination
storage. It does not accept an arbitrary byte span.

`DxfTextValueDecodeReceipt` records:

- the document `SourceId`;
- exact group occurrence;
- exact raw value `ByteSpan`;
- the document encoding resolution;
- an optional replacement-free `DxfTextDecodeResult`.

The decode result is `None` only for `UnsupportedLegacy` or `Indeterminate`.
In those states the destination is unchanged and no group-value source read is
performed. An invalid occurrence is a fatal, path-redacted invalid-data error
and also leaves the destination unchanged.

## Bounded streaming behavior

The implementation allocates nothing. It reads at most one fixed 4 KiB source
chunk at a time and preserves decoder state across chunk boundaries. A private
four-byte staging buffer handles dependency output constraints, so caller
capacity is measured by actual UTF-8 bytes: for example, the three-byte scalar
`U+3042` completes into exactly three destination bytes.

Only `destination[..written]` is defined output. The terminal status is:

- `Complete` after the full value is consumed;
- `OutputFull` at the last complete UTF-8 prefix that fits;
- `Malformed` at invalid input, without inserting U+FFFD.

An empty destination returns zero-progress `OutputFull` before source I/O.
After any `OutputFull`, callers retry the same occurrence from the beginning
with a larger destination; `read` is evidence, not a resume cursor.

## Snapshot precondition

`DxfByteSource` now states the pre-existing snapshot requirement explicitly:
length and contents must remain stable while a borrowing document exists.
SeaCad sources do not mutate their bytes. An external writer changing an open
file violates this precondition. Whole-source rehashing per text request would
make large-file lazy views non-viable; atomic mutation preconditions belong to
the M11 transaction layer.

## Deliberate M4.3b2 boundaries

- No CIF, MIF, `\U+xxxx`, surrogate, MTEXT, or formatting escape processing.
- No classification of which group codes contain text semantics.
- No allocating `String`, interning, normalization, or semantic LRU cache.
- No Binary DXF integration.
- No CLI/JSON v1 expansion.
- No encode path or canonical writer integration.
