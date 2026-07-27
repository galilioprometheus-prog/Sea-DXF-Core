# M6.1a HEADER bootstrap provenance

Normative source: Autodesk `HEADER Section Group Codes (DXF)`, topic
`GUID-A85E8E67-27CD-4C59-BE61-4DC9FADBE74A`.

<https://help.autodesk.com/cloudhelp/2026/ENU/AutoCAD-DXF/files/GUID-A85E8E67-27CD-4C59-BE61-4DC9FADBE74A.htm>

The ordered normalized `variable-name|group-code-expression` inventory was
independently checked across the Autodesk 2015, 2018, 2021, 2024, 2025, and
2026 pages. All six contained 206 rows and produced the same normalized facts
SHA-256:

`d1034c4246758f368ac79982fa1c21d59328851d9fdca8ccebe944f20f71cfaf`

M6.1a records only these three reviewed name/code shapes:

| SeaCad ID | Public DXF name | Ordered group codes | Storage shape |
| --- | --- | --- | --- |
| `acadver` | `$ACADVER` | `1` | exact text |
| `dwgcodepage` | `$DWGCODEPAGE` | `3` | exact text |
| `handseed` | `$HANDSEED` | `5` | handle |

All are explicitly marked `shape_only`. Dialect applicability, defaults,
requiredness, semantic decoding, duplicate behavior, and edit behavior remain
`not_yet_reviewed`. Generated metadata therefore cannot establish a public
semantic support claim.

No Autodesk prose or HTML is stored in the schema or generated Rust.
