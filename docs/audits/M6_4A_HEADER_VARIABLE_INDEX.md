# M6.4a exact HEADER variable index

M6.4a creates the structural directory required to scale from the three typed
bootstrap fields to the full public HEADER inventory without adding one parse
tracker per field.

## Normative boundary

The existing M4.1 Autodesk audit establishes that an exact HEADER section is
opened by `0/SECTION, 2/HEADER`, each variable begins with group code 9, and the
section ends with `0/ENDSEC`. Autodesk's numerical group-code reference also
defines code 9 as a HEADER variable-name identifier.

Reviewed sources already frozen by M4.1:

- [Autodesk HEADER section](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-EA9CDD11-19D1-4EBC-9F56-979ACF679E3C.htm)
- [Autodesk numerical group codes](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3F0380A5-1C15-464D-BC66-2C5F094BCFB9.htm)
- [Autodesk general DXF structure](https://help.autodesk.com/cloudhelp/2020/ENU/AutoCAD-DXF/files/GUID-D939EA11-0CEC-4636-91A8-756640A031D3.htm)

## Exact indexing contract

For every exact group-code 9 inside every exact HEADER section, the immutable
index stores:

- the containing `0/SECTION` group occurrence;
- the group-code 9 marker occurrence;
- the exact source span of the variable name;
- the half-open range of all following value groups up to the next group-code
  9, `0/ENDSEC`, an interrupting `0/SECTION`, framed EOF, or input end.

Names are not decoded, normalized, copied, deduplicated, or matched against the
schema. Unknown names, duplicate names, empty value ranges, and multi-group
values remain ordered and fully visible. Groups outside exact HEADER sections
and non-exact section names are not reclassified.

The ASCII and Binary parse loops call the same tracker and reuse their already
framed value bytes. The index adds no source scan, file-sized buffer, semantic
default, diagnostic, dependency, writer behavior, or mutation path. One compact
fixed-size entry is allocated per indexed marker, and the existing record limit
therefore bounds total index storage.

## Verification

Synthetic tests cover all nine AC1009-AC1032 versions in both physical formats
with identical marker/value occurrence ranges and exact name-span readback.
Additional cases cover variables outside HEADER, non-exact section names,
unknown names, pre-marker groups, multi-group values, an empty value range,
multiple HEADER sections, an interrupted section, EOF without ENDSEC, 256
ordered custom variables, compact metadata, and occurrence-width overflow.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `crates/seacad-dxf-core/src/header_index.rs` | `4ca4a32853c417a31c424bf43d0b2b4ce8515ef205ded87e6eb9144b666df52a` |
| `crates/seacad-dxf-core/src/ascii_document.rs` | `5605da5eedca4cca74c01e6c73374e0582da5e699abce95f6eda4d0f63fef164` |
| `crates/seacad-dxf-core/src/binary_document.rs` | `54aab14499cc457fa41ec9ab744f8d9f9fee88de3f216aea31373f626f01c1f6` |
| `crates/seacad-dxf-core/src/lib.rs` | `15df19958f2877107c60414e4062500eb2c0cb343103c0d5ea22e2ecc235fb86` |

This checkpoint claims structural HEADER variable accounting only. It does not
claim semantic support for the remaining Autodesk HEADER variables or replace
the three reviewed typed views from M6.3.
