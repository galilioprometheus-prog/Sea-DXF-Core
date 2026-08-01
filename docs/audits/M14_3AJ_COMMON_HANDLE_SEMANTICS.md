# M14.3aj Common Handle Semantics

## Scope

M14.3aj projects the five handle-valued common entity fields over the existing
generic field semantics and M7 document-local handle-resolution directory. It
does not rescan groups or select duplicate field occurrences or targets.

## Sources

- Autodesk common entity codes define group 5 identity, group 330 BLOCK_RECORD
  owner, group 360 inside the `ACAD_XDICTIONARY` control group, material group
  347 with BYLAYER omission, and plot-style group 390:
  `https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm`
- Autodesk's numerical group-code reference classifies 330..339 as soft
  pointers, 340..349 and 390..399 as hard pointers, and 360..369 as hard
  owners:
  `https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3F0380A5-1C15-464D-BC66-2C5F094BCFB9.htm`
- Autodesk distinguishes pointer usage references from ownership references:
  `https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-704F5152-B2A4-4DAC-A0FD-03D8ABFC0A4F.htm`

The user-authorized legacy repository was inspected read-only as a behavioral
oracle. Its evidence confirmed that resolved, unresolved, null, and ambiguous
extension/reference outcomes must remain separate and that dictionary topology
must not be promoted. No external or legacy code, fixture, data, or dependency
was copied, translated, vendored, linked, or added at runtime.

## Contract

- `DxfEntityCommonHandleDirectory` owns the generic common-field semantic
  directory, shared M7 resolution directory, and five stable entries per entity.
- Group 5 remains an `Identity` carrying the exact lexical semantic value. It
  is not treated as a reference to itself.
- Owner, extension dictionary, material, and plot style are `Reference` values.
  Field decode/cardinality failures remain typed `Field` issues with unchanged
  provenance.
- Explicit valid handles distinguish `Null`, `Missing`, `Ambiguous`, and one
  exact `Resolved` target. The resolved value retains both handle and exact
  source-backed target identity match.
- Optional extension dictionary remains absent. Omitted material remains the
  generated `ByLayer` default and is never converted into a null handle.
- Construction and lookups are source-bound, cancellation-aware,
  allocation-fallible, group-order independent, and payload-redacted.

## Nonclaims

Unique numeric resolution does not prove target record-kind compatibility,
authoritative owner selection, one-owner conformance, hard/soft lifecycle,
dictionary membership, graph closure, purge behavior, applicability, or safe
edit/clone/delete behavior. Names, transparency, proxy count/data relations,
family patches, handle assignment, and entity `Complete` support remain open.

## Verification

Paired ASCII/Binary fixtures cover AC1009 through AC1032. Modern fixtures prove
four unique reference classes and exact target identity; AC1009 proves missing-
required, absent, and `ByLayer` states without inventing inapplicable groups.
Negative fixtures cover invalid hexadecimal syntax, duplicate singleton,
missing required field, explicit null, dangling target, duplicate target
identity, exact provenance, cancellation, source mismatch, bounded lookup,
public traits, and debug redaction. Final gate counts, production diff, and
artifact hashes are recorded below. The focused suite passed 4/4 tests, the
handle-resolution and common-field regressions passed 7/7, and the full
workspace passed 842/842 tests. Schema generation and release-evidence checks,
`cargo deny --locked check`, formatting, workspace Clippy with warnings denied,
workspace tests, and `git diff --check` all passed. The production diff is 342
added lines: 336 in the common-handle projection and 6 module/exports. This
audit intentionally omits its own hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 184 | `77974dcafb3f595f5079699f828ec53bbfed0303500001656dacefca7510b60e` |
| `crates/seacad-dxf-core/src/entity_common_handle_semantic.rs` | 336 | `0128dbe5031c807f357c6cafeb81aa5dedbac7cd0e6ee34a0eeb76b916d0ccfe` |
| `crates/seacad-dxf-core/src/lib.rs` | 991 | `abbd59a316bbefc6885439912237638c6296d373c64871501c25787d1108cfb9` |
| `crates/seacad-dxf-core/tests/entity_common_handle_semantic_tests.rs` | 451 | `f380380a78f5189fff1691ee8d69d3d3e06f3cde697cd1bb3e750fddbf36ca34` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 694 | `108d55795e103fb7a9483d8d7de3e5fc1120c79ac43a891879fa7ba607d86269` |
| `docs/IMPLEMENTATION_PLAN.md` | 2121 | `a59f280631a1eada49a7d56637e5f07fc70d2264a385c385fc14f12ea6f4b99e` |
| `docs/SUPPORT_MATRIX.md` | 1780 | `78e62e871ff2e0cafd2d442dce5eca3f7891f21ffeeb9fc9d31d6f44f06ce06d` |
