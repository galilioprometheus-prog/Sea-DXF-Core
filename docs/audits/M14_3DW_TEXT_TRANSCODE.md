# M14.3dw Exact Text Transcoding

Retrieved: 2026-08-08

## Contract

M14.3dw adds a bounded, replacement-free encoder for UTF-8 and the reviewed
legacy code pages backed by `encoding_rs`. Johab/Windows-1361 encoding remains
explicitly unavailable; no fallback or replacement byte is emitted.

`DxfRawDocumentView::transcode_text_span_to` reads one exact source span under
the selected value-byte ceiling, resolves the source decoder and destination
encoder from independently parsed document reports, and rejects malformed,
unsupported, indeterminate, unmappable, output-full, or unavailable states.
Before a plan escapes, it decodes the encoded destination bytes again and
requires byte-exact equality with the intermediate UTF-8 sequence. The plan
binds both document identities, the source span, both encoding resolutions,
source/UTF-8 counts, and exact destination bytes. Debug output omits text bytes.

## Verification boundary

Four integration tests cover UTF-8 and Windows-1252 in both directions across
all four ASCII/Binary source-destination pairs, same and cross-legacy values,
empty text, unmappable Unicode, unavailable Johab encoding, malformed and
unsupported source storage, indeterminate destination storage, cancellation,
hostile over-limit spans, identity/count metadata, traits, and debug
redaction. Three encoder unit tests cover exact reviewed bytes, output-full,
unmappable, unavailable, traits, and absence of replacement output.

This checkpoint provides the primitive only. It does not yet rewrite POINT or
XDATA fields, infer destination symbol names, assign application-specific
meaning, insert entities, or increase any entity-family completion claim.

## Gate receipts

Focused transcode tests passed 4/4 and focused encoder unit tests passed 3/3.
The workspace passed exactly 1,064 tests. Cargo-deny, formatting, schema and
release-evidence checks, and workspace Clippy with warnings denied passed; the
production safety scan, local Markdown links, protected-surface diff, and
`git diff --check` also passed. No manifest, dependency, lockfile, schema,
corpus, legal, release, or license surface changed. This audit intentionally
omits its own hash.
