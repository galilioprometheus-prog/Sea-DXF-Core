# M2 bounded-source contract

Status: M2.1 complete; API contract only

M2.1 freezes the public safety types used by later source readers and parsers.
It does not claim DXF framing, file I/O, hashing, or format support.

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
- Until M3, the modes are contract values only; no parser consumes them.

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

## Dependency and support boundary

M2.1 adds no dependency and remains standard-library-only. It also imports no
legacy source, test, fixture, or corpus bytes. The support matrix remains
unchanged.

M2.2 will implement bounded in-memory and file-backed byte sources with
`u64` offsets. M2.3 will add the reviewed streaming SHA-256 and wire progress
and cancellation into source scanning.
