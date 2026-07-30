# M10.1a BLOCK Definition Topology

Retrieved: 2026-07-30

## Normative boundary

Autodesk [Blocks Group Codes in DXF Files
(DXF)](https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-A3E4F1D3-79C9-489C-B7EC-3924DA7F25C9.htm)
shows each BLOCKS-section definition beginning with `BLOCK`, followed by zero or
more entity records, and ending with `ENDBLK`.

Autodesk [About BLOCKS Section Group Codes
(DXF)](https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-9DDFC343-6C87-4AF8-B3B9-77E0AB5A3031.htm)
states that BLOCK and ENDBLK occur only in the BLOCKS section and that block
definitions are never nested, although a definition may contain INSERT.
M10.1a therefore treats a second exact BLOCK before ENDBLK as an interrupted
definition rather than silently inventing nesting.

## Implementation contract

`crates/seacad-dxf-core/src/block_definition.rs` indexes exact uppercase BLOCK
definitions only over records already proven to belong to complete BLOCKS
sections. It retains every intervening group-zero record in source order and
publishes exact closed, nested-BLOCK-interrupted, or section-end-unclosed
boundary evidence. Exact ENDBLK and BLOCK boundary records remain available;
definition and boundary payloads do not enter the member slice.

## Test evidence

`crates/seacad-dxf-core/tests/block_definition_tests.rs` covers ASCII/Binary
parity across all nine supported dialects, empty and non-empty definitions,
arbitrary member record types, exact marker spelling, orphan boundaries,
nested-BLOCK interruption, section-end unclosed state, exclusion outside
BLOCKS and from incomplete sections, source identity, bounded lookups,
cancellation, and public traits.

## Non-claims

M10.1a does not decode BLOCK names, flags, base points, xref paths,
descriptions, handles, member entities, INSERT references, transforms,
edits, writes, or rendering.

## Required checkpoint gates

All required gates passed on 2026-07-30: dependency policy, formatting,
generated-schema drift, workspace Clippy with warnings denied, 430 workspace
tests with zero failures/ignored tests, and `git diff --check`. The first
workspace-test invocation reached its 120-second process timeout without a test
failure; the complete rerun with a longer bound passed. No dependency manifest
or lockfile changed.

## Artifact receipt

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `crates/seacad-dxf-core/src/block_definition.rs` | 292 | `22c491666d04dab1dc16552949e3fc1f813d69cc42ebca26e0c4dbb3b3ed81f1` |
| `crates/seacad-dxf-core/src/lib.rs` | 429 | `a9fc56654f66c3a12c9b3f9007e405ab980fc49ccff32cf951e0703c6a13c720` |
| `crates/seacad-dxf-core/tests/block_definition_tests.rs` | 297 | `b8e06c1c499b554a69f804b64e1f5588e208719e4a9ac228cd3dae6e790a30e5` |
| `docs/IMPLEMENTATION_PLAN.md` | 721 | `d85aa70133a685a3e3159db1f3f3f4c51d44cd3a64d8a09ae0361c8068862375` |
| `docs/SUPPORT_MATRIX.md` | 613 | `1065d96b946804c0982254efde923f34296edad3a613dee4fefade9874241695` |
