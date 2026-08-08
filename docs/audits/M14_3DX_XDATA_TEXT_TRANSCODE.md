# M14.3dx XDATA Text Transcode Integration

Retrieved: 2026-08-09

## Contract

M14.3dx applies the replacement-free M14.3dw primitive only to exact XDATA
string values in group 1000. A non-ASCII value whose source and destination
decoders differ must produce a round-trip-verified destination encoding before
canonical ASCII/Binary group framing. Portable ASCII and non-ASCII values with
the same available decoder remain exact byte copies.

Successful conversions retain a compact `DxfTextTranscodeReceipt` in directory-
owned storage and entries keep only a 32-bit ordinal, preserving the existing
160-byte metadata ceiling. Receipt lookup validates the complete entry
identity. APPID group 1001 and LAYER group 1003 continue to use exact resolved
destination names; group-1002 list controls remain source-exact. A converted
group-1000 value above the 255-byte XDATA string limit is typed unavailable.

## Verification boundary

Five focused integration tests cover UTF-8/Windows-1252 in both directions
across all four ASCII/Binary pairs, exact receipt identities/spans/resolutions/
counts, application/entity readiness, APPID/LAYER/control non-transcoding,
ASCII portability with indeterminate destination storage, unmappable Unicode,
malformed and unsupported source storage, indeterminate and Johab-unavailable
destination storage, the 255-byte destination limit, traits, metadata bounds,
and debug redaction. Adjacent encoded-destination and primitive transcode suites
remain green.

One POINT clone test carries a converted Binary AC1018 group-1000 value into an
ASCII AC1021 family-plus-XDATA draft, create-new writer, strict reparse, exact
post-image XDATA verification, and executable byte-identical inverse.

Application-specific XDATA interpretation, missing destination symbol creation,
other entity text fields, higher entity families, and POINT `Complete` remain
open.

## Gate receipts

Focused suites passed 5/5 for XDATA transcode integration, 10/10 for adjacent
encoded destinations, 4/4 for the primitive, and 10/10 for draft/write
composition. The workspace passed exactly 1,070 tests. Cargo-deny, formatting,
schema and release-evidence checks, workspace Clippy with warnings denied,
production safety scan, local Markdown links, protected-surface diff, and
`git diff --check` passed. No manifest, dependency, lockfile, schema, corpus,
legal, release, or license surface changed. This audit intentionally omits its
own hash.
