# Format Support Matrix

SeaCad through M4.3c1 can open an immutable raw ASCII framing document, enforce
or recover its EOF envelope, attach a one-pass SHA-256 source identity,
discover an exact HEADER `$ACADVER`, account every parsed group inside or
outside non-overlapping sections, index every numeric group code 0, discover
the exact `$DWGCODEPAGE` declaration, derive a fail-closed text-storage policy,
resolve the reviewed 15-token Windows ANSI subset during the original parse,
decode any selected raw group value by document occurrence into caller-owned
UTF-8 without replacement, interpret documented CIF `\U+hhhh` controls
including valid UTF-16 surrogate pairs, recognize MIF syntax without guessing
its mapping, and write a separately verified byte-identical copy. Each storage
decode receipt retains source ID, occurrence, raw span, encoding, and terminal
status. The CLI exposes `inspect` and `verify` with
English/Vietnamese human output, JSON v1, stable exits, and path redaction.
These M4 reports and decode views remain core APIs and are not exposed in CLI
JSON v1 yet.

| Format | Version | Read | Preserve | Semantic | Edit/Write |
|---|---|---:|---:|---:|---:|
| DXF ASCII | AC1009-AC1032 | Raw framing + dialect/structure/text resolution + exact ANSI registry | Verified Verbatim only | Source-anchored caller-buffer UTF-8 view + bounded CIF decode; MIF mapping and record semantics not implemented | Not implemented |
| DXF Binary | AC1009-AC1032 | Not implemented | Not implemented | Not implemented | Not implemented |
| DWG | Any | Out of scope | Out of scope | Out of scope | Out of scope |
| DGN V7/V8 | Any | Out of scope | Out of scope | Out of scope | Out of scope |

A cell changes only after its milestone closes with deterministic evidence.
"Envelope structure" recognizes exact section boundaries and record starts
without interpreting section payloads. Unknown section names remain exact
source-backed bytes. "Text-storage policy" means UTF-8 by documented modern
version or a provenance-backed legacy declaration. The decoder is a low-level,
caller-buffer view tied to one document group occurrence. Escape
interpretation is a second bounded layer: exact CIF controls decode, while
valid MIF controls return typed selector/payload metadata without conversion.
Semantic strings, MTEXT formatting, DOS/OEM pages, and Johab are not
implemented.
"Verified Verbatim" only creates a new byte-identical file and is not an edit
or canonical writer.
