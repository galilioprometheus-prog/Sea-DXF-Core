# M14.3dy POINT Color-Name Transcode

Retrieved: 2026-08-09

## Contract

M14.3dy applies M14.3dw replacement-free, round-trip-verified transcoding to
one reviewed common free-text field: the POINT color-book name in group 430.
Clone snapshots retain exact `DxfEntityFieldTextValue` provenance with copied
bytes for layer, layout, linetype, and color name. Only color name may convert
storage; layer/layout/linetype remain explicit destination-local bindings.

Portable ASCII and non-ASCII text with equal available decoders stay exact.
Different reviewed storage decisions must produce a verified plan whose receipt
matches the exact source field span, source/destination identities and
resolutions, and source/UTF-8/encoded byte counts. The receipt survives family
projection, XDATA composition, insertion, strict verification, and create-new
write journals. Debug output exposes only receipt metadata or presence.

Group 430 remains unavailable before AC1015. Unmappable destination text is a
typed projection issue and no draft escapes. A four-byte bounded scratch margin
was added to destination round-trip decoding because a legacy decoder may need
terminal scalar slack even when final UTF-8 length is exact; actual accepted
payload bytes remain under the original profile ceiling.

## Verification boundary

Thirteen focused family-draft tests include UTF-8/Windows-1252 color names in
both directions across all four ASCII/Binary pairs, same-decoder exact bytes,
exact field-span/count receipts, unmappable rejection, AC1009 applicability,
and redaction. Eleven XDATA draft/write tests carry one Binary AC1018-to-ASCII
AC1021 receipt through strict verification, create-new writing, XDATA
postconditions, and exact inverse. Four primitive tests include longer legacy
color-book strings that exercise the bounded decoder slack.

Other common free-text fields, application-specific XDATA meaning, automatic
symbol creation, higher entity families, and POINT `Complete` remain open.

## Gate receipts

Focused family and XDATA suites passed 13/13 and 11/11; the primitive suite
passed 4/4. The workspace passed exactly 1,073 tests. Cargo-deny, formatting,
schema and release-evidence checks, workspace Clippy with warnings denied,
production safety scan, local Markdown links, protected-surface diff, and
`git diff --check` passed. No manifest, dependency, lockfile, schema, corpus,
legal, release, or license surface changed. This audit intentionally omits its
own hash.
