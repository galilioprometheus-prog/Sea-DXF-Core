# M6.4b borrowed raw-document adapter

M6.4b implements the owner-approved single borrowed adapter over validated
ASCII and Binary raw documents. It is an architectural checkpoint and adds no
new DXF semantic support claim.

## Contract

`DxfRawDocumentView` is a copyable borrow. It owns no source, group, index,
report, string, or payload. Its private adapter trait is implemented only by
SeaCad's two validated raw document types, so external code cannot substitute
an implementation with weaker invariants.

The common API exposes:

- validated `Ascii` or `Binary` identity, separate from the pre-open format
  probe's `AsciiCandidate` state;
- strict or recovered framing conformance;
- exact `SourceId`, source length, and group count;
- an occurrence-addressed `DxfRawGroup` with numeric group code, full wire
  span, and value payload span;
- explicit source reads through the existing exact-span contract;
- the already stored dialect, encoding, HANDSEED, structure, and HEADER
  variable reports/indexes.

For ASCII, the unified value payload is the exact value-line content without
its newline. For Binary, it is the exact framed payload without the
NUL terminator or binary-chunk length octet. Fixed-width numeric payload bytes
remain unchanged.

Metadata access performs no source I/O and allocates nothing. `read_span`
delegates to the original immutable document and preserves its bounds,
short-read, and source-snapshot behavior. The view exposes no mutation,
normalization, fallback decoding, filesystem path, or writer.

## Verification

Synthetic parity tests cover all nine AC1009-AC1032 dialects in ASCII and
Binary. Every group occurrence, code, exact payload readback, full-span
accounting, `SourceId`, report/index identity, and end-of-source boundary is
checked through the same API.

Additional tests verify recovered documents, out-of-range occurrences,
destination-length failure, no source reads for metadata access, explicit I/O
only for `read_span`, redacted Debug output, and compact `Copy + Send + Sync`
public metadata.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `crates/seacad-dxf-core/src/raw_document.rs` | `ca1a65ebc0280290bf1f047fe6b86e7140b7c02e0d596bc19724a690dcbab07f` |
| `crates/seacad-dxf-core/src/lib.rs` | `7b9e176a2a61f834cb5dccf2099e4287573499dabb9e5e7f8c392f72f5ef53cd` |

This checkpoint does not remove the specialized ASCII/Binary APIs. It provides
the one common semantic input path required before indexed field lookup and
the remaining typed HEADER families.
