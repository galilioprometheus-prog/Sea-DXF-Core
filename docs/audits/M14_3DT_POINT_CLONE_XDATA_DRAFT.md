# M14.3dt POINT Clone XDATA Draft Composition

Retrieved: 2026-08-08

## Contract

M14.3dt selects one exact entry from the encoded per-entity XDATA destination
directory, derives the entry's owning source POINT, performs the M14.3ds family
projection in an internal XDATA-composition mode, and immediately appends the
M14.3dn payload through the existing M14.3do validator. No insertion or file
write occurs inside the new API.

The immutable result retains source key, dialect, placement, optional owner,
dual-source identities, exact encoded entry/state, and the complete canonical
family-plus-XDATA record. Public standalone family projection continues to
reject XDATA groups, so callers cannot accidentally publish a draft that drops
application payload.

## Verification boundary

Nine focused XDATA draft tests pass. The new composition path spans all four
ASCII/Binary source-destination pairings for all nine Core dialects plus the
AC1009-to-AC1032 boundary. Every ready plan is consumed through existing atomic
insertion, strict family-plus-XDATA post-image verification, and byte-exact
inverse restoration. Tests also prove standalone rejection, cancellation,
foreign source identity, unavailable/orphan POINT XDATA, public traits, and
non-disclosing debug output.

The reverse AC1032-to-AC1009 boundary remains typed unavailable in M14.3ds when
an explicit modern common field is inapplicable. A direct convenience wrapper
for insertion/create-new writing, application-specific XDATA meaning, actual
text transcoding, reverse-boundary adaptation, and POINT `Complete` remain
open.

## Gate receipts

Focused XDATA draft tests passed 9/9; adjacent POINT draft and clone/insert
suites passed 11/11 and 18/18. The workspace passed exactly 1,057 tests. Cargo-
deny, formatting, schema and release-evidence checks, workspace Clippy with
warnings denied, production safety scan, protected-surface diff, local Markdown
links, and `git diff --check` passed. No manifest, dependency, lockfile, schema,
corpus, legal, or release surface changed.

Production adds 253 lines across the composition module, guarded projection
mode, extractor admission, and public exports; focused tests add 272 lines.
This audit intentionally omits its own hash.
