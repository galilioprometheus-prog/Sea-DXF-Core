# Format Support Matrix

SeaCad through M4.1 can open an immutable raw ASCII framing document, enforce
or recover its EOF envelope, attach a one-pass SHA-256 source identity,
discover an exact HEADER `$ACADVER`, and write a separately verified
byte-identical copy. The CLI exposes `inspect` and `verify` with
English/Vietnamese human output, JSON v1, stable exits, and path redaction.
The M4.1 dialect report is a core API and is not exposed in CLI JSON v1 yet.

| Format | Version | Read | Preserve | Semantic | Edit/Write |
|---|---|---:|---:|---:|---:|
| DXF ASCII | AC1009-AC1032 | Raw framing + exact dialect discovery | Verified Verbatim only | Dialect identification only | Not implemented |
| DXF Binary | AC1009-AC1032 | Not implemented | Not implemented | Not implemented | Not implemented |
| DWG | Any | Out of scope | Out of scope | Out of scope | Out of scope |
| DGN V7/V8 | Any | Out of scope | Out of scope | Out of scope | Out of scope |

A cell changes only after its milestone closes with deterministic evidence.
"Dialect identification" validates only the exact documented HEADER variable
shape and the supported `$ACADVER` registry. It is not section accounting,
encoding interpretation, entity semantics, or geometry support. "Verified
Verbatim" only creates a new byte-identical file and is not an edit or
canonical writer.
