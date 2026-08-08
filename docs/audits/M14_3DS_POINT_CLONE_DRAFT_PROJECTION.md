# M14.3ds Cross-Document POINT Draft Projection

Retrieved: 2026-08-08

## Contract

M14.3ds reuses the reviewed same-document POINT clone extractor as one
immutable source snapshot, then projects that snapshot into a separately parsed
destination's canonical family draft without insertion. Source identity, key,
dialect, placement, and optional owner remain bound to the projection plan.

Destination-local layer, layout, linetype, material, and plot-style bindings
are explicit caller inputs. Presence mismatches fail before encoding; the
existing destination draft validator remains authoritative for exact symbol,
handle, placement, dialect, and resource admission. The projection preserves
reviewed common scalars, exact proxy graphics, color-book tuples, and POINT
geometry. It does not yet compose XDATA or mutate a destination.

## Verification boundary

Eleven focused draft-record tests pass. Projection coverage spans all four
ASCII/Binary source-destination format pairings for all nine Core dialects.
AC1009-to-AC1032 proves the destination's documented required lineweight
default; AC1032-to-AC1009 rejects the inapplicable explicit field. Missing and
ambiguous destination layers, unexpected local bindings, same-document use,
cancellation, public traits, and non-disclosing debug output are explicit.

The adjacent same-document clone/insert suite passes 18/18, proving the
extractor refactor did not change existing behavior. Dedicated cross-document
material and plot-style fixtures prove required source mappings, four-format-
pair destination validation, and canonical mapped-handle encoding. XDATA
composition, complete cross-container insertion/write, application-specific
XDATA meaning, actual text transcoding, and POINT `Complete` remain open.

## Gate receipts

Focused projection tests passed 11/11 and adjacent clone/insert tests passed
18/18. The workspace passed exactly 1,055 tests. Cargo-deny, formatting, schema
and release-evidence checks, workspace Clippy with warnings denied, production
safety scan, and `git diff --check` passed. No manifest, dependency, lockfile,
schema, corpus, legal, or release surface changed.

Production adds 431 lines across the reused snapshot extractor, projection
module, and public exports; focused tests add 408 lines. This audit intentionally
omits its own hash.

## Artifact receipts

Artifact hashes are recorded in the active cumulative Antigravity batch after
this audit and the checkpoint documentation are finalized.
