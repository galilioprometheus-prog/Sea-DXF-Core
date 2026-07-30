# M10.1ad Classic ATTDEF Placement Anchor

Retrieved: 2026-07-30

## Normative boundary

Autodesk [ATTDEF
(DXF)](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-F0EA099B-6F88-4BCC-BEC7-247BA64838A4.htm)
states that text-start `10/20/30` applies when horizontal and vertical
justification are zero or absent, while alignment point `11/21/31` is present
and meaningful when either justification value is nonzero.

M10.1ad selects between already-projected M10.1z coordinate tuples using the
typed M10.1ac justification applicability. It does not recalculate either
stored tuple.

## Implementation contract

`crates/seacad-dxf-core/src/block_attribute_definition_anchor.rs` lazily
publishes one placement-anchor state per retained classic ATTDEF:

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
bounded M10.1z and M10.1ac directories. Raw-record, exact-entry, and BLOCK-local
ATTDEF lookups do not allocate payload-sized buffers.

## Test evidence

`crates/seacad-dxf-core/tests/block_attribute_definition_anchor_tests.rs`
covers ASCII/Binary parity across all nine supported dialects; text-start and
alignment selection; exact binary64 points; unavailable selected tuples;
ignored invalid unselected tuples; unsupported justification precedence;
raw-record, exact-entry, and BLOCK-local lookups; cancellation; source identity;
bounds; and public traits.

## Non-claims

M10.1ad does not recalculate AutoCAD's stored points, validate
horizontal/vertical combinations, apply extrusion, rotation, text-style
metrics, or INSERT/BLOCK transforms, decode MText extensions, compare ATTRIB
tags, associate inserted attributes, edit, write, or render.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, generated-schema
drift, formatting, workspace Clippy with warnings denied, 549 workspace tests
with zero failures/ignored tests, and `git diff --check`. No dependency
manifest or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/block_attribute_definition_anchor.rs` | 234 | `39da299448d07fa434ceb9da4657a03afa161a7e6e5d76aca1ddb164944eed3f` |
| `crates/seacad-dxf-core/src/lib.rs` | 591 | `a9cefcd3ea6b006e618d30049e6fae5e0d7eb07bec0815963a422170efd31943` |
| `crates/seacad-dxf-core/tests/block_attribute_definition_anchor_tests.rs` | 303 | `a53caf589d9577e4a4b9cc37253b44b47b38adb169df3c8a487f333f7cc3c56d` |
| `docs/IMPLEMENTATION_PLAN.md` | 1092 | `b154394e9625e9e2def968c39436855cc184317226403500b264244abe99c826` |
| `docs/SUPPORT_MATRIX.md` | 905 | `5908db04483354c418e45daf6dc61c80c7ffa37cf6720940c14d32981712bf3c` |
