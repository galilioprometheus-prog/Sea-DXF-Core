# M14.3du POINT Clone XDATA Insert and Write

Retrieved: 2026-08-08

## Contract

M14.3du consumes one M14.3dt family-plus-XDATA draft into the existing atomic
destination insertion transaction while retaining compact source POINT
identity, dialect, placement, and optional owner evidence. Dedicated wrapper
outcomes carry that provenance through strict post-image verification and
create-new writing beside the existing XDATA receipts and executable inverse.

The wrapper delegates to the already verified family-plus-XDATA insertion,
verification, writer, strict reparse, and cleanup implementations. It does not
introduce another transaction or file writer.

## Verification boundary

Nine focused XDATA draft tests pass. The POINT clone wrapper spans all four
ASCII/Binary source-destination pairings for all nine Core dialects plus
AC1009-to-AC1032. Every case creates a new file, strictly reparses it, verifies
typed family and exact XDATA postconditions, retains source evidence in both
journals, and restores the destination pre-image byte-for-byte through the
inverse.

Existing files remain unchanged, pre-cancelled planning and writing create
nothing, foreign destination planning fails by identity, and final-progress
tampering rejects verification and removes the created file. Public traits,
resource bounds inherited from the composed plans, and debug redaction are
covered. Application-specific XDATA meaning, actual text transcoding, reverse-
boundary adaptation, higher entity families, and POINT `Complete` remain open.

## Gate receipts

Focused XDATA draft tests passed 9/9; adjacent POINT draft and clone/insert
suites passed 11/11 and 18/18. The workspace passed exactly 1,057 tests. Cargo-
deny, formatting, schema and release-evidence checks, workspace Clippy with
warnings denied, production safety scan, protected-surface diff, local Markdown
links, and `git diff --check` passed. No manifest, dependency, lockfile, schema,
corpus, legal, or release surface changed.

Production adds 286 lines across the provenance-preserving wrapper and public
exports; focused tests add 170 and remove 21 lines. This audit intentionally
omits its own hash.
