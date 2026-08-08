# M14.3dv POINT Clone Legacy-Dialect Adaptation

Retrieved: 2026-08-08

## Contract

M14.3dv admits AC1032-to-AC1009 POINT cloning only for two reviewed semantic
equivalences. A modern explicit layout is represented by the selected legacy
destination placement and owner binding, and explicit `BY_LAYER` lineweight is
omitted where group 370 is not applicable. No numeric or visual approximation
is introduced.

A compact immutable adaptation mask records both decisions. The mask survives
family projection, XDATA composition, insertion planning, strict verification,
and create-new write journals. Any other explicit source lineweight returns
typed `DestinationFieldNotRepresentable` before a destination draft escapes.

## Verification boundary

Eleven focused family-draft tests cover both AC1009/AC1032 directions across
all four ASCII/Binary format pairings, exact destination bytes, empty forward
adaptation, both reverse adaptation flags, and typed rejection of explicit
lineweight 25. Nine focused XDATA draft tests carry the reverse adaptation
through create-new writing, strict reparse, family-plus-XDATA verification, and
byte-exact inverse restoration for all four format pairings.

Application-specific XDATA meaning, actual text transcoding, other explicit
modern common fields, higher entity families, and POINT `Complete` remain open.

## Gate receipts

Focused POINT draft and XDATA draft tests passed 11/11 and 9/9; the adjacent
same-document clone/insert suite passed 18/18. The workspace passed exactly
1,057 tests. Cargo-deny, formatting, schema and release-evidence checks,
workspace Clippy with warnings denied, production safety scan, protected-
surface diff, local Markdown links, and `git diff --check` passed. No manifest,
dependency, lockfile, schema, corpus, legal, or release surface changed.

Production adds 131 and removes 13 lines across adaptation evidence,
projection, XDATA propagation, and exports; focused tests add 76 and remove 16
lines. This audit intentionally omits its own hash.
