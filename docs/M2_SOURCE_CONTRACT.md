# M2 bounded-source contract

Status: M2 complete through M2.3; bounded source identity

M2.1 freezes the public safety types used by later source readers and parsers.
M2.2 implements bounded byte access. M2.3 binds the complete observed byte
stream to SHA-256 and connects bounded scanning to progress and cooperative
cancellation. M2 does not claim DXF framing, parsing, or format support.

## Default behavior

`DxfReadOptions::default()` is Strict mode with the Safe resource profile.
Large is always explicit opt-in.

| Limit | Safe | Large |
| --- | ---: | ---: |
| Source bytes | 512 MiB | 2 GiB |
| Group records | 5,000,000 | 25,000,000 |
| One value | 1 MiB | 16 MiB |
| Retained diagnostics | 10,000 | 100,000 |

All byte offsets, byte counts, record counts, and limits use `u64`.
`ByteSpan` is half-open: `[start, end)`. Its constructors reject reversed
bounds and integer overflow.

## Read modes

- Strict accepts only standard-conforming framing.
- Compatible may apply only recoveries explicitly added to the allowlist at M3.
- Selecting Compatible does not authorize semantic guessing.
- M3.2 first consumes the modes for ASCII group framing; its allowlist is
  maintained in `docs/M3_ASCII_FRAMING_CONTRACT.md`.

## Stable codes

Fatal errors:

| Code | Meaning |
| --- | --- |
| `DXF-E0001` | Source I/O operation failed |
| `DXF-E0002` | Operation cancelled |
| `DXF-E0101` | Source-byte limit exceeded |
| `DXF-E0102` | Record-count limit exceeded |
| `DXF-E0103` | Value-byte limit exceeded |
| `DXF-E0105` | A `u64` byte range overflowed |

Non-fatal diagnostics:

| Code | Meaning |
| --- | --- |
| `DXF-W0001` | Further diagnostics were truncated at the selected limit |

The diagnostic cap is non-fatal: hostile diagnostic volume is bounded while an
otherwise inspectable raw document may remain available. Fatal I/O metadata
stores the operation, `ErrorKind`, and raw OS code, but never stores a source
path or arbitrary OS error message.

## Progress and cancellation

`DxfReadProgress` represents monotonic processed/total source bytes and cannot
be constructed with processed bytes greater than the total. A synchronous
`DxfReadObserver` returns Continue or Cancel. Closures implement the observer
trait directly, and a no-op observer is provided.

`DxfCancellationToken` is cloneable and uses an atomic shared flag. It is
`Send + Sync`, allowing a UI or worker controller to request cooperative
cancellation without making the core asynchronous.

## Byte sources (M2.2)

`DxfByteSource` is a synchronous `Send + Sync` random-access contract. Its
`len()` is the source length captured at construction. `read_at` may return a
partial read near EOF; `read_exact_at` either fills the destination or returns
a path-redacted unexpected-EOF error. Every request validates `offset + length`
before indexing or I/O.

`DxfMemorySource` borrows an immutable byte slice without copying it.
`DxfFileSource` owns an already-open file or opens a path, records metadata
length, and uses a mutex-protected seek/read cursor. The mutex makes concurrent
callers safe and portable across Windows, macOS, and Linux. It does not mmap or
allocate a buffer proportional to file size.

File reads are clamped to the accepted length snapshot. Later external growth
is not exposed; shrinkage is reported as EOF by exact reads or source scans.
A scan identifies the bytes actually observed; it does not claim that external
writers cannot change file content during or after scanning.

## Source identity and bounded scan (M2.3)

`DxfSourceId` stores exactly 32 SHA-256 bytes. Its stable text form is 64
lowercase hexadecimal characters. `DxfSourceScanReceipt` pairs that identity
with the exact number of bytes hashed; SHA-256 is used for identity and
integrity preconditions, not as proof of authenticity.

`scan_dxf_source` rechecks the selected Safe or Large source-byte limit before
the first read, including for caller-defined sources. It reads through one
fixed 64 KiB buffer, accepts legitimate partial reads, retains no source-sized
buffer, and returns a receipt only after the accepted length is fully hashed.
Premature EOF and a source reporting more bytes than requested fail closed as
path-redacted read errors.

A successful scan reports initial progress, progress after every read, and a
final complete event. An empty source reports one `(0, 0)` event. Observer
cancellation and the thread-safe cancellation token both return `DXF-E0002`
before another read; a token is also checked immediately on entry and after
each potentially blocking read.

## Dependency and support boundary

M2.1-M2.2 are standard-library-only. M2.3 adds exactly `sha2 = 0.11.0`
with its default features disabled and commits Cargo's registry checksums.
Its eight locked transitive packages and their enabled feature tree are
recorded in `docs/audits/M2_3_SHA2_DEPENDENCY_REVIEW.md`; every package is
dual-licensed MIT OR Apache-2.0 and no reviewed advisory affects the locked
versions.

M2 imports no legacy code or fixture bytes and does not change the DXF support
matrix. Later framing work is specified separately in the M3 framing
contract.
