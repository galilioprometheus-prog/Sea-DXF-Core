# M4.2 Autodesk DXF structure reference audit

Status: completed 2026-07-27

This audit freezes the public evidence used for SeaCad's ASCII section
accounting and group-code 0 index. No external parser, implementation, or
fixture was copied, translated, or ported.

## Normative public references

- Autodesk, [About the General DXF File Structure](https://help.autodesk.com/cloudhelp/2020/ENU/AutoCAD-DXF/files/GUID-D939EA11-0CEC-4636-91A8-756640A031D3.htm):
  a section begins with `0/SECTION`, its immediate group-code 2 value names the
  section, and `0/ENDSEC` ends it. The page documents HEADER, CLASSES, TABLES,
  BLOCKS, ENTITIES, OBJECTS, and optional THUMBNAILIMAGE sections.
- Autodesk, [About Group Codes in DXF Files](https://help.autodesk.com/cloudhelp/2015/ENU/AutoCAD-DXF/files/GUID-89CB823D-614D-4D1E-8204-568EC72DF869.htm):
  entities, objects, classes, tables, table entries, and file separators are
  introduced by group code 0 followed by a name.
- Autodesk, [About Object and Entity Codes](https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-A35B8C2A-1885-4A8E-8533-E61D8A423D62.htm):
  the next group code 0 terminates the current entity and begins the next
  entity or the end of its section; future readers should not assume undefined
  group codes or fixed field order.
- Autodesk, [Entity Group Codes in DXF Files](https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-995ABB55-571A-4D0F-882E-8A74A738643E.htm):
  shows the exact ENTITIES envelope and one group-code 0 entry per entity.
- Autodesk, [Object Group Codes in DXF Files](https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-1038FDE4-745D-469D-972E-1F977D674882.htm):
  shows the equivalent group-code 0 organization for OBJECTS.
- Autodesk, [DXF Group Codes in Numerical Order](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3F0380A5-1C15-464D-BC66-2C5F094BCFB9.htm):
  group code 0 is the fixed text-string entity/type discriminator.

References were reviewed on 2026-07-27.

## Clean-room interpretation

- The numeric group code is taken from the already validated lossless framer.
  Every numeric code 0 is indexed regardless of how the integer was spelled in
  ASCII.
- Marker values `SECTION` and `ENDSEC`, section names, and known-name matching
  are byte-exact and case-sensitive. Compatible mode never trims or changes
  their semantic meaning.
- Framed EOF uses the already frozen M3 Strict/Compatible envelope decision.
  A recovered padded EOF remains raw and diagnostic while still terminating
  section accounting at the same physical boundary.
- Unknown section names are valid opaque names. SeaCad stores their source span
  and never discards or decodes their bytes at this milestone.
- Malformed envelopes remain openable as immutable raw documents. SeaCad emits
  diagnostics and constructs conservative, non-overlapping ranges rather than
  inventing a nesting model not documented by Autodesk.

No AutoCAD oracle is required for this checkpoint because the indexed
boundaries are directly documented. AutoCAD becomes useful again when later
milestones interpret versioned section contents.
