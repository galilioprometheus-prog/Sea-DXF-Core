# Format Support Matrix

SeaCad through M3.5 can open an immutable raw ASCII framing document, enforce
or recover its EOF envelope, attach a one-pass SHA-256 source identity, and
write a separately verified byte-identical copy. The CLI now exposes
`inspect` and `verify` with JSON v1, stable exits, and path redaction. It still
makes no versioned or semantic file-format support claim because section
accounting and `$ACADVER` validation begin at M4.

| Format | Version | Read | Preserve | Semantic | Edit/Write |
|---|---|---:|---:|---:|---:|
| DXF ASCII | AC1009-AC1032 | Raw framing only | Verified Verbatim only | Not implemented | Not implemented |
| DXF Binary | AC1009-AC1032 | Not implemented | Not implemented | Not implemented | Not implemented |
| DWG | Any | Out of scope | Out of scope | Out of scope | Out of scope |
| DGN V7/V8 | Any | Out of scope | Out of scope | Out of scope | Out of scope |

A cell changes only after its milestone closes with deterministic evidence.
"Raw framing" is version-agnostic physical/envelope handling; it does not yet
validate `$ACADVER`, section semantics, or entity meaning. "Verified Verbatim"
only creates a new byte-identical file and is not an edit or canonical writer.
