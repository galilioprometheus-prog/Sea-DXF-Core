# M5.1b Binary DXF group-cursor contract

Status: frozen 2026-07-27

This checkpoint adds bounded source streaming and lossless value framing on
top of the M5.1a wire registry. It still does not infer the group-code
encoding, validate SECTION/EOF semantics, build a raw document/index, expose a
Binary file through CLI `inspect`/`verify`, or write Binary DXF.

## Entry and explicit encoding

`DxfBinaryGroupCursor::new` requires:

- an immutable `DxfByteSource`;
- one explicit `DxfBinaryGroupCodeEncoding`;
- `DxfReadOptions` for the Safe/Large resource profile.

The source must begin with the exact 22-byte Autodesk sentinel. The cursor
starts at byte 22. Strict and Compatible modes behave identically because this
checkpoint has no approved Binary framing recovery.

Encoding detection remains outside the cursor. A later document opener will
probe canonical opening records, read `$ACADVER`, and reject disagreement
between the declared dialect and physical group-code encoding.

## Lossless borrowed group

Each successful `DxfBinaryGroup` exposes:

- zero-based occurrence, typed group code, and M5.1a value family;
- exact borrowed group-code bytes and complete wire value bytes;
- exact group-code, value, payload, and full-pair half-open source spans.

String `raw_value` includes its terminal NUL while `payload_span` excludes it.
Binary-chunk `raw_value` includes its u8 length prefix while `payload_span`
excludes it. Fixed-width payload and value spans are identical. Concatenating
all raw code/value bytes after the sentinel reconstructs every successfully
framed source byte exactly.

The view borrows cursor-owned buffers and remains valid only until the next
cursor call. Debug output shows metadata and lengths, never raw contents.

## Streaming and resource bounds

- source length, record count, and value payload length obey the selected
  profile;
- one 8 KiB read window prevents per-byte file I/O;
- only the current value is retained; the cursor never maps or loads the
  entire source;
- fixed values and chunks use bounded copies; NUL strings scan buffered spans;
- cancellation is checked before a record and before/after each source read;
- partial source reads are supported; a source that ends before its declared
  length fails closed;
- time is linear in consumed bytes and memory is `8 KiB + current value`.

The value limit counts payload bytes. It excludes a string terminator and a
chunk length prefix. A value exactly at the limit followed by its NUL is valid.

## Typed terminal failures

Stable fatal errors cover:

- invalid or truncated sentinel;
- truncated/invalid group-code header from M5.1a;
- a reserved group code with no documented wire family;
- truncated fixed-width or binary-chunk value;
- unterminated NUL string;
- selected source, record, and value resource limits;
- cancellation and source I/O contract failures.

After any error the cursor must be discarded. The cursor frames, but does not
semantically validate, Boolean bytes, group 999 conformance, XDATA group 1004's
127-byte semantic maximum, handle syntax, text encoding, numeric values, or
the terminal `0/EOF` envelope.

## Deferred boundary

M5.2 will detect/verify encoding and `$ACADVER`, construct the immutable Binary
raw document/index, account sections and group-0 records, enforce or report the
document envelope, provide byte-identical unchanged replay, and integrate the
CLI. Numeric semantic views, canonical writers, edits, and conversions remain
in later milestones.
