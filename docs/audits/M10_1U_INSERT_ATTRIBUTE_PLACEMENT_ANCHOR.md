# M10.1u Classic ATTRIB Placement Anchor

Retrieved: 2026-07-30

## Normative boundary

Autodesk [ATTRIB
(DXF)](https://help.autodesk.com/cloudhelp/2020/ENU/AutoCAD-DXF/files/GUID-7DD8B495-C3F8-48CD-A766-14F9D7D0DD9B.htm)
states that text-start `10/20/30` applies when horizontal and vertical
justification are zero or absent, while alignment point `11/21/31` is present
and meaningful when either justification value is nonzero.

M10.1u selects between already-projected M10.1q coordinate tuples using the
typed M10.1t justification applicability. It does not recalculate either
stored tuple.

## Implementation contract

`crates/seacad-dxf-core/src/insert_attribute_anchor.rs` lazily publishes one
placement-anchor state per retained classic ATTRIB:

- usable baseline/left justification selects text start;
- any usable nonzero horizontal or vertical justification selects alignment
  point;
- the selected tuple must have all three usable components; and
- unavailable justification, text start, and alignment point remain distinct
  states.

Successful states preserve exact binary64 components and retain both underlying
justification and double semantic views. Invalid or missing components in the
unselected tuple remain inspectable but do not contaminate the selected
anchor.

Directory construction preserves source identity and reuses the cancellable,
bounded M10.1q and M10.1t directories. Raw-record, exact-entry, and
INSERT-sequence-local lookups do not allocate payload-sized buffers.

## Test evidence

`crates/seacad-dxf-core/tests/insert_attribute_anchor_tests.rs` covers
ASCII/Binary parity across all nine supported dialects; text-start and
alignment selection; exact binary64 points; unavailable selected tuples;
ignored invalid unselected tuples; unsupported justification precedence;
raw-record, exact-entry, and sequence-local lookups; cancellation; source
identity; bounds; and public traits.

## Non-claims

M10.1u does not recalculate AutoCAD's stored points, validate
horizontal/vertical combinations, apply extrusion, rotation, text-style
metrics, or INSERT transforms, decode MText extensions, associate ATTDEF
definitions, edit, write, or render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, formatting,
generated-schema drift, workspace Clippy with warnings denied, 514 workspace
tests with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/insert_attribute_anchor.rs` | 224 | `42abdbb7f5b85fe00116e3843c275fe59b4637158f7caecbc22e0e5370a710e5` |
| `crates/seacad-dxf-core/src/lib.rs` | 535 | `dd058a728da58d08121c89eb5c7294985f4a5e9e648c1a28127f0bcad9e11cd7` |
| `crates/seacad-dxf-core/tests/insert_attribute_anchor_tests.rs` | 297 | `823360f5c3af89d4c6092e7c15a4f0dadd625d7086a709658eda7ea30e5f989c` |
| `docs/IMPLEMENTATION_PLAN.md` | 970 | `e1005dd839bc96ba5ab348d9d9091c3ae6ec5e1dbe028fc2699d610dfdf463a1` |
| `docs/SUPPORT_MATRIX.md` | 810 | `ae135ae69a86f38180041cf0133aa68498d10f191155cde8b63c00426d059168` |
