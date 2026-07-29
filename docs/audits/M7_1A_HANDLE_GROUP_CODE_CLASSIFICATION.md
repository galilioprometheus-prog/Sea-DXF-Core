# M7.1a handle group-code classification

M7.1a adds a context-neutral public registry for numeric DXF group codes whose
values are handles. It is deliberately smaller than a document topology API:
the registry classifies the role assigned by a group code but never reads a
payload, follows a target, validates object existence, or assigns ownership.

## Normative basis

Autodesk's numerical group-code reference defines code `5` as an entity handle,
code `105` as the DIMVAR symbol-table-entry object handle, ranges `320..329` as
arbitrary handles, `330..339` as soft pointers, `340..349` as hard pointers,
`350..359` as soft owners, `360..369` as hard owners, `390..399` as plot-style
handles that are basically hard pointers, `480..481` as hard pointers, and
`1005` as an extended-data entity handle:

<https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3F0380A5-1C15-464D-BC66-2C5F094BCFB9.htm>

Autodesk separately states that `1005` extended-data handles have the same
behavior and semantics as soft pointers:

<https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-8243079C-B44F-493A-BAF7-1D11A6E6C78C.htm>

The common entity and object tables show concrete `330`, `347`, `360`, and
`390` pointer/owner uses and the application control strings that delimit
persistent-reactor and extension-dictionary groups:

<https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm>

<https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-6939D69E-04CB-4F4C-87B2-67BC540FCF58.htm>

## Public contract

`DxfHandleGroupClass` exposes six roles:

| Numeric code | Class |
| --- | --- |
| `5`, `105` | `ObjectIdentity` |
| `320..329` | `Arbitrary` |
| `330..339`, `1005` | `SoftPointer` |
| `340..349`, `390..399`, `480..481` | `HardPointer` |
| `350..359` | `SoftOwner` |
| `360..369` | `HardOwner` |

`classify_dxf_handle_group_code` is a bounded `const` function over an already
validated `DxfGroupCode`. It returns `None` for every other group code. The
original code stays with the caller, so plot-style, XDATA, and other contexts
remain distinguishable without multiplying semantic variants.

Both public additions are dependency-free, allocation-free, thread-safe, and
copyable. Existing exact hexadecimal parsing and HEADER handle contracts are
unchanged.

## Explicit non-claims

This checkpoint does not parse handle payloads outside the HEADER directory,
identify record boundaries, build a handle index or reference graph, resolve
targets, validate dangling references, interpret pointer strength during an
edit, or implement dictionaries, XDATA containers, reactors, cloning, INSERT,
or XREF translation. Those operations require later M7 evidence and APIs.

## Antigravity handoff

Before implementation, Antigravity Agent Hub task 18 performed a read-only
inventory at HEAD `4ffae8b201a6fa324b92a392d47038edfb856599`. It reported the
existing exact handle parser, public exports, raw-group access, Binary wire
families, and five passing handle-filtered tests. Its semantic search found only
framing-level XDATA mentions and no ownership, graph, dictionary, XDATA decode,
or reactor implementation. The worker left the repository unchanged. Codex
independently reproduced the inventory and tests before accepting the task.

## Verification scope

Boundary tests cover both ends of every documented range plus codes `5`, `105`,
`1005`, all six public variants, and adjacent or unrelated non-handle codes.
Trait tests retain the public `Send + Sync + Copy` contract. The handwritten
production diff adds 90 lines and removes two across the handle module and its
public re-export. No manifest, dependency, lockfile, parser branch, source scan,
writer, fixture, schema, or generated file changes.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `crates/seacad-dxf-core/src/handle.rs` | `0489048ab9cfdf59925281ab44eb191e4c64786bdd0f66dd7f7504d01385dc76` |
| `crates/seacad-dxf-core/src/lib.rs` | `854fefac7dc040263f2bdb7c1d0199303a6aee1e2a62fefe7d7eff142befc1b0` |
| `docs/IMPLEMENTATION_PLAN.md` | `54c408f850061d44e2dbbcbdf4655e602afebd9b08e155ce594b2b4307b47a4d` |
| `docs/SUPPORT_MATRIX.md` | `11254775b307271e19a91268142f12b2e7e0616f2314f8cae46c9cfb7baa4b65` |

## Required gates

Rust 1.97.1 passed:

- `cargo deny --locked check` (advisories, bans, licenses, and sources);
- `cargo +1.97.1 fmt --all -- --check`;
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`;
- `cargo +1.97.1 clippy --workspace --all-targets -- -D warnings`;
- `cargo +1.97.1 test --workspace`: 259 passed, comprising 204 core unit
  tests, four handle integration tests, 12 numeric integration tests, four
  text integration tests, 16 CLI tests, six corpus-receipt tests, and 13
  schema-generator tests;
- `git diff --check`.

The focused handle-filter command passed seven tests with zero failures. The
reviewed production diff contains no added `unsafe`, `panic!`, `unwrap`,
`expect`, `todo!`, or `unimplemented!` use.
