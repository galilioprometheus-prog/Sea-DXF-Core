# M5.2a Binary DXF raw-document contract

Status: frozen 2026-07-27

M5.2a adds automatic physical group-code encoding selection, exact
`$ACADVER` agreement, and an immutable source-backed raw snapshot. M5.2 is
split into reviewable checkpoints: M5.2b adds section/EOF accounting, then
M5.2c adds unchanged replay and CLI integration. This split does not reduce
the accepted M5.2 scope.

## Fail-closed encoding selection

After the exact 22-byte Binary DXF sentinel, the opener requires the canonical
first `0/SECTION` record:

- `00 53 45 43 54 49 4f 4e 00` selects the pre-R13 one-byte encoding;
- `00 00 53 45 43 54 49 4f 4e 00` selects R13-and-later two-byte
  little-endian group codes.

The patterns are mutually exclusive. SeaCad does not score candidates, scan
arbitrary payload bytes for `$ACADVER`, or retry a malformed source under a
different encoding. Strict and Compatible modes behave identically because no
Binary recovery is approved.

The selected cursor must then frame the complete physical source and discover
exactly one supported `$ACADVER` in an exact HEADER section. AC1009 must use
the one-byte form; AC1012 through AC1032 must use the two-byte form. Missing,
invalid, unsupported, duplicate, or encoding-disagreeing declarations are
typed fatal errors with stable codes and source spans.

## Immutable raw snapshot

`DxfBinaryRawDocument` retains:

- the complete one-pass SHA-256 `SourceId` and physical source length;
- the verified group-code encoding and existing provenance-rich dialect
  report;
- one compact owned metadata record per group: occurrence, code, wire family,
  and exact code/value/payload/full spans;
- a borrowed stable source used only for checked `read_span` calls.

The parse and SHA-256 share one sequential pass after a bounded canonical
opening probe. The document does not retain raw values, map the file, or load
the complete source. Group metadata is at most 40 bytes and stored in
independent 4,096-record chunks; growth never reallocates one contiguous
gigabyte-scale table. Memory is the chunked group table plus the cursor's 8 KiB
window and current bounded value. Progress is monotonic from zero to source
length; cancellation and Safe/Large limits remain enforced.

## Shared dialect behavior

The existing `$ACADVER` tracker now accepts an internal format-neutral raw
group observation. ASCII calls the same observation path and its public API,
tests, exact matching, diagnostics, and behavior remain unchanged. Binary
string payload spans exclude their NUL terminator, so dialect provenance points
to the exact text bytes.

## Deferred boundary

M5.2a frames every physical group but does not yet classify EOF conformance,
account sections/group-zero records, recover malformed Binary input, decode
text, expose Binary through `seacad inspect`/`verify`, or write/replay Binary
DXF. Those M5.2b/M5.2c boundaries remain explicit. Numeric semantics,
canonical writers, edits, and version conversion remain later milestones.
