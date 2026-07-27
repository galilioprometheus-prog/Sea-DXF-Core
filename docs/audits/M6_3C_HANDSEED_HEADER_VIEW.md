# M6.3c `$HANDSEED` typed HEADER view

M6.3c implements the reviewed public DXF meaning and syntax of `$HANDSEED`
without extending the claim into handle allocation or document topology.

## Normative sources and contract

Autodesk's HEADER reference defines `$HANDSEED` at group code 5 as the next
available handle. Autodesk's group-code reference defines code 5 as an entity
handle stored as a fixed text string of up to 16 hexadecimal digits. Autodesk's
ObjectARX filer reference describes that string as a representation of a 64-bit
integer handle.

Reviewed sources:

- [Autodesk DXF HEADER variables](https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-A85E8E67-27CD-4C59-BE61-4DC9FADBE74A.htm)
- [Autodesk DXF group codes](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3F0380A5-1C15-464D-BC66-2C5F094BCFB9.htm)
- [Autodesk ObjectARX `writeAcDbHandle`](https://help.autodesk.com/cloudhelp/2017/ENU/OARXMAC-RefGuide/files/OREFMAC-AcDbDxfFiler__writeAcDbHandle_AcDb__DxfCode_AcDbHandle_.html)

SeaCad therefore accepts exactly 1-16 ASCII hexadecimal digits, in either
case, with optional leading zeroes. It does not trim whitespace, accept a
`0x` prefix, decode Unicode, or replace malformed bytes. The semantic value is
an exact `u64`; original source spelling and byte span remain authoritative.

Zero is syntactically valid and exposed by `DxfHandle::is_null`. Whether zero
is permitted for a future allocator is deliberately deferred to M7/M11 because
the reviewed HEADER references do not establish that policy.

## Implementation boundary

ASCII and Binary documents call the same fixed-state `$HANDSEED` tracker from
their existing parse loops. Binary reuses the already framed string payload.
There is no second source scan, per-record allocation, new dependency, writer
behavior, mutation API, or default-value inference.

`DxfHeaderView` exposes `Explicit`, `Absent`, or `Invalid` with schema and raw
provenance. Empty, overlength, non-hex, wrong-group, missing, and duplicate
declarations are distinct typed failures. Diagnostics use stable codes
`DXF-E0430` and `DXF-E0431`.

Tests cover all nine AC1009-AC1032 versions in ASCII and Binary, lower/upper
case, leading zeroes, zero, the 16-digit maximum, every parser boundary,
wrong group, duplicate, compatible missing value, exact raw readback, and
source-anchored diagnostics.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `crates/seacad-dxf-core/src/handle.rs` | `a0cc7438a1cecb6d3e3627345ef92885e357e431c6ff3b017ae40bec88335b1d` |
| `crates/seacad-dxf-core/src/handseed.rs` | `36f525b0bc23beb8bc572a94f1955a3f852e5b86676ce4186a4a49b4b0aeaad7` |
| `crates/seacad-dxf-core/src/ascii_document.rs` | `0a24852800d463126518c242fc0759b3ba4bbc8b7dea6de59d7cc1373370abcb` |
| `crates/seacad-dxf-core/src/binary_document.rs` | `94d578f2ce5e192ef3cd44bb055ce68088024b70438d32c7fe8b34d5151223f6` |
| `crates/seacad-dxf-core/src/diagnostic.rs` | `1b99df2dabaabdce597fbca0b7549b61cc73aa1c039692b920d522693b495e89` |
| `crates/seacad-dxf-core/src/header_view.rs` | `f81b2e7e6212f1791d7db8d9f8dfae85a0a89d0299155e39d5bf9b17ea95f572` |
| `crates/seacad-dxf-core/src/lib.rs` | `736e82164f344a9db7a2c2f534ab30323f9ac6a7b4b46d191d79b7530a4efb10` |

This checkpoint claims only exact syntactic handle parsing and the documented
HEADER meaning "next available handle." Allocation, collision handling,
ownership, references, and topology remain future milestones.
