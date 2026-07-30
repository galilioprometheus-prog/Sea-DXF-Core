# M10.1m INSERT Rectangular Array

Retrieved: 2026-07-30

## Normative boundary

Autodesk [INSERT
(DXF)](https://help.autodesk.com/cloudhelp/2018/ENU/AutoCAD-DXF/files/GUID-28FA4CFB-9D5E-4880-9F11-36C97578252F.htm)
documents optional column/row counts `70/71`, column/row spacing `44/45`, their
defaults, the OCS insertion point, rotation, and extrusion. Autodesk
[MINSERT](https://help.autodesk.com/cloudhelp/2027/ENU/AutoCAD-Core/files/GUID-A780A2FA-4A2E-4574-950F-E788AB71F527.htm)
describes a rectangular array and states that its rotation applies to both the
individual block insertions and the entire array.

SeaCad therefore rotates the two array step directions with the INSERT OCS
axes. Treating the documented spacing values as drawing-unit array offsets
independent of BLOCK scale is an explicit SeaCad placement policy, not a quoted
Autodesk conformance rule.

## Implementation contract

`crates/seacad-dxf-core/src/insert_array.rs` publishes one entry for every
M10.1l transform entry. A usable entry retains:

- positive column and row counts;
- finite column and row step vectors transformed through the rotated OCS basis;
- the M10.1l base transform;
- an exact `u64` instance count; and
- bounded `(column, row)` lookup returning one translated affine transform.

The layout is constant-space. It never allocates or enumerates the rectangular
product, so two maximum signed-16-bit counts remain a small evidence object
even though they describe 1,073,676,289 positions. Out-of-range lookups return
no instance, while non-finite requested placement remains a typed failure.

Invalid array semantics, non-positive counts, non-finite spacing, and any
unavailable M10.1l transform remain typed. Directory construction preserves
source identity, checks cancellation, and uses fallible bounded allocation
only for the already-bounded INSERT entry count.

## Test evidence

`crates/seacad-dxf-core/tests/insert_array_tests.rs` covers ASCII/Binary parity
across all nine supported dialects; documented defaults; nonzero counts and
spacing; rotation of the entire array; non-default extrusion; exact
column/row WCS steps; per-index translation; out-of-range lookup; invalid,
zero, and negative counts; non-finite spacing; unresolved targets; maximum
counts without expansion; placement overflow; cancellation; source identity;
bounded directory lookup; and public traits.

## Non-claims

M10.1m does not enumerate every instance, convert BLOCK units, follow
ATTRIB/SEQEND, recursively transform member geometry, edit, write, or render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, formatting,
generated-schema drift, workspace Clippy with warnings denied, 483 workspace
tests with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/insert_array.rs` | 330 | `5ec207e699a799eceeea6798e738995dbd7214f57df144a290a87071603b8adc` |
| `crates/seacad-dxf-core/src/insert_transform.rs` | 419 | `63001c3683805bb27840affbb140435a212cf77fc60addcf422a6f9e6463d71c` |
| `crates/seacad-dxf-core/src/lib.rs` | 490 | `0f247a5f68d1cd80c090711eae86e1d82d6f59c07b6c6a0cdf117419eba1ebb2` |
| `crates/seacad-dxf-core/tests/insert_array_tests.rs` | 409 | `c405ed7b8c44daeae431939775e4f7dc8f553070e3341ecd050491a2b4246211` |
| `docs/IMPLEMENTATION_PLAN.md` | 860 | `db7b853c233a8f9c2aa60314f05d8e51917a811fb443f229779e573858fd00c1` |
| `docs/SUPPORT_MATRIX.md` | 730 | `a32086ed71c85954153b8dfb5fdf5738e7fba47f23d70fc120168e9be7be78e2` |
