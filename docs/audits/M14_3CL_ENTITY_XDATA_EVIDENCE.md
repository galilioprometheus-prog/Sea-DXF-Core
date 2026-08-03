# M14.3cl Entity XDATA Evidence

Retrieved: 2026-08-03

## Normative basis

Autodesk's *About Extended Data (DXF)* states that entity XDATA follows normal
entity definition data, uses group codes 1000 through 1071, preserves source
order, and starts each registered-application list with group 1001. It also
states that application names correspond to APPID table entries and that a
second 1001 begins another application list:

<https://help.autodesk.com/cloudhelp/2016/ENU/AutoCAD-DXF/files/GUID-A2A628B0-3699-4740-A215-C560E7242F63.htm>

Autodesk's numerical group-code reference supplies the physical value families
within that range, including group 1002 control strings, group 1004 binary
chunks, group 1005 handles, point/vector components, doubles, and signed
integers:

<https://help.autodesk.com/view/OARX/2025/ENU/?guid=GUID-3F0380A5-1C15-464D-BC66-2C5F094BCFB9>

## Contract

- `DxfEntityXDataDirectory` owns the existing unified entity directory and
  retains every record-local group in the inclusive 1000..=1071 range that is
  outside a group-102 application-control payload.
- Every group 1001 creates a separate `DxfEntityXDataApplication`; duplicate
  raw application names are not merged or selected.
- Each following XDATA-code occurrence retains its exact `DxfRawGroup`, entity,
  directory context, and source order.
- An XDATA code before any current group 1001 remains an `Orphan` occurrence.
- A normal entity group ends the current list as `Interrupted`. The following
  XDATA codes remain orphans until another group 1001 starts a new list.
- A new group 1001 or the raw-record boundary records the current list as
  `Contiguous` without claiming that its name, braces, values, or application
  payload are semantically valid.
- Construction is cancellation-aware, fallible on allocation, source-bound,
  non-disclosing in `Debug`, and linear in the already bounded raw group set.

## Verification boundary

One paired fixture runs over ASCII and Binary AC1009 through AC1032. It covers
an orphan value, two same-spelling application names, ordered string/control/
integer/handle/double values, a list interrupted by a normal entity group, an
orphan after interruption, a restarted list, and XDATA-shaped codes hidden
inside a group-102 application control. Separate checks cover cancellation,
foreign-source rejection, lookup bounds, metadata size bounds, and redacted
debug output.

## Nonclaims

This checkpoint does not resolve application names against APPID, validate the
31-byte name limit or symbol-name syntax, validate group-1002 brace structure,
project typed XDATA values, enforce the separate per-entity 16-KiB AutoCAD
policy, interpret application payloads, validate or remap group-1005 targets,
clone/write XDATA, or advance POINT or any other entity to `Complete`.

## Verification

The focused entity-XDATA suite passed 3/3 tests. The first non-quiet workspace
run reached its 180-second wrapper timeout after only passing output; its quiet
retry passed 971/971 listed tests in 72.9 seconds. After the final `Contiguous`
state-name refinement, the first cold quiet run likewise reached its 180-second
wrapper after only passing output, and the complete retry passed 971/971 in
54.2 seconds. Generated schema and release-evidence checks, `cargo deny
--locked check`, formatting, workspace Clippy with warnings denied, the
production forbidden-construct scan, and `git diff --check` all passed.
Production changes are 437 insertions; the focused test target adds 353 lines.
No manifest, lockfile, dependency, generated schema, locale source, committed
fixture, or external corpus changed. This audit intentionally omits its own
hash.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 404 | `59f0b55aff21cd0c7839cf82acbaf6d9541cb78fed29c247bdb5bb3b29ec116e` |
| `crates/seacad-dxf-core/src/entity_xdata.rs` | 432 | `5776bdc69ee2def07851df0630eb6c4099cdca73eb636242acc47b7b7dc3b75e` |
| `crates/seacad-dxf-core/src/lib.rs` | 1,073 | `85464a715fc7a0acd449944c9f045611fe5ffec78aceb58c97f74e9297274b68` |
| `crates/seacad-dxf-core/tests/entity_xdata_tests.rs` | 353 | `66cde78a3ee486380090c775e2c874021685538f84537358110b7662e5ff24b1` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 1,336 | `32d5c5b9740d70d41937b36e9c774dcb3697a1c5822ccc707d37b19fac67c305` |
| `docs/IMPLEMENTATION_PLAN.md` | 2,762 | `e61565c9e6022e625bde9a72cbc8754d02067832fd3b3c5074f7d3250bdd40e2` |
| `docs/SUPPORT_MATRIX.md` | 2,409 | `9e9a54ea1250191ea58de1734378f51e66d52786e71dfef7009fb46943cb2ba0` |
