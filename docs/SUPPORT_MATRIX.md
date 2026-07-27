# Format Support Matrix

SeaCad through M4.2 can open an immutable raw ASCII framing document, enforce
or recover its EOF envelope, attach a one-pass SHA-256 source identity,
discover an exact HEADER `$ACADVER`, account every parsed group inside or
outside non-overlapping sections, index every numeric group code 0, and write a
separately verified byte-identical copy. The CLI exposes `inspect` and `verify`
with English/Vietnamese human output, JSON v1, stable exits, and path
redaction. Dialect and structure indexes remain core APIs and are not exposed
in CLI JSON v1 yet.

| Format | Version | Read | Preserve | Semantic | Edit/Write |
|---|---|---:|---:|---:|---:|
| DXF ASCII | AC1009-AC1032 | Raw framing + dialect/section/group-0 indexes | Verified Verbatim only | Dialect and envelope structure only | Not implemented |
| DXF Binary | AC1009-AC1032 | Not implemented | Not implemented | Not implemented | Not implemented |
| DWG | Any | Out of scope | Out of scope | Out of scope | Out of scope |
| DGN V7/V8 | Any | Out of scope | Out of scope | Out of scope | Out of scope |

A cell changes only after its milestone closes with deterministic evidence.
"Envelope structure" recognizes exact section boundaries and record starts
without interpreting section payloads. Unknown section names remain exact
source-backed bytes. This is not encoding interpretation, entity semantics, or
geometry support. "Verified Verbatim" only creates a new byte-identical file
and is not an edit or canonical writer.
