# M6.4c exact HEADER name lookup

M6.4c adds exact indexed lookup over the complete HEADER variable directory.
It adds no new DXF semantic-support claim and no third-party dependency.

## Contract

`DxfRawDocumentView::lookup_header_variable` accepts raw name bytes. It does
not decode, trim, normalize, case-fold, or infer a leading dollar sign. The
result is one of:

- `Absent`, with no retained occurrence;
- `Unique`, with the one exact occurrence as primary evidence;
- `Ambiguous`, with the first two exact occurrences in source order plus the
  total exact occurrence count.

Every result retains the source identity. Primary and conflicting evidence are
the existing compact `DxfHeaderVariable` records, so callers can inspect the
exact marker span and complete value-group range without copying payloads.

## Collision and resource behavior

The one-pass HEADER index stores one keyed, process-local 64-bit fingerprint
beside each variable. The fingerprint is private and is only a candidate
filter. Every candidate is compared against the authoritative source bytes
before it can affect the result. A forced-collision test verifies that equal
fingerprints cannot create a false match.

Comparison uses one 256-byte stack buffer and exact span reads. Long names are
read in bounded chunks; lookup allocates no name, map, or result collection.
One lookup scans `V` compact variable records and reads only fingerprint
candidates, for `O(V + candidate bytes)` time and `O(1)` temporary memory.
The index adds eight fingerprint bytes per HEADER variable. Batch lookup and
semantic caching remain later work; core code must not repeatedly call this
single-name operation in a way that creates an unreviewed quadratic path.

## Verification

Tests cover all nine AC1009-AC1032 dialects through the shared ASCII/Binary
document view, exact case sensitivity, duplicate ordering/counts, empty names,
absent names, a 1,025-byte name crossing five comparison chunks, no extra
source I/O for a length-distinct miss, source I/O for a candidate, and an
artificial fingerprint collision. Public result metadata is checked for
`Copy + Send + Sync`.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `crates/seacad-dxf-core/src/header_index.rs` | `80b4cff4a221e47e44e0edb6a520a90a149801bbfd5de93fbaff580d8f4640aa` |
| `crates/seacad-dxf-core/src/raw_document.rs` | `7bbb4efca1e4adca6ef3499d9d6da5e5f61ff16410306213dc5540f676e38b1c` |
| `crates/seacad-dxf-core/src/lib.rs` | `dcf187bf5ec977cb2c3f1cc82153962eb51f225b3688bb1cc6bf3097158a6a64` |
