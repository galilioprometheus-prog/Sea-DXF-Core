# M8.1a Autodesk Basic-Geometry Reference

Retrieved: 2026-07-29

## Primary references

- Autodesk, [POINT (DXF)](https://help.autodesk.com/cloudhelp/2025/ENU/AutoCAD-DXF/files/GUID-9C6AD32D-769D-4213-85A4-CA9CCB5C5317.htm)
- Autodesk, [LINE (DXF)](https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-FCEF5726-53AE-4C43-B4EA-C84EB8686A66.htm)
- Autodesk, [Common Group Codes for Entities (DXF)](https://help.autodesk.com/cloudhelp/2025/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm)
- Autodesk ObjectARX 2027, [DXF Group Codes](https://help.autodesk.com/cloudhelp/2027/ENU/OARX-RefGuide/files/OARX-RefGuide-DXF_Group_Codes.html)

## Reviewed facts

- `POINT` group codes `10`, `20`, and `30` are the WCS point location.
- `LINE` group codes `10`, `20`, and `30` are the WCS start point, while
  `11`, `21`, and `31` are the WCS endpoint.
- Both record types use optional `210`, `220`, and `230` extrusion-direction
  components. Autodesk documents the omitted extrusion default as `(0, 0, 1)`.
- The ObjectARX filer reference classifies the `10..17` family as point,
  vector, or scale doubles and the paired `20..27` and `30..37` families as
  Y and Z coordinate doubles.
- Autodesk explicitly warns consumers not to depend on the order shown in the
  entity tables because group order can change.

## M8.1a decision

M8.1a indexes only the exact source occurrences documented above for exact
uppercase `POINT` and `LINE` records in complete `BLOCKS` or `ENTITIES`
sections. It assigns a coordinate role by numeric group code, decodes the raw
double with the shared ASCII/Binary path, and retains source order, duplicates,
invalid ASCII numbers, and raw spans.

The directory does not assemble a canonical point or line. It does not choose
among duplicate components, require a component, apply the optional extrusion
default, interpret thickness or the POINT display angle, normalize a vector,
or convert between WCS, OCS, UCS, DCS, or any other coordinate system. Those
decisions require separate reviewed milestones.
