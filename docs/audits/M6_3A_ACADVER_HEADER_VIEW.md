# M6.3a typed `$ACADVER` HEADER view

M6.3a is the first read-only consumer of the generated HEADER schema and the
four-state semantic value contract. It projects the existing evidence-backed
`DxfAcadVersionReport`; it does not scan or read source bytes again.

Construction fails closed if generated metadata no longer identifies
`acadver` as exact `$ACADVER`, group code 1, `exact_text`. The resulting field
provenance uses the raw document `SourceId` and generated field ID.

The mapping is exact:

- supported values are `Explicit` with the original value group occurrence and
  byte span;
- absence is `Absent` without fabricated raw evidence;
- unsupported, wrong-group, missing, and duplicate cases are `Invalid` with a
  typed reason;
- invalid values retain the best exact candidate or variable-marker evidence.

ASCII and Binary raw documents call the same O(1) projection over their stored
dialect report. Synthetic tests cover all nine supported AC1009-AC1032 versions
in both physical formats and verify raw span readback. Boundary tests cover
absent, unsupported, wrong group, missing value, and duplicate values.

The normative behavior and support claim already belong to the reviewed M4.1
Autodesk dialect audit and receipt. M6.3a adds no dependency, cache, writer, or
new DXF semantic claim. `$DWGCODEPAGE` and `$HANDSEED` remain outside this
checkpoint.
